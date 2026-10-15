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

All three are examples of the crate and run in release mode:

```bash
cargo run --release --features full --example bench
cargo run --release --features full --example bench_scale
cargo run --release --features full --example bench_parallel
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
| Memory | 22 GB |
| Toolchain | rustc 1.99.0, release |
| Dependencies | none; the benchmarks are examples |

The absolute numbers mean nothing off this machine. Compare ratios within one
tool, not values across machines.

## Result one: a single call

| Path | Per call |
| --- | --- |
| Hermite ephemeris query | 27.3 ns |
| TEME to Earth-fixed rotation | 24.0 ns |
| Keplerian elements to a state | 47.3 ns |
| Earth-fixed to geodetic | 50.0 ns |
| Link budget C/N0 | 60.3 ns |
| SGP4 propagation, thirty minutes from epoch | 264.5 ns |
| Access search, 120 samples | 6.38 us |
| Planning 60 tasks on one resource | 240 us |

A query against the ephemeris is about ten times cheaper than the
propagation it stands in for, which is the reason `apsis_ephemeris` exists:
when a track is queried many times, build the table once and interpolate.

## Result two: growth with size

### One day of a constellation, sixty second step

| Satellites | Cold | Steady | Per sample |
| --- | --- | --- | --- |
| 24 | 8.1 ms | 8.1 ms | 234 ns |
| 240 | 80.7 ms | 81.2 ms | 235 ns |
| 1008 | 339.8 ms | 339.8 ms | 234 ns |
| **2016** | 681.9 ms | 681.9 ms | 235 ns |

**The cost per sample is flat, an exponent of 0.00**, so one SGP4 propagation
is constant time and the whole day is linear in the number of satellites. A
day over two thousand satellites takes about seven tenths of a second. Cold
and steady are nearly equal, so the path has no significant one-off
allocation.

### Access search, one station

| Tracks | Per track |
| --- | --- |
| 8 | 7.2 us |
| 128 | 6.5 us |

**Essentially flat, an exponent of −0.03.** Each track is independent, so the
search is linear in the number of satellites.

### Topology construction and routing

| Satellites | Per satellite | Exponent |
| --- | --- | --- |
| 8 | 178 ns | — |
| 64 | 1.6 us | 1.06 |
| 120 | 4.7 us | 1.70 |
| 300 | 33.3 us | 2.14 |
| **600** | **159.7 us** | 2.26 |

**Roughly O(n squared).** The cost per satellite grows with the count, the
exponent rising from one to just over two. The cause is inherent: the
geometry test is an all pairs check, and n satellites have n squared pairs to
judge. The parallel scan confirms where the time goes.

### Task planning

| Tasks | Per task | Exponent |
| --- | --- | --- |
| 60 | 4.6 us | — |
| 200 | 14.6 us | 0.96 |
| 1000 | 111.6 us | 1.27 |
| **2000** | **305.7 us** | 1.45 |

**Roughly O(n squared), easing towards O(n) at size.** A greedy placement
arranges tasks in time order and each arrangement is compared against the
placements already made, so the quadratic is the algorithm rather than a
defect in the implementation.

## Result three: threads

### Propagation, split by satellite

| Satellites | 1 thread | 2 | 4 | 8 | 16 | Best |
| --- | --- | --- | --- | --- | --- | --- |
| 550 | 182 ms | 1.91x | 3.84x | 7.37x | 10.08x | 10.08x at 16 |
| 1000 | 332 ms | 1.95x | 3.87x | 7.38x | 10.15x | 10.15x at 16 |
| 2000 | 669 ms | 1.97x | 3.85x | 7.42x | 9.87x | 9.87x at 16 |

**Propagation is embarrassingly parallel.** Every satellite is independent,
and two to eight threads come close to linear. Efficiency at sixteen threads
falls to about sixty percent, which is the contention of running two threads
per core. Eight threads is the better setting per core, about ninety-two
percent efficiency; sixteen still gives the lowest wall time.

### Access search

| Tracks | 1 thread | 4 | 8 | 16 | Best |
| --- | --- | --- | --- | --- | --- |
| 64 | 0.41 ms | 1.78x | 2.61x | 1.73x | 2.61x at 8 |
| 256 | 1.65 ms | 3.43x | 4.36x | 4.91x | 4.91x at 16 |

**The gain is small on a small batch**, because the fixed cost of starting
threads dominates when there is little work. At 256 tracks it reaches about
4.9x.

### Topology construction, split by phase

| Satellites | Visibility, parallelisable | Edge fold, serial tail | Best speedup |
| --- | --- | --- | --- |
| 120 | 0.15 ms | 0.21 ms | 1.16x |
| 300 | 0.78 ms | 3.85 ms | 1.33x |

**Threading helps this path only a little.** At 300 satellites the serial
edge fold takes about five times the parallelisable part, so the total gains
at most 1.3x rather than the near-linear gain of the numerical kernels. The
cause is that `add_edge` scans the edges already stored to reject a
duplicate, which makes the insertion itself cubic.

## Conclusions

| Path | Complexity | Threads | At useful scale |
| --- | --- | --- | --- |
| SGP4 propagation | O(1) a sample | excellent, about 10x on sixteen | 2016 satellites a day in 0.68 s, 68 ms on sixteen |
| Ephemeris query | O(log n) | follows propagation | 27 ns |
| Access search | O(1) a satellite | good, about 5x on sixteen | 256 tracks in 0.34 ms |
| Coordinate and link | O(1) | not worth it | 24 to 60 ns |
| Task planning | about O(n squared) | serial | 2000 tasks in 0.61 s |
| Topology construction | about O(n squared) | limited, a serial tail dominates | 600 satellites in 96 ms |

**The points that matter**

1. The numerical kernels are linear and parallelise well. A day over two
   thousand satellites costs 0.68 seconds on one core and 68 ms on sixteen.
2. Interpolating the ephemeris is about ten times cheaper than propagating,
   so a densely queried track should be tabulated once.
3. Two combined operations are limited by their algorithm rather than by the
   available cores. Greedy planning is quadratically serial, and the
   duplicate check on edge insertion squares that quadratic; the second gains
   only about 1.3x from threading. Both would need a different algorithm to
   gain more.
4. Eight threads is the better setting per core for propagation. Efficiency
   is about ninety-two percent, while sixteen threads falls to about sixty
   percent because two threads share a core.

## Reproducing

```bash
# The gates first, so that what is measured is the right code.
cargo test --workspace --all-features

# The three measurements.
cargo run --release --features full --example bench
cargo run --release --features full --example bench_scale
cargo run --release --features full --example bench_parallel
```

Do not run other work while measuring. The absolute values move with the
machine, so the conclusions rest on the ratios within one tool.
