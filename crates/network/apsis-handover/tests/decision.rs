//! Tests for handover decisions.
//!
//! 切换决策的测试。

use apsis_handover::{
    Candidate, HandoverDecision, HandoverErrorKind, HandoverPolicy, HandoverRecord,
    HandoverTracker, decide, decide_with_history,
};

/// The default policy: minimum quality 0.2, margin 0.1, dwell 5 s.
///
/// 缺省策略：最低质量 0.2、余量 0.1、驻留 5 s。
fn policy() -> HandoverPolicy {
    HandoverPolicy::default()
}

/// A candidate with the given identifier and quality.
///
/// 具有给定标识符与质量的候选。
fn candidate(satellite: usize, quality: f64) -> Candidate {
    Candidate::new(satellite, quality, 100.0)
}

#[test]
fn the_default_policy_has_the_documented_values() {
    let policy = policy();
    assert_eq!(policy.minimum_quality, 0.2);
    assert_eq!(policy.handover_margin, 0.1);
    assert_eq!(policy.minimum_dwell_seconds, 5.0);
    assert!(policy.is_valid());
}

#[test]
fn a_policy_is_validated() {
    assert!(HandoverPolicy::new(0.2, 0.0, 0.0).is_valid());
    // A negative margin would make a handover worthwhile to a worse
    // candidate.
    // 负余量会使切换到更差候选也值得。
    assert!(!HandoverPolicy::new(0.2, -0.1, 5.0).is_valid());
    assert!(!HandoverPolicy::new(0.2, 0.1, -1.0).is_valid());
    assert!(!HandoverPolicy::new(f64::NAN, 0.1, 5.0).is_valid());
    assert_eq!(
        HandoverPolicy::new(0.2, -0.1, 5.0)
            .validate()
            .expect_err("negative margin")
            .kind(),
        HandoverErrorKind::InvalidPolicy
    );
}

#[test]
fn the_best_candidate_has_the_greatest_quality() {
    let candidates = [candidate(1, 0.3), candidate(2, 0.9), candidate(3, 0.5)];
    let best = policy().best(&candidates).expect("non-empty");
    assert_eq!(best.satellite, 2);
}

#[test]
fn ties_are_broken_by_the_lower_identifier() {
    let candidates = [candidate(5, 0.7), candidate(2, 0.7), candidate(9, 0.7)];
    let best = policy().best(&candidates).expect("non-empty");
    assert_eq!(best.satellite, 2);
}

#[test]
fn a_terminal_with_no_server_acquires_the_best_candidate() {
    let candidates = [candidate(1, 0.5), candidate(2, 0.8)];
    let decision = decide(&policy(), None, &candidates).expect("valid");
    assert_eq!(decision, HandoverDecision::Acquire { satellite: 2 });
    assert!(decision.is_change());
    assert_eq!(decision.resulting_satellite(), Some(2));
}

#[test]
fn a_terminal_with_no_server_drops_when_nothing_is_good_enough() {
    let candidates = [candidate(1, 0.1), candidate(2, 0.15)];
    let decision = decide(&policy(), None, &candidates).expect("valid");
    assert_eq!(decision, HandoverDecision::Drop);
    assert!(!decision.is_change());
    assert_eq!(decision.resulting_satellite(), None);
}

#[test]
fn a_good_current_server_is_kept() {
    // The alternative is better, but by less than the margin.
    // 备选更好，但幅度小于余量。
    let candidates = [candidate(1, 0.80), candidate(2, 0.85)];
    let decision = decide(&policy(), Some(1), &candidates).expect("valid");
    assert_eq!(decision, HandoverDecision::Stay);
}

#[test]
fn a_clearly_better_candidate_causes_a_handover() {
    // The alternative beats the current server by more than the margin.
    // 备选超出当前服务星多于余量。
    let candidates = [candidate(1, 0.50), candidate(2, 0.80)];
    let decision = decide(&policy(), Some(1), &candidates).expect("valid");
    assert_eq!(
        decision,
        HandoverDecision::HandOver {
            from: 1,
            to: 2,
            from_quality: 0.50,
            to_quality: 0.80,
        }
    );
    assert_eq!(decision.resulting_satellite(), Some(2));
}

#[test]
fn the_margin_is_the_threshold_for_a_handover() {
    let policy = policy();
    // Exactly at the margin is not enough, because the comparison is
    // strict.
    // 恰在余量处不够，因为该比较是严格的。
    let at_margin = [candidate(1, 0.5), candidate(2, 0.6)];
    assert_eq!(
        decide(&policy, Some(1), &at_margin).expect("valid"),
        HandoverDecision::Stay
    );

    // Just above the margin is enough.
    // 恰高于余量即可。
    let above = [candidate(1, 0.5), candidate(2, 0.600_1)];
    assert!(decide(&policy, Some(1), &above).expect("valid").is_change());
}

#[test]
fn a_zero_margin_hands_over_to_any_better_candidate() {
    let policy = HandoverPolicy::new(0.2, 0.0, 0.0);
    let candidates = [candidate(1, 0.5), candidate(2, 0.500_1)];
    assert!(
        decide(&policy, Some(1), &candidates)
            .expect("valid")
            .is_change()
    );
}

#[test]
fn a_current_server_below_the_minimum_is_abandoned() {
    let candidates = [candidate(1, 0.1), candidate(2, 0.3)];
    let decision = decide(&policy(), Some(1), &candidates).expect("valid");
    assert_eq!(
        decision,
        HandoverDecision::HandOver {
            from: 1,
            to: 2,
            from_quality: 0.1,
            to_quality: 0.3,
        }
    );
}

#[test]
fn a_weak_terminal_drops_when_no_candidate_meets_the_minimum() {
    let candidates = [candidate(1, 0.1), candidate(2, 0.15)];
    let decision = decide(&policy(), Some(1), &candidates).expect("valid");
    assert_eq!(decision, HandoverDecision::Drop);
}

#[test]
fn a_current_server_that_disappears_causes_a_handover() {
    // The current server is no longer in the candidate list, which means
    // its access window has ended.
    // 当前服务星不再在候选列表中，这意味着它的可见性窗口已结束。
    let candidates = [candidate(2, 0.6), candidate(3, 0.4)];
    let decision = decide(&policy(), Some(1), &candidates).expect("valid");
    assert_eq!(
        decision,
        HandoverDecision::HandOver {
            from: 1,
            to: 2,
            from_quality: 0.0,
            to_quality: 0.6,
        }
    );
}

#[test]
fn a_current_server_that_disappears_and_nothing_replaces_it_drops() {
    let candidates = [candidate(2, 0.1)];
    let decision = decide(&policy(), Some(1), &candidates).expect("valid");
    assert_eq!(decision, HandoverDecision::Drop);
}

#[test]
fn the_only_candidate_being_the_current_server_keeps_it() {
    let candidates = [candidate(1, 0.5)];
    let decision = decide(&policy(), Some(1), &candidates).expect("valid");
    assert_eq!(decision, HandoverDecision::Stay);
}

#[test]
fn a_weak_current_server_with_no_alternative_drops() {
    // The current server is below the minimum and is the only candidate, so
    // there is nowhere to move and the service is lost.
    // 当前服务星低于最低值且是唯一候选，故无处可迁，服务丢失。
    let candidates = [candidate(1, 0.1)];
    let decision = decide(&policy(), Some(1), &candidates).expect("valid");
    assert_eq!(decision, HandoverDecision::Drop);
}

#[test]
fn the_dwell_time_refuses_an_opportunistic_handover() {
    let candidates = [candidate(1, 0.5), candidate(2, 0.9)];
    // The last handover was one second ago, which is inside the five second
    // dwell time, so the better candidate is not taken.
    // 上一次切换在一秒前，仍在五秒驻留时间内，故不取更好的候选。
    let decision =
        decide_with_history(&policy(), Some(1), Some(9.0), &candidates, 10.0).expect("valid");
    assert_eq!(decision, HandoverDecision::Stay);

    // After the dwell time the handover proceeds.
    // 驻留时间之后切换进行。
    let decision =
        decide_with_history(&policy(), Some(1), Some(1.0), &candidates, 10.0).expect("valid");
    assert!(decision.is_change());
}

#[test]
fn the_dwell_time_does_not_block_a_forced_handover() {
    // The current server has fallen below the minimum, so the handover must
    // happen even though the dwell time has not elapsed: refusing it would
    // drop the link.
    // 当前服务星已跌至最低值以下，故即便驻留时间未过也必须切换：拒绝它会掉链。
    let candidates = [candidate(1, 0.1), candidate(2, 0.9)];
    let decision =
        decide_with_history(&policy(), Some(1), Some(9.9), &candidates, 10.0).expect("valid");
    assert!(decision.is_change());
    assert_eq!(decision.resulting_satellite(), Some(2));
}

#[test]
fn the_dwell_time_does_not_block_an_acquisition() {
    let candidates = [candidate(1, 0.9)];
    let decision =
        decide_with_history(&policy(), None, Some(9.9), &candidates, 10.0).expect("valid");
    assert_eq!(decision, HandoverDecision::Acquire { satellite: 1 });
}

#[test]
fn the_dwell_time_does_not_block_a_handover_for_a_lost_server() {
    let candidates = [candidate(2, 0.9)];
    let decision =
        decide_with_history(&policy(), Some(1), Some(9.9), &candidates, 10.0).expect("valid");
    assert!(decision.is_change());
}

#[test]
fn a_zero_dwell_time_always_permits_a_handover() {
    let policy = HandoverPolicy::new(0.2, 0.1, 0.0);
    let candidates = [candidate(1, 0.5), candidate(2, 0.9)];
    let decision =
        decide_with_history(&policy, Some(1), Some(10.0), &candidates, 10.0).expect("valid");
    assert!(decision.is_change());
}

#[test]
fn invalid_candidates_are_rejected() {
    let empty: [Candidate; 0] = [];
    assert_eq!(
        decide(&policy(), None, &empty)
            .expect_err("no candidates")
            .kind(),
        HandoverErrorKind::NoCandidates
    );

    let duplicated = [candidate(1, 0.5), candidate(1, 0.6)];
    assert_eq!(
        decide(&policy(), None, &duplicated)
            .expect_err("a duplicate")
            .kind(),
        HandoverErrorKind::DuplicateSatellite
    );

    let not_finite = [Candidate::new(1, f64::NAN, 100.0)];
    assert_eq!(
        decide(&policy(), None, &not_finite)
            .expect_err("not finite")
            .kind(),
        HandoverErrorKind::InvalidQuality
    );

    let negative_remaining = [Candidate::new(1, 0.5, -1.0)];
    assert_eq!(
        decide(&policy(), None, &negative_remaining)
            .expect_err("negative remaining time")
            .kind(),
        HandoverErrorKind::InvalidQuality
    );
}

#[test]
fn the_decision_names_are_stable() {
    assert_eq!(HandoverDecision::Stay.name(), "stay");
    assert_eq!(
        HandoverDecision::HandOver {
            from: 1,
            to: 2,
            from_quality: 0.1,
            to_quality: 0.2
        }
        .name(),
        "hand over"
    );
    assert_eq!(HandoverDecision::Acquire { satellite: 1 }.name(), "acquire");
    assert_eq!(HandoverDecision::Drop.name(), "drop");
}

#[test]
fn a_tracker_acquires_and_then_hands_over() {
    let mut tracker = HandoverTracker::new(policy());

    // First step: acquire the best satellite.
    // 第一步：获取最优卫星。
    let decision = tracker
        .step(0.0, &[candidate(1, 0.5), candidate(2, 0.8)])
        .expect("valid");
    assert_eq!(decision, HandoverDecision::Acquire { satellite: 2 });
    assert_eq!(tracker.current(), Some(2));
    assert_eq!(tracker.handover_count(), 0);

    // The current server stays while it remains the best.
    // 当前服务星在仍为最优期间保持不变。
    let decision = tracker.step(1.0, &[candidate(2, 0.8)]).expect("valid");
    assert_eq!(decision, HandoverDecision::Stay);

    // A much better candidate after the dwell time causes a handover.
    // 驻留时间之后好得多的候选引起一次切换。
    let decision = tracker
        .step(10.0, &[candidate(2, 0.3), candidate(3, 0.9)])
        .expect("valid");
    assert_eq!(decision.resulting_satellite(), Some(3));
    assert_eq!(tracker.current(), Some(3));
    assert_eq!(tracker.handover_count(), 1);
    assert_eq!(tracker.last_handover_seconds(), Some(10.0));
}

#[test]
fn a_tracker_records_the_handover_history() {
    let mut tracker = HandoverTracker::serving(policy(), 1);
    // Force two handovers by dropping the current server below the minimum.
    // 通过把当前服务星降到最低值以下强制两次切换。
    tracker
        .step(0.0, &[candidate(1, 0.1), candidate(2, 0.9)])
        .expect("valid");
    tracker
        .step(1.0, &[candidate(2, 0.1), candidate(3, 0.9)])
        .expect("valid");

    assert_eq!(tracker.handover_count(), 2);
    let history = tracker.history();
    assert_eq!(history[0].from, 1);
    assert_eq!(history[0].to, 2);
    assert_eq!(history[1].from, 2);
    assert_eq!(history[1].to, 3);
    // The quality gain is the difference across each handover.
    // 质量增益是每次切换处的差值。
    assert!((history[0].quality_gain() - 0.8).abs() < 1e-12);
    assert!((tracker.total_quality_gain() - 1.6).abs() < 1e-12);
}

#[test]
fn a_tracker_reports_the_handover_rate() {
    let mut tracker = HandoverTracker::serving(policy(), 1);
    tracker
        .step(0.0, &[candidate(1, 0.1), candidate(2, 0.9)])
        .expect("valid");
    tracker
        .step(100.0, &[candidate(2, 0.1), candidate(3, 0.9)])
        .expect("valid");

    // Two handovers a hundred seconds apart give a mean interval of a
    // hundred seconds.
    // 相隔一百秒的两次切换给出百秒的平均间隔。
    assert_eq!(tracker.mean_interval_seconds(), Some(100.0));
    assert_eq!(tracker.handover_rate_per_second(), Some(0.01));
}

#[test]
fn a_tracker_with_fewer_than_two_handovers_has_no_rate() {
    let tracker = HandoverTracker::new(policy());
    assert_eq!(tracker.mean_interval_seconds(), None);
    assert_eq!(tracker.handover_rate_per_second(), None);
}

#[test]
fn a_tracker_that_loses_all_candidates_has_no_server() {
    let mut tracker = HandoverTracker::serving(policy(), 1);
    let decision = tracker.step(0.0, &[candidate(2, 0.05)]).expect("valid");
    assert_eq!(decision, HandoverDecision::Drop);
    assert_eq!(tracker.current(), None);
}

#[test]
fn a_tracker_rejects_invalid_candidates() {
    let mut tracker = HandoverTracker::new(policy());
    let empty: [Candidate; 0] = [];
    assert_eq!(
        tracker.step(0.0, &empty).expect_err("no candidates").kind(),
        HandoverErrorKind::NoCandidates
    );
}

#[test]
fn a_handover_record_reports_its_gain() {
    let record = HandoverRecord::new(1, 2, 5.0, 0.3, 0.8);
    assert!((record.quality_gain() - 0.5).abs() < 1e-12);
    // A forced handover can show a loss.
    // 强制切换可能显示为损失。
    let forced = HandoverRecord::new(1, 2, 5.0, 0.9, 0.3);
    assert!(forced.quality_gain() < 0.0);
}
