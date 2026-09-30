# Layer boundaries

English | [简体中文](./layers.zh.md)

This file records the layering rule of the workspace and why certain
cross-layer dependencies are allowed. The "Layers" section of the README is
the rule itself; what belongs here is how it is decided and where it has edge
cases.

## The rule

The layers run left to right:

```text
foundation -> physics -> orbit -> rf -> network -> ops -> sim -> apsis-rs
```

A dependency edge runs from crate A to crate B. Map the layer of A and the
layer of B to indices, and the rule is:

```text
index(B) <= index(A)     allowed
index(B) >  index(A)     a violation, a reverse dependency
```

In other words, **dependencies go to the left and never to the right.**
`scripts/check-layers.sh` applies this to every manifest in the workspace,
and CI runs it.

## Crossing layers is allowed

The rule constrains direction, not distance. `network` may depend on `orbit`
even though `rf` sits between them, because `orbit` is to the left of
`network` and the direction is correct.

Allowing a leftward crossing is deliberate. Forbidding it would create two
kinds of problem:

- **There may be no type to carry the value.** When a position from the
  `orbit` layer is handed to the `network` layer, the `rf` layer need not
  offer a type that carries it. Requiring the trip through `rf` would only
  produce a shell type that forwards.
- **It would be coupling without need.** The constellation calculation in
  `network` wants the Keplerian elements from `apsis-kepler` and has no use
  for `rf`. Forbidding the crossing would make `network` depend on `rf` only
  to reach a type it does not use.

The rule therefore constrains **direction**, because direction is what
carries the meaning of layering: a concept of a higher layer may refer to a
concept of a lower one, and the other way round would mean the layering is
wrong.

## The global baselines

`apsis-core` (the general error, the constants, the unit marker) and
`apsis-math` (`Vec3`, `Mat3`, `Quat`) are shared by every layer. They sit in
`foundation` themselves, so "to the left only" already lets any layer use
them and no special case is needed.

This differs from a within-layer baseline. `apsis-core` is also the baseline
inside `foundation`, but its reach is the whole workspace rather than one
layer.

**Declare a dependency only where it is used.** Any crate may use
`apsis-core` and `apsis-math`, but a crate that does not reference them
should not list them: an unused dependency slows the build, adds noise to
`cargo tree` and to the layer check, and misstates who depends on what.
Adding one later is a single line whenever it is needed. The edge count that
`scripts/check-layers.sh` reports is therefore a real usage relation rather
than a list of intentions.

## The facade within a layer

Each layer has one facade crate as its integration point:

| Layer | Facade |
| --- | --- |
| orbit | `apsis-orbit` |
| rf | `apsis-rf` |
| sim | `apsis-sim` |
| outward | `apsis-rs` |

A facade combines the capabilities of its layer into a few types and
re-exports the remaining crates of the same layer. `foundation`, `physics`,
`network` and `ops` have no separate facade: their crates are each useful on
their own, and `apsis-rs` re-exports them together.

## Edge cases

- **The `apsis-rs` facade is not in the layer sequence.** It depends on
  every layer, because it is the exit. `scripts/check-layers.sh` decides by
  the layer directory name, and `crates/apsis-rs` does not have the shape
  `crates/<layer>/<crate>`, so it is skipped.
- **Dev-dependencies count.** A reverse dependency in a test shows the
  layering is just as wrong there, so the fix is to change the code rather
  than relax the test.
- **A facade re-export is not a new dependency.** `apsis-rs` re-exporting
  `apsis-orbit` is what a facade does, not a layer reaching into `orbit`.

## Checks

```bash
bash scripts/check-layers.sh
```

The script prints the number of dependency edges it checked, and lists each
reverse dependency before exiting with status one if it found any.
