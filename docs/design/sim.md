# The simulation layer

English | [简体中文](./sim.zh.md)

This file records the cross-crate decisions of the simulation layer. The
design inside each crate is in its module documentation.

## Composition and dependencies

```text
apsis-event      the event type and the event queue
apsis-sim-engine the discrete event engine
apsis-scenario   scenario definition and text loading
apsis-output     simulation output recording
apsis-sim        the facade, which joins the four above
```

The dependency direction:

```text
apsis-event -> apsis-sim-engine -> apsis-sim -> apsis-rs
apsis-scenario -----------------> apsis-sim
apsis-output -------------------> apsis-sim
```

`apsis-sim` also depends on the ops layer (`apsis-scheduler`,
`apsis-schedule`, `apsis-task`, `apsis-resource`), because it plans a
scenario and reads the plan. The remaining crates depend on foundation alone.

## The order of events

Several events at the same instant must be delivered in a reproducible
order, or the same input would give a different output and there would be
nothing to cross-check against a reference implementation. The convention:

- Every event takes an increasing sequence number when it is scheduled.
- Events are ordered by time, and events at the same time by that number,
  which is the order they were scheduled in.
- An event that a handler schedules while handling another goes behind the
  events already queued for that instant.

`apsis-event` provides this order and the engine and the facade carry it
through. The queue assigns the numbers, so a caller cannot create two events
with the same one.

## The origin of time

An event time is a number of seconds from the origin of the simulation and
has nothing to do with a calendar; a scenario that needs one converts it
above this layer.

The `epoch` of a scenario is where the clock starts. The times in a timeline
are relative to that origin and `apsis-sim` adds the `epoch` before
delivering. Without the offset the engine would move the clock backwards when
it delivered the first event.

A time limit is compared on the same absolute axis: an event later than the
limit stays in the queue instead of running, so a caller can see what
remained when the run ended. The clock stops at the last event it executed
rather than at the limit.

## The event limit is cumulative

The event limit in `apsis-sim-engine` bounds the total for the life of the
engine, not the count of a single `run`. Once the limit is reached a further
`run` executes nothing. The limit is therefore a bound that cannot be
circumvented, and a caller who wants the remaining events to run has to
raise it explicitly.

## What a timeline means

`apsis-sim` expands a plan into start and end events on a timeline: every
placed task contributes one start and one end, as separate events, so the
engine can order them in time with everything else and the order of
overlapping tasks follows from the clock.

The timeline is the plan, not a physical model. What physical computation a
handler runs when a start or end event fires is up to the handler; the
facade only orders and records.

## The scenario text format

`apsis-scenario` carries its own line oriented text format with no
dependency, for storing a scenario and loading it back. Parsing runs in three
passes, over the resources, the tasks and the precedence constraints, so a
`require` may name a resource defined later in the file and a `precedence`
may name tasks in any order. Every failure carries the line it came from.

The output of `Scenario::to_text` must parse back with `Scenario::parse` to
an equal scenario, so the text form is stable under a round trip, which a
test asserts.

A task with no window, whose two ends are infinite, cannot be written as
text, so it is written as a finite interval that contains the task and is
still usable once read back.

## Determinism

A simulation result must be reproducible, because that is what makes a
cross-check possible. Three things give it:

- the same-instant ordering rule of the event queue;
- the stable ordering the planner applies to tasks of equal weight, described
  in the ops layer;
- the recorder, which keeps the order of insertion, where
  `Trace::in_time_order` is a stable sort.

The `integrate_sim` example asserts that running one prepared simulation
twice produces the same trace.
