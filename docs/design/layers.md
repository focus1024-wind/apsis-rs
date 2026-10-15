# Layer boundaries

English | [简体中文](./layers.zh.md)

This file records the layering rule of the crate and why certain cross-layer
references are allowed. The "Layers" section of the README is the rule
itself; what belongs here is how it is decided and where it has edge cases.

## The rule

The layers run left to right:

```text
foundation -> physics -> orbit -> rf -> network -> ops -> sim
```

A reference edge runs from module A to module B. Map the layer of A and the
layer of B to indices, and the rule is:

```text
index(B) <= index(A)     allowed
index(B) >  index(A)     a violation, a reverse reference
```

In other words, **references go to the left and never to the right.**
`scripts/check-layers.sh` applies this to every module directory of the
crate, and CI runs it.

## Crossing layers is allowed

The rule constrains direction, not distance. `network` may reference `orbit`
even though `rf` sits between them, because `orbit` is to the left of
`network` and the direction is correct.

Allowing a leftward crossing is deliberate. Forbidding it would create two
kinds of problem:

- **There may be no type to carry the value.** When a position from the
  `orbit` layer is handed to the `network` layer, the `rf` layer need not
  offer a type that carries it. Requiring the trip through `rf` would only
  produce a shell type that forwards.
- **It would be coupling without need.** The constellation calculation in
  `network` wants the Keplerian elements from `apsis_kepler` and has no use
  for `rf`. Forbidding the crossing would make `network` reference `rf` only
  to reach a type it does not use.

The rule therefore constrains **direction**, because direction is what
carries the meaning of layering: a concept of a higher layer may refer to a
concept of a lower one, and the other way round would mean the layering is
wrong.

## The global baselines

`apsis_core` (the general error, the constants, the unit marker) and
`apsis_math` (`Vec3`, `Mat3`, `Quat`) are shared by every layer. They sit in
`foundation` themselves, so "to the left only" already lets any layer use
them and no special case is needed.

This differs from a within-layer baseline. `apsis_core` is also the baseline
inside `foundation`, but its reach is the whole crate rather than one layer.

**Reference a module only where it is used.** Any module may reference
`apsis_core` and `apsis_math`, but a module that does not use them should not
import them: an unused import slows the build, adds noise to the layer check,
and misstates who depends on what. Adding one later is a single line whenever
it is needed. The edge count that `scripts/check-layers.sh` reports is
therefore a real usage relation rather than a list of intentions.

## The facade within a layer

Each layer has one facade module as its integration point:

| Layer | Facade |
| --- | --- |
| orbit | `apsis_orbit` |
| rf | `apsis_rf` |
| sim | `apsis_sim` |
| outward | the crate root, `apsis` |

A facade combines the capabilities of its layer into a few types and
re-exports the remaining modules of the same layer. `foundation`, `physics`,
`network` and `ops` have no separate facade: their modules are each useful on
their own, and the crate root re-exports them together.

## Edge cases

- **The crate root is not in the layer sequence.** It re-exports every
  layer, because it is the exit. `scripts/check-layers.sh` decides by the
  layer directory name, and `src/lib.rs` does not have the shape
  `src/<layer>/<module>`, so it is skipped.
- **Tests count.** A reverse reference in a test shows the layering is just
  as wrong there, so the fix is to change the code rather than relax the
  test.
- **A re-export is not a new reference.** The crate root re-exporting
  `apsis_orbit` is what a facade does, not a layer reaching into `orbit`.

## Checks

```bash
bash scripts/check-layers.sh
```

The script prints the number of reference edges it checked, and lists each
reverse reference before exiting with status one if it found any.
