//! SEC-7.2 (bd-8lppo): Graduated enforcement rollout with rollback guards.
//!
//! Tests for the phased enforcement rollout state machine, automatic rollback
//! triggers, operator-inspectable state, and integration with the risk controller
//! enforce flag.

use pi::extensions::{
    RollbackTrigger, RollbackWindowStats, RolloutPhase, RolloutState, RolloutTracker,
};

// ── RolloutPhase enum ──

#[test]
fn phase_ordering_shadow_lt_log_only_lt_enforce_new_lt_enforce_all() {
    assert!(RolloutPhase::Shadow < RolloutPhase::LogOnly);
    assert!(RolloutPhase::LogOnly < RolloutPhase::EnforceNew);
    assert!(RolloutPhase::EnforceNew < RolloutPhase::EnforceAll);
}

#[test]
fn phase_as_str_matches_serde_rename() {
    assert_eq!(RolloutPhase::Shadow.as_str(), "shadow");
    assert_eq!(RolloutPhase::LogOnly.as_str(), "log_only");
    assert_eq!(RolloutPhase::EnforceNew.as_str(), "enforce_new");
    assert_eq!(RolloutPhase::EnforceAll.as_str(), "enforce_all");
}

#[test]
fn phase_display_matches_as_str() {
    for phase in [
        RolloutPhase::Shadow,
        RolloutPhase::LogOnly,
        RolloutPhase::EnforceNew,
        RolloutPhase::EnforceAll,
    ] {
        assert_eq!(format!("{phase}"), phase.as_str());
    }
}

#[test]
fn phase_is_enforcing_only_for_enforce_phases() {
    assert!(!RolloutPhase::Shadow.is_enforcing());
    assert!(!RolloutPhase::LogOnly.is_enforcing());
    assert!(RolloutPhase::EnforceNew.is_enforcing());
    assert!(RolloutPhase::EnforceAll.is_enforcing());
}

#[test]
fn phase_serde_roundtrip() {
    for phase in [
        RolloutPhase::Shadow,
        RolloutPhase::LogOnly,
        RolloutPhase::EnforceNew,
        RolloutPhase::EnforceAll,
    ] {
        let json = serde_json::to_string(&phase).unwrap();
        let back: RolloutPhase = serde_json::from_str(&json).unwrap();
        assert_eq!(phase, back);
    }
}

#[test]
fn phase_serde_uses_snake_case() {
    let json = serde_json::to_string(&RolloutPhase::EnforceNew).unwrap();
    assert_eq!(json, "\"enforce_new\"");
    let json = serde_json::to_string(&RolloutPhase::LogOnly).unwrap();
    assert_eq!(json, "\"log_only\"");
}

// ── RollbackTrigger defaults ──

#[test]
fn rollback_trigger_defaults_are_reasonable() {
    let t = RollbackTrigger::default();
    assert!(t.max_false_positive_rate > 0.0);
    assert!(t.max_false_positive_rate < 1.0);
    assert!(t.max_error_rate > 0.0);
    assert!(t.max_error_rate < 1.0);
    assert!(t.window_size >= 10);
    assert!(t.max_latency_ms > 0);
}

#[test]
fn rollback_trigger_serde_roundtrip() {
    let t = RollbackTrigger {
        max_false_positive_rate: 0.03,
        max_error_rate: 0.08,
        window_size: 50,
        max_latency_ms: 150,
    };
    let json = serde_json::to_string(&t).unwrap();
    let back: RollbackTrigger = serde_json::from_str(&json).unwrap();
    assert_eq!(t, back);
}

// ── RolloutTracker: phase transitions ──

#[test]
fn tracker_new_starts_at_given_phase() {
    let tracker = RolloutTracker::new(RolloutPhase::Shadow);
    assert_eq!(tracker.phase, RolloutPhase::Shadow);
    assert_eq!(tracker.transition_count, 0);
    assert!(tracker.rolled_back_from.is_none());
}

#[test]
fn tracker_default_starts_at_enforce_all() {
    let tracker = RolloutTracker::default();
    assert_eq!(tracker.phase, RolloutPhase::EnforceAll);
}

#[test]
fn advance_progresses_through_all_phases() {
    let mut tracker = RolloutTracker::new(RolloutPhase::Shadow);

    assert!(tracker.advance());
    assert_eq!(tracker.phase, RolloutPhase::LogOnly);
    assert_eq!(tracker.transition_count, 1);

    assert!(tracker.advance());
    assert_eq!(tracker.phase, RolloutPhase::EnforceNew);
    assert_eq!(tracker.transition_count, 2);

    assert!(tracker.advance());
    assert_eq!(tracker.phase, RolloutPhase::EnforceAll);
    assert_eq!(tracker.transition_count, 3);

    // Cannot advance past EnforceAll
    assert!(!tracker.advance());
    assert_eq!(tracker.phase, RolloutPhase::EnforceAll);
    assert_eq!(tracker.transition_count, 3);
}

#[test]
fn advance_clears_rolled_back_from() {
    let mut tracker = RolloutTracker::new(RolloutPhase::EnforceNew);
    tracker.rollback();
    assert_eq!(tracker.rolled_back_from, Some(RolloutPhase::EnforceNew));

    tracker.advance(); // Shadow -> LogOnly
    assert!(tracker.rolled_back_from.is_none());
}

#[test]
fn rollback_from_enforce_all_goes_to_shadow() {
    let mut tracker = RolloutTracker::new(RolloutPhase::EnforceAll);
    tracker.rollback();
    assert_eq!(tracker.phase, RolloutPhase::Shadow);
    assert_eq!(tracker.rolled_back_from, Some(RolloutPhase::EnforceAll));
    assert_eq!(tracker.transition_count, 1);
}

#[test]
fn rollback_from_enforce_new_goes_to_shadow() {
    let mut tracker = RolloutTracker::new(RolloutPhase::EnforceNew);
    tracker.rollback();
    assert_eq!(tracker.phase, RolloutPhase::Shadow);
    assert_eq!(tracker.rolled_back_from, Some(RolloutPhase::EnforceNew));
}

#[test]
fn rollback_from_log_only_goes_to_shadow() {
    let mut tracker = RolloutTracker::new(RolloutPhase::LogOnly);
    tracker.rollback();
    assert_eq!(tracker.phase, RolloutPhase::Shadow);
    assert_eq!(tracker.rolled_back_from, Some(RolloutPhase::LogOnly));
}

#[test]
fn rollback_from_shadow_is_noop() {
    let mut tracker = RolloutTracker::new(RolloutPhase::Shadow);
    let count_before = tracker.transition_count;
    tracker.rollback();
    assert_eq!(tracker.phase, RolloutPhase::Shadow);
    assert_eq!(tracker.transition_count, count_before);
    assert!(tracker.rolled_back_from.is_none());
}

#[test]
fn set_phase_transitions_to_target() {
    let mut tracker = RolloutTracker::new(RolloutPhase::Shadow);
    tracker.set_phase(RolloutPhase::EnforceAll);
    assert_eq!(tracker.phase, RolloutPhase::EnforceAll);
    assert_eq!(tracker.transition_count, 1);
    assert!(tracker.rolled_back_from.is_none());
}

#[test]
fn set_phase_same_is_noop() {
    let mut tracker = RolloutTracker::new(RolloutPhase::LogOnly);
    tracker.set_phase(RolloutPhase::LogOnly);
    assert_eq!(tracker.transition_count, 0);
}

#[test]
fn set_phase_can_skip_phases() {
    let mut tracker = RolloutTracker::new(RolloutPhase::Shadow);
    tracker.set_phase(RolloutPhase::EnforceAll);
    assert_eq!(tracker.phase, RolloutPhase::EnforceAll);
    assert_eq!(tracker.transition_count, 1);
}

#[test]
fn set_phase_can_go_backwards() {
    let mut tracker = RolloutTracker::new(RolloutPhase::EnforceAll);
    tracker.set_phase(RolloutPhase::LogOnly);
    assert_eq!(tracker.phase, RolloutPhase::LogOnly);
    assert_eq!(tracker.transition_count, 1);
}

// ── RolloutTracker: rollback triggers ──

#[test]
fn no_rollback_when_too_few_samples() {
    let mut tracker = RolloutTracker::new(RolloutPhase::EnforceAll);
    // Record fewer than 10 decisions - should not trigger
    for _ in 0..9 {
        let triggered = tracker.record_decision(10, true, true);
        assert!(!triggered);
    }
    assert_eq!(tracker.phase, RolloutPhase::EnforceAll);
}

#[test]
fn false_positive_rate_triggers_rollback() {
    let mut tracker = RolloutTracker::new(RolloutPhase::EnforceAll);
    tracker.trigger.max_false_positive_rate = 0.05;
    tracker.trigger.window_size = 20;

    // Record 18 clean decisions first, then 2 FPs at the end.
    // After 20 decisions: 2/20 = 10% > 5% threshold → triggers.
    for _ in 0..18 {
        tracker.record_decision(10, false, false);
    }
    assert_eq!(tracker.phase, RolloutPhase::EnforceAll);
    tracker.record_decision(10, false, true); // FP #1
    tracker.record_decision(10, false, true); // FP #2
    assert_eq!(tracker.phase, RolloutPhase::Shadow);
    assert_eq!(tracker.rolled_back_from, Some(RolloutPhase::EnforceAll));
}

#[test]
fn error_rate_triggers_rollback() {
    let mut tracker = RolloutTracker::new(RolloutPhase::EnforceNew);
    tracker.trigger.max_error_rate = 0.10;
    tracker.trigger.window_size = 20;

    // Record 20 decisions, 3 are errors (15% > 10%)
    for i in 0..20 {
        let is_err = i < 3;
        tracker.record_decision(10, is_err, false);
    }
    assert_eq!(tracker.phase, RolloutPhase::Shadow);
    assert_eq!(tracker.rolled_back_from, Some(RolloutPhase::EnforceNew));
}

#[test]
fn high_latency_triggers_rollback() {
    let mut tracker = RolloutTracker::new(RolloutPhase::EnforceAll);
    tracker.trigger.max_latency_ms = 100;
    tracker.trigger.window_size = 20;

    // Record 20 decisions with avg latency 150ms > 100ms threshold
    for _ in 0..20 {
        tracker.record_decision(150, false, false);
    }
    assert_eq!(tracker.phase, RolloutPhase::Shadow);
}

#[test]
fn no_rollback_in_shadow_mode() {
    let mut tracker = RolloutTracker::new(RolloutPhase::Shadow);
    tracker.trigger.max_error_rate = 0.01;

    // All decisions are errors, but no rollback because we're in Shadow
    for _ in 0..50 {
        let triggered = tracker.record_decision(10, true, true);
        assert!(!triggered);
    }
    assert_eq!(tracker.phase, RolloutPhase::Shadow);
}

#[test]
fn no_rollback_in_log_only_mode() {
    let mut tracker = RolloutTracker::new(RolloutPhase::LogOnly);
    tracker.trigger.max_error_rate = 0.01;

    for _ in 0..50 {
        let triggered = tracker.record_decision(10, true, true);
        assert!(!triggered);
    }
    assert_eq!(tracker.phase, RolloutPhase::LogOnly);
}

#[test]
fn below_threshold_does_not_trigger_rollback() {
    let mut tracker = RolloutTracker::new(RolloutPhase::EnforceAll);
    tracker.trigger.max_false_positive_rate = 0.10;
    tracker.trigger.max_error_rate = 0.10;
    tracker.trigger.max_latency_ms = 200;
    tracker.trigger.window_size = 100;

    // Record 100 decisions: 5% FP, 5% error, 100ms avg latency - all below thresholds
    for i in 0..100 {
        let is_fp = i % 20 == 0; // 5%
        let is_err = i % 20 == 1; // 5%
        let triggered = tracker.record_decision(100, is_err, is_fp);
        assert!(!triggered);
    }
    assert_eq!(tracker.phase, RolloutPhase::EnforceAll);
}

// ── Window statistics ──

#[test]
fn window_stats_empty_returns_defaults() {
    let tracker = RolloutTracker::new(RolloutPhase::Shadow);
    let stats = tracker.window_stats();
    assert_eq!(stats.total_decisions, 0);
    assert_eq!(stats.error_count, 0);
    assert_eq!(stats.false_positive_count, 0);
}

#[test]
fn window_stats_correctly_counts() {
    let mut tracker = RolloutTracker::new(RolloutPhase::Shadow);
    tracker.record_decision(10, false, false);
    tracker.record_decision(20, true, false);
    tracker.record_decision(30, false, true);
    tracker.record_decision(40, true, true);

    let stats = tracker.window_stats();
    assert_eq!(stats.total_decisions, 4);
    assert_eq!(stats.error_count, 2);
    assert_eq!(stats.false_positive_count, 2);
    // avg latency = (10 + 20 + 30 + 40) / 4 = 25.0
    assert!((stats.avg_latency_ms - 25.0).abs() < f64::EPSILON);
}

#[test]
fn window_evicts_old_samples_beyond_limit() {
    let mut tracker = RolloutTracker::new(RolloutPhase::Shadow);
    tracker.trigger.window_size = 5;

    // Record 10 decisions
    for i in 0u64..10 {
        tracker.record_decision(i, false, false);
    }
    let stats = tracker.window_stats();
    // Only last 5 should remain
    assert_eq!(stats.total_decisions, 5);
    // avg latency = (5 + 6 + 7 + 8 + 9) / 5 = 7.0
    assert!((stats.avg_latency_ms - 7.0).abs() < f64::EPSILON);
}

// ── RollbackWindowStats serde ──

#[test]
fn window_stats_serde_roundtrip() {
    let stats = RollbackWindowStats {
        total_decisions: 42,
        error_count: 3,
        false_positive_count: 2,
        avg_latency_ms: 15.5,
    };
    let json = serde_json::to_string(&stats).unwrap();
    let back: RollbackWindowStats = serde_json::from_str(&json).unwrap();
    assert_eq!(stats, back);
}

// ── RolloutState snapshot ──

#[test]
fn rollout_state_captures_tracker_fields() {
    let mut tracker = RolloutTracker::new(RolloutPhase::LogOnly);
    tracker.record_decision(10, false, false);
    tracker.record_decision(20, true, false);
    tracker.advance(); // LogOnly -> EnforceNew

    let stats = tracker.window_stats();
    assert_eq!(tracker.phase, RolloutPhase::EnforceNew);
    assert_eq!(tracker.transition_count, 1);
    assert!(tracker.rolled_back_from.is_none());
    assert_eq!(stats.total_decisions, 2);
    assert_eq!(stats.error_count, 1);
}

#[test]
fn rollout_state_serde_roundtrip() {
    let state = RolloutState {
        phase: RolloutPhase::EnforceNew,
        enforce: true,
        enabled: true,
        last_transition_ms: 1_000_000,
        transition_count: 3,
        rolled_back_from: Some(RolloutPhase::EnforceAll),
        window_stats: RollbackWindowStats {
            total_decisions: 100,
            error_count: 5,
            false_positive_count: 2,
            avg_latency_ms: 12.3,
        },
    };
    let json = serde_json::to_string(&state).unwrap();
    let back: RolloutState = serde_json::from_str(&json).unwrap();
    assert_eq!(state, back);
}

// ── Combined lifecycle scenarios ──

#[test]
fn full_lifecycle_shadow_to_enforce_all_then_rollback_then_recover() {
    let mut tracker = RolloutTracker::new(RolloutPhase::Shadow);
    tracker.trigger.max_error_rate = 0.20;
    tracker.trigger.window_size = 20;

    // Phase 1: Shadow - record decisions, no triggers
    for _ in 0..20 {
        assert!(!tracker.record_decision(10, false, false));
    }
    assert_eq!(tracker.phase, RolloutPhase::Shadow);

    // Phase 2: Advance through all phases
    assert!(tracker.advance()); // -> LogOnly
    assert_eq!(tracker.phase, RolloutPhase::LogOnly);

    assert!(tracker.advance()); // -> EnforceNew
    assert_eq!(tracker.phase, RolloutPhase::EnforceNew);

    assert!(tracker.advance()); // -> EnforceAll
    assert_eq!(tracker.phase, RolloutPhase::EnforceAll);

    // Phase 3: Heavy errors trigger rollback
    for _ in 0..20 {
        tracker.record_decision(10, true, false); // all errors
    }
    assert_eq!(tracker.phase, RolloutPhase::Shadow);
    assert_eq!(tracker.rolled_back_from, Some(RolloutPhase::EnforceAll));

    // Phase 4: Recover by advancing again
    assert!(tracker.advance()); // -> LogOnly
    assert!(tracker.rolled_back_from.is_none());
    assert!(tracker.advance()); // -> EnforceNew
    assert!(tracker.advance()); // -> EnforceAll
    assert_eq!(tracker.phase, RolloutPhase::EnforceAll);
}

#[test]
fn operator_override_skips_phases() {
    let mut tracker = RolloutTracker::new(RolloutPhase::Shadow);
    tracker.set_phase(RolloutPhase::EnforceAll);
    assert_eq!(tracker.phase, RolloutPhase::EnforceAll);
    assert_eq!(tracker.transition_count, 1);

    // Operator can also downgrade
    tracker.set_phase(RolloutPhase::LogOnly);
    assert_eq!(tracker.phase, RolloutPhase::LogOnly);
    assert_eq!(tracker.transition_count, 2);
}

#[test]
fn rollback_after_partial_advancement() {
    let mut tracker = RolloutTracker::new(RolloutPhase::Shadow);
    tracker.trigger.max_false_positive_rate = 0.05;
    tracker.trigger.window_size = 20;

    tracker.advance(); // -> LogOnly
    tracker.advance(); // -> EnforceNew

    // Record decisions with high FP rate
    for i in 0..20 {
        let is_fp = i < 2; // 10% > 5%
        tracker.record_decision(10, false, is_fp);
    }
    assert_eq!(tracker.phase, RolloutPhase::Shadow);
    assert_eq!(tracker.rolled_back_from, Some(RolloutPhase::EnforceNew));
}

#[test]
fn multiple_rollbacks_track_latest_source() {
    let mut tracker = RolloutTracker::new(RolloutPhase::Shadow);
    tracker.trigger.max_error_rate = 0.05;
    tracker.trigger.window_size = 20;

    // First: advance to EnforceNew, trigger rollback
    tracker.set_phase(RolloutPhase::EnforceNew);
    for _ in 0..20 {
        tracker.record_decision(10, true, false);
    }
    assert_eq!(tracker.rolled_back_from, Some(RolloutPhase::EnforceNew));

    // Second: advance to EnforceAll, trigger rollback again
    tracker.set_phase(RolloutPhase::EnforceAll);
    // Clear old decisions by filling with good ones
    for _ in 0..20 {
        tracker.record_decision(10, false, false);
    }
    // Now fill with errors
    for _ in 0..20 {
        tracker.record_decision(10, true, false);
    }
    assert_eq!(tracker.rolled_back_from, Some(RolloutPhase::EnforceAll));
}

#[test]
fn transition_timestamps_increase() {
    let mut tracker = RolloutTracker::new(RolloutPhase::Shadow);
    let ts1 = tracker.last_transition_ms;

    tracker.advance(); // -> LogOnly
    let ts2 = tracker.last_transition_ms;
    assert!(ts2 >= ts1, "timestamps should be monotonic");

    tracker.advance(); // -> EnforceNew
    let ts3 = tracker.last_transition_ms;
    assert!(ts3 >= ts2, "timestamps should be monotonic");
}

// ── Custom trigger thresholds ──

#[test]
fn custom_trigger_strict_thresholds() {
    let mut tracker = RolloutTracker::new(RolloutPhase::EnforceAll);
    tracker.trigger = RollbackTrigger {
        max_false_positive_rate: 0.01, // 1% - very strict
        max_error_rate: 0.01,
        window_size: 100,
        max_latency_ms: 50,
    };

    // Even 2 errors in 100 decisions triggers rollback (2%)
    for i in 0..100 {
        let is_err = i < 2;
        tracker.record_decision(10, is_err, false);
    }
    assert_eq!(tracker.phase, RolloutPhase::Shadow);
}

#[test]
fn custom_trigger_relaxed_thresholds() {
    let mut tracker = RolloutTracker::new(RolloutPhase::EnforceAll);
    tracker.trigger = RollbackTrigger {
        max_false_positive_rate: 0.50, // 50% - very relaxed
        max_error_rate: 0.50,
        window_size: 20,
        max_latency_ms: 10_000,
    };

    // Distribute 8 errors across 20 decisions (40% < 50% threshold).
    // Put errors at positions 2,4,6,8,10,12,14,16 so that the running
    // rate in the first 10 decisions never exceeds 50%.
    for i in 0..20 {
        let is_err = i % 5 < 2; // alternating pattern, ~40%
        let triggered = tracker.record_decision(100, is_err, false);
        assert!(!triggered, "unexpected trigger at i={i}");
    }
    assert_eq!(tracker.phase, RolloutPhase::EnforceAll);
}

// ── Phase enforcement flag sync ──

#[test]
fn phase_enforce_flag_correctly_synced() {
    // This tests the conceptual contract: shadow/log_only = no enforce,
    // enforce_new/enforce_all = enforce
    assert!(!RolloutPhase::Shadow.is_enforcing());
    assert!(!RolloutPhase::LogOnly.is_enforcing());
    assert!(RolloutPhase::EnforceNew.is_enforcing());
    assert!(RolloutPhase::EnforceAll.is_enforcing());
}

// ── Edge cases ──

#[test]
fn window_size_one() {
    let mut tracker = RolloutTracker::new(RolloutPhase::EnforceAll);
    tracker.trigger.window_size = 1;
    tracker.trigger.max_error_rate = 0.5;

    // Not enough data (< 10) so no trigger even with window_size=1
    let triggered = tracker.record_decision(10, true, true);
    assert!(!triggered, "fewer than 10 samples should not trigger");
}

#[test]
fn exactly_at_threshold_does_not_trigger() {
    let mut tracker = RolloutTracker::new(RolloutPhase::EnforceAll);
    tracker.trigger.max_false_positive_rate = 0.10;
    tracker.trigger.max_error_rate = 0.50; // relax error threshold
    tracker.trigger.window_size = 20;

    // Distribute 2 FPs at the end so running rate never exceeds 10% during fill.
    // After 20 decisions: 2/20 = 0.10, which is NOT > 0.10
    for _ in 0..18 {
        tracker.record_decision(10, false, false);
    }
    tracker.record_decision(10, false, true); // FP at position 19
    // At 19 decisions: 1/19 = 5.2% — still under 10%
    assert_eq!(tracker.phase, RolloutPhase::EnforceAll);
    tracker.record_decision(10, false, true); // FP at position 20
    // At 20 decisions: 2/20 = 10% — exactly at threshold, NOT above
    assert_eq!(
        tracker.phase,
        RolloutPhase::EnforceAll,
        "exactly at threshold should NOT trigger (uses > not >=)"
    );
}

#[test]
fn just_above_threshold_triggers() {
    let mut tracker = RolloutTracker::new(RolloutPhase::EnforceAll);
    tracker.trigger.max_false_positive_rate = 0.10;
    tracker.trigger.window_size = 20;

    // Record 3/20 = 15% FP rate > 10%
    for i in 0..20 {
        let is_fp = i < 3;
        tracker.record_decision(10, false, is_fp);
    }
    assert_eq!(tracker.phase, RolloutPhase::Shadow);
}

#[test]
fn rollout_state_json_fields_present() {
    let state = RolloutState {
        phase: RolloutPhase::Shadow,
        enforce: false,
        enabled: true,
        last_transition_ms: 12345,
        transition_count: 2,
        rolled_back_from: None,
        window_stats: RollbackWindowStats::default(),
    };
    let json: serde_json::Value = serde_json::to_value(&state).unwrap();
    assert!(json.get("phase").is_some());
    assert!(json.get("enforce").is_some());
    assert!(json.get("enabled").is_some());
    assert!(json.get("last_transition_ms").is_some());
    assert!(json.get("transition_count").is_some());
    assert!(json.get("rolled_back_from").is_some());
    assert!(json.get("window_stats").is_some());
}
