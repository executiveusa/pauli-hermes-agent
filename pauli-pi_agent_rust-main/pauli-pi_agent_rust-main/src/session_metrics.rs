//! Session hot-path observability.
//!
//! Thread-safe atomic counters and timing for save/append/serialize/IO
//! operations. Gated behind `PI_PERF_TELEMETRY=1` for zero overhead in
//! production: when disabled, all recording methods are instant no-ops.
//!
//! ## Design
//!
//! - [`TimingCounter`]: atomic count + total microseconds + max microseconds.
//! - [`ByteCounter`]: atomic count + total bytes.
//! - [`SessionMetrics`]: composes counters for every instrumented phase.
//! - [`ScopedTimer`]: RAII guard that records elapsed time on drop.
//! - [`global()`]: returns `&'static SessionMetrics` (lazy-initialized once).
//!
//! ## Integration points
//!
//! Currently instrumented (files owned by this bead):
//! - `session_sqlite.rs`: save, load, metadata load
//! - `session_index.rs`: lock acquisition, upsert, list, reindex
//!
//! Future instrumentation (requires session.rs access):
//! - `Session::save()` JSONL path: queue wait, serialization, IO, persist
//! - `Session::append_*()`: in-memory append timing

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

// ---------------------------------------------------------------------------
// TimingCounter
// ---------------------------------------------------------------------------

/// Atomic counter that tracks invocation count, cumulative time (µs), and
/// peak time (µs) for a single instrumented phase.
///
/// All operations use `Relaxed` ordering because we tolerate slightly stale
/// reads in exchange for zero contention. The counters are append-only
/// (monotonically increasing), so torn reads cannot produce logically
/// inconsistent results.
#[derive(Debug)]
pub struct TimingCounter {
    count: AtomicU64,
    total_us: AtomicU64,
    max_us: AtomicU64,
}

impl TimingCounter {
    const fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
            total_us: AtomicU64::new(0),
            max_us: AtomicU64::new(0),
        }
    }

    /// Record one observation of `elapsed_us` microseconds.
    #[inline]
    pub fn record(&self, elapsed_us: u64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.total_us.fetch_add(elapsed_us, Ordering::Relaxed);
        // Relaxed CAS loop for max — bounded to one retry on contention.
        let mut current = self.max_us.load(Ordering::Relaxed);
        while elapsed_us > current {
            match self.max_us.compare_exchange_weak(
                current,
                elapsed_us,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current = actual,
            }
        }
    }

    /// Snapshot the current counters.
    pub fn snapshot(&self) -> TimingSnapshot {
        let count = self.count.load(Ordering::Relaxed);
        let total_us = self.total_us.load(Ordering::Relaxed);
        let max_us = self.max_us.load(Ordering::Relaxed);
        TimingSnapshot {
            count,
            total_us,
            max_us,
            avg_us: total_us.checked_div(count).unwrap_or(0),
        }
    }

    /// Reset all counters to zero.
    pub fn reset(&self) {
        self.count.store(0, Ordering::Relaxed);
        self.total_us.store(0, Ordering::Relaxed);
        self.max_us.store(0, Ordering::Relaxed);
    }
}

/// Point-in-time snapshot of a [`TimingCounter`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimingSnapshot {
    pub count: u64,
    pub total_us: u64,
    pub max_us: u64,
    pub avg_us: u64,
}

impl std::fmt::Display for TimingSnapshot {
    #[allow(clippy::cast_precision_loss)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.count == 0 {
            write!(f, "n=0")
        } else {
            write!(
                f,
                "n={} avg={:.1}ms max={:.1}ms total={:.1}ms",
                self.count,
                self.avg_us as f64 / 1000.0,
                self.max_us as f64 / 1000.0,
                self.total_us as f64 / 1000.0,
            )
        }
    }
}

// ---------------------------------------------------------------------------
// ByteCounter
// ---------------------------------------------------------------------------

/// Atomic counter for tracking bytes written/read.
#[derive(Debug)]
pub struct ByteCounter {
    count: AtomicU64,
    total_bytes: AtomicU64,
}

impl ByteCounter {
    const fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
            total_bytes: AtomicU64::new(0),
        }
    }

    /// Record one observation of `bytes` bytes.
    #[inline]
    pub fn record(&self, bytes: u64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.total_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Snapshot the current counters.
    pub fn snapshot(&self) -> ByteSnapshot {
        let count = self.count.load(Ordering::Relaxed);
        let total_bytes = self.total_bytes.load(Ordering::Relaxed);
        ByteSnapshot {
            count,
            total_bytes,
            avg_bytes: total_bytes.checked_div(count).unwrap_or(0),
        }
    }

    /// Reset all counters to zero.
    pub fn reset(&self) {
        self.count.store(0, Ordering::Relaxed);
        self.total_bytes.store(0, Ordering::Relaxed);
    }
}

/// Point-in-time snapshot of a [`ByteCounter`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteSnapshot {
    pub count: u64,
    pub total_bytes: u64,
    pub avg_bytes: u64,
}

impl std::fmt::Display for ByteSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.count == 0 {
            write!(f, "n=0")
        } else {
            write!(
                f,
                "n={} avg={}B total={}B",
                self.count, self.avg_bytes, self.total_bytes,
            )
        }
    }
}

// ---------------------------------------------------------------------------
// SessionMetrics
// ---------------------------------------------------------------------------

/// Centralized session hot-path metrics collector.
///
/// Covers four key phases identified in the PERF-3X measurement plan:
/// 1. **Queueing**: time between save request and actual IO start
/// 2. **Serialization**: `serde_json` encoding time and output size
/// 3. **IO / Fsync**: file write, flush, and atomic-rename time
/// 4. **Index update**: session-index upsert time
///
/// Plus additional breakdowns for SQLite paths and lock contention.
pub struct SessionMetrics {
    enabled: AtomicBool,

    // -- JSONL save path (session.rs — future integration) --
    /// Total wall-clock time for a complete `Session::save()` JSONL call.
    pub jsonl_save: TimingCounter,
    /// Time spent in `serde_json::to_writer` for header + entries.
    pub jsonl_serialize: TimingCounter,
    /// Time spent in `BufWriter::flush()` + `tempfile::persist()`.
    pub jsonl_io: TimingCounter,
    /// Bytes written per JSONL save (header + entries).
    pub jsonl_bytes: ByteCounter,
    /// Queue wait: time from `save()` entry to IO thread start.
    pub jsonl_queue_wait: TimingCounter,

    // -- SQLite save path (session_sqlite.rs) --
    /// Total wall-clock time for `save_session()` (full rewrite).
    pub sqlite_save: TimingCounter,
    /// Total wall-clock time for `append_entries()` (incremental append).
    pub sqlite_append: TimingCounter,
    /// Time spent serializing entries to JSON strings within SQLite save.
    pub sqlite_serialize: TimingCounter,
    /// Total JSON bytes produced during SQLite save serialization.
    pub sqlite_bytes: ByteCounter,

    // -- SQLite load path (session_sqlite.rs) --
    /// Total wall-clock time for `load_session()`.
    pub sqlite_load: TimingCounter,
    /// Total wall-clock time for `load_session_meta()`.
    pub sqlite_load_meta: TimingCounter,

    // -- Session index (session_index.rs) --
    /// Lock acquisition time in `SessionIndex::with_lock()`.
    pub index_lock: TimingCounter,
    /// Total wall-clock time for `upsert_meta()` (including lock).
    pub index_upsert: TimingCounter,
    /// Total wall-clock time for `list_sessions()` (including lock).
    pub index_list: TimingCounter,
    /// Total wall-clock time for `reindex_all()`.
    pub index_reindex: TimingCounter,

    // -- In-memory append (session.rs — future integration) --
    /// Time for `Session::append_message()` and similar in-memory ops.
    pub append: TimingCounter,
}

impl SessionMetrics {
    const fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false),
            jsonl_save: TimingCounter::new(),
            jsonl_serialize: TimingCounter::new(),
            jsonl_io: TimingCounter::new(),
            jsonl_bytes: ByteCounter::new(),
            jsonl_queue_wait: TimingCounter::new(),
            sqlite_save: TimingCounter::new(),
            sqlite_append: TimingCounter::new(),
            sqlite_serialize: TimingCounter::new(),
            sqlite_bytes: ByteCounter::new(),
            sqlite_load: TimingCounter::new(),
            sqlite_load_meta: TimingCounter::new(),
            index_lock: TimingCounter::new(),
            index_upsert: TimingCounter::new(),
            index_list: TimingCounter::new(),
            index_reindex: TimingCounter::new(),
            append: TimingCounter::new(),
        }
    }

    /// Whether metrics collection is active.
    #[inline]
    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Explicitly enable metrics (useful for tests).
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Relaxed);
    }

    /// Explicitly disable metrics.
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }

    /// Start a scoped timer for `counter`. If metrics are disabled, returns
    /// a no-op timer that does nothing on drop.
    #[inline]
    pub fn start_timer<'a>(&'a self, counter: &'a TimingCounter) -> ScopedTimer<'a> {
        if self.enabled() {
            ScopedTimer {
                counter: Some(counter),
                start: Instant::now(),
                finished: false,
            }
        } else {
            ScopedTimer {
                counter: None,
                start: Instant::now(), // unused but cheap
                finished: false,
            }
        }
    }

    /// Record bytes if metrics are enabled.
    #[inline]
    pub fn record_bytes(&self, counter: &ByteCounter, bytes: u64) {
        if self.enabled() {
            counter.record(bytes);
        }
    }

    /// Reset all counters to zero.
    pub fn reset_all(&self) {
        self.jsonl_save.reset();
        self.jsonl_serialize.reset();
        self.jsonl_io.reset();
        self.jsonl_bytes.reset();
        self.jsonl_queue_wait.reset();
        self.sqlite_save.reset();
        self.sqlite_append.reset();
        self.sqlite_serialize.reset();
        self.sqlite_bytes.reset();
        self.sqlite_load.reset();
        self.sqlite_load_meta.reset();
        self.index_lock.reset();
        self.index_upsert.reset();
        self.index_list.reset();
        self.index_reindex.reset();
        self.append.reset();
    }

    /// Produce a structured snapshot of all metrics.
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            enabled: self.enabled(),
            jsonl_save: self.jsonl_save.snapshot(),
            jsonl_serialize: self.jsonl_serialize.snapshot(),
            jsonl_io: self.jsonl_io.snapshot(),
            jsonl_bytes: self.jsonl_bytes.snapshot(),
            jsonl_queue_wait: self.jsonl_queue_wait.snapshot(),
            sqlite_save: self.sqlite_save.snapshot(),
            sqlite_append: self.sqlite_append.snapshot(),
            sqlite_serialize: self.sqlite_serialize.snapshot(),
            sqlite_bytes: self.sqlite_bytes.snapshot(),
            sqlite_load: self.sqlite_load.snapshot(),
            sqlite_load_meta: self.sqlite_load_meta.snapshot(),
            index_lock: self.index_lock.snapshot(),
            index_upsert: self.index_upsert.snapshot(),
            index_list: self.index_list.snapshot(),
            index_reindex: self.index_reindex.snapshot(),
            append: self.append.snapshot(),
        }
    }

    /// Human-readable multi-line summary for diagnostics.
    pub fn summary(&self) -> String {
        if !self.enabled() {
            return "Session telemetry disabled (set PI_PERF_TELEMETRY=1 to enable)".to_string();
        }
        let s = self.snapshot();
        format!(
            "Session hot-path metrics:\n  \
             JSONL save:       {}\n  \
             JSONL serialize:  {}\n  \
             JSONL IO:         {}\n  \
             JSONL bytes:      {}\n  \
             JSONL queue wait: {}\n  \
             SQLite save:      {}\n  \
             SQLite append:    {}\n  \
             SQLite serialize: {}\n  \
             SQLite bytes:     {}\n  \
             SQLite load:      {}\n  \
             SQLite load meta: {}\n  \
             Index lock:       {}\n  \
             Index upsert:     {}\n  \
             Index list:       {}\n  \
             Index reindex:    {}\n  \
             Append:           {}",
            s.jsonl_save,
            s.jsonl_serialize,
            s.jsonl_io,
            s.jsonl_bytes,
            s.jsonl_queue_wait,
            s.sqlite_save,
            s.sqlite_append,
            s.sqlite_serialize,
            s.sqlite_bytes,
            s.sqlite_load,
            s.sqlite_load_meta,
            s.index_lock,
            s.index_upsert,
            s.index_list,
            s.index_reindex,
            s.append,
        )
    }

    /// Emit the summary to `tracing::debug` (called periodically or on demand).
    pub fn emit(&self) {
        if self.enabled() {
            tracing::debug!("{}", self.summary());
        }
    }
}

// ---------------------------------------------------------------------------
// MetricsSnapshot
// ---------------------------------------------------------------------------

/// Complete point-in-time snapshot of all session metrics.
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub enabled: bool,
    pub jsonl_save: TimingSnapshot,
    pub jsonl_serialize: TimingSnapshot,
    pub jsonl_io: TimingSnapshot,
    pub jsonl_bytes: ByteSnapshot,
    pub jsonl_queue_wait: TimingSnapshot,
    pub sqlite_save: TimingSnapshot,
    pub sqlite_append: TimingSnapshot,
    pub sqlite_serialize: TimingSnapshot,
    pub sqlite_bytes: ByteSnapshot,
    pub sqlite_load: TimingSnapshot,
    pub sqlite_load_meta: TimingSnapshot,
    pub index_lock: TimingSnapshot,
    pub index_upsert: TimingSnapshot,
    pub index_list: TimingSnapshot,
    pub index_reindex: TimingSnapshot,
    pub append: TimingSnapshot,
}

// ---------------------------------------------------------------------------
// ScopedTimer
// ---------------------------------------------------------------------------

/// RAII timer that records elapsed microseconds into a [`TimingCounter`]
/// when dropped. If `counter` is `None` (metrics disabled), drop is a no-op.
pub struct ScopedTimer<'a> {
    counter: Option<&'a TimingCounter>,
    start: Instant,
    finished: bool,
}

impl ScopedTimer<'_> {
    /// Manually finish the timer and return elapsed microseconds.
    /// Consumes self so drop won't double-record.
    #[allow(clippy::cast_possible_truncation)]
    pub fn finish(mut self) -> u64 {
        let elapsed_us = self.start.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
        if let Some(counter) = self.counter {
            counter.record(elapsed_us);
        }
        // Prevent drop from recording again.
        self.finished = true;
        elapsed_us
    }
}

impl Drop for ScopedTimer<'_> {
    #[allow(clippy::cast_possible_truncation)]
    fn drop(&mut self) {
        if !self.finished {
            if let Some(counter) = self.counter {
                let elapsed_us = self.start.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
                counter.record(elapsed_us);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Global accessor
// ---------------------------------------------------------------------------

static GLOBAL_METRICS: OnceLock<SessionMetrics> = OnceLock::new();

/// Return the global `SessionMetrics` singleton.
///
/// On first call, reads `PI_PERF_TELEMETRY` to decide whether collection
/// is enabled. The singleton lives for the process lifetime.
pub fn global() -> &'static SessionMetrics {
    GLOBAL_METRICS.get_or_init(|| {
        let metrics = SessionMetrics::new();
        let enabled =
            std::env::var_os("PI_PERF_TELEMETRY").is_some_and(|v| v == "1" || v == "true");
        if enabled {
            metrics.enabled.store(true, Ordering::Relaxed);
        }
        metrics
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timing_counter_starts_at_zero() {
        let counter = TimingCounter::new();
        let snap = counter.snapshot();
        assert_eq!(snap.count, 0);
        assert_eq!(snap.total_us, 0);
        assert_eq!(snap.max_us, 0);
        assert_eq!(snap.avg_us, 0);
    }

    #[test]
    fn timing_counter_records_single_observation() {
        let counter = TimingCounter::new();
        counter.record(500);
        let snap = counter.snapshot();
        assert_eq!(snap.count, 1);
        assert_eq!(snap.total_us, 500);
        assert_eq!(snap.max_us, 500);
        assert_eq!(snap.avg_us, 500);
    }

    #[test]
    fn timing_counter_records_multiple_observations() {
        let counter = TimingCounter::new();
        counter.record(100);
        counter.record(300);
        counter.record(200);
        let snap = counter.snapshot();
        assert_eq!(snap.count, 3);
        assert_eq!(snap.total_us, 600);
        assert_eq!(snap.max_us, 300);
        assert_eq!(snap.avg_us, 200);
    }

    #[test]
    fn timing_counter_max_tracks_peak() {
        let counter = TimingCounter::new();
        counter.record(50);
        counter.record(999);
        counter.record(100);
        assert_eq!(counter.snapshot().max_us, 999);
    }

    #[test]
    fn timing_counter_reset_clears_all() {
        let counter = TimingCounter::new();
        counter.record(100);
        counter.record(200);
        counter.reset();
        let snap = counter.snapshot();
        assert_eq!(snap.count, 0);
        assert_eq!(snap.total_us, 0);
        assert_eq!(snap.max_us, 0);
    }

    #[test]
    fn byte_counter_starts_at_zero() {
        let counter = ByteCounter::new();
        let snap = counter.snapshot();
        assert_eq!(snap.count, 0);
        assert_eq!(snap.total_bytes, 0);
        assert_eq!(snap.avg_bytes, 0);
    }

    #[test]
    fn byte_counter_records_observations() {
        let counter = ByteCounter::new();
        counter.record(1024);
        counter.record(2048);
        let snap = counter.snapshot();
        assert_eq!(snap.count, 2);
        assert_eq!(snap.total_bytes, 3072);
        assert_eq!(snap.avg_bytes, 1536);
    }

    #[test]
    fn byte_counter_reset_clears_all() {
        let counter = ByteCounter::new();
        counter.record(512);
        counter.reset();
        let snap = counter.snapshot();
        assert_eq!(snap.count, 0);
        assert_eq!(snap.total_bytes, 0);
    }

    #[test]
    fn scoped_timer_records_on_drop() {
        let counter = TimingCounter::new();
        {
            let _timer = ScopedTimer {
                counter: Some(&counter),
                start: Instant::now(),
                finished: false,
            };
            // Simulate some work
            std::thread::sleep(std::time::Duration::from_micros(100));
        }
        let snap = counter.snapshot();
        assert_eq!(snap.count, 1);
        assert!(
            snap.total_us > 0,
            "Timer should record nonzero elapsed time"
        );
    }

    #[test]
    fn scoped_timer_finish_returns_elapsed_and_records() {
        let counter = TimingCounter::new();
        let timer = ScopedTimer {
            counter: Some(&counter),
            start: Instant::now(),
            finished: false,
        };
        std::thread::sleep(std::time::Duration::from_micros(100));
        let elapsed = timer.finish();
        assert!(elapsed > 0);
        assert_eq!(counter.snapshot().count, 1);
    }

    #[test]
    fn scoped_timer_noop_when_disabled() {
        let counter = TimingCounter::new();
        {
            let _timer = ScopedTimer {
                counter: None,
                start: Instant::now(),
                finished: false,
            };
        }
        assert_eq!(counter.snapshot().count, 0);
    }

    #[test]
    fn session_metrics_disabled_by_default() {
        let metrics = SessionMetrics::new();
        assert!(!metrics.enabled());
    }

    #[test]
    fn session_metrics_enable_disable() {
        let metrics = SessionMetrics::new();
        metrics.enable();
        assert!(metrics.enabled());
        metrics.disable();
        assert!(!metrics.enabled());
    }

    #[test]
    fn session_metrics_start_timer_noop_when_disabled() {
        let metrics = SessionMetrics::new();
        assert!(!metrics.enabled());
        {
            let _timer = metrics.start_timer(&metrics.sqlite_save);
        }
        assert_eq!(metrics.sqlite_save.snapshot().count, 0);
    }

    #[test]
    fn session_metrics_start_timer_records_when_enabled() {
        let metrics = SessionMetrics::new();
        metrics.enable();
        {
            let _timer = metrics.start_timer(&metrics.sqlite_save);
            std::thread::sleep(std::time::Duration::from_micros(50));
        }
        assert_eq!(metrics.sqlite_save.snapshot().count, 1);
        assert!(metrics.sqlite_save.snapshot().total_us > 0);
    }

    #[test]
    fn session_metrics_record_bytes_noop_when_disabled() {
        let metrics = SessionMetrics::new();
        metrics.record_bytes(&metrics.jsonl_bytes, 1024);
        assert_eq!(metrics.jsonl_bytes.snapshot().count, 0);
    }

    #[test]
    fn session_metrics_record_bytes_when_enabled() {
        let metrics = SessionMetrics::new();
        metrics.enable();
        metrics.record_bytes(&metrics.jsonl_bytes, 1024);
        metrics.record_bytes(&metrics.jsonl_bytes, 2048);
        let snap = metrics.jsonl_bytes.snapshot();
        assert_eq!(snap.count, 2);
        assert_eq!(snap.total_bytes, 3072);
    }

    #[test]
    fn session_metrics_reset_all() {
        let metrics = SessionMetrics::new();
        metrics.enable();
        metrics.sqlite_save.record(100);
        metrics.index_upsert.record(200);
        metrics.jsonl_bytes.record(512);
        metrics.reset_all();
        assert_eq!(metrics.sqlite_save.snapshot().count, 0);
        assert_eq!(metrics.index_upsert.snapshot().count, 0);
        assert_eq!(metrics.jsonl_bytes.snapshot().count, 0);
    }

    #[test]
    fn session_metrics_snapshot_captures_all_counters() {
        let metrics = SessionMetrics::new();
        metrics.enable();
        metrics.sqlite_save.record(100);
        metrics.sqlite_load.record(200);
        metrics.index_lock.record(50);
        metrics.jsonl_bytes.record(4096);
        let snap = metrics.snapshot();
        assert!(snap.enabled);
        assert_eq!(snap.sqlite_save.count, 1);
        assert_eq!(snap.sqlite_load.count, 1);
        assert_eq!(snap.index_lock.count, 1);
        assert_eq!(snap.jsonl_bytes.count, 1);
        assert_eq!(snap.jsonl_bytes.total_bytes, 4096);
    }

    #[test]
    fn session_metrics_summary_disabled() {
        let metrics = SessionMetrics::new();
        let summary = metrics.summary();
        assert!(summary.contains("disabled"));
    }

    #[test]
    fn session_metrics_summary_enabled_contains_all_labels() {
        let metrics = SessionMetrics::new();
        metrics.enable();
        metrics.sqlite_save.record(100);
        let summary = metrics.summary();
        assert!(summary.contains("JSONL save:"));
        assert!(summary.contains("JSONL serialize:"));
        assert!(summary.contains("JSONL IO:"));
        assert!(summary.contains("JSONL bytes:"));
        assert!(summary.contains("JSONL queue wait:"));
        assert!(summary.contains("SQLite save:"));
        assert!(summary.contains("SQLite append:"));
        assert!(summary.contains("SQLite serialize:"));
        assert!(summary.contains("SQLite bytes:"));
        assert!(summary.contains("SQLite load:"));
        assert!(summary.contains("SQLite load meta:"));
        assert!(summary.contains("Index lock:"));
        assert!(summary.contains("Index upsert:"));
        assert!(summary.contains("Index list:"));
        assert!(summary.contains("Index reindex:"));
        assert!(summary.contains("Append:"));
    }

    #[test]
    fn timing_snapshot_display_zero() {
        let snap = TimingSnapshot {
            count: 0,
            total_us: 0,
            max_us: 0,
            avg_us: 0,
        };
        assert_eq!(format!("{snap}"), "n=0");
    }

    #[test]
    fn timing_snapshot_display_nonzero() {
        let snap = TimingSnapshot {
            count: 3,
            total_us: 6000,
            max_us: 3000,
            avg_us: 2000,
        };
        let display = format!("{snap}");
        assert!(display.contains("n=3"));
        assert!(display.contains("avg=2.0ms"));
        assert!(display.contains("max=3.0ms"));
        assert!(display.contains("total=6.0ms"));
    }

    #[test]
    fn byte_snapshot_display_zero() {
        let snap = ByteSnapshot {
            count: 0,
            total_bytes: 0,
            avg_bytes: 0,
        };
        assert_eq!(format!("{snap}"), "n=0");
    }

    #[test]
    fn byte_snapshot_display_nonzero() {
        let snap = ByteSnapshot {
            count: 2,
            total_bytes: 3072,
            avg_bytes: 1536,
        };
        let display = format!("{snap}");
        assert!(display.contains("n=2"));
        assert!(display.contains("avg=1536B"));
        assert!(display.contains("total=3072B"));
    }

    #[test]
    fn global_returns_same_instance() {
        let a = global();
        let b = global();
        assert!(std::ptr::eq(a, b));
    }

    #[test]
    fn timing_counter_concurrent_recording() {
        use std::sync::Arc;

        let counter = Arc::new(TimingCounter::new());
        let threads: Vec<_> = (0..4)
            .map(|_| {
                let c = Arc::clone(&counter);
                std::thread::spawn(move || {
                    for i in 0..100 {
                        c.record(i);
                    }
                })
            })
            .collect();
        for t in threads {
            t.join().expect("thread join");
        }
        let snap = counter.snapshot();
        assert_eq!(snap.count, 400);
        // Total should be 4 * sum(0..100) = 4 * 4950 = 19800
        assert_eq!(snap.total_us, 19800);
        assert_eq!(snap.max_us, 99);
    }

    mod proptest_session_metrics {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            /// After recording n values, count == n and total == sum.
            #[test]
            fn timing_counter_sum_and_count(
                values in prop::collection::vec(0u64..10_000, 0..50)
            ) {
                let counter = TimingCounter::new();
                for &v in &values {
                    counter.record(v);
                }
                let snap = counter.snapshot();
                assert_eq!(snap.count, values.len() as u64);
                assert_eq!(
                    snap.total_us,
                    values.iter().copied().sum::<u64>()
                );
            }

            /// max_us tracks the maximum recorded value.
            #[test]
            fn timing_counter_tracks_max(
                values in prop::collection::vec(0u64..100_000, 1..50)
            ) {
                let counter = TimingCounter::new();
                for &v in &values {
                    counter.record(v);
                }
                let snap = counter.snapshot();
                assert_eq!(snap.max_us, *values.iter().max().unwrap());
            }

            /// avg_us == total_us / count (integer division).
            #[test]
            fn timing_snapshot_avg_is_floor_division(
                values in prop::collection::vec(1u64..10_000, 1..50)
            ) {
                let counter = TimingCounter::new();
                for &v in &values {
                    counter.record(v);
                }
                let snap = counter.snapshot();
                let expected = snap.total_us / snap.count;
                assert_eq!(snap.avg_us, expected);
            }

            /// Empty counter snapshot has all zeros.
            #[test]
            fn empty_counter_snapshot(_dummy in 0..1u8) {
                let counter = TimingCounter::new();
                let snap = counter.snapshot();
                assert_eq!(snap.count, 0);
                assert_eq!(snap.total_us, 0);
                assert_eq!(snap.max_us, 0);
                assert_eq!(snap.avg_us, 0);
            }

            /// After reset, snapshot returns all zeros.
            #[test]
            fn timing_reset_clears(values in prop::collection::vec(1u64..1000, 1..20)) {
                let counter = TimingCounter::new();
                for &v in &values {
                    counter.record(v);
                }
                counter.reset();
                let snap = counter.snapshot();
                assert_eq!(snap.count, 0);
                assert_eq!(snap.total_us, 0);
                assert_eq!(snap.max_us, 0);
            }

            /// `ByteCounter` tracks sum and count correctly.
            #[test]
            fn byte_counter_sum_and_count(
                values in prop::collection::vec(0u64..100_000, 0..50)
            ) {
                let counter = ByteCounter::new();
                for &v in &values {
                    counter.record(v);
                }
                let snap = counter.snapshot();
                assert_eq!(snap.count, values.len() as u64);
                assert_eq!(snap.total_bytes, values.iter().copied().sum::<u64>());
            }

            /// `ByteCounter` avg_bytes is floor division.
            #[test]
            fn byte_counter_avg(
                values in prop::collection::vec(1u64..10_000, 1..50)
            ) {
                let counter = ByteCounter::new();
                for &v in &values {
                    counter.record(v);
                }
                let snap = counter.snapshot();
                assert_eq!(snap.avg_bytes, snap.total_bytes / snap.count);
            }

            /// `ByteCounter` reset clears all.
            #[test]
            fn byte_counter_reset(values in prop::collection::vec(1u64..1000, 1..10)) {
                let counter = ByteCounter::new();
                for &v in &values {
                    counter.record(v);
                }
                counter.reset();
                let snap = counter.snapshot();
                assert_eq!(snap.count, 0);
                assert_eq!(snap.total_bytes, 0);
            }

            /// `TimingSnapshot` display is "n=0" when count == 0.
            #[test]
            fn timing_display_zero(_dummy in 0..1u8) {
                let snap = TimingSnapshot {
                    count: 0,
                    total_us: 0,
                    max_us: 0,
                    avg_us: 0,
                };
                assert_eq!(format!("{snap}"), "n=0");
            }

            /// `TimingSnapshot` display contains all fields when count > 0.
            #[test]
            fn timing_display_nonzero(
                count in 1u64..1000,
                total_us in 1u64..1_000_000,
                max_us in 1u64..1_000_000
            ) {
                let snap = TimingSnapshot {
                    count,
                    total_us,
                    max_us,
                    avg_us: total_us / count,
                };
                let display = format!("{snap}");
                assert!(display.contains(&format!("n={count}")));
                assert!(display.contains("avg="));
                assert!(display.contains("max="));
                assert!(display.contains("total="));
            }

            /// `ByteSnapshot` display is "n=0" when count == 0.
            #[test]
            fn byte_display_zero(_dummy in 0..1u8) {
                let snap = ByteSnapshot {
                    count: 0,
                    total_bytes: 0,
                    avg_bytes: 0,
                };
                assert_eq!(format!("{snap}"), "n=0");
            }

            /// `ByteSnapshot` display contains count and bytes when count > 0.
            #[test]
            fn byte_display_nonzero(
                count in 1u64..1000,
                total in 1u64..1_000_000
            ) {
                let snap = ByteSnapshot {
                    count,
                    total_bytes: total,
                    avg_bytes: total / count,
                };
                let display = format!("{snap}");
                assert!(display.contains(&format!("n={count}")));
                assert!(display.contains("avg="));
                assert!(display.contains("total="));
            }
        }
    }
}
