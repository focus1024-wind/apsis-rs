# Benchmark method and results

English | [简体中文](./bench.zh.md)

This file records how the benchmarks are run and what they measured, so
that someone else can reproduce them and judge whether the conclusions hold.

## Method

### Three measurements, each answering a different question

| Tool | Question it answers |
| --- | --- |
| `bench` | How expensive is one call |
| `bench_scale` | How does the cost grow with size |
| `bench_parallel` | How much do the other cores save |

All three are examples of the `apsis-rs` facade and run in release mode:

```bash
cargo run --release -p apsis-rs --features full --example bench
cargo run --release -p apsis-rs --features full --example bench_scale
cargo run --release -p apsis-rs --features full --example bench_parallel
```

### Three rules

**No warm-up.** No call is discarded before a measurement begins. The first
call pays for whatever the path allocates or caches, and that is a cost the
caller waits for; hiding it behind a discarded run would report a figure
nobody ever waited for. `bench_scale` therefore reports a cold column beside
the steady one.

**Take the best of several runs.** The serial baseline is run a few times and
the fastest kept. A scheduling hiccup slows one run, not all of them, so the
fastest is the clean signal rather than a floor polluted by whatever else the
machine was doing. `bench_parallel` works this way.

**Report cost per unit.** The absolute figures move with the machine, the
nanoseconds per unit do not. The second is what shows whether the complexity
is right.

### How complexity is decided

For an input size `n`, a constant `ns per unit` means constant work per item
and linear total time. A `ns per unit` that grows as a power of `n` means the
total is that power. The exponent is the logarithm of the ratio of two
neighbouring points.

## Machine

| Item | Value |
| --- | --- |
| CPU | AMD Ryzen 7 8845HS, eight cores, sixteen threads |
| Memory | 28 GB |
| Toolchain | rustc 1.98.1, release |
| Dependencies | none; the benchmarks are examples |

The absolute numbers mean nothing off this machine. Compare ratios within one
tool, not values across machines.

## Result one: a single call

| Path | Per call |
| --- | --- |
| Hermite ephemeris query | 24.3 ns |
| TEME to Earth-fixed rotation | 26.8 ns |
| Keplerian elements to a state | 38.1 ns |
| Earth-fixed to geodetic | 50.1 ns |
| Link budget C/N0 | 60.6 ns |
| SGP4 propagation, thirty minutes from epoch | 248.7 ns |
| Access search, 120 samples | 7.49 us |
| Planning 60 tasks on one resource | 239 us |

A query against the ephemeris is about ten times cheaper than the
propagation it stands in for, which is the reason `apsis-ephemeris` exists:
when a track is queried many times, build the table once and interpolate.

## Result two: growth with size

### One day of a constellation, sixty second step

| Satellites | Cold | Steady | Per sample |
| --- | --- | --- | --- |
| 24 | 8.7 ms | 8.7 ms | 252 ns |
| 240 | 80.5 ms | 80.0 ms | 231 ns |
| 1008 | 339.5 ms | 339.5 ms | 234 ns |
| **2016** | 679.4 ms | 679.4 ms | 234 ns |

**The cost per sample is flat, an exponent of 0.00**, so one SGP4 propagation
is constant time and the whole day is linear in the number of satellites. A
day over two thousand satellites takes about seven tenths of a second. Cold
and steady are nearly equal, so the path has no significant one-off
allocation.

### Access search, one station

| Tracks | Per track |
| --- | --- |
| 8 | 7.3 us |
| 128 | 7.3 us |

**Flat, an exponent of 0.00.** Each track is independent, so the search is
linear in the number of satellites.

### Topology construction and routing

| Satellites | Per satellite | Exponent |
| --- | --- | --- |
| 8 | 237 ns | — |
| 64 | 2.2 us | 1.07 |
| 120 | 5.7 us | 1.52 |
| 300 | 33.0 us | 1.93 |
| **600** | **182.6 us** | 2.47 |

**Roughly O(n squared).** The cost per satellite grows with the count, the
exponent rising from one towards two and a half. The cause is inherent: the
geometry test is an all pairs check, and n satellites have n squared pairs to
judge. The parallel scan confirms where the time goes.

### Task planning

| Tasks | Per task | Exponent |
| --- | --- | --- |
| 60 | 4.5 us | — |
| 200 | 16.0 us | 1.05 |
| 1000 | 108.4 us | 1.33 |
| **2000** | **314.1 us** | 1.54 |

**Roughly O(n squared), easing towards O(n) at size.** A greedy placement
arranges tasks in time order and each arrangement is compared against the
placements already made, so the quadratic is the algorithm rather than a
defect in the implementation.

## Result three: threads

### Propagation, split by satellite

| Satellites | 1 thread | 2 | 4 | 8 | 16 | Best |
| --- | --- | --- | --- | --- | --- | --- |
| 550 | 184 ms | 1.96x | 3.83x | 6.86x | 8.12x | 8.12x at 16 |
| 1000 | 336 ms | 1.89x | 3.73x | 6.49x | 8.24x | 8.24x at 16 |
| 2000 | 671 ms | 1.89x | 3.74x | 6.61x | 7.93x | 7.93x at 16 |

**Propagation is embarrassingly parallel.** Every satellite is independent,
and two to eight threads come close to linear. Efficiency at sixteen threads
falls to about one half, which is the contention of running two threads per
core. Eight threads is the better setting, about eighty-five percent
efficiency; sixteen buys more absolute throughput at a lower return.

### Access search

| Tracks | 1 thread | 4 | 8 | 16 | Best |
| --- | --- | --- | --- | --- | --- |
| 64 | 0.47 ms | 2.74x | 2.68x | 1.87x | 2.74x at 4 |
| 256 | 1.95 ms | 3.52x | 4.36x | 5.27x | 5.27x at 16 |

**The gain is small on a small batch**, because the fixed cost of starting
threads dominates when there is little work. At 256 tracks it reaches about
5.3x.

### Topology construction, split by phase

| Satellites | Visibility, parallelisable | Edge fold, serial tail | Best speedup |
| --- | --- | --- | --- |
| 120 | 0.31 ms | 0.30 ms | 1.37x |
| 300 | 1.93 ms | 3.84 ms | 1.72x |

**Threading helps this path only a little.** At 300 satellites the serial
edge fold takes twice the parallelisable part, so the total gains at most
1.7x rather than the near-linear gain of the numerical kernels. The cause is
that `add_edge` scans the edges already stored to reject a duplicate, which
makes the insertion itself cubic.

## Conclusions

| Path | Complexity | Threads | At useful scale |
| --- | --- | --- | --- |
| SGP4 propagation | O(1) a sample | excellent, about 8x on sixteen | 2016 satellites a day in 0.68 s, 85 ms on sixteen |
| Ephemeris query | O(log n) | follows propagation | 24 ns |
| Access search | O(1) a satellite | good, about 5x on sixteen | 256 tracks in 0.37 ms |
| Coordinate and link | O(1) | not worth it | 27 to 61 ns |
| Task planning | about O(n squared) | serial | 2000 tasks in 0.63 s |
| Topology construction | about O(n squared) | limited, a serial tail dominates | 600 satellites in 110 ms |

**The points that matter**

1. The numerical kernels are linear and parallelise well. A day over two
   thousand satellites costs 0.68 seconds on one core and 85 ms on sixteen.
2. Interpolating the ephemeris is about ten times cheaper than propagating,
   so a densely queried track should be tabulated once.
3. Two combined operations are limited by their algorithm rather than by the
   available cores. Greedy planning is quadratically serial, and the
   duplicate check on edge insertion squares that quadratic; the second gains
   only about 1.7x from threading. Both would need a different algorithm to
   gain more.
4. Eight threads is the better setting for propagation. Efficiency is about
   eighty-five percent, while sixteen threads falls to about one half because
   two threads share a core.

## Reproducing

```bash
# The gates first, so that what is measured is the right code.
cargo test --workspace --all-features

# The three measurements.
cargo run --release -p apsis-rs --features full --example bench
cargo run --release -p apsis-rs --features full --example bench_scale
cargo run --release -p apsis-rs --features full --example bench_parallel
```

Do not run other work while measuring. The absolute values move with the
machine, so the conclusions rest on the ratios within one tool.
