# Parallelism

English | [简体中文](./parallel.zh.md)

This file records how a loop is judged worth threading, which interfaces the
crate offers, and what the measurements were. The cross-layer unit and
coordinate conventions are in [Units and coordinates](units.md), and the
layering is in [Layer boundaries](layers.md).

## Why there is no thread pool

The crate forbids every third-party crate, so a pool such as `rayon` is
not available. None is needed either: every computation here that is worth
several cores is a loop over independent items, which the standard
library's `std::thread::scope` expresses directly.

Scoped threads borrow their data, so a caller shares a slice or a read-only
value without an `Arc`, and the scope joins every thread before it returns,
so no result is read while a thread is still writing it.

## The interfaces

### apsis_math: the primitives

`apsis_math::parallel` provides the entry points every other interface builds
on:

| Function | Purpose |
| --- | --- |
| `thread_count()` | the parallelism of the machine |
| `threads_for(len)` | the threads worth starting for a length |
| `for_each_index(len, f)` | call `f` for every index |
| `map_index(len, f)` | produce a value per index, returned in index order |

Both loop entries require only that the closure can be **called** from
several threads (`Fn` plus `Sync`), because they treat the input as shared
and the output as a slot each thread owns; the caller synchronises nothing.

`threads_for` returns one below a thousand items per thread, because at that
scale starting a thread costs more than the work it would save.

### The batch interfaces of the layers

| Module | Interface | Scale it suits |
| --- | --- | --- |
| `apsis_sgp4` | `propagate_minutes_at_parallel` | a long track, tens of thousands of samples or more |
| `apsis_sgp4` | `propagate_at_parallel` | a whole shell in one call |
| `apsis_access` | `find_all_parallel` | hundreds of tracks |
| `apsis_isl` | `visibility_matrix_parallel` | a few hundred satellites or more, see below |

Each interface has a sequential counterpart of the same name. The two agree
element for element and in the same order, so a caller may switch freely.

## When a loop is worth threading

Not every batch is worth threading. The decision rests on the **cost of one
item** and the **size of the batch**:

- **Worth threading**: the work per item is expensive, at a microsecond or
  more, the items are independent, and the batch reaches hundreds. The
  typical cases are SGP4 propagation at about 235 ns a sample over millions
  of samples, and the access search at about 7 us a track.
- **Not worth threading**:
  - `Constellation::instantiate`: pure arithmetic, microseconds for a few
    hundred satellites, which the thread overhead entirely drowns.
  - The scheduler: a greedy placement is serial in substance, since each
    placement changes the feasible interval of the tasks after it, so the
    loop is not independent and there is no parallel interface. Its search
    is nonetheless much faster than it was, as the last section describes.
  - The event engine: one queue in time order, which cannot be split.
  - The geodetic and coordinate transforms: a single call is already
    constant time, so there is no batch.
  - One `mutual_visibility` call: about 4 ns, which is not worth threading
    on its own but is as a matrix, as below.

## Measurements (sixteen cores, release)

### One satellite track (`propagate_minutes_at_parallel`)

| Track length | Sequential | Parallel | Speed-up |
| --- | --- | --- | --- |
| 1 day (1,440 samples) | 0.3 ms | 0.3 ms | 1.28x |
| 7 days (10,080) | 2.3 ms | 0.9 ms | 2.46x |
| 30 days (43,200) | 10.1 ms | 1.9 ms | 5.45x |
| 365 days (525,600) | 144.1 ms | 26.0 ms | 5.54x |

**Conclusion**: the longer the track the larger the gain, settling near 5.5x
past a month. A day or less gains little, because splitting 1,440 samples
across the threads leaves each with fewer than the thousand items
`threads_for` asks for.

**Practical point**: with many short tracks, call `propagate_at_parallel` for
the whole shell rather than the parallel form once per satellite, because the
latter pays the thread start-up once per satellite.

### The visibility matrix (`visibility_matrix_parallel`)

| Satellites | Pairs | Sequential | Parallel | Speed-up |
| --- | --- | --- | --- | --- |
| 200 | 40 thousand | 0.16 ms | 0.31 ms | 0.53x |
| 600 | 360 thousand | 1.50 ms | 1.37 ms | 1.09x |
| 1,000 | 1 million | 4.09 ms | 2.50 ms | 1.64x |

**Conclusion**: one visibility test costs a few nanoseconds, so **below a
few hundred satellites the parallel form is slower**, because the thread
overhead dominates. It only starts to gain past that and is nowhere near
saturated. The interface exists for a large shell; a small one should use
the sequential form, and the interface documentation says so.

### Across layers

The measurements in `bench_parallel` show that propagating a whole shell and
building a topology benefit most, because they hand the entire amount of work
to the threads at once. Run the example for the figures.

## Determinism

Threading does not change the result. Every batch interface compares its
output element by element against the sequential form, including the order:
the states from SGP4, the boolean matrix from visibility, the pass summaries
from access. Propagation is deterministic and depends only on the instant, not
on how it was split, and visibility and the pass search are independent per
item. The same input therefore gives the same output at any thread count, and
the tests assert exactly that.

## The scheduler search

The scheduler has no parallel interface, but its worst case once cost
hundreds of microseconds a task. The fix came in two steps, neither of which
changes a single bit of the result.

**First, one sweep in place of a test per candidate.** The search used to
list every instant at which feasibility could change and test them one by
one, recomputing the load profile of every resource at each step. The profile
does not depend on the candidate, so it is now swept once: the intervals
where a resource has no room are cut from its availability windows, the
per-resource results are intersected, and the first surviving start is read
off the front. The answer is still confirmed by `feasible_at`, and the
enumeration remains as the fallback for the rare case where rounding at a
capacity boundary makes the two disagree.

**Second, the confirmation reuses the index.** Removing the confirmation
entirely cut the crowded case by eight, which located the real cost: the
public feasibility test that does the confirming scans the task list once to
collect the boundaries and then, for every boundary, once more to sum the
demand of each placement covering it. Passing the index the sweep already
built turns both scans into a read by identifier.

A task identifier is a small dense integer, so the index is a vector rather
than a map: an array index is cheaper than hashing a small integer, which
this repository has now measured twice, once for the adjacency index in
`apsis_topology` and once here.

**Result**: with every task contending for one resource and the capacity the
binding constraint, planning three thousand two hundred tasks falls from 6.7
seconds to 0.48, a factor of fourteen, and six thousand four hundred takes
2.2 seconds. The plan is unchanged: a test compares the start and end of
every placement before and after, across two resources with disjoint
availability windows.
