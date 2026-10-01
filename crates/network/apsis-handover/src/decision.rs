//! The handover decision and the rules that produce it.

use crate::error::{HandoverError, HandoverErrorKind};
use crate::policy::HandoverPolicy;

/// A satellite that could serve the terminal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candidate {
    /// The satellite identifier.
    pub satellite: usize,
    /// The quality of the link to this satellite.
    ///
    /// The meaning is the caller's: an elevation angle, a received power, a
    /// carrier to noise ratio, or any other measure of how good the link is.
    /// The policy thresholds are expressed in the same units. The value must
    /// be finite.
    pub quality: f64,
    /// The time until this satellite stops being available, in seconds.
    ///
    /// This is the remaining time in the access window. It lets the policy
    /// prefer a satellite that will last, and it is reported so a caller can
    /// plan the next decision.
    pub remaining_seconds: f64,
}

impl Candidate {
    /// Creates a candidate.
    pub const fn new(satellite: usize, quality: f64, remaining_seconds: f64) -> Self {
        Self {
            satellite,
            quality,
            remaining_seconds,
        }
    }

    /// Returns `true` when the candidate is usable.
    pub fn is_valid(&self) -> bool {
        self.quality.is_finite()
            && self.remaining_seconds.is_finite()
            && self.remaining_seconds >= 0.0
    }
}

/// What the terminal should do at one instant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HandoverDecision {
    /// Keep the current server.
    Stay,
    /// Switch from one satellite to another.
    HandOver {
        /// The satellite being left.
        from: usize,
        /// The satellite being taken up.
        to: usize,
        /// The quality of the satellite being left.
        from_quality: f64,
        /// The quality of the satellite being taken up.
        to_quality: f64,
    },
    /// Establish service for the first time.
    Acquire {
        /// The satellite to use.
        satellite: usize,
    },
    /// Give up service, because no candidate meets the minimum quality.
    Drop,
}

impl HandoverDecision {
    /// Returns `true` when the decision changes the serving satellite.
    pub const fn is_change(self) -> bool {
        matches!(self, Self::HandOver { .. } | Self::Acquire { .. })
    }

    /// Returns the satellite the terminal serves after the decision.
    ///
    /// Returns `None` for [`HandoverDecision::Drop`] and for
    /// [`HandoverDecision::Stay`] with no current server.
    pub const fn resulting_satellite(self) -> Option<usize> {
        match self {
            Self::HandOver { to, .. } => Some(to),
            Self::Acquire { satellite } => Some(satellite),
            Self::Stay => None,
            Self::Drop => None,
        }
    }

    /// Returns a short name for the decision.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Stay => "stay",
            Self::HandOver { .. } => "hand over",
            Self::Acquire { .. } => "acquire",
            Self::Drop => "drop",
        }
    }
}

/// Validates a candidate list against the policy.
fn validate(policy: &HandoverPolicy, candidates: &[Candidate]) -> Result<(), HandoverError> {
    policy.validate()?;
    if candidates.is_empty() {
        return Err(HandoverError::new(HandoverErrorKind::NoCandidates));
    }
    for (index, candidate) in candidates.iter().enumerate() {
        if !candidate.is_valid() {
            return Err(
                HandoverError::new(HandoverErrorKind::InvalidQuality).with_detail(format!(
                    "candidate {index} has quality {} and {} s remaining",
                    candidate.quality, candidate.remaining_seconds
                )),
            );
        }
        // A duplicate would make the choice between two entries ambiguous
        // and would let a caller weight a satellite by listing it twice.
        if candidates[..index]
            .iter()
            .any(|earlier| earlier.satellite == candidate.satellite)
        {
            return Err(
                HandoverError::new(HandoverErrorKind::DuplicateSatellite).with_detail(format!(
                    "satellite {} appears at index {index}",
                    candidate.satellite
                )),
            );
        }
    }
    Ok(())
}

/// Returns the decision implied by the policy, without a dwell time.
///
/// This is the form to use when the terminal makes an independent decision
/// at each instant and the caller tracks the history itself. Use
/// [`decide_with_history`] when the dwell time must be enforced.
///
/// The rules, in the order they are applied:
///
/// 1. With no current server, acquire the best candidate, or drop when none
///    meets the minimum quality.
/// 2. When the current server is not among the candidates, it has become
///    unavailable, so switch to the best candidate or drop when none meets
///    the minimum quality.
/// 3. When the current server is below the minimum quality, abandon it for
///    the best candidate, or drop when none meets the minimum quality.
/// 4. Otherwise stay, unless a candidate exceeds the current server's
///    quality by more than the margin.
///
/// Returns an error when the candidates or the policy are not usable.
///
/// 候选或策略不可用时返回错误。
pub fn decide(
    policy: &HandoverPolicy,
    current: Option<usize>,
    candidates: &[Candidate],
) -> Result<HandoverDecision, HandoverError> {
    decide_internal(policy, current, None, candidates, 0.0)
}

/// Returns the decision implied by the policy, enforcing the dwell time.
///
/// This adds one rule to those of [`decide`]. When the time since the last
/// handover is less than the dwell time, a handover that would otherwise
/// happen for a quality gain is refused and the terminal stays. A handover
/// that is forced, because the current server has become unavailable or has
/// fallen below the minimum quality, is allowed regardless: refusing it
/// would drop the link, which is worse than the churn the dwell time
/// prevents.
///
/// Returns an error when the candidates or the policy are not usable.
///
/// 候选或策略不可用时返回错误。
pub fn decide_with_history(
    policy: &HandoverPolicy,
    current: Option<usize>,
    last_handover_seconds: Option<f64>,
    candidates: &[Candidate],
    seconds: f64,
) -> Result<HandoverDecision, HandoverError> {
    decide_internal(policy, current, last_handover_seconds, candidates, seconds)
}

/// Applies the rules with an optional dwell time.
fn decide_internal(
    policy: &HandoverPolicy,
    current: Option<usize>,
    last_handover_seconds: Option<f64>,
    candidates: &[Candidate],
    seconds: f64,
) -> Result<HandoverDecision, HandoverError> {
    validate(policy, candidates)?;

    let best = policy
        .best(candidates)
        .expect("the candidate list is not empty");
    let current_candidate = current.and_then(|satellite| {
        candidates
            .iter()
            .find(|candidate| candidate.satellite == satellite)
    });

    // Rule 1: no current server, so acquire the best that meets the
    // minimum, or drop.
    let Some(current_satellite) = current else {
        return Ok(if best.quality >= policy.minimum_quality {
            HandoverDecision::Acquire {
                satellite: best.satellite,
            }
        } else {
            HandoverDecision::Drop
        });
    };

    // Rule 2: the current server is no longer a candidate, so it has become
    // unavailable and the terminal must move.
    let Some(current_candidate) = current_candidate else {
        return Ok(if best.quality >= policy.minimum_quality {
            HandoverDecision::HandOver {
                from: current_satellite,
                to: best.satellite,
                from_quality: 0.0,
                to_quality: best.quality,
            }
        } else {
            HandoverDecision::Drop
        });
    };

    // Rule 3: the current server has fallen below the minimum, so abandon
    // it. This is a forced handover and the dwell time does not apply.
    if current_candidate.quality < policy.minimum_quality {
        if best.satellite == current_satellite {
            // The best candidate is the current server, which is too weak,
            // so there is nothing to move to.
            return Ok(HandoverDecision::Drop);
        }
        return Ok(if best.quality >= policy.minimum_quality {
            HandoverDecision::HandOver {
                from: current_satellite,
                to: best.satellite,
                from_quality: current_candidate.quality,
                to_quality: best.quality,
            }
        } else {
            HandoverDecision::Drop
        });
    }

    // Rule 4: consider a handover for a quality gain, which the dwell time
    // may refuse.
    if best.satellite != current_satellite
        && best.quality > current_candidate.quality + policy.handover_margin
    {
        if let Some(last) = last_handover_seconds {
            if seconds - last < policy.minimum_dwell_seconds {
                return Ok(HandoverDecision::Stay);
            }
        }
        return Ok(HandoverDecision::HandOver {
            from: current_satellite,
            to: best.satellite,
            from_quality: current_candidate.quality,
            to_quality: best.quality,
        });
    }

    Ok(HandoverDecision::Stay)
}
