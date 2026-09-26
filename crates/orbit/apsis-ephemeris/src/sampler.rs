//! Filling an ephemeris by calling a state function on a grid.

use crate::ephemeris::{Ephemeris, Sample};
use crate::error::{EphemerisError, EphemerisErrorKind};
use apsis_math::Vec3;

/// A uniform time grid that a state function is evaluated on.
///
/// A sampler describes when to sample and how finely. The grid is defined by
/// a start time, an end time, and a step, and the number of samples follows
/// from them: the samples are placed at the start and at every step up to
/// the end, with the end itself included whether or not it falls on the
/// grid. Including the end matters, because a caller that asks for coverage
/// up to a time expects the table to reach it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sampler {
    /// The time of the first sample, in seconds.
    pub start_seconds: f64,
    /// The time of the last sample, in seconds.
    pub end_seconds: f64,
    /// The spacing between samples, in seconds.
    pub step_seconds: f64,
}

impl Sampler {
    /// Creates a sampler over an interval with a step.
    pub const fn new(start_seconds: f64, end_seconds: f64, step_seconds: f64) -> Self {
        Self {
            start_seconds,
            end_seconds,
            step_seconds,
        }
    }

    /// Returns `true` when the sampler describes a usable grid.
    ///
    /// The times must be finite, the end must be strictly after the start,
    /// and the step must be finite and positive. A zero length interval is
    /// refused because a table needs at least two distinct times and the
    /// interpolation has no interval to work over.
    pub fn is_valid(&self) -> bool {
        self.start_seconds.is_finite()
            && self.end_seconds.is_finite()
            && self.step_seconds.is_finite()
            && self.step_seconds > 0.0
            && self.end_seconds > self.start_seconds
    }

    /// Returns the number of samples the grid produces.
    ///
    /// This is the number of whole steps that fit in the interval plus one
    /// for the start and one for the end when the end is off the grid.
    /// Returns `None` when the sampler is not usable.
    pub fn sample_count(&self) -> Option<usize> {
        if !self.is_valid() {
            return None;
        }
        let span = self.end_seconds - self.start_seconds;
        let whole_steps = (span / self.step_seconds).floor();
        // The end sample is added unless it already coincides with the last
        // whole step within a rounding tolerance.
        let remainder = span - whole_steps * self.step_seconds;
        let extra = if remainder > self.step_seconds * 1e-9 {
            1
        } else {
            0
        };
        Some(whole_steps as usize + 1 + extra)
    }

    /// Returns the times of the grid, in order.
    ///
    /// Returns `None` when the sampler is not usable. The last time is the
    /// end when the end does not fall on the grid, and the last grid point
    /// otherwise.
    pub fn times(&self) -> Option<Vec<f64>> {
        let count = self.sample_count()?;
        let mut times = Vec::with_capacity(count);
        for index in 0..count {
            let time = self.start_seconds + index as f64 * self.step_seconds;
            if time >= self.end_seconds {
                break;
            }
            times.push(time);
        }
        // The end is always included, whether or not it landed on the grid.
        if times.last().copied() != Some(self.end_seconds) {
            times.push(self.end_seconds);
        }
        Some(times)
    }

    /// Fills an ephemeris by evaluating a state function on the grid.
    ///
    /// The function maps a time in seconds to a position and velocity. In
    /// this workspace it is normally a closure over
    /// `apsis_sgp4::Satellite::propagate_at`, but any function works, so
    /// the sampler can be tested against an analytic orbit.
    ///
    /// Returns an error when the sampler is not usable or when the function
    /// produces a state that is not finite.
    pub fn sample<F>(&self, mut state_at: F) -> Result<Ephemeris, EphemerisError>
    where
        F: FnMut(f64) -> Sample,
    {
        if !self.is_valid() {
            return Err(
                EphemerisError::new(EphemerisErrorKind::NonPositiveStep).with_detail(format!(
                    "start {}, end {}, step {}",
                    self.start_seconds, self.end_seconds, self.step_seconds
                )),
            );
        }
        let times = self
            .times()
            .ok_or_else(|| EphemerisError::new(EphemerisErrorKind::NotFinite))?;
        let mut samples = Vec::with_capacity(times.len());
        for time in times {
            let sample = state_at(time);
            if !sample.is_valid() {
                return Err(EphemerisError::new(EphemerisErrorKind::InvalidState)
                    .with_detail(format!("at {time} s")));
            }
            // The time is set from the grid rather than trusted from the
            // function, so the table stays strictly ordered even if the
            // function returns a slightly different time.
            samples.push(Sample::new(time, sample.position, sample.velocity));
        }
        Ephemeris::from_samples(samples)
    }

    /// Fills an ephemeris from a function that returns only a position.
    ///
    /// The velocity is reconstructed from neighbouring positions by a
    /// central difference. This is for a position-only source, such as a
    /// tabulated file; where a velocity is available it should be used, so
    /// prefer [`Sampler::sample`].
    pub fn sample_positions<F>(&self, mut position_at: F) -> Result<Ephemeris, EphemerisError>
    where
        F: FnMut(f64) -> Vec3,
    {
        if !self.is_valid() {
            return Err(EphemerisError::new(EphemerisErrorKind::NonPositiveStep));
        }
        let times = self
            .times()
            .ok_or_else(|| EphemerisError::new(EphemerisErrorKind::NotFinite))?;
        let mut positions = Vec::with_capacity(times.len());
        for &time in &times {
            let position = position_at(time);
            if !position.is_finite() {
                return Err(EphemerisError::new(EphemerisErrorKind::InvalidState)
                    .with_detail(format!("at {time} s")));
            }
            positions.push(position);
        }

        // A central difference inside the interval and a one sided one at
        // each end. The finite difference is second order, which matches the
        // accuracy of the Hermite interpolation it feeds.
        let count = positions.len();
        let mut samples = Vec::with_capacity(count);
        for (index, &position) in positions.iter().enumerate() {
            let velocity = if count == 1 {
                Vec3::new(0.0, 0.0, 0.0)
            } else if index == 0 {
                (positions[1] - positions[0]) / (times[1] - times[0])
            } else if index == count - 1 {
                (positions[count - 1] - positions[count - 2])
                    / (times[count - 1] - times[count - 2])
            } else {
                (positions[index + 1] - positions[index - 1])
                    / (times[index + 1] - times[index - 1])
            };
            samples.push(Sample::new(times[index], position, velocity));
        }
        Ephemeris::from_samples(samples)
    }
}
