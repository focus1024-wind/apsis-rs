//! The access window finder.

use crate::error::{AccessError, AccessErrorKind};
use crate::pass::{Pass, PassSummary};
use apsis_coverage::{GroundPoint, elevation_angle};
use apsis_math::Vec3;

/// One sample of a satellite track, as the finder consumes it.
///
/// The position is in the Earth-fixed frame, in kilometers, and the time is
/// in seconds from the caller's origin. The caller supplies a sequence of
/// these, which decouples the finder from the propagation and the frame
/// transformation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrbitSample {
    /// The time of the sample, in seconds from the caller's origin.
    pub seconds: f64,
    /// The satellite position in the Earth-fixed frame, in kilometers.
    pub position_ecef_km: Vec3,
}

impl OrbitSample {
    /// Creates a sample from a time and a position.
    pub const fn new(seconds: f64, position_ecef_km: Vec3) -> Self {
        Self {
            seconds,
            position_ecef_km,
        }
    }

    /// Returns `true` when the sample is usable.
    pub fn is_valid(&self) -> bool {
        self.seconds.is_finite() && self.position_ecef_km.is_finite()
    }
}

/// Finds access windows for one station.
///
/// The finder holds the station and the numerical settings of the search.
/// The settings are separated from the search so that the same finder can be
/// applied to many satellites.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccessFinder {
    /// The station and its elevation mask.
    pub station: GroundPoint,
    /// The interval between samples of the satellite track, in seconds.
    ///
    /// A smaller step finds shorter passes but costs proportionally more
    /// evaluations. A step of one minute resolves any pass a low Earth
    /// satellite makes, since such a pass lasts several minutes.
    pub sample_step_seconds: f64,
    /// The tolerance on a crossing time, in seconds.
    ///
    /// The refinement of a rise or set continues until the interval
    /// containing the crossing is this narrow.
    pub crossing_tolerance_seconds: f64,
}

impl AccessFinder {
    /// Returns `true` when the finder settings are usable.
    pub fn is_valid(&self) -> bool {
        self.station.is_valid()
            && self.sample_step_seconds.is_finite()
            && self.sample_step_seconds > 0.0
            && self.crossing_tolerance_seconds.is_finite()
            && self.crossing_tolerance_seconds > 0.0
            && self.crossing_tolerance_seconds <= self.sample_step_seconds
    }

    /// Validates the settings, returning the reason when they are not
    /// usable.
    fn validate(&self) -> Result<(), AccessError> {
        if !self.station.is_valid() {
            return Err(AccessError::new(AccessErrorKind::InvalidStation));
        }
        if !self.sample_step_seconds.is_finite() || self.sample_step_seconds <= 0.0 {
            return Err(AccessError::new(AccessErrorKind::InvalidSampleStep)
                .with_detail(format!("step is {}", self.sample_step_seconds)));
        }
        if !self.crossing_tolerance_seconds.is_finite() || self.crossing_tolerance_seconds <= 0.0 {
            return Err(AccessError::new(AccessErrorKind::InvalidTolerance)
                .with_detail(format!("tolerance is {}", self.crossing_tolerance_seconds)));
        }
        if self.crossing_tolerance_seconds > self.sample_step_seconds {
            return Err(AccessError::new(AccessErrorKind::InvalidTolerance)
                .with_detail("the tolerance exceeds the sampling step"));
        }
        Ok(())
    }

    /// Returns the elevation of a sample, in radians.
    ///
    /// Returns `None` when the sample or the station is not usable.
    pub fn elevation_of(&self, sample: &OrbitSample) -> Option<f64> {
        if !sample.is_valid() {
            return None;
        }
        elevation_angle(sample.position_ecef_km, &self.station)
    }

    /// Finds the access windows in an ordered sequence of samples.
    ///
    /// The samples must be ordered by time and cover the interval of
    /// interest. The finder assumes the elevation is continuous between
    /// samples, so the step must be short enough that the satellite does not
    /// rise and set within one step. A low Earth satellite is above a
    /// ten degree mask for several minutes, so a step of a minute or less
    /// satisfies that.
    ///
    /// The crossing times are found by interpolating the satellite position
    /// linearly between the two samples that straddle the crossing. The
    /// chord between two samples departs from the true arc by an amount of
    /// the second order in the step, so a ten second step gives a crossing
    /// accurate to about a tenth of a second and a one second step to about
    /// a millisecond. Use [`AccessFinder::find_with`] when the propagator is
    /// available and the crossing is wanted to the full tolerance.
    ///
    /// A pass that is already in progress at the first sample is reported
    /// with its rise at the first sample time, and one still in progress at
    /// the last sample is reported with its set at the last sample time.
    /// That convention means the pass is clipped to the interval searched
    /// rather than extrapolated beyond it.
    ///
    /// Returns an error when the settings are not usable, when the sample
    /// sequence has fewer than two entries, when a sample is not usable, or
    /// when the sequence is not in increasing time order. The order is
    /// checked rather than assumed, because the interpolation between two
    /// samples binary searches the sequence and a binary search over an
    /// unordered list returns a wrong index without failing.
    pub fn find(&self, samples: &[OrbitSample]) -> Result<PassSummary, AccessError> {
        self.find_internal(samples, |seconds| {
            // Interpolate the position between the bracketing samples. The
            // order of the sequence is checked by `find_internal` before
            // this closure runs, and the step is the caller's to choose.
            let index = match samples.binary_search_by(|sample| {
                sample
                    .seconds
                    .partial_cmp(&seconds)
                    .unwrap_or(core::cmp::Ordering::Equal)
            }) {
                Ok(exact) => exact,
                Err(insert) => insert.min(samples.len() - 1),
            };
            let upper = index.max(1);
            let lower = upper - 1;
            let span = samples[upper].seconds - samples[lower].seconds;
            let fraction = if span == 0.0 {
                0.0
            } else {
                (seconds - samples[lower].seconds) / span
            };
            let position = samples[lower]
                .position_ecef_km
                .lerp(samples[upper].position_ecef_km, fraction);
            self.elevation_of(&OrbitSample::new(seconds, position))
        })
    }

    /// Runs the search given a function that returns the elevation at a
    /// time.
    ///
    /// The function must be defined over the whole interval and must be
    /// continuous, and the sample sequence must be ordered by time. The
    /// closure returns `None` when the position at that time is not usable.
    fn find_internal<F>(
        &self,
        samples: &[OrbitSample],
        mut elevation_at: F,
    ) -> Result<PassSummary, AccessError>
    where
        F: FnMut(f64) -> Option<f64>,
    {
        self.validate()?;
        if samples.len() < 2 {
            return Err(AccessError::new(AccessErrorKind::InvalidInterval)
                .with_detail("at least two samples are needed"));
        }

        // The whole sequence must be in strictly increasing time, not just its
        // ends. The interpolation between two samples binary searches this
        // list, and a binary search over an unordered list returns a wrong
        // index rather than failing, so an unordered sequence would refine a
        // crossing against the wrong pair of positions and report a pass that
        // never happened. Checking only the ends, as this once did, let that
        // through silently.
        for (index, pair) in samples.windows(2).enumerate() {
            // The test is written out rather than as `!(a < b)` so that it
            // also rejects a pair whose times are not finite, and so that a
            // reader can see that equality and disorder are both refused.
            let ordered = pair[0].seconds.is_finite()
                && pair[1].seconds.is_finite()
                && pair[0].seconds < pair[1].seconds;
            if !ordered {
                return Err(
                    AccessError::new(AccessErrorKind::UnorderedSamples).with_detail(format!(
                        "sample {} at {} s does not precede {} s",
                        index + 1,
                        pair[0].seconds,
                        pair[1].seconds
                    )),
                );
            }
        }

        let first = samples[0].seconds;
        let last = samples[samples.len() - 1].seconds;
        if !first.is_finite() || !last.is_finite() {
            return Err(AccessError::new(AccessErrorKind::InvalidInterval)
                .with_detail(format!("times run from {first} to {last}")));
        }

        let mask = self.station.elevation_mask;
        let mut elevations = Vec::with_capacity(samples.len());
        for sample in samples {
            let elevation = elevation_at(sample.seconds).ok_or_else(|| {
                AccessError::new(AccessErrorKind::InvalidPosition)
                    .with_detail(format!("sample at {} s is not usable", sample.seconds))
            })?;
            if !elevation.is_finite() {
                return Err(AccessError::new(AccessErrorKind::InvalidPosition)
                    .with_detail(format!("elevation at {} s is not finite", sample.seconds)));
            }
            elevations.push(elevation);
        }

        let mut passes = Vec::new();
        let mut rise: Option<f64> = None;
        // Track the highest elevation seen inside the current pass so that
        // the peak is known without a second pass over the samples.
        let mut peak = f64::NEG_INFINITY;
        let mut peak_seconds = first;
        let mut peak_position = samples[0].position_ecef_km;

        for index in 0..samples.len() {
            let above = elevations[index] >= mask;
            if above {
                if rise.is_none() {
                    rise = Some(if index == 0 {
                        // The pass was already open at the start of the
                        // window, so it is clipped to the first sample.
                        samples[0].seconds
                    } else {
                        self.refine_crossing(samples, index, true, &mut elevation_at)
                    });
                }
                if elevations[index] > peak {
                    peak = elevations[index];
                    peak_seconds = samples[index].seconds;
                    peak_position = samples[index].position_ecef_km;
                }
            } else if let Some(rise_seconds) = rise.take() {
                let set_seconds = self.refine_crossing(samples, index, false, &mut elevation_at);
                passes.push(Pass {
                    rise_seconds,
                    set_seconds,
                    peak_seconds,
                    peak_elevation: peak,
                    mask_elevation: mask,
                    peak_position,
                });
                peak = f64::NEG_INFINITY;
            }
        }

        // A pass still open at the end of the sequence is clipped to the
        // last sample.
        if let Some(rise_seconds) = rise {
            passes.push(Pass {
                rise_seconds,
                set_seconds: last,
                peak_seconds,
                peak_elevation: peak,
                mask_elevation: mask,
                peak_position,
            });
        }

        Ok(PassSummary::new(passes, last - first))
    }

    /// Refines the time at which the elevation crosses the mask.
    ///
    /// The crossing lies between the sample at `index` and the one before
    /// it, and the two straddle the mask. Bisection halves the interval
    /// until it is narrower than the tolerance.
    ///
    /// When `rising` is true the crossing is a rise, where the elevation
    /// increases through the mask; otherwise it is a set. The flag selects
    /// which side of the crossing is above the mask, which keeps the search
    /// bracket correct.
    fn refine_crossing<F>(
        &self,
        samples: &[OrbitSample],
        index: usize,
        rising: bool,
        elevation_at: &mut F,
    ) -> f64
    where
        F: FnMut(f64) -> Option<f64>,
    {
        let mask = self.station.elevation_mask;
        let mut low = samples[index - 1].seconds;
        let mut high = samples[index].seconds;

        // The elevation difference from the mask, with the sign arranged so
        // that the function is negative at `low` and positive at `high`.
        let sign = if rising { 1.0 } else { -1.0 };
        let mut difference = |seconds: f64| {
            let elevation = elevation_at(seconds).unwrap_or(mask);
            sign * (elevation - mask)
        };

        let mut low_value = difference(low);
        for _ in 0..64 {
            if high - low <= self.crossing_tolerance_seconds {
                break;
            }
            let middle = 0.5 * (low + high);
            let value = difference(middle);
            if value == 0.0 {
                return middle;
            }
            if (value > 0.0) == (low_value > 0.0) {
                low = middle;
                low_value = value;
            } else {
                high = middle;
            }
        }
        0.5 * (low + high)
    }

    /// Finds the access windows over an interval, calling a propagator for
    /// each sample.
    ///
    /// The propagator receives a time in seconds from the origin of the
    /// search and returns the satellite position in the Earth-fixed frame.
    /// This adapts the finder to any orbit source without this crate
    /// depending on one.
    ///
    /// Returns an error when the settings are not usable, when the interval
    /// is not ordered, or when the propagator fails at a sample.
    pub fn find_with<F>(
        &self,
        start_seconds: f64,
        end_seconds: f64,
        mut propagator: F,
    ) -> Result<PassSummary, AccessError>
    where
        F: FnMut(f64) -> Option<Vec3>,
    {
        self.validate()?;
        if !start_seconds.is_finite() || !end_seconds.is_finite() || end_seconds <= start_seconds {
            return Err(
                AccessError::new(AccessErrorKind::InvalidInterval).with_detail(format!(
                    "interval runs from {start_seconds} to {end_seconds}"
                )),
            );
        }

        // Build the sample sequence, including the end point.
        let step = self.sample_step_seconds;
        let count = ((end_seconds - start_seconds) / step).ceil() as usize + 1;
        let mut samples = Vec::with_capacity(count);
        for index in 0..count {
            let seconds = (start_seconds + index as f64 * step).min(end_seconds);
            let position = propagator(seconds).ok_or_else(|| {
                AccessError::new(AccessErrorKind::InvalidPosition)
                    .with_detail(format!("the propagator failed at {seconds} s"))
            })?;
            samples.push(OrbitSample::new(seconds, position));
        }
        // The loop above may append the end point twice when the interval is
        // an exact multiple of the step, so the last sample is replaced when
        // it duplicates the end.
        if let Some(last) = samples.last_mut() {
            if last.seconds != end_seconds {
                let position = propagator(end_seconds).ok_or_else(|| {
                    AccessError::new(AccessErrorKind::InvalidPosition)
                        .with_detail(format!("the propagator failed at {end_seconds} s"))
                })?;
                samples.push(OrbitSample::new(end_seconds, position));
            } else {
                last.seconds = end_seconds;
            }
        }

        // The refinement calls the propagator at each bisection point, so
        // the crossing is found to the tolerance rather than to the
        // accuracy of a chord between two samples.
        let mut failure = None;
        let summary = self.find_internal(&samples, |seconds| match propagator(seconds) {
            Some(position) if position.is_finite() => elevation_angle(position, &self.station),
            _ => {
                failure = Some(seconds);
                None
            }
        });
        match summary {
            Ok(summary) => Ok(summary),
            Err(error) => Err(match failure {
                Some(seconds) => error.with_detail(format!("the propagator failed at {seconds} s")),
                None => error,
            }),
        }
    }
}
