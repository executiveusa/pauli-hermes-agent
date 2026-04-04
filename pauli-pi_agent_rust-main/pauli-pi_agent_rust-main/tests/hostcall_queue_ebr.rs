use pi::hostcall_queue::{
    BravoBiasMode, ContentionSample, ContentionSignature, HostcallQueueEnqueueResult,
    HostcallQueueMode, HostcallRequestQueue, QueueTenant, S3FifoFallbackReason, S3FifoMode,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct TenantRequest {
    tenant: Option<&'static str>,
    value: u8,
}

impl QueueTenant for TenantRequest {
    fn tenant_key(&self) -> Option<&str> {
        self.tenant
    }
}

const fn starvation_sample() -> ContentionSample {
    ContentionSample {
        read_acquires: 80,
        write_acquires: 20,
        read_wait_p95_us: 120,
        write_wait_p95_us: 9_000,
        write_timeouts: 3,
    }
}

const fn write_dominant_sample() -> ContentionSample {
    ContentionSample {
        read_acquires: 20,
        write_acquires: 80,
        read_wait_p95_us: 120,
        write_wait_p95_us: 420,
        write_timeouts: 0,
    }
}

#[test]
fn ebr_mode_reports_retired_backlog_until_epoch_pins_release() {
    let mut queue = HostcallRequestQueue::with_mode(2, 2, HostcallQueueMode::Ebr);
    let pin = queue.pin_epoch();

    assert!(matches!(
        queue.push_back(1_u8),
        HostcallQueueEnqueueResult::FastPath { .. }
    ));
    assert!(matches!(
        queue.push_back(2_u8),
        HostcallQueueEnqueueResult::FastPath { .. }
    ));
    assert!(matches!(
        queue.push_back(3_u8),
        HostcallQueueEnqueueResult::OverflowPath { .. }
    ));

    let drained = queue.drain_all();
    assert_eq!(drained.into_iter().collect::<Vec<_>>(), vec![1, 2, 3]);

    queue.force_reclaim();
    let pinned = queue.snapshot();
    assert_eq!(pinned.reclamation_mode, HostcallQueueMode::Ebr);
    assert_eq!(pinned.active_epoch_pins, 1);
    assert!(pinned.retired_backlog >= 3);
    assert_eq!(pinned.reclaimed_total, 0);

    drop(pin);
    queue.force_reclaim();
    let reclaimed = queue.snapshot();
    assert_eq!(reclaimed.active_epoch_pins, 0);
    assert_eq!(reclaimed.retired_backlog, 0);
    assert!(reclaimed.reclaimed_total >= 3);
    assert!(reclaimed.reclamation_latency_max_epochs >= 1);
}

#[test]
fn enqueue_depths_and_backpressure_counters_stay_consistent() {
    let mut queue = HostcallRequestQueue::with_mode(2, 2, HostcallQueueMode::SafeFallback);

    assert!(matches!(
        queue.push_back(10_u8),
        HostcallQueueEnqueueResult::FastPath { depth: 1 }
    ));
    assert!(matches!(
        queue.push_back(11_u8),
        HostcallQueueEnqueueResult::FastPath { depth: 2 }
    ));
    assert!(matches!(
        queue.push_back(12_u8),
        HostcallQueueEnqueueResult::OverflowPath {
            depth: 3,
            overflow_depth: 1
        }
    ));
    assert!(matches!(
        queue.push_back(13_u8),
        HostcallQueueEnqueueResult::OverflowPath {
            depth: 4,
            overflow_depth: 2
        }
    ));
    assert!(matches!(
        queue.push_back(14_u8),
        HostcallQueueEnqueueResult::Rejected {
            depth: 4,
            overflow_depth: 2
        }
    ));

    let snapshot = queue.snapshot();
    assert_eq!(snapshot.total_depth, 4);
    assert_eq!(snapshot.fast_depth, 2);
    assert_eq!(snapshot.overflow_depth, 2);
    assert_eq!(snapshot.max_depth_seen, 4);
    assert_eq!(snapshot.overflow_enqueued_total, 2);
    assert_eq!(snapshot.overflow_rejected_total, 1);
}

#[test]
fn drain_preserves_fifo_when_overflow_lane_is_engaged() {
    let mut queue = HostcallRequestQueue::with_mode(1, 3, HostcallQueueMode::SafeFallback);

    assert!(matches!(
        queue.push_back(0_u8),
        HostcallQueueEnqueueResult::FastPath { depth: 1 }
    ));
    for (value, expected_depth, expected_overflow_depth) in [
        (1_u8, 2_usize, 1_usize),
        (2_u8, 3_usize, 2_usize),
        (3_u8, 4_usize, 3_usize),
    ] {
        assert!(matches!(
            queue.push_back(value),
            HostcallQueueEnqueueResult::OverflowPath {
                depth,
                overflow_depth
            } if depth == expected_depth && overflow_depth == expected_overflow_depth
        ));
    }
    assert!(matches!(
        queue.push_back(4_u8),
        HostcallQueueEnqueueResult::Rejected {
            depth: 4,
            overflow_depth: 3
        }
    ));

    let drained = queue.drain_all();
    assert_eq!(drained.into_iter().collect::<Vec<_>>(), vec![0, 1, 2, 3]);
}

#[test]
fn force_safe_fallback_is_idempotent_for_transition_counter() {
    let mut queue: HostcallRequestQueue<u8> =
        HostcallRequestQueue::with_mode(2, 2, HostcallQueueMode::Ebr);

    let initial = queue.snapshot();
    assert_eq!(initial.reclamation_mode, HostcallQueueMode::Ebr);
    assert_eq!(initial.fallback_transitions, 0);

    queue.force_safe_fallback();
    let first = queue.snapshot();
    assert_eq!(first.reclamation_mode, HostcallQueueMode::SafeFallback);
    assert_eq!(first.fallback_transitions, 1);

    queue.force_safe_fallback();
    let second = queue.snapshot();
    assert_eq!(second.reclamation_mode, HostcallQueueMode::SafeFallback);
    assert_eq!(second.fallback_transitions, 1);
}

#[test]
fn safe_fallback_mode_remains_operational_and_fifo() {
    let mut queue = HostcallRequestQueue::with_mode(2, 2, HostcallQueueMode::Ebr);
    assert!(matches!(
        queue.push_back(10_u8),
        HostcallQueueEnqueueResult::FastPath { .. }
    ));
    assert!(matches!(
        queue.push_back(11_u8),
        HostcallQueueEnqueueResult::FastPath { .. }
    ));

    queue.force_safe_fallback();
    let snapshot = queue.snapshot();
    assert_eq!(snapshot.reclamation_mode, HostcallQueueMode::SafeFallback);
    assert_eq!(snapshot.fallback_transitions, 1);

    let drained = queue.drain_all();
    assert_eq!(drained.into_iter().collect::<Vec<_>>(), vec![10, 11]);
}

#[test]
fn ebr_stress_run_reclaims_without_backlog_growth() {
    let mut queue = HostcallRequestQueue::with_mode(8, 32, HostcallQueueMode::Ebr);

    for value in 0..20_000_u32 {
        let _ = queue.push_back(value);
        let drained = queue.drain_all();
        assert_eq!(drained.len(), 1);
        if value % 128 == 0 {
            queue.force_reclaim();
        }
    }

    queue.force_reclaim();
    let snapshot = queue.snapshot();
    assert_eq!(snapshot.reclamation_mode, HostcallQueueMode::Ebr);
    assert_eq!(snapshot.retired_backlog, 0);
    assert!(snapshot.reclaimed_total >= 20_000);
}

#[test]
fn s3fifo_fallback_clears_ghost_and_active_tenants_until_reset() {
    let mut queue = HostcallRequestQueue::with_mode(1, 1, HostcallQueueMode::SafeFallback);

    assert!(matches!(
        queue.push_back(TenantRequest {
            tenant: Some("ext.noisy"),
            value: 0,
        }),
        HostcallQueueEnqueueResult::FastPath { .. }
    ));
    assert!(matches!(
        queue.push_back(TenantRequest {
            tenant: Some("ext.noisy"),
            value: 1,
        }),
        HostcallQueueEnqueueResult::OverflowPath { .. }
    ));
    assert!(matches!(
        queue.push_back(TenantRequest {
            tenant: Some("ext.noisy"),
            value: 2,
        }),
        HostcallQueueEnqueueResult::Rejected { .. }
    ));

    let pre_fallback = queue.snapshot();
    assert_eq!(pre_fallback.s3fifo_mode, S3FifoMode::Active);
    assert!(pre_fallback.s3fifo_ghost_depth >= 1);
    assert_eq!(pre_fallback.s3fifo_active_tenants, 1);

    for value in 3_u8..40_u8 {
        let _ = queue.push_back(TenantRequest {
            tenant: None,
            value,
        });
    }

    let fallback = queue.snapshot();
    assert_eq!(fallback.s3fifo_mode, S3FifoMode::ConservativeFifo);
    assert_eq!(
        fallback.s3fifo_fallback_reason,
        Some(S3FifoFallbackReason::FairnessInstability)
    );
    assert_eq!(fallback.s3fifo_fallback_transitions, 1);
    assert_eq!(fallback.s3fifo_ghost_depth, 0);
    assert_eq!(fallback.s3fifo_active_tenants, 0);
    let fairness_rejections_before = fallback.s3fifo_fairness_rejected_total;

    for value in 40_u8..80_u8 {
        let _ = queue.push_back(TenantRequest {
            tenant: Some("ext.noisy"),
            value,
        });
        let _ = queue.drain_all();
    }

    let stable = queue.snapshot();
    assert_eq!(stable.s3fifo_mode, S3FifoMode::ConservativeFifo);
    assert_eq!(
        stable.s3fifo_fallback_reason,
        Some(S3FifoFallbackReason::FairnessInstability)
    );
    assert_eq!(stable.s3fifo_fallback_transitions, 1);
    assert_eq!(stable.s3fifo_ghost_depth, 0);
    assert_eq!(stable.s3fifo_active_tenants, 0);
    assert_eq!(
        stable.s3fifo_fairness_rejected_total,
        fairness_rejections_before
    );
}

#[test]
fn s3fifo_fallback_latch_does_not_mutate_ebr_pin_reclaim_accounting() {
    let mut queue = HostcallRequestQueue::with_mode(1, 1, HostcallQueueMode::Ebr);
    let pin = queue.pin_epoch();

    assert!(matches!(
        queue.push_back(TenantRequest {
            tenant: Some("ext.noisy"),
            value: 0,
        }),
        HostcallQueueEnqueueResult::FastPath { .. }
    ));
    assert!(matches!(
        queue.push_back(TenantRequest {
            tenant: Some("ext.noisy"),
            value: 1,
        }),
        HostcallQueueEnqueueResult::OverflowPath { .. }
    ));
    assert!(matches!(
        queue.push_back(TenantRequest {
            tenant: Some("ext.noisy"),
            value: 2,
        }),
        HostcallQueueEnqueueResult::Rejected { .. }
    ));

    for value in 3_u8..40_u8 {
        let _ = queue.push_back(TenantRequest {
            tenant: None,
            value,
        });
    }

    let fallback = queue.snapshot();
    assert_eq!(fallback.reclamation_mode, HostcallQueueMode::Ebr);
    assert_eq!(fallback.active_epoch_pins, 1);
    assert_eq!(fallback.s3fifo_mode, S3FifoMode::ConservativeFifo);
    assert_eq!(
        fallback.s3fifo_fallback_reason,
        Some(S3FifoFallbackReason::FairnessInstability)
    );
    assert_eq!(fallback.s3fifo_fallback_transitions, 1);

    let drained = queue.drain_all();
    assert!(!drained.is_empty());
    queue.force_reclaim();
    let pinned = queue.snapshot();
    assert_eq!(pinned.active_epoch_pins, 1);
    assert!(pinned.retired_backlog >= drained.len());
    assert_eq!(pinned.reclaimed_total, 0);
    assert_eq!(pinned.s3fifo_mode, S3FifoMode::ConservativeFifo);
    assert_eq!(pinned.s3fifo_fallback_transitions, 1);

    drop(pin);
    queue.force_reclaim();
    let reclaimed = queue.snapshot();
    assert_eq!(reclaimed.active_epoch_pins, 0);
    assert_eq!(reclaimed.retired_backlog, 0);
    assert!(reclaimed.reclaimed_total >= drained.len() as u64);
    assert_eq!(reclaimed.s3fifo_mode, S3FifoMode::ConservativeFifo);
    assert_eq!(
        reclaimed.s3fifo_fallback_reason,
        Some(S3FifoFallbackReason::FairnessInstability)
    );
    assert_eq!(reclaimed.s3fifo_fallback_transitions, 1);
}

#[test]
fn bravo_observations_do_not_perturb_latched_s3fifo_fallback_telemetry() {
    let mut queue = HostcallRequestQueue::with_mode(1, 1, HostcallQueueMode::Ebr);

    assert!(matches!(
        queue.push_back(TenantRequest {
            tenant: Some("ext.noisy"),
            value: 0,
        }),
        HostcallQueueEnqueueResult::FastPath { .. }
    ));
    assert!(matches!(
        queue.push_back(TenantRequest {
            tenant: Some("ext.noisy"),
            value: 1,
        }),
        HostcallQueueEnqueueResult::OverflowPath { .. }
    ));
    assert!(matches!(
        queue.push_back(TenantRequest {
            tenant: Some("ext.noisy"),
            value: 2,
        }),
        HostcallQueueEnqueueResult::Rejected { .. }
    ));
    for value in 3_u8..40_u8 {
        let _ = queue.push_back(TenantRequest {
            tenant: None,
            value,
        });
    }

    let fallback = queue.snapshot();
    assert_eq!(fallback.s3fifo_mode, S3FifoMode::ConservativeFifo);
    assert_eq!(
        fallback.s3fifo_fallback_reason,
        Some(S3FifoFallbackReason::FairnessInstability)
    );
    assert_eq!(fallback.s3fifo_fallback_transitions, 1);

    let read_dominant = queue.observe_contention_window(ContentionSample {
        read_acquires: 220,
        write_acquires: 10,
        read_wait_p95_us: 15,
        write_wait_p95_us: 150,
        write_timeouts: 0,
    });
    assert_eq!(read_dominant.signature, ContentionSignature::ReadDominant);

    let starvation_risk = queue.observe_contention_window(ContentionSample {
        read_acquires: 20,
        write_acquires: 80,
        read_wait_p95_us: 40,
        write_wait_p95_us: 12_000,
        write_timeouts: 3,
    });
    assert_eq!(
        starvation_risk.signature,
        ContentionSignature::WriterStarvationRisk
    );

    let post_observe = queue.snapshot();
    assert_eq!(post_observe.s3fifo_mode, fallback.s3fifo_mode);
    assert_eq!(
        post_observe.s3fifo_fallback_reason,
        fallback.s3fifo_fallback_reason
    );
    assert_eq!(
        post_observe.s3fifo_fallback_transitions,
        fallback.s3fifo_fallback_transitions
    );
    assert_eq!(post_observe.s3fifo_ghost_depth, fallback.s3fifo_ghost_depth);
    assert_eq!(
        post_observe.s3fifo_active_tenants,
        fallback.s3fifo_active_tenants
    );

    assert_eq!(
        post_observe.bravo_last_signature,
        ContentionSignature::WriterStarvationRisk
    );
    assert!(post_observe.bravo_transitions >= fallback.bravo_transitions);
}

#[test]
fn latched_s3fifo_fallback_freezes_signal_and_fairness_counters_under_queue_activity() {
    let mut queue = HostcallRequestQueue::with_mode(1, 1, HostcallQueueMode::Ebr);

    assert!(matches!(
        queue.push_back(TenantRequest {
            tenant: Some("ext.noisy"),
            value: 0,
        }),
        HostcallQueueEnqueueResult::FastPath { .. }
    ));
    assert!(matches!(
        queue.push_back(TenantRequest {
            tenant: Some("ext.noisy"),
            value: 1,
        }),
        HostcallQueueEnqueueResult::OverflowPath { .. }
    ));
    assert!(matches!(
        queue.push_back(TenantRequest {
            tenant: Some("ext.noisy"),
            value: 2,
        }),
        HostcallQueueEnqueueResult::Rejected { .. }
    ));

    for value in 3_u8..40_u8 {
        let _ = queue.push_back(TenantRequest {
            tenant: None,
            value,
        });
    }

    let fallback = queue.snapshot();
    assert_eq!(fallback.s3fifo_mode, S3FifoMode::ConservativeFifo);
    assert_eq!(
        fallback.s3fifo_fallback_reason,
        Some(S3FifoFallbackReason::FairnessInstability)
    );
    assert_eq!(fallback.s3fifo_fallback_transitions, 1);

    let frozen_fairness_rejections = fallback.s3fifo_fairness_rejected_total;
    let frozen_ghost_hits = fallback.s3fifo_ghost_hits_total;
    let frozen_signal_samples = fallback.s3fifo_signal_samples;
    let frozen_signalless_streak = fallback.s3fifo_signalless_streak;
    let frozen_active_tenants = fallback.s3fifo_active_tenants;
    let frozen_ghost_depth = fallback.s3fifo_ghost_depth;

    for value in 40_u8..100_u8 {
        let tenant = if value % 2 == 0 {
            Some("ext.noisy")
        } else {
            None
        };
        let _ = queue.push_back(TenantRequest { tenant, value });
        let _ = queue.drain_all();
    }

    let stable = queue.snapshot();
    assert_eq!(stable.s3fifo_mode, S3FifoMode::ConservativeFifo);
    assert_eq!(
        stable.s3fifo_fallback_reason,
        fallback.s3fifo_fallback_reason
    );
    assert_eq!(stable.s3fifo_fallback_transitions, 1);
    assert_eq!(
        stable.s3fifo_fairness_rejected_total,
        frozen_fairness_rejections
    );
    assert_eq!(stable.s3fifo_ghost_hits_total, frozen_ghost_hits);
    assert_eq!(stable.s3fifo_signal_samples, frozen_signal_samples);
    assert_eq!(stable.s3fifo_signalless_streak, frozen_signalless_streak);
    assert_eq!(stable.s3fifo_active_tenants, frozen_active_tenants);
    assert_eq!(stable.s3fifo_ghost_depth, frozen_ghost_depth);
}

#[test]
fn ebr_bravo_writer_recovery_window_stays_bounded_and_exits_without_stale_counter() {
    let mut queue = HostcallRequestQueue::<u8>::with_mode(4, 4, HostcallQueueMode::Ebr);

    let first = queue.observe_contention_window(starvation_sample());
    assert_eq!(first.signature, ContentionSignature::WriterStarvationRisk);

    let after_first = queue.snapshot();
    assert_eq!(after_first.bravo_mode, BravoBiasMode::WriterRecovery);
    assert!(after_first.bravo_rollbacks >= 1);
    assert!(
        after_first.bravo_writer_recovery_remaining <= 2,
        "writer recovery window must be bounded by config default (2)"
    );

    let second = queue.observe_contention_window(starvation_sample());
    assert_eq!(second.signature, ContentionSignature::WriterStarvationRisk);

    let after_second = queue.snapshot();
    assert_eq!(after_second.bravo_mode, BravoBiasMode::WriterRecovery);
    assert!(after_second.bravo_writer_recovery_remaining <= 2);
    assert!(
        after_second.bravo_writer_recovery_remaining >= after_first.bravo_writer_recovery_remaining
    );

    for _ in 0..4 {
        let decision = queue.observe_contention_window(write_dominant_sample());
        assert_eq!(decision.signature, ContentionSignature::WriteDominant);
    }

    let stable = queue.snapshot();
    assert_eq!(stable.bravo_mode, BravoBiasMode::Balanced);
    assert_eq!(stable.bravo_writer_recovery_remaining, 0);
    assert_eq!(
        stable.bravo_last_signature,
        ContentionSignature::WriteDominant
    );
    assert!(stable.bravo_transitions > after_second.bravo_transitions);
}
