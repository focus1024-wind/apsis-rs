//! The ephemeris table and its Hermite interpolation.

use crate::error::{EphemerisError, EphemerisErrorKind};
use apsis_math::Vec3;

/// One sampled state.
///
/// A sample is a position and velocity at a time, which is the form every
/// propagator in the workspace produces and the form the interpolation
/// needs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    /// The time of the sample, in seconds from the caller's origin.
    pub seconds: f64,
    /// Position, in kilometers.
    pub position: Vec3,
    /// Velocity, in kilometers per second.
    pub velocity: Vec3,
}

impl Sample {
    /// Creates a sample.
    pub const fn new(seconds: f64, position: Vec3, velocity: Vec3) -> Self {
        Self {
            seconds,
            position,
            velocity,
        }
    }

    /// Returns `true` when the time, position, and velocity are finite.
    pub fn is_valid(&self) -> bool {
        self.seconds.is_finite() && self.position.is_finite() && self.velocity.is_finite()
    }

    /// Returns the distance from the origin, in kilometers.
    pub fn radius(&self) -> f64 {
        self.position.norm()
    }

    /// Returns the speed, in kilometers per second.
    pub fn speed(&self) -> f64 {
        self.velocity.norm()
    }
}

/// Evaluates the Hermite cubic for one component.
///
/// `p0` and `p1` are the endpoint values, `v0` and `v1` the endpoint
/// derivatives with respect to time, `dt` the interval length, and `s` the
/// fraction along the interval in `[0, 1]`.
///
/// The four terms are the standard cubic Hermite basis: the first and third
/// carry the endpoints, the second and fourth the slopes. The derivative
/// terms are scaled by `dt` because the slopes are per unit time while `s`
/// is per unit interval.
pub fn hermite_position(p0: Vec3, v0: Vec3, p1: Vec3, v1: Vec3, dt: f64, s: f64) -> Vec3 {
    let s2 = s * s;
    let s3 = s2 * s;
    let h00 = 2.0 * s3 - 3.0 * s2 + 1.0;
    let h10 = s3 - 2.0 * s2 + s;
    let h01 = -2.0 * s3 + 3.0 * s2;
    let h11 = s3 - s2;
    p0 * h00 + v0 * (h10 * dt) + p1 * h01 + v1 * (h11 * dt)
}

/// Evaluates the derivative of the Hermite cubic for one component.
///
/// This is the velocity that goes with [`hermite_position`]. It is the exact
/// derivative of that cubic, so the reconstructed state is internally
/// consistent: a caller that differentiates the position gets this velocity.
pub fn hermite_velocity(p0: Vec3, v0: Vec3, p1: Vec3, v1: Vec3, dt: f64, s: f64) -> Vec3 {
    let s2 = s * s;
    let dh00 = 6.0 * s2 - 6.0 * s;
    let dh10 = 3.0 * s2 - 4.0 * s + 1.0;
    let dh01 = -6.0 * s2 + 6.0 * s;
    let dh11 = 3.0 * s2 - 2.0 * s;
    // The chain rule turns the per interval derivative into a per second
    // one, which divides the position terms by `dt` and leaves the velocity
    // terms, already per second, untouched.
    p0 * (dh00 / dt) + v0 * dh10 + p1 * (dh01 / dt) + v1 * dh11
}

/// Evaluates both the Hermite position and velocity.
pub fn hermite_state(p0: Vec3, v0: Vec3, p1: Vec3, v1: Vec3, dt: f64, s: f64) -> (Vec3, Vec3) {
    (
        hermite_position(p0, v0, p1, v1, dt, s),
        hermite_velocity(p0, v0, p1, v1, dt, s),
    )
}

/// A table of sampled states that answers queries by interpolation.
///
/// The table holds at least two samples in strictly increasing time order.
/// A query at a sample time returns that sample exactly; a query between two
/// samples returns the Hermite cubic through the four endpoint values. A
/// query outside the covered interval is refused rather than extrapolated,
/// because the interpolation has no information there and an extrapolated
/// orbit can diverge quickly.
#[derive(Debug, Clone, PartialEq)]
pub struct Ephemeris {
    samples: Vec<Sample>,
}

impl Ephemeris {
    /// Builds a table from samples in increasing time order.
    ///
    /// Returns an error when there are fewer than two samples, when a sample
    /// is not finite, or when the times do not strictly increase.
    pub fn from_samples(samples: Vec<Sample>) -> Result<Self, EphemerisError> {
        if samples.len() < 2 {
            return Err(EphemerisError::new(EphemerisErrorKind::TooFewSamples)
                .with_detail(format!("{} sample(s)", samples.len())));
        }
        for sample in &samples {
            if !sample.is_valid() {
                return Err(EphemerisError::new(EphemerisErrorKind::InvalidState)
                    .with_detail(format!("at {} s", sample.seconds)));
            }
        }
        for pair in samples.windows(2) {
            if pair[1].seconds <= pair[0].seconds {
                return Err(
                    EphemerisError::new(EphemerisErrorKind::NotMonotonic).with_detail(format!(
                        "{} s follows {} s",
                        pair[1].seconds, pair[0].seconds
                    )),
                );
            }
        }
        Ok(Self { samples })
    }

    /// Returns the samples in time order.
    pub fn samples(&self) -> &[Sample] {
        &self.samples
    }

    /// Returns the number of samples.
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Returns `true` when the table is empty.
    ///
    /// A table built through [`Ephemeris::from_samples`] is never empty, so
    /// this is always `false` for one that exists; it is provided so that
    /// generic code that accepts a slice can use the usual pair of methods.
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Returns the first sample time, in seconds.
    pub fn start_time(&self) -> f64 {
        self.samples.first().map_or(0.0, |sample| sample.seconds)
    }

    /// Returns the last sample time, in seconds.
    pub fn end_time(&self) -> f64 {
        self.samples.last().map_or(0.0, |sample| sample.seconds)
    }

    /// Returns the time the table covers, in seconds.
    pub fn duration(&self) -> f64 {
        self.end_time() - self.start_time()
    }

    /// Returns `true` when a time falls within the covered interval.
    ///
    /// The interval is closed, so its endpoints are covered.
    pub fn covers(&self, seconds: f64) -> bool {
        seconds.is_finite() && seconds >= self.start_time() && seconds <= self.end_time()
    }

    /// Returns the nominal step, when the samples are evenly spaced.
    ///
    /// Returns `None` when the spacing varies, which a caller checks before
    /// indexing samples by arithmetic.
    pub fn uniform_step(&self) -> Option<f64> {
        let first = self.samples.windows(2).next()?[1].seconds - self.samples[0].seconds;
        let uniform = self
            .samples
            .windows(2)
            .all(|pair| (pair[1].seconds - pair[0].seconds - first).abs() <= 1e-9);
        uniform.then_some(first)
    }

    /// Returns `true` when the samples are evenly spaced.
    pub fn is_uniform(&self) -> bool {
        self.uniform_step().is_some()
    }

    /// Returns the index of the interval that contains a time.
    ///
    /// The index `i` is such that `samples[i].seconds <= seconds <=
    /// samples[i + 1].seconds`. A time outside the interval returns `None`.
    fn interval_of(&self, seconds: f64) -> Option<usize> {
        if !self.covers(seconds) {
            return None;
        }
        // A binary search for the last sample at or before the time. The
        // partition point is the first sample strictly after it, so the
        // interval starts one before, clamped to the last interval.
        let after = self
            .samples
            .partition_point(|sample| sample.seconds <= seconds);
        let index = after.saturating_sub(1);
        Some(index.min(self.samples.len() - 2))
    }

    /// Returns the state at a time, interpolating between samples.
    ///
    /// A time equal to a sample time returns that sample. Returns `None`
    /// when the time is outside the covered interval.
    pub fn state_at(&self, seconds: f64) -> Option<Sample> {
        let index = self.interval_of(seconds)?;
        let first = self.samples[index];
        let second = self.samples[index + 1];

        // At a sample the interpolation must reproduce the sample exactly,
        // which avoids the round off of evaluating the cubic at its end.
        if seconds == first.seconds {
            return Some(first);
        }
        if seconds == second.seconds {
            return Some(second);
        }

        let dt = second.seconds - first.seconds;
        let s = (seconds - first.seconds) / dt;
        let (position, velocity) = hermite_state(
            first.position,
            first.velocity,
            second.position,
            second.velocity,
            dt,
            s,
        );
        Some(Sample::new(seconds, position, velocity))
    }

    /// Returns the state at a time, or the nearest endpoint when the time is
    /// outside the covered interval.
    ///
    /// This is for a caller that must produce a value for any time and is
    /// content to hold the last known state rather than fail.
    pub fn clamped_state_at(&self, seconds: f64) -> Option<Sample> {
        if self.covers(seconds) {
            return self.state_at(seconds);
        }
        if !seconds.is_finite() {
            return None;
        }
        if seconds < self.start_time() {
            self.samples.first().copied()
        } else {
            self.samples.last().copied()
        }
    }

    /// Returns the sample nearest to a time, without interpolating.
    pub fn nearest_sample(&self, seconds: f64) -> Option<Sample> {
        if !seconds.is_finite() {
            return None;
        }
        let index = match self.interval_of(seconds) {
            Some(index) => {
                let first = self.samples[index];
                let second = self.samples[index + 1];
                if (seconds - first.seconds).abs() <= (second.seconds - seconds).abs() {
                    index
                } else {
                    index + 1
                }
            }
            None if seconds < self.start_time() => 0,
            None => self.samples.len() - 1,
        };
        Some(self.samples[index])
    }

    /// Returns the times of every sample, in order.
    pub fn times(&self) -> Vec<f64> {
        self.samples.iter().map(|sample| sample.seconds).collect()
    }

    /// Returns `true` when every state satisfies a predicate.
    ///
    /// This is a convenience for a caller that wants to check a property of
    /// the whole table, such as every radius being above the surface, once.
    pub fn all_samples<F>(&self, predicate: F) -> bool
    where
        F: Fn(&Sample) -> bool,
    {
        self.samples.iter().all(predicate)
    }
}
