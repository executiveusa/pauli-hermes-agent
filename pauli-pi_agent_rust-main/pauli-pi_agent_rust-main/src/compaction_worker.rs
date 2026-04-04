//! Background compaction worker with basic quota controls.
//!
//! This keeps LLM compaction off the foreground turn path by running compaction
//! on the existing runtime and applying results on subsequent turns.

use crate::compaction::{self, CompactionPreparation, CompactionResult};
use crate::error::{Error, Result};
use crate::provider::Provider;
use asupersync::runtime::{JoinHandle, RuntimeHandle};
use futures::FutureExt;
use futures::channel::oneshot;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Quota controls that bound background compaction resource usage.
#[derive(Debug, Clone)]
pub struct CompactionQuota {
    /// Minimum elapsed time between compaction starts.
    pub cooldown: Duration,
    /// Maximum wall-clock time to wait for a background compaction result.
    pub timeout: Duration,
    /// Maximum compaction attempts allowed in a single session.
    pub max_attempts_per_session: u32,
}

impl Default for CompactionQuota {
    fn default() -> Self {
        Self {
            cooldown: Duration::from_secs(60),
            timeout: Duration::from_secs(120),
            max_attempts_per_session: 100,
        }
    }
}

type CompactionOutcome = Result<CompactionResult>;

struct PendingCompaction {
    join: JoinHandle<CompactionOutcome>,
    abort_tx: Option<oneshot::Sender<()>>,
    started_at: Instant,
}

impl PendingCompaction {
    fn is_finished(&self) -> bool {
        self.join.is_finished()
    }

    fn abort(&mut self) {
        if let Some(abort_tx) = self.abort_tx.take() {
            let _ = abort_tx.send(());
        }
    }
}

/// Per-session background compaction state.
pub(crate) struct CompactionWorkerState {
    pending: Option<PendingCompaction>,
    last_start: Option<Instant>,
    attempt_count: u32,
    quota: CompactionQuota,
}

impl CompactionWorkerState {
    pub const fn new(quota: CompactionQuota) -> Self {
        Self {
            pending: None,
            last_start: None,
            attempt_count: 0,
            quota,
        }
    }

    /// Whether a new background compaction is allowed to start now.
    pub fn can_start(&self) -> bool {
        if self.pending.is_some() {
            return false;
        }
        if self.attempt_count >= self.quota.max_attempts_per_session {
            return false;
        }
        if let Some(last) = self.last_start {
            if last.elapsed() < self.quota.cooldown {
                return false;
            }
        }
        true
    }

    /// Non-blocking check for a completed compaction result.
    pub async fn try_recv(&mut self) -> Option<CompactionOutcome> {
        // Check timeout first (read-only borrow, then drop before mutation).
        let timed_out = self
            .pending
            .as_ref()
            .is_some_and(|p| p.started_at.elapsed() > self.quota.timeout);

        if timed_out {
            if let Some(mut pending) = self.pending.take() {
                pending.abort();
            }
            return Some(Err(Error::session(
                "Background compaction timed out".to_string(),
            )));
        }

        if !self
            .pending
            .as_ref()
            .is_some_and(PendingCompaction::is_finished)
        {
            return None;
        }

        let pending = self.pending.take()?;
        Some(pending.join.await)
    }

    /// Spawn a background compaction on the provided runtime.
    pub fn start(
        &mut self,
        runtime_handle: &RuntimeHandle,
        preparation: CompactionPreparation,
        provider: Arc<dyn Provider>,
        api_key: String,
        custom_instructions: Option<String>,
    ) {
        debug_assert!(
            self.can_start(),
            "start() called while can_start() is false"
        );

        let (abort_tx, abort_rx) = oneshot::channel();
        let now = Instant::now();
        let join = runtime_handle.spawn(async move {
            run_compaction_task(
                preparation,
                provider,
                api_key,
                custom_instructions,
                abort_rx,
            )
            .await
        });

        self.pending = Some(PendingCompaction {
            join,
            abort_tx: Some(abort_tx),
            started_at: now,
        });
        self.last_start = Some(now);
        self.attempt_count = self.attempt_count.saturating_add(1);
    }
}

impl Drop for CompactionWorkerState {
    fn drop(&mut self) {
        if let Some(mut pending) = self.pending.take() {
            pending.abort();
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
async fn run_compaction_task(
    preparation: CompactionPreparation,
    provider: Arc<dyn Provider>,
    api_key: String,
    custom_instructions: Option<String>,
    abort_rx: oneshot::Receiver<()>,
) -> CompactionOutcome {
    let abort_fut = async move {
        let _ = abort_rx.await;
        Err(Error::session("Background compaction aborted".to_string()))
    }
    .fuse();
    let compaction_fut = std::panic::AssertUnwindSafe(compaction::compact(
        preparation,
        provider,
        &api_key,
        custom_instructions.as_deref(),
    ))
    .catch_unwind()
    .fuse();

    futures::pin_mut!(abort_fut, compaction_fut);

    match futures::future::select(abort_fut, compaction_fut).await {
        futures::future::Either::Left((abort_result, _)) => abort_result,
        futures::future::Either::Right((Ok(result), _)) => result,
        futures::future::Either::Right((Err(_), _)) => Err(Error::session(
            "Background compaction worker panicked".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    fn make_worker(quota: CompactionQuota) -> CompactionWorkerState {
        CompactionWorkerState::new(quota)
    }

    fn default_worker() -> CompactionWorkerState {
        make_worker(CompactionQuota::default())
    }

    fn run_async<T, F>(make_future: impl FnOnce(RuntimeHandle) -> F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
            .build()
            .expect("build test runtime");
        let runtime_handle = runtime.handle();
        runtime.block_on(make_future(runtime_handle))
    }

    fn inject_pending(worker: &mut CompactionWorkerState, pending: PendingCompaction) {
        worker.pending = Some(pending);
        worker.last_start = Some(Instant::now());
        worker.attempt_count += 1;
    }

    async fn ready_pending_with_handle(
        runtime_handle: RuntimeHandle,
        outcome: CompactionOutcome,
    ) -> PendingCompaction {
        let join = runtime_handle.spawn(async move { outcome });
        PendingCompaction {
            join,
            abort_tx: None,
            started_at: Instant::now(),
        }
    }

    async fn parked_pending_with_handle(
        runtime_handle: RuntimeHandle,
        aborted: Option<Arc<AtomicBool>>,
    ) -> PendingCompaction {
        let (abort_tx, abort_rx) = oneshot::channel();
        let join = runtime_handle.spawn(async move {
            let _ = abort_rx.await;
            if let Some(flag) = aborted {
                flag.store(true, Ordering::SeqCst);
            }
            Err(Error::session("Background compaction aborted".to_string()))
        });
        PendingCompaction {
            join,
            abort_tx: Some(abort_tx),
            started_at: Instant::now(),
        }
    }

    #[test]
    fn fresh_worker_can_start() {
        let w = default_worker();
        assert!(w.can_start());
    }

    #[test]
    fn cannot_start_while_pending() {
        run_async(|runtime_handle| async move {
            let mut w = default_worker();
            let pending = parked_pending_with_handle(runtime_handle, None).await;
            inject_pending(&mut w, pending);
            assert!(!w.can_start());
        });
    }

    #[test]
    fn cannot_start_during_cooldown() {
        let mut w = make_worker(CompactionQuota {
            cooldown: Duration::from_secs(3600),
            ..CompactionQuota::default()
        });
        w.last_start = Some(Instant::now());
        w.attempt_count = 1;
        assert!(!w.can_start());
    }

    #[test]
    fn can_start_after_cooldown() {
        let mut w = make_worker(CompactionQuota {
            cooldown: Duration::from_millis(0),
            ..CompactionQuota::default()
        });
        w.last_start = Some(
            Instant::now()
                .checked_sub(Duration::from_secs(1))
                .unwrap_or_else(Instant::now),
        );
        w.attempt_count = 1;
        assert!(w.can_start());
    }

    #[test]
    fn max_attempts_blocks_start() {
        let mut w = make_worker(CompactionQuota {
            max_attempts_per_session: 2,
            cooldown: Duration::from_millis(0),
            ..CompactionQuota::default()
        });
        w.attempt_count = 2;
        assert!(!w.can_start());
    }

    #[test]
    fn try_recv_none_when_no_pending() {
        run_async(|_runtime_handle| async move {
            let mut w = default_worker();
            assert!(w.try_recv().await.is_none());
        });
    }

    #[test]
    fn try_recv_none_when_not_ready() {
        run_async(|runtime_handle| async move {
            let mut w = default_worker();
            let pending = parked_pending_with_handle(runtime_handle, None).await;
            inject_pending(&mut w, pending);
            // Nothing completed yet.
            assert!(w.try_recv().await.is_none());
            // Pending should still be there.
            assert!(w.pending.is_some());
        });
    }

    #[test]
    fn dropping_worker_aborts_pending_task() {
        run_async(|runtime_handle| async move {
            let aborted = Arc::new(AtomicBool::new(false));
            let mut w = default_worker();
            let pending =
                parked_pending_with_handle(runtime_handle, Some(Arc::clone(&aborted))).await;
            inject_pending(&mut w, pending);

            drop(w);
            asupersync::time::sleep(
                asupersync::time::wall_now(),
                std::time::Duration::from_millis(50),
            )
            .await;

            assert!(
                aborted.load(Ordering::SeqCst),
                "dropping the worker should abort the pending task"
            );
        });
    }

    #[test]
    fn try_recv_timeout() {
        run_async(|runtime_handle| async move {
            let aborted = Arc::new(AtomicBool::new(false));
            let mut w = make_worker(CompactionQuota {
                timeout: Duration::from_millis(0),
                ..CompactionQuota::default()
            });
            let mut pending =
                parked_pending_with_handle(runtime_handle, Some(Arc::clone(&aborted))).await;
            pending.started_at = Instant::now()
                .checked_sub(Duration::from_secs(1))
                .unwrap_or_else(Instant::now);
            inject_pending(&mut w, pending);

            let outcome = w.try_recv().await.expect("should return timeout error");
            assert!(outcome.is_err());
            let err_msg = outcome.unwrap_err().to_string();
            assert!(err_msg.contains("timed out"), "got: {err_msg}");

            asupersync::time::sleep(
                asupersync::time::wall_now(),
                std::time::Duration::from_millis(50),
            )
            .await;
            assert!(
                aborted.load(Ordering::SeqCst),
                "timing out the worker should abort the pending task"
            );
        });
    }

    #[test]
    fn try_recv_success() {
        run_async(|runtime_handle| async move {
            let mut w = default_worker();

            // Simulate a successful compaction result.
            let result = CompactionResult {
                summary: "test summary".to_string(),
                first_kept_entry_id: "entry-1".to_string(),
                tokens_before: 1000,
                details: compaction::CompactionDetails {
                    read_files: vec![],
                    modified_files: vec![],
                },
            };
            let pending = ready_pending_with_handle(runtime_handle, Ok(result)).await;
            inject_pending(&mut w, pending);
            asupersync::time::sleep(
                asupersync::time::wall_now(),
                std::time::Duration::from_millis(50),
            )
            .await;

            let outcome = w.try_recv().await.expect("should have result");
            let result = outcome.expect("should be Ok");
            assert_eq!(result.summary, "test summary");
            assert!(w.pending.is_none());
        });
    }
}
