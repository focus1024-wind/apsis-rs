//! Running a prepared simulation.

use crate::activity::Activity;
use crate::simulation::Simulation;
use apsis_output::{RecordKind, Recorder, Trace};
use apsis_sim_engine::{Engine, EngineOutcome};

/// What a run produced.
///
/// The report pairs the engine's own outcome with the output trace, so a
/// caller has everything a run produced without reaching back into the
/// session.
#[derive(Debug, Clone, PartialEq)]
pub struct RunReport {
    /// What the engine did.
    pub outcome: EngineOutcome,
    /// The records the run made.
    pub trace: Trace,
    /// The number of records the recorder refused because they were not
    /// usable.
    pub refused: u64,
}

impl RunReport {
    /// Returns the number of activities the run executed.
    pub const fn executed(&self) -> u64 {
        self.outcome.executed
    }

    /// Returns `true` when the run finished because the timeline was
    /// exhausted rather than because a limit stopped it.
    pub fn is_complete(&self) -> bool {
        self.outcome.is_natural()
    }

    /// Returns the number of warnings recorded.
    pub fn warning_count(&self) -> usize {
        self.trace.warning_count()
    }
}

/// One run of a prepared simulation.
///
/// A session owns the engine and the recorder for a single run, so it can be
/// driven from one place without the caller holding either. It is created by
/// [`Simulation::start`].
#[derive(Debug, Clone)]
pub struct Session {
    engine: Engine<Activity>,
    recorder: Recorder,
}

impl Session {
    /// Builds a session from a prepared simulation.
    pub(crate) fn new(simulation: &Simulation) -> Self {
        let scenario = simulation.scenario();
        let mut engine = Engine::new()
            .with_time_limit(scenario.time_limit().unwrap_or(f64::INFINITY))
            .with_event_limit(scenario.event_limit().unwrap_or(0));
        engine.set_now(scenario.epoch());
        // The timeline is relative to the scenario origin, so the epoch is
        // added to every event time. Without the offset the engine would
        // move the clock backwards when it delivered the first event.
        let epoch = scenario.epoch();
        for (time, activity) in simulation.timeline() {
            engine.schedule(epoch + time, *activity);
        }
        Self {
            engine,
            recorder: Recorder::new(),
        }
    }

    /// Returns the engine, for a caller that needs its state between calls.
    pub const fn engine(&self) -> &Engine<Activity> {
        &self.engine
    }

    /// Returns the engine for modification.
    pub const fn engine_mut(&mut self) -> &mut Engine<Activity> {
        &mut self.engine
    }

    /// Returns the recorder, for a caller that reads the trace as it grows.
    pub const fn recorder(&self) -> &Recorder {
        &self.recorder
    }

    /// Returns the recorder for modification.
    pub const fn recorder_mut(&mut self) -> &mut Recorder {
        &mut self.recorder
    }

    /// Returns the current simulation time, in seconds.
    pub const fn now(&self) -> f64 {
        self.engine.now()
    }

    /// Runs the engine, calling the handler at each activity.
    ///
    /// The handler receives the engine, the recorder, and the activity. It
    /// normally records something for the activity and may schedule further
    /// events; the engine delivers them in time order alongside the timeline.
    ///
    /// Returns a report holding the engine outcome and the trace made during
    /// this call. The recorder is left empty for the next call.
    pub fn run<F>(&mut self, mut handler: F) -> RunReport
    where
        F: FnMut(&mut Engine<Activity>, &mut Recorder, apsis_event::Event<Activity>),
    {
        let outcome = self.engine.run(|engine, event| {
            handler(engine, &mut self.recorder, event);
        });
        RunReport {
            outcome,
            trace: self.recorder.take_trace(),
            refused: self.recorder.refused(),
        }
    }

    /// Records an activity as a state change without running anything.
    ///
    /// This is for a caller that drives the engine itself and only wants the
    /// recorder.
    pub fn note(&mut self, time: f64, label: impl Into<String>, value: f64) -> bool {
        self.recorder.note(time, label, value)
    }

    /// Records a value of a given kind without running anything.
    pub fn record(
        &mut self,
        time: f64,
        kind: RecordKind,
        label: impl Into<String>,
        value: f64,
    ) -> bool {
        self.recorder.record(time, kind, label, value)
    }
}
