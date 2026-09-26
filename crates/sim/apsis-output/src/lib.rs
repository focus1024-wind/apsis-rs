//! Simulation output recording.
//!
//! A simulation produces more output than a caller can act on at once: event
//! times, the values a model computes as it runs, and the warnings it raises.
//! This crate collects those into one trace, so that a run can be examined
//! after it finishes rather than only as it happens.
//!
//! A record has a time, a kind, a label, and a value. The time is the
//! simulation instant the record was made at, which is what makes a trace
//! orderable and joinable against the event log. The kind separates the three
//! things a caller usually wants to tell apart: an observation the model made,
//! a state change it decided on, and a warning it wants to surface. The label
//! names what the value is about, and the value is a single number, which
//! covers the great majority of what a simulation records without imposing a
//! schema the caller has to design.
//!
//! The trace keeps the records in the order they were made, not in time order.
//! That distinction matters: a handler may record a value for an event it is
//! handling after recording one for a follow-up it just scheduled, and the
//! order the model reasoned in is worth keeping. A caller that wants time
//! order asks for it with [`Trace::in_time_order`].
//!
//! ```
//! use apsis_output::{Recorder, RecordKind};
//!
//! let mut recorder = Recorder::new();
//! recorder.observe(10.0, "altitude", 550.0);
//! recorder.note(10.0, "eclipse", 1.0);
//!
//! assert_eq!(recorder.len(), 2);
//! assert!(recorder.trace().is_stable());
//! assert_eq!(recorder.trace().value_of("altitude"), Some(550.0));
//! let _ = RecordKind::Warning;
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod record;
mod recorder;
mod trace;

pub use record::{Record, RecordKind};
pub use recorder::Recorder;
pub use trace::{SeriesSummary, Trace};
