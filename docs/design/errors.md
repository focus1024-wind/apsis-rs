# Errors and tolerances

English | [简体中文](./errors.zh.md)

This file records how errors are expressed across modules and how numerical
tolerances are chosen. The error enumeration of a single module is documented
in its `error.rs`.

## Expressing an error

### A domain error

Every module with a domain specific failure defines **its own** error type,
named and shaped consistently:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FooErrorKind {
    /// Each variant names a concrete condition a caller may want to tell apart.
    ReversedInterval,
    DuplicateResource,
    // ...
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FooError {
    kind: FooErrorKind,
    detail: Option<String>,
}

impl FooError {
    pub fn new(kind: FooErrorKind) -> Self;
    pub fn with_detail(self, detail: impl Into<String>) -> Self;
    pub fn kind(&self) -> FooErrorKind;
    pub fn detail(&self) -> Option<&str>;
}

impl fmt::Display for FooError { /* "kind", or "kind: detail" */ }
impl core::error::Error for FooError {}
```

The conventions:

- **An `ErrorKind` is marked `#[non_exhaustive]`**, so that adding a failure
  category does not break a match downstream. All fifteen `ErrorKind`
  enumerations carry the attribute.
- **An error enum that carries data is marked too**, including one whose
  variants wrap a lower error, such as `apsis_orbit`'s
  `EphemerisSamplingError`.
- **A state enum that is not an error is not marked.** A domain state such as
  `TaskState`, `LinkState`, `TimeScale`, `HandoverDecision` or the antenna
  `Pattern` gains from being matchable exhaustively inside a single
  repository: when a variant is added, the compiler points at every `match`
  that needs updating. The reasoning is the opposite of an error's, because
  one more error category should let a caller fall through to a default
  branch, while one more state usually has to be handled explicitly. The test
  is whether a caller will `match` the enum and take a default on some
  branches: if so, mark it; if not, leave it.
- **`kind` is matched, `detail` is read.** A caller matches on `kind` to
  write recovery logic and logs `detail`. Because `detail` takes no part in
  matching, its wording may change freely.
- **`Display` prints the kind**, followed by `": detail"` when there is one,
  so a log line always begins with the category.
- **`core::error::Error` is implemented**, so an error can nest as the
  `source` of another. All fifteen error types implement it.
- **A bare `String` or `Box<dyn Error>` is never a public error.** The first
  cannot be matched and the second loses the category. A private helper may
  pass a `String` as a reason internally, but the public boundary is an enum.
  The `precedence_floor` in `apsis_scheduler` is an instance: its private
  `String` ends up inside the public `TaskState::Infeasible { reason }`.

### The general error

`apsis_core::Error` and `ErrorKind` are a general purpose fallback carrying
a coarse category such as `InvalidInput` or `NotFound`. They are **not** the
error surface of the layers: no layer uses them, because a coarse category
loses the distinction a caller needs. `InvalidInput` cannot tell a reversed
interval from a duplicate identifier.

They are kept for two reasons: to give a function whose failure has no
interesting structure a ready return type, and to give the crate one
extensible fallback shape. New code whose failure has a domain meaning should
define a domain error.

### Failure against the absence of a value

- **`Option`** means that having no value is an ordinary outcome rather than
  a failure, such as an instant outside the covered interval, which
  `Ephemeris::state_at` reports as `None`, or a tangent that is undefined at
  an angle.
- **`Result`** means an operation failed for a diagnosable reason, such as a
  reversed interval, a duplicate resource, or a value outside the domain the
  model covers.

The test is whether the caller needs to know **why**. If it only needs to
know that there is nothing, use `Option`; if it needs to tell the failures
apart and recover, use `Result`.

## Numerical tolerances

### Convergence of an iteration

An iterative solve, whether of Kepler's equation, a nutation series or a ray
trace, is governed by a private constant inside its module:

| Module | Constant | Value | Meaning |
| --- | --- | --- | --- |
| `apsis_kepler` | `TOLERANCE` | `1e-14` | the step of the Newton iteration for the eccentric anomaly |
| `apsis_sgp4` | `KEPLER_TOLERANCE` | `1e-12` | the residual of the Kepler equation iteration |

These values trade floating point precision against iteration count: they sit
close to the machine precision of an `f64` while still guaranteeing that the
iteration terminates, each with its own `MAX_ITERATIONS` bound.

### Tolerances of the cross-checks

A cross-check sets its tolerance by the **physical quantity** rather than by
one number for everything:

| Quantity | Tolerance |
| --- | --- |
| Position | < 1e-3 km |
| Velocity | < 1e-6 km/s |
| Time | < 1e-6 s |
| Coverage angle | 1e-12 rad |
| Coverage area and fraction | 1e-12 relative |

A tolerance sits far above the disagreement actually observed, which is
usually at the floating point level, and far below the accuracy of the model
itself. That way it catches a mistyped radius or a missing unit conversion
without reporting on the last bit of a floating point value. The table in
`validation/README.md` lists them per script.

### Comparing exactly

A tolerance should be avoided where it is not needed. An integer index, a
count or a flag is compared for equality. An angle reduction such as
`wrap_to_pi` uses a half-open interval at the boundary rather than an
epsilon. A floating point comparison is for a quantity that is genuinely a
continuous approximation.

## expect in library code

The crate convention is that library code uses no `unwrap`, `expect` or
`panic!`. The **intent** of the rule is that they must not handle a failure
on external input, which has to become a `Result` or an `Option` the caller
can act on.

Allowing no `expect` at all would push the code towards `unwrap`, which is
worse, or into long hand written branches. So one case is permitted:
**an `expect` on an internal invariant that a check has already guarded.**
Such an assertion does not turn untrusted data into a panic; it turns "if
this were empty it would be a bug in this module" into a readable failure
message.

The test is whether a guard making the branch unreachable sits on the line
above, or whether the value was just built by this function. All thirteen
current sites satisfy it. The table groups them by module; the `apsis_kepler`
and `apsis_constellation` rows each cover three sites:

| Site | Invariant | Guard |
| --- | --- | --- |
| `apsis_math/parallel.rs` | every slot was filled by one thread | the loop assigns each index exactly once |
| `apsis_schedule/schedule.rs` | the placement just pushed exists | the line above |
| `apsis_scheduler/improve.rs` | the value whose `is_none()` was just checked is not none | the `is_none()` branch above |
| `apsis_scheduler/planner.rs` | an identifier in the order is in the task list | the order is built from the task list |
| `apsis_topology/series.rs` | the time is finite | validated on construction |
| `apsis_handover/decision.rs` | the candidate list is not empty | the `validate` call above |
| `apsis_constellation/walker.rs` | the plane count and the satellites per plane are positive | the divisor is validated before |
| `apsis_kepler/elements.rs` | the node, the radius and the angular momentum are non-zero | the `norm() == 0.0` branch just above |
| `apsis_event/queue.rs` | a popped entry has its payload | guaranteed by the construction order of the queue |

Removing a guard means the matching `expect` must become a `Result`; the two
change together.

## Checks

- Each module's tests cover every `kind` of its error, including the path
  that constructs it and the path that recovers from it.
- A cross-check reports through its exit code: zero to pass, one to exceed
  the tolerance, two for a missing input.
- `cargo clippy -- -D warnings` keeps an `unwrap` or `expect` from reaching
  library code unnoticed.
