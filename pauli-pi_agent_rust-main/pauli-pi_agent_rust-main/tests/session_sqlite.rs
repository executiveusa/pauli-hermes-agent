#![cfg(feature = "sqlite-sessions")]

use asupersync::runtime::RuntimeBuilder;
use pi::model::UserContent;
use pi::session::{Session, SessionMessage, SessionStoreKind};
use std::future::Future;
use std::path::Path;

fn make_test_message(text: &str) -> SessionMessage {
    SessionMessage::User {
        content: UserContent::Text(text.to_string()),
        timestamp: Some(0),
    }
}

fn run_async<T>(future: impl Future<Output = T>) -> T {
    let runtime = RuntimeBuilder::current_thread()
        .build()
        .expect("build runtime");
    runtime.block_on(future)
}

#[test]
fn sqlite_session_round_trip_smoke() {
    let temp = tempfile::tempdir().expect("tempdir");
    let mut session = Session::create_with_dir_and_store(
        Some(temp.path().to_path_buf()),
        SessionStoreKind::Sqlite,
    );

    let id1 = session.append_message(make_test_message("Hello"));
    let id2 = session.append_message(make_test_message("World"));
    assert_eq!(session.leaf_id(), Some(id2.as_str()));

    run_async(async { session.save().await }).expect("save session");
    let path = session
        .path
        .as_ref()
        .expect("session path")
        .to_string_lossy()
        .to_string();
    assert!(
        Path::new(&path)
            .extension()
            .is_some_and(|ext| ext == "sqlite")
    );

    let loaded = run_async(async { Session::open(path.as_str()).await }).expect("open session");

    assert_eq!(loaded.header.id, session.header.id);
    assert_eq!(loaded.entries.len(), session.entries.len());
    assert_eq!(loaded.leaf_id(), Some(id2.as_str()));
    assert!(
        loaded
            .entries
            .iter()
            .any(|entry| entry.base_id() == Some(&id1))
    );
}

#[test]
fn migrate_jsonl_to_sqlite_round_trip_smoke() {
    let temp = tempfile::tempdir().expect("tempdir");

    let mut jsonl = Session::create_with_dir_and_store(
        Some(temp.path().to_path_buf()),
        SessionStoreKind::Jsonl,
    );
    jsonl.append_message(make_test_message("A"));
    jsonl.append_message(make_test_message("B"));
    run_async(async { jsonl.save().await }).expect("save jsonl");
    let jsonl_path = jsonl
        .path
        .as_ref()
        .expect("jsonl path")
        .to_string_lossy()
        .to_string();

    let loaded = run_async(async { Session::open(jsonl_path.as_str()).await }).expect("open jsonl");

    let mut sqlite = Session::create_with_dir_and_store(
        Some(temp.path().to_path_buf()),
        SessionStoreKind::Sqlite,
    );
    sqlite.header = loaded.header.clone();
    sqlite.entries = loaded.entries.clone();
    sqlite._test_set_leaf_id(loaded.leaf_id().map(String::from));

    run_async(async { sqlite.save().await }).expect("save sqlite");
    let sqlite_path = sqlite
        .path
        .as_ref()
        .expect("sqlite path")
        .to_string_lossy()
        .to_string();

    let reopened =
        run_async(async { Session::open(sqlite_path.as_str()).await }).expect("open sqlite");

    assert_eq!(reopened.header.id, loaded.header.id);
    assert_eq!(reopened.entries.len(), loaded.entries.len());
}
