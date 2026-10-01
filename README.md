# ApsisRS

Computational kernels for commercial spaceflight and satellite internet, in Rust.

English | [简体中文](./README.zh.md)

## Scope

- Pure Rust, standard library only, no third-party crate dependency
- Clear domain boundaries with a one-way dependency order between layers
- The public facade crate is `apsis-rs`; its library name is `apsis`
- Every crate is `publish = false` at this stage, so none of them is released

## Layers

```text
foundation -> physics -> orbit -> rf -> network -> ops -> sim -> apsis-rs
```

The rules:

- Dependencies flow to the left only. A lower layer may use any layer to its
  left; a higher layer may not use any layer to its right. So `network` may
  use `orbit`, but `orbit` may not use `network`. A leftward dependency may
  cross as many layers as it needs, because the layers in between need not
  offer a type that carries the value forward.
- `apsis-core` in `foundation` is the global baseline that any crate in any
  layer may use. `apsis-math` is the same, because linear algebra is shared
  across the layers.
- Within a layer, that layer's facade crate (`apsis-orbit`, `apsis-rf`,
  `apsis-sim`) is the integration point.

`scripts/check-layers.sh` enforces these rules and CI runs it.

| Layer | Responsibility | Crates |
| --- | --- | --- |
| foundation | Base types, units, mathematics, macros | `apsis-core`, `apsis-units`, `apsis-math`, `apsis-macros` |
| physics | Time, leap seconds, coordinates, datum | `apsis-time`, `apsis-leap`, `apsis-coord`, `apsis-geodesy`, `apsis-frames` |
| orbit | TLE, SGP4, two-body, ephemeris, maneuvers | `apsis-tle`, `apsis-sgp4`, `apsis-kepler`, `apsis-ephemeris`, `apsis-maneuver`, `apsis-orbit` |
| rf | Radio frequency, antennas, link budget | `apsis-rf`, `apsis-antenna`, `apsis-doppler`, `apsis-atmosphere`, `apsis-noise`, `apsis-interference`, `apsis-link` |
| network | Constellations, coverage, access, topology, routing | `apsis-constellation`, `apsis-coverage`, `apsis-access`, `apsis-topology`, `apsis-isl`, `apsis-routing`, `apsis-handover`, `apsis-mesh` |
| ops | Resources, tasks, scheduling, conflicts, schedules | `apsis-resource`, `apsis-task`, `apsis-scheduler`, `apsis-conflict`, `apsis-schedule` |
| sim | Simulation engine, scenarios, events, output | `apsis-sim-engine`, `apsis-scenario`, `apsis-event`, `apsis-output`, `apsis-sim` |
| facade | Public facade | `apsis-rs` |

## Building

Run every Rust command from the repository root:

```bash
cargo check --workspace
cargo test -p apsis-core -p apsis-units -p apsis-math
```

The `apsis-rs` facade selects the layers it needs by feature:

```toml
[dependencies]
apsis-rs = { path = "...", features = ["orbit"] }
```

The features are `foundation`, `physics`, `orbit`, `rf`, `network`, `ops`,
`sim` and `full`. A layer feature enables the layers below it. `full` enables
everything including `sim`; `ops` is every layer except `sim`, for a project
that does not run simulations.

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
- Tests live in each crate's `tests/` directory, separate from `src/`. They are
  integration tests and reach the crate only through its public API; no inline
  `#[cfg(test)]` module is used
- Documentation follows the `X.md` and `X.zh.md` convention, with English in
  `X.md` and Simplified Chinese in `X.zh.md`

## Layout

```text
apsis-rs/
├── Cargo.toml
├── crates/          # Rust workspace members
│   └── apsis-rs/
│       ├── examples/   # validation dumps and cross-layer checks
│       └── ...
├── scripts/         # repository check scripts
├── validation/      # Python cross-checks
└── docs/design/     # design notes
```

Tests live in each crate's `tests/` directory. The cross-layer integration
checks and the dependency-free benchmarks are examples of the facade:

```bash
# Cross-layer integration checks, which CI runs
cargo run -p apsis-rs --features network --example integrate_network
cargo run -p apsis-rs --features ops --example integrate_ops
cargo run -p apsis-rs --features sim --example integrate_sim

# Hot path benchmark, run in release
cargo run --release -p apsis-rs --features full --example bench

# Scale sweep: the input grows and the cost per unit of work shows the trend
cargo run --release -p apsis-rs --features full --example bench_scale

# Parallel sweep: the same work over 1, 2, 4, 8 and 16 threads
cargo run --release -p apsis-rs --features full --example bench_parallel
```

## Documentation

The design notes record the cross-crate decisions that a reader of a single
crate cannot infer:

- [Units and coordinates](docs/design/units.md)
- [Errors and tolerances](docs/design/errors.md)
- [Layer boundaries](docs/design/layers.md)
- [Parallelism](docs/design/parallel.md)
- [Glossary and formula conventions](docs/design/glossary.md)
- [The simulation layer](docs/design/sim.md)

## Licence

GPL-3.0-or-later, see `LICENSE`.
