//! The handover policy, its record of past decisions, and the tracker.

use crate::decision::Candidate;
use crate::error::{HandoverError, HandoverErrorKind};

/// The parameters of a handover policy.
///
/// The three parameters work together. The minimum quality decides when the
/// current server must be abandoned, the margin decides when a better
/// candidate is worth the switch, and the dwell time limits how often
/// switches may happen at all.
///
/// 切换策略的参数。
///
/// 三个参数协同工作。最低质量决定当前服务星何时必须被放弃，余量决定
/// 更优候选何时值得切换，驻留时间则限制切换至多能多频繁发生。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HandoverPolicy {
    /// The quality below which the current server is abandoned regardless of
    /// what else is available.
    ///
    /// Quality is a number the caller defines, and the policy is expressed
    /// in the same units. A value of 0.2 on a normalised scale means the
    /// server must stay above a fifth of full quality.
    ///
    /// 低于该质量时，无论还有什么可用，当前服务星都被放弃。
    ///
    /// 质量是调用方定义的数，策略以相同单位表达。在归一化尺度上取 0.2
    /// 表示服务星必须保持在全质量的五分之一以上。
    pub minimum_quality: f64,
    /// The amount by which a candidate must exceed the current server's
    /// quality before a handover is worthwhile.
    ///
    /// This is the hysteresis that suppresses oscillation between two
    /// satellites of similar quality.
    ///
    /// 候选超出当前服务星质量多少才值得切换的量。
    ///
    /// 这是抑制质量相近的两颗卫星之间振荡的滞回量。
    pub handover_margin: f64,
    /// The minimum time between handovers, in seconds.
    ///
    /// A handover is refused when the previous one was more recent than
    /// this, unless the current server has fallen below the minimum quality,
    /// in which case the link would fail anyway.
    ///
    /// 两次切换之间的最短时间，单位为秒。
    ///
    /// 若上一次切换比这更近，则切换被拒绝，除非当前服务星已跌至最低
    /// 质量以下，此时链路无论如何都会失效。
    pub minimum_dwell_seconds: f64,
}

impl Default for HandoverPolicy {
    /// Returns a policy with a minimum quality of 0.2, a margin of 0.1, and
    /// a dwell time of five seconds.
    ///
    /// 返回最低质量为 0.2、余量为 0.1、驻留时间为五秒的策略。
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
    ///
    /// 由其三个参数创建策略。
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
    ///
    /// 当策略参数可用时返回 `true`。
    ///
    /// 余量不得为负，因为负余量会使切换到更差候选也值得做；驻留时间也
    /// 不得为负。最低质量可以是任何有限值，因为调用方可以把它放在质量
    /// 尺度范围之外以禁用强制切换。
    pub fn is_valid(&self) -> bool {
        self.minimum_quality.is_finite()
            && self.handover_margin.is_finite()
            && self.handover_margin >= 0.0
            && self.minimum_dwell_seconds.is_finite()
            && self.minimum_dwell_seconds >= 0.0
    }

    /// Validates the policy, returning the reason when it is not usable.
    ///
    /// 校验策略，不可用时返回原因。
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
    ///
    /// 返回该策略下的最优候选。
    ///
    /// 最优候选是质量最大者，并列时以较小的卫星标识符打破，使结果确定。
    pub fn best<'a>(&self, candidates: &'a [Candidate]) -> Option<&'a Candidate> {
        candidates.iter().max_by(|first, second| {
            first
                .quality
                .partial_cmp(&second.quality)
                .unwrap_or(core::cmp::Ordering::Equal)
                // Reversed, so that among equal qualities the lower
                // identifier wins in the maximum.
                // 反转，使质量相等时较小的标识符在取最大中胜出。
                .then_with(|| second.satellite.cmp(&first.satellite))
        })
    }
}

/// A record of one handover that has occurred.
///
/// 一次已发生切换的记录。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HandoverRecord {
    /// The satellite handed over from.
    ///
    /// 被切换离开的卫星。
    pub from: usize,
    /// The satellite handed over to.
    ///
    /// 被切换至的卫星。
    pub to: usize,
    /// The time of the handover, in seconds.
    ///
    /// 该切换的时间，单位为秒。
    pub seconds: f64,
    /// The quality of the satellite handed over from.
    ///
    /// 被切换离开卫星的质量。
    pub from_quality: f64,
    /// The quality of the satellite handed over to.
    ///
    /// 被切换至卫星的质量。
    pub to_quality: f64,
}

impl HandoverRecord {
    /// Creates a record.
    ///
    /// 创建一条记录。
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
    ///
    /// 返回该切换所取得的质量增益。
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
///
/// 跟踪服务卫星与切换历史。
///
/// 跟踪器拥有策略与决策历史，故调用方只需在每一步提供候选。这是仿真或
/// 控制回路所用的形式——其中一步的决策经驻留时间影响下一步。
#[derive(Debug, Clone, PartialEq)]
pub struct HandoverTracker {
    policy: HandoverPolicy,
    serving: Option<usize>,
    last_handover_seconds: Option<f64>,
    history: Vec<HandoverRecord>,
}

impl HandoverTracker {
    /// Creates a tracker with no serving satellite.
    ///
    /// 创建没有服务卫星的跟踪器。
    pub fn new(policy: HandoverPolicy) -> Self {
        Self {
            policy,
            serving: None,
            last_handover_seconds: None,
            history: Vec::new(),
        }
    }

    /// Creates a tracker that is already serving a satellite.
    ///
    /// 创建已在服务某卫星的跟踪器。
    pub fn serving(policy: HandoverPolicy, satellite: usize) -> Self {
        Self {
            policy,
            serving: Some(satellite),
            last_handover_seconds: None,
            history: Vec::new(),
        }
    }

    /// Returns the policy.
    ///
    /// 返回策略。
    pub fn policy(&self) -> HandoverPolicy {
        self.policy
    }

    /// Returns the satellite currently being served, if any.
    ///
    /// 返回当前正在服务的卫星（若有）。
    pub fn current(&self) -> Option<usize> {
        self.serving
    }

    /// Returns the time of the last handover, if there has been one.
    ///
    /// 返回最后一次切换的时间（若曾发生）。
    pub fn last_handover_seconds(&self) -> Option<f64> {
        self.last_handover_seconds
    }

    /// Returns the handover history, in time order.
    ///
    /// 按时间次序返回切换历史。
    pub fn history(&self) -> &[HandoverRecord] {
        &self.history
    }

    /// Returns the number of handovers that have occurred.
    ///
    /// 返回已发生的切换次数。
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
    ///
    /// 候选或策略不可用时返回错误。
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
    ///
    /// 返回两次切换之间的平均时间，单位为秒。
    ///
    /// 发生的切换少于两次时返回 `None`，因为单个间隔算不上均值。
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
    ///
    /// 返回整个历史上单位时间的切换次数。
    ///
    /// 速率跨越整个历史度量，从首次切换到末次。切换少于两次或它们都
    /// 发生在同一瞬时时返回 `None`。
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
    ///
    /// 返回切换所获得的质量总和。
    ///
    /// 该值是对整个历史求和，当切换是被下降的质量所迫而非为增益所选择
    /// 时可为负。
    pub fn total_quality_gain(&self) -> f64 {
        self.history.iter().map(HandoverRecord::quality_gain).sum()
    }
}
