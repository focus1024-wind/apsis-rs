//! The handover policy, its record of past decisions, and the tracker.

use crate::decision::Candidate;
use crate::error::{HandoverError, HandoverErrorKind};

/// The parameters of a handover policy.
///
/// The three parameters work together. The minimum quality decides when the
/// current server must be abandoned, the margin decides when a better
/// candidate is worth the switch, and the dwell time limits how often
/// switches may happen at all.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HandoverPolicy {
    /// The quality below which the current server is abandoned regardless of
    /// what else is available.
    ///
    /// Quality is a number the caller defines, and the policy is expressed
    /// in the same units. A value of 0.2 on a normalised scale means the
    /// server must stay above a fifth of full quality.
    pub minimum_quality: f64,
    /// The amount by which a candidate must exceed the current server's
    /// quality before a handover is worthwhile.
    ///
    /// This is the hysteresis that suppresses oscillation between two
    /// satellites of similar quality.
    pub handover_margin: f64,
    /// The minimum time between handovers, in seconds.
    ///
    /// A handover is refused when the previous one was more recent than
    /// this, unless the current server has fallen below the minimum quality,
    /// in which case the link would fail anyway.
    pub minimum_dwell_seconds: f64,
}

impl Default for HandoverPolicy {
    /// Returns a policy with a minimum quality of 0.2, a margin of 0.1, and
    /// a dwell time of five seconds.
    fn default() -> Self {
        Self {
            minimum_quality: 0.2,
            handover_margin: 0.1,
            minimum_dwell_seconds: 5.0,
        }
    }
}

impl HandoverPolicy {
    /// Creates a policy from its three parameters.
    pub const fn new(
        minimum_quality: f64,
        handover_margin: f64,
        minimum_dwell_seconds: f64,
    ) -> Self {
        Self {
            minimum_quality,
            handover_margin,
            minimum_dwell_seconds,
        }
    }

    /// Returns `true` when the policy parameters are usable.
    ///
    /// The margin must not be negative, since a negative margin would make a
    /// handover worth doing to a worse candidate, and the dwell time must
    /// not be negative. The minimum quality may be anything finite, because
    /// a caller may place it outside the range of the quality scale to
    /// disable the forced handover.
    pub fn is_valid(&self) -> bool {
        self.minimum_quality.is_finite()
            && self.handover_margin.is_finite()
            && self.handover_margin >= 0.0
            && self.minimum_dwell_seconds.is_finite()
            && self.minimum_dwell_seconds >= 0.0
    }

    /// Validates the policy, returning the reason when it is not usable.
    pub fn validate(&self) -> Result<(), HandoverError> {
        if !self.is_valid() {
            return Err(
                HandoverError::new(HandoverErrorKind::InvalidPolicy).with_detail(format!(
                    "minimum quality {}, margin {}, dwell {}",
                    self.minimum_quality, self.handover_margin, self.minimum_dwell_seconds
                )),
            );
        }
        Ok(())
    }

    /// Returns the best candidate under the policy.
    ///
    /// The best candidate is the one with the greatest quality, with ties
    /// broken by the lower satellite identifier so that the result is
    /// deterministic.
    pub fn best<'a>(&self, candidates: &'a [Candidate]) -> Option<&'a Candidate> {
        candidates.iter().max_by(|first, second| {
            first
                .quality
                .partial_cmp(&second.quality)
                .unwrap_or(core::cmp::Ordering::Equal)
                // Reversed, so that among equal qualities the lower
                // identifier wins in the maximum.
                .then_with(|| second.satellite.cmp(&first.satellite))
        })
    }
}

/// A record of one handover that has occurred.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HandoverRecord {
    /// The satellite handed over from.
    pub from: usize,
    /// The satellite handed over to.
    pub to: usize,
    /// The time of the handover, in seconds.
    pub seconds: f64,
    /// The quality of the satellite handed over from.
    pub from_quality: f64,
    /// The quality of the satellite handed over to.
    pub to_quality: f64,
}

impl HandoverRecord {
    /// Creates a record.
    pub const fn new(
        from: usize,
        to: usize,
        seconds: f64,
        from_quality: f64,
        to_quality: f64,
    ) -> Self {
        Self {
            from,
            to,
            seconds,
            from_quality,
            to_quality,
        }
    }

    /// Returns the gain in quality that the handover achieved.
    pub fn quality_gain(&self) -> f64 {
        self.to_quality - self.from_quality
    }
}

/// Tracks the serving satellite and the history of handovers.
///
/// The tracker owns the policy and the decision history, so a caller only
/// has to supply the candidates at each step. It is the form a simulation or
/// a control loop uses, where the decision made at one step affects the one
/// at the next through the dwell time.
#[derive(Debug, Clone, PartialEq)]
pub struct HandoverTracker {
    policy: HandoverPolicy,
    serving: Option<usize>,
    last_handover_seconds: Option<f64>,
    history: Vec<HandoverRecord>,
}

impl HandoverTracker {
    /// Creates a tracker with no serving satellite.
    pub fn new(policy: HandoverPolicy) -> Self {
        Self {
            policy,
            serving: None,
            last_handover_seconds: None,
            history: Vec::new(),
        }
    }

    /// Creates a tracker that is already serving a satellite.
    pub fn serving(policy: HandoverPolicy, satellite: usize) -> Self {
        Self {
            policy,
            serving: Some(satellite),
            last_handover_seconds: None,
            history: Vec::new(),
        }
    }

    /// Returns the policy.
    pub fn policy(&self) -> HandoverPolicy {
        self.policy
    }

    /// Returns the satellite currently being served, if any.
    pub fn current(&self) -> Option<usize> {
        self.serving
    }

    /// Returns the time of the last handover, if there has been one.
    pub fn last_handover_seconds(&self) -> Option<f64> {
        self.last_handover_seconds
    }

    /// Returns the handover history, in time order.
    pub fn history(&self) -> &[HandoverRecord] {
        &self.history
    }

    /// Returns the number of handovers that have occurred.
    pub fn handover_count(&self) -> usize {
        self.history.len()
    }

    /// Applies the policy to a set of candidates at a time.
    ///
    /// The tracker updates its state according to the decision and returns
    /// that decision, so a caller can both react to it and rely on the
    /// tracker having recorded it.
    ///
    /// Returns an error when the candidates or the policy are not usable.
    pub fn step(
        &mut self,
        seconds: f64,
        candidates: &[Candidate],
    ) -> Result<crate::decision::HandoverDecision, HandoverError> {
        let decision = crate::decision::decide_with_history(
            &self.policy,
            self.serving,
            self.last_handover_seconds,
            candidates,
            seconds,
        )?;
        match decision {
            crate::decision::HandoverDecision::HandOver {
                from,
                to,
                from_quality,
                to_quality,
            } => {
                self.history.push(HandoverRecord::new(
                    from,
                    to,
                    seconds,
                    from_quality,
                    to_quality,
                ));
                self.serving = Some(to);
                self.last_handover_seconds = Some(seconds);
            }
            crate::decision::HandoverDecision::Acquire { satellite } => {
                self.serving = Some(satellite);
                self.last_handover_seconds = Some(seconds);
            }
            crate::decision::HandoverDecision::Stay => {}
            crate::decision::HandoverDecision::Drop => {
                self.serving = None;
            }
        }
        Ok(decision)
    }

    /// Returns the mean time between handovers, in seconds.
    ///
    /// Returns `None` when fewer than two handovers have occurred, because a
    /// single interval is not a mean.
    pub fn mean_interval_seconds(&self) -> Option<f64> {
        if self.history.len() < 2 {
            return None;
        }
        let span = self.history.last()?.seconds - self.history.first()?.seconds;
        Some(span / (self.history.len() - 1) as f64)
    }

    /// Returns the number of handovers per unit time over the history.
    ///
    /// The rate is measured across the whole history, from the first
    /// handover to the last. Returns `None` when there are fewer than two
    /// handovers or when they all occur at the same instant.
    pub fn handover_rate_per_second(&self) -> Option<f64> {
        let interval = self.mean_interval_seconds()?;
        if interval <= 0.0 {
            return None;
        }
        Some(1.0 / interval)
    }

    /// Returns the total quality gained by handing over.
    ///
    /// The value is the sum over the history, which can be negative when a
    /// handover was forced by a falling quality rather than chosen for a
    /// gain.
    pub fn total_quality_gain(&self) -> f64 {
        self.history.iter().map(HandoverRecord::quality_gain).sum()
    }
}
