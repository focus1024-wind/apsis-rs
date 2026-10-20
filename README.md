# ApsisRS

Computational kernels for commercial spaceflight and satellite internet, in Rust.

English | [简体中文](./README.zh.md)

## Scope

- Pure Rust, standard library only, no third-party crate dependency
- Clear domain boundaries with a one-way dependency order between layers
- One crate, `apsis`; each layer is a module re-exported at the crate root
- The crate is ready for its first crates.io release

## Layers

```text
foundation -> physics -> orbit -> rf -> network -> ops -> sim
```

The rules:

- References flow to the left only. A lower layer may use any layer to its
  left; a higher layer may not use any layer to its right. So `network` may
  use `orbit`, but `orbit` may not use `network`. A leftward reference may
  cross as many layers as it needs, because the layers in between need not
  offer a type that carries the value forward.
- `apsis_core` in `foundation` is the global baseline that any module in any
  layer may use. `apsis_math` is the same, because linear algebra is shared
  across the layers.
- Within a layer, that layer's facade module (`apsis_orbit`, `apsis_rf`,
  `apsis_sim`) is the integration point.

`scripts/check-layers.sh` enforces these rules and CI runs it.

| Layer | Responsibility | Modules |
| --- | --- | --- |
| foundation | Base types, units, mathematics, macros | `apsis_core`, `apsis_units`, `apsis_math`, `apsis_macros` |
| physics | Time, leap seconds, coordinates, datum | `apsis_time`, `apsis_leap`, `apsis_coord`, `apsis_geodesy`, `apsis_frames` |
| orbit | TLE, SGP4, two-body, ephemeris, maneuvers, mission sequences, EOIR optics, conjunction assessment | `apsis_tle`, `apsis_sgp4`, `apsis_kepler`, `apsis_ephemeris`, `apsis_maneuver`, `apsis_orbit`, `apsis_mission`, `apsis_eoir`, `apsis_cat` |
| rf | Radio frequency, antennas, atmosphere, aircraft performance, link budget | `apsis_rf`, `apsis_antenna`, `apsis_doppler`, `apsis_atmosphere`, `apsis_aviator`, `apsis_noise`, `apsis_interference`, `apsis_link` |
| network | Constellations, coverage, access, topology, routing | `apsis_constellation`, `apsis_coverage`, `apsis_access`, `apsis_topology`, `apsis_isl`, `apsis_routing`, `apsis_handover`, `apsis_mesh` |
| ops | Resources, tasks, scheduling, conflicts, schedules | `apsis_resource`, `apsis_task`, `apsis_scheduler`, `apsis_conflict`, `apsis_schedule` |
| sim | Simulation engine, scenarios, events, output | `apsis_sim_engine`, `apsis_scenario`, `apsis_event`, `apsis_output`, `apsis_sim` |
| facade | Crate root re-exports | the crate root, `apsis` |

## Building

Run every Rust command from the repository root:

```bash
cargo check --workspace
cargo test --workspace --all-features
```

The crate selects the layers it needs by feature:

```toml
[dependencies]
apsis = { path = "crates/apsis", features = ["orbit"] }
```

The features are `foundation`, `physics`, `orbit`, `rf`, `network`, `ops`,
`sim` and `full`. A layer feature enables the layers below it. `full` enables
everything including `sim`; `ops` is every layer except `sim`, for a project
that does not run simulations.

## Performance

The three benchmarks are examples of the crate. They run in release mode and
depend on nothing outside the workspace, so a build is the compiler and
nothing else. The figures below are one run on an AMD Ryzen 7 8845HS (eight
cores, sixteen threads); absolute times move with the machine, so compare the
ratios within a table. The method, every size in the sweeps and the reasoning
behind each conclusion are in
[Benchmark method and results](docs/design/bench.md).

**One call on the hot paths**

| Path | Per call |
| --- | --- |
| Hermite ephemeris query | 27 ns |
| TEME to Earth-fixed rotation | 24 ns |
| Keplerian elements to a state | 47 ns |
| Earth-fixed to geodetic | 50 ns |
| Link budget C/N0 | 60 ns |
| SGP4 propagation, thirty minutes from epoch | 265 ns |
| Access search, 120 samples | 6.4 us |
| Planning 60 tasks on one resource | 240 us |

An ephemeris query is an order of magnitude cheaper than the propagation it
stands in for, which is why a densely queried track is tabulated once and then
interpolated.

**Growth with size.** A flat cost per unit is what shows the complexity is
right:

- A day of a constellation at a sixty second step is linear in the satellite
  count: 24 satellites in 8.1 ms, 2016 in 0.68 s, about 235 ns a sample, an
  exponent of 0.00.
- Access search is flat at about 7 us a track from 8 to 128 tracks.
- Topology construction is about O(n squared) and greedy planning eases from
  O(n squared) towards O(n); both are limited by their algorithm, and the
  design note says why.

**Threads.** The numerical kernels split by satellite. Propagating 2000
satellites over a day takes 669 ms on one thread and 68 ms on sixteen, a
speedup of 9.9x; access search over 256 tracks reaches 4.9x. Efficiency is
about ninety-two percent at eight threads, one per core; sixteen puts two on
a core and lowers it to about sixty percent. Every parallel result equals the serial one,
so threading buys throughput and never changes a number.

## Conventions

- Toolchain `stable`, edition 2024, rust-version 1.85
- No third-party crate by default, including dev-dependencies and build scripts
- Library code uses no `unwrap`, `expect` or `panic`. A failure on external
  input is always a `Result` or an `Option`. An `expect` is allowed on an
  internal invariant that a check just above already guards, such as the
  element just pushed onto a vector, or a value whose absence the line above
  ruled out. There are thirteen such sites and each is listed with its guard in
  `docs/design/errors.md`.
- Every public item carries a documentation comment, and the comment is
  self-contained
- Tests live in `crates/apsis/tests/`, mirroring the layer and module
  structure under `src/`. They are integration tests and reach the crate only
  through its public API, so a re-export missing at the crate root fails here
- Documentation follows the `X.md` and `X.zh.md` convention, with English in
  `X.md` and Simplified Chinese in `X.zh.md`

## Layout

```text
apsis-rs/
├── Cargo.toml
├── crates/
│   └── apsis/              # the single crate
│       ├── build.rs
│       ├── examples/       # validation dumps and cross-layer checks
│       ├── src/
│       │   ├── lib.rs      # feature-gated re-exports
│       │   ├── foundation/ # one directory per layer
│       │   ├── physics/
│       │   ├── orbit/
│       │   ├── rf/
│       │   ├── network/
│       │   ├── ops/
│       │   └── sim/
│       └── tests/          # integration tests, mirroring the layers
├── scripts/                # repository check scripts
├── validation/             # Python cross-checks
└── docs/design/            # design notes
```

Tests live in `crates/apsis/tests/`, in a single integration target named
`integration`. The cross-layer integration checks and the dependency-free
benchmarks are examples of the crate:

```bash
# Cross-layer integration checks, which CI runs
cargo run --features network --example integrate_network
cargo run --features ops --example integrate_ops
cargo run --features sim --example integrate_sim

# Hot path benchmark, run in release
cargo run --release --features full --example bench

# Scale sweep: the input grows and the cost per unit of work shows the trend
cargo run --release --features full --example bench_scale

# Parallel sweep: the same work over 1, 2, 4, 8 and 16 threads
cargo run --release --features full --example bench_parallel
```

## Documentation

The design notes record the cross-layer decisions that a reader of a single
module cannot infer:

- [Units and coordinates](docs/design/units.md)
- [Errors and tolerances](docs/design/errors.md)
- [Layer boundaries](docs/design/layers.md)
- [Parallelism](docs/design/parallel.md)
- [Benchmark method and results](docs/design/bench.md)
- [Glossary and formula conventions](docs/design/glossary.md)
- [The simulation layer](docs/design/sim.md)

The generated API documentation renders mathematical formulas offline. A copy
of MathJax is vendored under `docs/vendor/mathjax`, `docs/mathjax.html` loads
it, and `crates/apsis/build.rs` copies it into the documentation root, so
the generated pages need no network access. Build them with
`bash scripts/build-docs.sh --workspace --all-features --no-deps`; a plain
`cargo doc` also works, and the script only makes the copy reliable. A display
formula is written as `$$ ... $$` inside a `<div>`; the surrounding div keeps
the markdown layer from rewriting the LaTeX.

## Licence

GPL-3.0-or-later, see `LICENSE`.

The vendored MathJax under `docs/vendor/mathjax` is Apache-2.0; see the
`LICENSE` in that directory.
