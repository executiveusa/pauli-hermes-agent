#[path = "bench_env.rs"]
mod bench_env;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use pi::model::UserContent;
use pi::session::{Session, SessionMessage};
use std::hint::black_box;

fn build_large_session(message_count: usize) -> Session {
    let mut session = Session::in_memory();
    let content = "a".repeat(1000);

    for _ in 0..message_count {
        session.append_message(SessionMessage::User {
            content: UserContent::Text(content.clone()),
            timestamp: Some(1_234_567_890),
        });
    }

    session
}

fn bench_session_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_clone");
    for count in [100, 1_000, 10_000] {
        let session = build_large_session(count);
        group.throughput(Throughput::Elements(count as u64));
        group.bench_function(BenchmarkId::new("clone_entries", count), |b| {
            b.iter(|| black_box(session.entries.clone()));
        });
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = bench_env::criterion_config();
    targets = bench_session_clone,
}
criterion_main!(benches);
