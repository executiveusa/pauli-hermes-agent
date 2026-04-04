#[cfg(test)]
mod tests {
    use pi::scheduler::{Clock, Scheduler};
    use std::sync::Arc;
    use std::sync::atomic::AtomicU64;

    #[derive(Debug)]
    pub struct DeterministicClock {
        current_ms: AtomicU64,
    }

    impl DeterministicClock {
        pub fn new(start_ms: u64) -> Self {
            Self {
                current_ms: AtomicU64::new(start_ms),
            }
        }
        pub fn advance(&self, ms: u64) {
            self.current_ms
                .fetch_add(ms, std::sync::atomic::Ordering::SeqCst);
        }
    }

    impl Clock for DeterministicClock {
        fn now_ms(&self) -> u64 {
            self.current_ms.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    #[test]
    fn scheduler_invariant_violation_repro() {
        let clock = Arc::new(DeterministicClock::new(0));
        let mut sched = Scheduler::with_clock(clock.clone());

        // 1. Set a timer T1 (will get seq=1). Deadline = 100ms.
        let t1_id = sched.set_timeout(100);
        println!("Set timer T1: id={t1_id}");

        // 2. Enqueue an event E1 (will get seq=2).
        sched.enqueue_event("E1".to_string(), serde_json::json!({}));
        println!("Enqueued event E1");

        // 3. Advance time to 100ms. T1 is now due.
        clock.advance(100);
        println!("Advanced clock to 100ms");

        // 4. Tick 1.
        // Expected (by seq): T1 (seq=1) should execute first.
        // Actual (suspected): E1 (seq=2) is already in queue, T1 is appended. E1 executes.
        let task1 = sched.tick().expect("Should have a task");
        println!("Task 1: {:?}", task1.kind);

        // 5. Tick 2.
        let task2 = sched.tick().expect("Should have a task");
        println!("Task 2: {:?}", task2.kind);

        let seq1 = task1.seq.value();
        let seq2 = task2.seq.value();

        println!("Task 1 Seq: {seq1}");
        println!("Task 2 Seq: {seq2}");

        // Check I5: "all observable scheduling is ordered by seq"
        // This implies task1.seq < task2.seq
        assert!(
            seq1 < seq2,
            "Invariant I5 violation: Task execution not ordered by seq. Executed {seq1} then {seq2}"
        );
    }
}
