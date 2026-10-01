//! The handover decision and the rules that produce it.
//!
//! 切换决策以及产生它的各条规则。

use crate::error::{HandoverError, HandoverErrorKind};
use crate::policy::HandoverPolicy;

/// A satellite that could serve the terminal.
///
/// 一颗可能服务终端的卫星。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candidate {
    /// The satellite identifier.
    ///
    /// 卫星标识符。
    pub satellite: usize,
    /// The quality of the link to this satellite.
    ///
    /// The meaning is the caller's: an elevation angle, a received power, a
    /// carrier to noise ratio, or any other measure of how good the link is.
    /// The policy thresholds are expressed in the same units. The value must
    /// be finite.
    ///
    /// 到该卫星链路的质量。
    ///
    /// 其含义由调用方定义：仰角、接收功率、载噪比，或任何其他衡量链路
    /// 好坏的度量。策略阈值以相同单位表达。该值必须有限。
    pub quality: f64,
    /// The time until this satellite stops being available, in seconds.
    ///
    /// This is the remaining time in the access window. It lets the policy
    /// prefer a satellite that will last, and it is reported so a caller can
    /// plan the next decision.
    ///
    /// 到该卫星不再可用所剩的时间，单位为秒。
    ///
    /// 这是可见性窗口中的剩余时间。它使策略可以偏好将持续更久的卫星，
    /// 并被报告出来以便调用方规划下一步决策。
    pub remaining_seconds: f64,
}

impl Candidate {
    /// Creates a candidate.
    ///
    /// 创建一个候选。
    pub const fn new(satellite: usize, quality: f64, remaining_seconds: f64) -> Self {
        Self {
            satellite,
            quality,
            remaining_seconds,
        }
    }

    /// Returns `true` when the candidate is usable.
    ///
    /// 当该候选可用时返回 `true`。
    pub fn is_valid(&self) -> bool {
        self.quality.is_finite()
            && self.remaining_seconds.is_finite()
            && self.remaining_seconds >= 0.0
    }
}

/// What the terminal should do at one instant.
///
/// 终端在某一瞬时应做的动作。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HandoverDecision {
    /// Keep the current server.
    ///
    /// 保持当前服务星。
    Stay,
    /// Switch from one satellite to another.
    ///
    /// 从一颗卫星切换到另一颗。
    HandOver {
        /// The satellite being left.
        ///
        /// 被离开的卫星。
        from: usize,
        /// The satellite being taken up.
        ///
        /// 被接替的卫星。
        to: usize,
        /// The quality of the satellite being left.
        ///
        /// 被离开卫星的质量。
        from_quality: f64,
        /// The quality of the satellite being taken up.
        ///
        /// 被接替卫星的质量。
        to_quality: f64,
    },
    /// Establish service for the first time.
    ///
    /// 首次建立服务。
    Acquire {
        /// The satellite to use.
        ///
        /// 要使用的卫星。
        satellite: usize,
    },
    /// Give up service, because no candidate meets the minimum quality.
    ///
    /// 放弃服务，因为没有候选满足最低质量。
    Drop,
}

impl HandoverDecision {
    /// Returns `true` when the decision changes the serving satellite.
    ///
    /// 当该决策改变服务卫星时返回 `true`。
    pub const fn is_change(self) -> bool {
        matches!(self, Self::HandOver { .. } | Self::Acquire { .. })
    }

    /// Returns the satellite the terminal serves after the decision.
    ///
    /// Returns `None` for [`HandoverDecision::Drop`] and for
    /// [`HandoverDecision::Stay`] with no current server.
    ///
    /// 返回决策之后终端所服务的卫星。
    ///
    /// 对 [`HandoverDecision::Drop`] 以及没有当前服务星的
    /// [`HandoverDecision::Stay`] 返回 `None`。
    pub const fn resulting_satellite(self) -> Option<usize> {
        match self {
            Self::HandOver { to, .. } => Some(to),
            Self::Acquire { satellite } => Some(satellite),
            Self::Stay => None,
            Self::Drop => None,
        }
    }

    /// Returns a short name for the decision.
    ///
    /// 返回该决策的简称。
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
///
/// 针对策略校验候选列表。
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
        // 重复项会使两项之间的选择变得含糊，并让调用方通过列出两次来
        // 加重某卫星的权重。
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
/// 返回策略在不设驻留时间时所蕴含的决策。
///
/// 这是终端在每一瞬时独立决策、而调用方自行跟踪历史时所用的形式。当必须强制
/// 驻留时间时，用 [`decide_with_history`]。
///
/// 各条规则，按其施加次序：
///
/// 1. 没有当前服务星时，获取最优候选；若没有一个满足最低质量，则放弃。
/// 2. 当前服务星不在候选中时，它已不可用，故切换到最优候选；若没有一个满足
///    最低质量，则放弃。
/// 3. 当前服务星低于最低质量时，放弃它而取最优候选；若没有一个满足最低质量，
///    则放弃。
/// 4. 否则保持，除非某候选超出当前服务星的质量多于余量。
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
/// 返回策略在强制驻留时间时所蕴含的决策。
///
/// 它在 [`decide`] 的各条规则之上再加一条。当距上次切换的时间少于驻留时间时，
/// 本会因质量增益而发生的切换被拒绝，终端保持。被强制的切换——因为当前服务星
/// 已不可用或已跌至最低质量以下——无论如何都允许：拒绝它会掉链，这比驻留时间
/// 所要防止的频繁切换更糟。
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
///
/// 以可选驻留时间施加各条规则。
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
    // 规则 1：没有当前服务星，故获取满足最低质量的最优者，否则放弃。
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
    // 规则 2：当前服务星不再是候选，故它已不可用，终端必须迁移。
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
    // 规则 3：当前服务星已跌至最低质量以下，故放弃它。这是强制切换，
    // 驻留时间不适用。
    if current_candidate.quality < policy.minimum_quality {
        if best.satellite == current_satellite {
            // The best candidate is the current server, which is too weak,
            // so there is nothing to move to.
            // 最优候选就是当前服务星，而它太弱，故无处可迁。
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
    // 规则 4：为质量增益考虑切换，驻留时间可能拒绝它。
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
