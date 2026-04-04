use crate::agent_cx::AgentCx;
use crate::error::{Error, Result};
use crate::session::{SessionEntry, SessionHeader};
use crate::session_metrics;
use asupersync::Outcome;
use asupersync::database::{SqliteConnection, SqliteError, SqliteRow, SqliteValue};
use std::path::Path;

const INIT_SQL: &str = r"
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS pi_session_header (
  id TEXT PRIMARY KEY,
  json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS pi_session_entries (
  seq INTEGER PRIMARY KEY,
  json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS pi_session_meta (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
";

#[derive(Debug, Clone)]
pub struct SqliteSessionMeta {
    pub header: SessionHeader,
    pub message_count: u64,
    pub name: Option<String>,
}

fn map_outcome<T>(outcome: Outcome<T, SqliteError>) -> Result<T> {
    match outcome {
        Outcome::Ok(value) => Ok(value),
        Outcome::Err(err) => Err(Error::session(format!("SQLite session error: {err}"))),
        Outcome::Cancelled(_) => Err(Error::Aborted),
        Outcome::Panicked(payload) => Err(Error::session(format!(
            "SQLite session operation panicked: {payload:?}"
        ))),
    }
}

fn row_get_str<'a>(row: &'a SqliteRow, column: &str) -> Result<&'a str> {
    row.get_str(column)
        .map_err(|err| Error::session(format!("SQLite row read failed: {err}")))
}

fn compute_message_count_and_name(entries: &[SessionEntry]) -> (u64, Option<String>) {
    let mut message_count = 0u64;
    let mut name = None;

    for entry in entries {
        match entry {
            SessionEntry::Message(_) => message_count += 1,
            SessionEntry::SessionInfo(info) => {
                if info.name.is_some() {
                    name.clone_from(&info.name);
                }
            }
            _ => {}
        }
    }

    (message_count, name)
}

pub async fn load_session(path: &Path) -> Result<(SessionHeader, Vec<SessionEntry>)> {
    let metrics = session_metrics::global();
    let _timer = metrics.start_timer(&metrics.sqlite_load);

    if !path.exists() {
        return Err(Error::SessionNotFound {
            path: path.display().to_string(),
        });
    }

    let cx = AgentCx::for_request();
    let conn = map_outcome(SqliteConnection::open(cx.cx(), path).await)?;

    let header_rows = map_outcome(
        conn.query(cx.cx(), "SELECT json FROM pi_session_header LIMIT 1", &[])
            .await,
    )?;
    let header_row = header_rows
        .first()
        .ok_or_else(|| Error::session("SQLite session missing header row"))?;
    let header_json = row_get_str(header_row, "json")?;
    let header: SessionHeader = serde_json::from_str(header_json)?;
    header
        .validate()
        .map_err(|reason| Error::session(format!("Invalid session header: {reason}")))?;

    let entry_rows = map_outcome(
        conn.query(
            cx.cx(),
            "SELECT json FROM pi_session_entries ORDER BY seq ASC",
            &[],
        )
        .await,
    )?;

    let mut entries = Vec::with_capacity(entry_rows.len());
    for row in entry_rows {
        let json = row_get_str(&row, "json")?;
        let entry: SessionEntry = serde_json::from_str(json)?;
        entries.push(entry);
    }

    Ok((header, entries))
}

pub async fn load_session_meta(path: &Path) -> Result<SqliteSessionMeta> {
    let metrics = session_metrics::global();
    let _timer = metrics.start_timer(&metrics.sqlite_load_meta);

    if !path.exists() {
        return Err(Error::SessionNotFound {
            path: path.display().to_string(),
        });
    }

    let cx = AgentCx::for_request();
    let conn = map_outcome(SqliteConnection::open(cx.cx(), path).await)?;

    let header_rows = map_outcome(
        conn.query(cx.cx(), "SELECT json FROM pi_session_header LIMIT 1", &[])
            .await,
    )?;
    let header_row = header_rows
        .first()
        .ok_or_else(|| Error::session("SQLite session missing header row"))?;
    let header_json = row_get_str(header_row, "json")?;
    let header: SessionHeader = serde_json::from_str(header_json)?;
    header
        .validate()
        .map_err(|reason| Error::session(format!("Invalid session header: {reason}")))?;

    let meta_rows = match conn
        .query(
            cx.cx(),
            "SELECT key,value FROM pi_session_meta WHERE key IN ('message_count','name')",
            &[],
        )
        .await
    {
        Outcome::Ok(rows) => rows,
        _ => Vec::new(),
    };

    let mut message_count: Option<u64> = None;
    let mut name: Option<String> = None;
    for row in meta_rows {
        let key = row_get_str(&row, "key")?;
        let value = row_get_str(&row, "value")?;
        match key {
            "message_count" => message_count = value.parse::<u64>().ok(),
            "name" => {
                if !value.is_empty() {
                    name = Some(value.to_string());
                }
            }
            _ => {}
        }
    }

    let message_count = if let Some(message_count) = message_count {
        message_count
    } else {
        let entry_rows = map_outcome(
            conn.query(
                cx.cx(),
                "SELECT json FROM pi_session_entries ORDER BY seq ASC",
                &[],
            )
            .await,
        )?;

        let mut entries = Vec::with_capacity(entry_rows.len());
        for row in entry_rows {
            let json = row_get_str(&row, "json")?;
            let entry: SessionEntry = serde_json::from_str(json)?;
            entries.push(entry);
        }

        let (message_count, fallback_name) = compute_message_count_and_name(&entries);
        if name.is_none() {
            name = fallback_name;
        }
        message_count
    };
    Ok(SqliteSessionMeta {
        header,
        message_count,
        name,
    })
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;
    use crate::model::UserContent;
    use crate::session::{EntryBase, MessageEntry, SessionInfoEntry, SessionMessage};

    fn dummy_base() -> EntryBase {
        EntryBase {
            id: Some("test-id".to_string()),
            parent_id: None,
            timestamp: "2026-01-01T00:00:00.000Z".to_string(),
        }
    }

    fn message_entry() -> SessionEntry {
        SessionEntry::Message(MessageEntry {
            base: dummy_base(),
            message: SessionMessage::User {
                content: UserContent::Text("hello".to_string()),
                timestamp: None,
            },
        })
    }

    fn session_info_entry(name: Option<String>) -> SessionEntry {
        SessionEntry::SessionInfo(SessionInfoEntry {
            base: dummy_base(),
            name,
        })
    }

    #[test]
    fn compute_counts_empty() {
        let (count, name) = compute_message_count_and_name(&[]);
        assert_eq!(count, 0);
        assert!(name.is_none());
    }

    #[test]
    fn compute_counts_messages_only() {
        let entries = vec![message_entry(), message_entry(), message_entry()];
        let (count, name) = compute_message_count_and_name(&entries);
        assert_eq!(count, 3);
        assert!(name.is_none());
    }

    #[test]
    fn compute_counts_session_info_with_name() {
        let entries = vec![
            message_entry(),
            session_info_entry(Some("My Session".to_string())),
            message_entry(),
        ];
        let (count, name) = compute_message_count_and_name(&entries);
        assert_eq!(count, 2);
        assert_eq!(name, Some("My Session".to_string()));
    }

    #[test]
    fn compute_counts_session_info_none_name_ignored() {
        let entries = vec![
            session_info_entry(Some("First".to_string())),
            session_info_entry(None),
            message_entry(),
        ];
        let (count, name) = compute_message_count_and_name(&entries);
        assert_eq!(count, 1);
        // The second SessionInfo has name=None, so it doesn't overwrite.
        assert_eq!(name, Some("First".to_string()));
    }

    #[test]
    fn compute_counts_latest_name_wins() {
        let entries = vec![
            session_info_entry(Some("First".to_string())),
            session_info_entry(Some("Second".to_string())),
        ];
        let (_, name) = compute_message_count_and_name(&entries);
        assert_eq!(name, Some("Second".to_string()));
    }

    // -- Non-message / non-session-info entries are ignored --

    #[test]
    fn compute_counts_ignores_model_change_entries() {
        use crate::session::ModelChangeEntry;
        let entries = vec![
            message_entry(),
            SessionEntry::ModelChange(ModelChangeEntry {
                base: dummy_base(),
                provider: "anthropic".to_string(),
                model_id: "claude-sonnet-4-5".to_string(),
            }),
            message_entry(),
        ];
        let (count, name) = compute_message_count_and_name(&entries);
        assert_eq!(count, 2);
        assert!(name.is_none());
    }

    #[test]
    fn compute_counts_ignores_label_entries() {
        use crate::session::LabelEntry;
        let entries = vec![
            message_entry(),
            SessionEntry::Label(LabelEntry {
                base: dummy_base(),
                target_id: "some-id".to_string(),
                label: Some("important".to_string()),
            }),
        ];
        let (count, name) = compute_message_count_and_name(&entries);
        assert_eq!(count, 1);
        assert!(name.is_none());
    }

    #[test]
    fn compute_counts_ignores_custom_entries() {
        use crate::session::CustomEntry;
        let entries = vec![
            SessionEntry::Custom(CustomEntry {
                base: dummy_base(),
                custom_type: "my_custom".to_string(),
                data: Some(serde_json::json!({"key": "value"})),
            }),
            message_entry(),
        ];
        let (count, name) = compute_message_count_and_name(&entries);
        assert_eq!(count, 1);
        assert!(name.is_none());
    }

    #[test]
    fn compute_counts_ignores_compaction_entries() {
        use crate::session::CompactionEntry;
        let entries = vec![
            message_entry(),
            SessionEntry::Compaction(CompactionEntry {
                base: dummy_base(),
                summary: "summary text".to_string(),
                first_kept_entry_id: "e1".to_string(),
                tokens_before: 500,
                details: None,
                from_hook: None,
            }),
            message_entry(),
            message_entry(),
        ];
        let (count, name) = compute_message_count_and_name(&entries);
        assert_eq!(count, 3);
        assert!(name.is_none());
    }

    #[test]
    fn compute_counts_mixed_entry_types() {
        use crate::session::{CompactionEntry, CustomEntry, LabelEntry, ModelChangeEntry};
        let entries = vec![
            message_entry(),
            SessionEntry::ModelChange(ModelChangeEntry {
                base: dummy_base(),
                provider: "openai".to_string(),
                model_id: "gpt-4".to_string(),
            }),
            session_info_entry(Some("Named".to_string())),
            SessionEntry::Label(LabelEntry {
                base: dummy_base(),
                target_id: "t1".to_string(),
                label: None,
            }),
            message_entry(),
            SessionEntry::Compaction(CompactionEntry {
                base: dummy_base(),
                summary: "s".to_string(),
                first_kept_entry_id: "e1".to_string(),
                tokens_before: 100,
                details: None,
                from_hook: None,
            }),
            SessionEntry::Custom(CustomEntry {
                base: dummy_base(),
                custom_type: "ct".to_string(),
                data: None,
            }),
            message_entry(),
        ];
        let (count, name) = compute_message_count_and_name(&entries);
        assert_eq!(count, 3);
        assert_eq!(name, Some("Named".to_string()));
    }

    // -- map_outcome tests --

    #[test]
    fn map_outcome_ok() {
        let outcome: Outcome<i32, SqliteError> = Outcome::Ok(42);
        let result = map_outcome(outcome);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn map_outcome_err() {
        let outcome: Outcome<i32, SqliteError> = Outcome::Err(SqliteError::ConnectionClosed);
        let result = map_outcome(outcome);
        let err = result.unwrap_err();
        match err {
            Error::Session(message) => {
                assert!(message.contains("SQLite session error"));
            }
            other => unreachable!("Unexpected error: {:?}", other),
        }
    }

    #[test]
    fn map_outcome_cancelled() {
        use asupersync::types::CancelKind;
        let reason = asupersync::CancelReason::new(CancelKind::User);
        let outcome: Outcome<i32, SqliteError> = Outcome::Cancelled(reason);
        let result = map_outcome(outcome);
        assert!(matches!(result.unwrap_err(), Error::Aborted));
    }

    #[test]
    fn map_outcome_panicked() {
        use asupersync::types::PanicPayload;
        let outcome: Outcome<i32, SqliteError> = Outcome::Panicked(PanicPayload::new("test panic"));
        let result = map_outcome(outcome);
        let err = result.unwrap_err();
        match err {
            Error::Session(message) => {
                assert!(message.contains("panicked"));
            }
            other => unreachable!("Unexpected error: {:?}", other),
        }
    }

    // -- SqliteSessionMeta struct --

    #[test]
    fn sqlite_session_meta_fields() {
        let meta = SqliteSessionMeta {
            header: SessionHeader {
                id: "test-session".to_string(),
                ..SessionHeader::default()
            },
            message_count: 42,
            name: Some("My Session".to_string()),
        };
        assert_eq!(meta.header.id, "test-session");
        assert_eq!(meta.message_count, 42);
        assert_eq!(meta.name.as_deref(), Some("My Session"));
    }

    #[test]
    fn sqlite_session_meta_no_name() {
        let meta = SqliteSessionMeta {
            header: SessionHeader::default(),
            message_count: 0,
            name: None,
        };
        assert_eq!(meta.message_count, 0);
        assert!(meta.name.is_none());
    }

    // -- compute_message_count_and_name: large input --

    #[test]
    fn compute_counts_large_message_set() {
        let entries: Vec<SessionEntry> = (0..1000).map(|_| message_entry()).collect();
        let (count, name) = compute_message_count_and_name(&entries);
        assert_eq!(count, 1000);
        assert!(name.is_none());
    }

    // -- compute_message_count_and_name: name then messages only --

    #[test]
    fn compute_counts_name_set_early_persists() {
        let entries = vec![
            session_info_entry(Some("Early Name".to_string())),
            message_entry(),
            message_entry(),
            message_entry(),
        ];
        let (count, name) = compute_message_count_and_name(&entries);
        assert_eq!(count, 3);
        assert_eq!(name, Some("Early Name".to_string()));
    }

    // -- compute_message_count_and_name: branch summary entry --

    #[test]
    fn compute_counts_ignores_branch_summary() {
        use crate::session::BranchSummaryEntry;
        let entries = vec![
            message_entry(),
            SessionEntry::BranchSummary(BranchSummaryEntry {
                base: dummy_base(),
                from_id: "parent-id".to_string(),
                summary: "branch summary".to_string(),
                details: None,
                from_hook: None,
            }),
        ];
        let (count, name) = compute_message_count_and_name(&entries);
        assert_eq!(count, 1);
        assert!(name.is_none());
    }

    // -- compute_message_count_and_name: thinking level change --

    #[test]
    fn compute_counts_ignores_thinking_level_change() {
        use crate::session::ThinkingLevelChangeEntry;
        let entries = vec![
            SessionEntry::ThinkingLevelChange(ThinkingLevelChangeEntry {
                base: dummy_base(),
                thinking_level: "high".to_string(),
            }),
            message_entry(),
        ];
        let (count, name) = compute_message_count_and_name(&entries);
        assert_eq!(count, 1);
        assert!(name.is_none());
    }

    #[test]
    fn save_session_rejects_semantically_invalid_header() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("invalid.sqlite");
        let header = SessionHeader {
            r#type: "note".to_string(),
            ..SessionHeader::default()
        };

        let err = futures::executor::block_on(async { save_session(&path, &header, &[]).await })
            .expect_err("invalid header should fail");
        let message = err.to_string();
        assert!(
            message.contains("Invalid session header"),
            "expected invalid session header error, got {message}"
        );
    }

    #[test]
    fn load_session_meta_rejects_semantically_invalid_header() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("invalid.sqlite");
        let header = SessionHeader {
            id: "sqlite-test".to_string(),
            ..SessionHeader::default()
        };

        futures::executor::block_on(async { save_session(&path, &header, &[]).await })
            .expect("save sqlite session");

        let invalid_header = SessionHeader {
            r#type: "note".to_string(),
            ..header
        };
        let invalid_json =
            serde_json::to_string(&invalid_header).expect("serialize invalid session header");
        let config = sqlmodel_sqlite::SqliteConfig::file(path.to_string_lossy())
            .flags(sqlmodel_sqlite::OpenFlags::create_read_write());
        let conn = sqlmodel_sqlite::SqliteConnection::open(&config).expect("open sqlite db");
        conn.execute_sync(
            "UPDATE pi_session_header SET json = ?1",
            &[sqlmodel_core::Value::Text(invalid_json)],
        )
        .expect("corrupt sqlite header row");

        let err = futures::executor::block_on(async { load_session_meta(&path).await })
            .expect_err("invalid header should fail");
        let message = err.to_string();
        assert!(
            message.contains("Invalid session header"),
            "expected invalid session header error, got {message}"
        );
    }
}

pub async fn save_session(
    path: &Path,
    header: &SessionHeader,
    entries: &[SessionEntry],
) -> Result<()> {
    header
        .validate()
        .map_err(|reason| Error::session(format!("Invalid session header: {reason}")))?;
    let metrics = session_metrics::global();
    let _save_timer = metrics.start_timer(&metrics.sqlite_save);

    if let Some(parent) = path.parent() {
        asupersync::fs::create_dir_all(parent).await?;
    }

    let cx = AgentCx::for_request();
    let conn = map_outcome(SqliteConnection::open(cx.cx(), path).await)?;
    map_outcome(conn.execute_batch(cx.cx(), INIT_SQL).await)?;

    let tx = map_outcome(conn.begin_immediate(cx.cx()).await)?;

    map_outcome(
        tx.execute(cx.cx(), "DELETE FROM pi_session_entries", &[])
            .await,
    )?;
    map_outcome(
        tx.execute(cx.cx(), "DELETE FROM pi_session_header", &[])
            .await,
    )?;
    map_outcome(
        tx.execute(cx.cx(), "DELETE FROM pi_session_meta", &[])
            .await,
    )?;

    // Serialize header + entries and track serialization time + bytes.
    let serialize_timer = metrics.start_timer(&metrics.sqlite_serialize);
    let header_json = serde_json::to_string(header)?;
    let mut total_json_bytes = header_json.len() as u64;

    let mut entry_jsons = Vec::with_capacity(entries.len());
    for entry in entries {
        let json = serde_json::to_string(entry)?;
        total_json_bytes += json.len() as u64;
        entry_jsons.push(json);
    }
    serialize_timer.finish();
    metrics.record_bytes(&metrics.sqlite_bytes, total_json_bytes);

    map_outcome(
        tx.execute(
            cx.cx(),
            "INSERT INTO pi_session_header (id,json) VALUES (?1,?2)",
            &[
                SqliteValue::Text(header.id.clone()),
                SqliteValue::Text(header_json),
            ],
        )
        .await,
    )?;

    for (idx, json) in entry_jsons.into_iter().enumerate() {
        map_outcome(
            tx.execute(
                cx.cx(),
                "INSERT INTO pi_session_entries (seq,json) VALUES (?1,?2)",
                &[
                    SqliteValue::Integer(i64::try_from(idx + 1).unwrap_or(i64::MAX)),
                    SqliteValue::Text(json),
                ],
            )
            .await,
        )?;
    }

    let (message_count, name) = compute_message_count_and_name(entries);
    map_outcome(
        tx.execute(
            cx.cx(),
            "INSERT INTO pi_session_meta (key,value) VALUES (?1,?2)",
            &[
                SqliteValue::Text("message_count".to_string()),
                SqliteValue::Text(message_count.to_string()),
            ],
        )
        .await,
    )?;
    let name_value = name.unwrap_or_else(String::new);
    map_outcome(
        tx.execute(
            cx.cx(),
            "INSERT INTO pi_session_meta (key,value) VALUES (?1,?2)",
            &[
                SqliteValue::Text("name".to_string()),
                SqliteValue::Text(name_value),
            ],
        )
        .await,
    )?;

    map_outcome(tx.commit(cx.cx()).await)?;
    Ok(())
}

/// Incrementally append new entries to an existing SQLite session database.
///
/// Only the entries in `new_entries` (starting at 1-based sequence `start_seq`)
/// are inserted. The header row is left unchanged, while the `message_count`
/// and `name` meta rows are upserted to reflect the current totals.
///
/// This avoids the DELETE+reinsert cost of [`save_session`] for the common
/// case where a few entries are appended between saves.
pub async fn append_entries(
    path: &Path,
    new_entries: &[SessionEntry],
    start_seq: usize,
    message_count: u64,
    session_name: Option<&str>,
) -> Result<()> {
    let metrics = session_metrics::global();
    let _timer = metrics.start_timer(&metrics.sqlite_append);

    let cx = AgentCx::for_request();
    let conn = map_outcome(SqliteConnection::open(cx.cx(), path).await)?;

    // Ensure WAL mode is active and tables exist (especially pi_session_meta for old DBs).
    map_outcome(conn.execute_batch(cx.cx(), INIT_SQL).await)?;

    let tx = map_outcome(conn.begin_immediate(cx.cx()).await)?;

    // Serialize and insert only the new entries.
    let serialize_timer = metrics.start_timer(&metrics.sqlite_serialize);
    let mut total_json_bytes = 0u64;
    let mut entry_jsons = Vec::with_capacity(new_entries.len());
    for entry in new_entries {
        let json = serde_json::to_string(entry)?;
        total_json_bytes += json.len() as u64;
        entry_jsons.push(json);
    }
    serialize_timer.finish();
    metrics.record_bytes(&metrics.sqlite_bytes, total_json_bytes);

    for (i, json) in entry_jsons.into_iter().enumerate() {
        let seq = start_seq + i + 1; // 1-based
        map_outcome(
            tx.execute(
                cx.cx(),
                "INSERT INTO pi_session_entries (seq,json) VALUES (?1,?2)",
                &[
                    SqliteValue::Integer(i64::try_from(seq).unwrap_or(i64::MAX)),
                    SqliteValue::Text(json),
                ],
            )
            .await,
        )?;
    }

    // Upsert meta counters (INSERT OR REPLACE).
    map_outcome(
        tx.execute(
            cx.cx(),
            "INSERT OR REPLACE INTO pi_session_meta (key,value) VALUES (?1,?2)",
            &[
                SqliteValue::Text("message_count".to_string()),
                SqliteValue::Text(message_count.to_string()),
            ],
        )
        .await,
    )?;
    let name_value = session_name.unwrap_or("");
    map_outcome(
        tx.execute(
            cx.cx(),
            "INSERT OR REPLACE INTO pi_session_meta (key,value) VALUES (?1,?2)",
            &[
                SqliteValue::Text("name".to_string()),
                SqliteValue::Text(name_value.to_string()),
            ],
        )
        .await,
    )?;

    map_outcome(tx.commit(cx.cx()).await)?;
    Ok(())
}
