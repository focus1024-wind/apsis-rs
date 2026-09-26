//! The access window type and its summary.

use apsis_math::Vec3;

/// One continuous interval during which a satellite is above a station's
/// elevation mask.
///
/// Times are seconds from an origin chosen by the caller, so the same
/// structure serves a search over a scenario clock and one over elapsed
/// time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pass {
    /// The time the satellite rises above the mask, in seconds from the
    /// caller's origin.
    pub rise_seconds: f64,
    /// The time the satellite falls below the mask.
    pub set_seconds: f64,
    /// The time of greatest elevation.
    pub peak_seconds: f64,
    /// The greatest elevation during the pass, in radians.
    pub peak_elevation: f64,
    /// The elevation at which the satellite crossed the mask, in radians.
    ///
    /// This is the station's mask, recorded so that a pass carries the
    /// geometry it was found under.
    pub mask_elevation: f64,
    /// The satellite position in the Earth-fixed frame at the peak, in
    /// kilometers.
    pub peak_position: Vec3,
}

impl Pass {
    /// Returns the duration of the pass, in seconds.
    pub fn duration(&self) -> f64 {
        self.set_seconds - self.rise_seconds
    }

    /// Returns the time from the rise to the peak, in seconds.
    pub fn rise_to_peak(&self) -> f64 {
        self.peak_seconds - self.rise_seconds
    }

    /// Returns the elevation at a time within the pass, when the caller can
    /// supply it.
    ///
    /// The pass stores only the peak, so this helper reports the stored
    /// values at the three defining times and defers to the caller's
    /// elevation function otherwise.
    pub fn contains(&self, seconds: f64) -> bool {
        seconds >= self.rise_seconds && seconds <= self.set_seconds
    }

    /// Returns the maximum elevation of the pass, in radians.
    ///
    /// This is the same as the peak elevation, named for readability at a
    /// call site that compares passes.
    pub fn maximum_elevation(&self) -> f64 {
        self.peak_elevation
    }
}

/// A summary of the access windows over an interval.
#[derive(Debug, Clone, PartialEq)]
pub struct PassSummary {
    /// The passes, in time order.
    pub passes: Vec<Pass>,
    /// The length of the interval searched, in seconds.
    pub interval_seconds: f64,
}

impl PassSummary {
    /// Creates a summary from a list of passes and the interval length.
    pub fn new(passes: Vec<Pass>, interval_seconds: f64) -> Self {
        Self {
            passes,
            interval_seconds,
        }
    }

    /// Returns the number of passes.
    pub fn count(&self) -> usize {
        self.passes.len()
    }

    /// Returns the total access time, in seconds.
    ///
    /// The passes are disjoint, so the durations add.
    pub fn total_access_seconds(&self) -> f64 {
        self.passes.iter().map(Pass::duration).sum()
    }

    /// Returns the fraction of the interval during which the satellite was
    /// accessible.
    ///
    /// Returns `None` when the interval length is not positive.
    pub fn access_fraction(&self) -> Option<f64> {
        if self.interval_seconds > 0.0 && self.interval_seconds.is_finite() {
            Some(self.total_access_seconds() / self.interval_seconds)
        } else {
            None
        }
    }

    /// Returns the mean pass duration, in seconds.
    ///
    /// Returns `None` when there are no passes.
    pub fn mean_duration_seconds(&self) -> Option<f64> {
        if self.passes.is_empty() {
            return None;
        }
        Some(self.total_access_seconds() / self.passes.len() as f64)
    }

    /// Returns the longest pass, if there is one.
    pub fn longest(&self) -> Option<&Pass> {
        self.passes.iter().max_by(|first, second| {
            first
                .duration()
                .partial_cmp(&second.duration())
                .unwrap_or(core::cmp::Ordering::Equal)
        })
    }

    /// Returns the highest pass, if there is one.
    pub fn highest(&self) -> Option<&Pass> {
        self.passes.iter().max_by(|first, second| {
            first
                .peak_elevation
                .partial_cmp(&second.peak_elevation)
                .unwrap_or(core::cmp::Ordering::Equal)
        })
    }

    /// Returns the gap between two consecutive passes, in seconds.
    ///
    /// Returns `None` when either index is out of range or the two indices
    /// are not consecutive and increasing.
    pub fn gap_seconds(&self, first: usize, second: usize) -> Option<f64> {
        if second != first + 1 {
            return None;
        }
        let earlier = self.passes.get(first)?;
        let later = self.passes.get(second)?;
        Some(later.rise_seconds - earlier.set_seconds)
    }

    /// Returns the largest gap between consecutive passes, in seconds.
    ///
    /// This is the longest time a station waits between contacts.
    /// Returns `None` when there are fewer than two passes.
    pub fn longest_gap_seconds(&self) -> Option<f64> {
        if self.passes.len() < 2 {
            return None;
        }
        self.passes
            .windows(2)
            .map(|pair| pair[1].rise_seconds - pair[0].set_seconds)
            .max_by(|first, second| {
                first
                    .partial_cmp(second)
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
    }
}
