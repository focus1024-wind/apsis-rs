//! Batch propagation over many instants, optionally across several threads.
//!
//! A single satellite's track is the common case for a coverage run or an
//! access search: the caller wants the state at every minute of a day, for
//! every satellite in a shell. Each state is independent of the others, so
//! the loop is a candidate for several cores, and the work is large enough
//! that starting a thread pays for itself.
//!
//! The parallel entry point produces the same values as the sequential one,
//! in the same order. It differs only in how the work is distributed.

use crate::{Satellite, State};
use apsis_math::parallel::map_index;

/// Propagates a satellite at a list of minutes from its epoch, in order.
///
/// The sequential form, provided for comparison and for small lists where
/// starting a thread would cost more than the work.
pub fn propagate_minutes_at(satellite: &Satellite, minutes: &[f64]) -> Vec<Option<State>> {
    minutes
        .iter()
        .map(|&value| satellite.propagate_minutes(value).ok())
        .collect()
}

/// Propagates a satellite at a list of minutes from its epoch, across several
/// threads.
///
/// The result matches [`propagate_minutes_at`] element for element and in the
/// same order. A minute at which the model reports a decay yields `None`, as
/// it does in the sequential form; the threads do not stop at the first such
/// minute, because a caller usually wants the whole track and will decide for
/// itself what a decay means.
///
/// Propagation is deterministic and depends only on the minutes, so the
/// result does not depend on how the work was split.
pub fn propagate_minutes_at_parallel(satellite: &Satellite, minutes: &[f64]) -> Vec<Option<State>> {
    map_index(minutes.len(), |index| {
        satellite.propagate_minutes(minutes[index]).ok()
    })
}

/// Propagates several satellites at one instant, in order.
///
/// The per satellite work is independent, so a shell's state at a single
/// instant is a natural batch. The satellites are read only, so they are
/// shared across the threads by reference.
pub fn propagate_at(satellites: &[Satellite], tai_mjd: f64) -> Vec<Option<State>> {
    satellites
        .iter()
        .map(|satellite| satellite.propagate_at(tai_mjd).ok())
        .collect()
}

/// Propagates several satellites at one instant, across several threads.
///
/// The result matches [`propagate_at`] element for element and in the same
/// order.
pub fn propagate_at_parallel(satellites: &[Satellite], tai_mjd: f64) -> Vec<Option<State>> {
    map_index(satellites.len(), |index| {
        satellites[index].propagate_at(tai_mjd).ok()
    })
}
