# Design notes

English | [简体中文](./README.zh.md)

This directory records the cross-crate design decisions of ApsisRS, such as
the coordinate and unit conventions, the error and tolerance strategy, the
layer boundaries and the parallelism. The design inside a single crate is
documented in that crate's module documentation.

A note is self-contained and refers only to code in this repository or to a
public standard.

## Cross-layer conventions

- [Units and coordinates](units.md)
- [Errors and tolerances](errors.md)
- [Layer boundaries](layers.md)
- [Parallelism](parallel.md)

## Per-layer design

- [The simulation layer](sim.md)
