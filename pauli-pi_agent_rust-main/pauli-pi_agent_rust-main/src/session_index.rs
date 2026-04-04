//! SQLite session index (derived from JSONL sessions).

use crate::config::Config;
use crate::error::{Error, Result};
use crate::session::{Session, SessionEntry, SessionHeader};
use fs4::fs_std::FileExt;
use serde::Deserialize;
use sqlmodel_core::Value;
use sqlmodel_sqlite::{OpenFlags, SqliteConfig, SqliteConnection};
use std::borrow::Borrow;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct SessionMeta {
    pub path: String,
    pub id: String,
    pub cwd: String,
    pub timestamp: String,
    pub message_count: u64,
    pub last_modified_ms: i64,
    pub size_bytes: u64,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SessionIndex {
    db_path: PathBuf,
    lock_path: PathBuf,
}

impl SessionIndex {
    pub fn new() -> Self {
        let root = Config::sessions_dir();
        Self::for_sessions_root(&root)
    }

    pub fn for_sessions_root(root: &Path) -> Self {
        Self {
            db_path: root.join("session-index.sqlite"),
            lock_path: root.join("session-index.lock"),
        }
    }

    pub fn index_session(&self, session: &Session) -> Result<()> {
        let Some(path) = session.path.as_ref() else {
            return Ok(());
        };

        let meta = build_meta(path, &session.header, &session.entries)?;
        self.upsert_meta(meta)
    }

    /// Update index metadata for an already-persisted session snapshot.
    ///
    /// This avoids requiring a full `Session` clone when callers already have
    /// header + aggregate entry stats.
    pub fn index_session_snapshot(
        &self,
        path: &Path,
        header: &SessionHeader,
        message_count: u64,
        name: Option<String>,
    ) -> Result<()> {
        let (last_modified_ms, size_bytes) = session_file_stats(path)?;
        let meta = SessionMeta {
            path: path.display().to_string(),
            id: header.id.clone(),
            cwd: header.cwd.clone(),
            timestamp: header.timestamp.clone(),
            message_count,
            last_modified_ms,
            size_bytes,
            name,
        };
        self.upsert_meta(meta)
    }

    pub(crate) fn upsert_session_meta(&self, meta: SessionMeta) -> Result<()> {
        self.upsert_meta(meta)
    }

    fn upsert_meta(&self, meta: SessionMeta) -> Result<()> {
        self.with_lock(|conn| {
            init_schema(conn)?;

            conn.execute_raw("BEGIN IMMEDIATE")
                .map_err(|e| Error::session(format!("BEGIN failed: {e}")))?;

            let result = (|| -> Result<()> {
                let message_count = sqlite_i64_from_u64("message_count", meta.message_count)?;
                let size_bytes = sqlite_i64_from_u64("size_bytes", meta.size_bytes)?;
                conn.execute_sync(
                    "INSERT INTO sessions (path,id,cwd,timestamp,message_count,last_modified_ms,size_bytes,name)
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
                     ON CONFLICT(path) DO UPDATE SET
                       id=excluded.id,
                       cwd=excluded.cwd,
                       timestamp=excluded.timestamp,
                       message_count=excluded.message_count,
                       last_modified_ms=excluded.last_modified_ms,
                       size_bytes=excluded.size_bytes,
                       name=excluded.name",
                    &[
                        Value::Text(meta.path),
                        Value::Text(meta.id),
                        Value::Text(meta.cwd),
                        Value::Text(meta.timestamp),
                        Value::BigInt(message_count),
                        Value::BigInt(meta.last_modified_ms),
                        Value::BigInt(size_bytes),
                        meta.name.map_or(Value::Null, Value::Text),
                    ],
                ).map_err(|e| Error::session(format!("Insert failed: {e}")))?;

                conn.execute_sync(
                    "INSERT INTO meta (key,value) VALUES ('last_sync_epoch_ms', ?1)
                     ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                    &[Value::Text(current_epoch_ms())],
                ).map_err(|e| Error::session(format!("Meta update failed: {e}")))?;
                Ok(())
            })();

            match result {
                Ok(()) => {
                    conn.execute_raw("COMMIT")
                        .map_err(|e| Error::session(format!("COMMIT failed: {e}")))?;
                    Ok(())
                }
                Err(e) => {
                    let _ = conn.execute_raw("ROLLBACK");
                    Err(e)
                }
            }
        })
    }

    pub fn list_sessions(&self, cwd: Option<&str>) -> Result<Vec<SessionMeta>> {
        self.with_lock(|conn| {
            init_schema(conn)?;

            let (sql, params): (&str, Vec<Value>) = cwd.map_or_else(
                || {
                    (
                        "SELECT path,id,cwd,timestamp,message_count,last_modified_ms,size_bytes,name
                         FROM sessions ORDER BY last_modified_ms DESC",
                        vec![],
                    )
                },
                |cwd| {
                    (
                        "SELECT path,id,cwd,timestamp,message_count,last_modified_ms,size_bytes,name
                         FROM sessions WHERE cwd=?1 ORDER BY last_modified_ms DESC",
                        vec![Value::Text(cwd.to_string())],
                    )
                },
            );

            let rows = conn
                .query_sync(sql, &params)
                .map_err(|e| Error::session(format!("Query failed: {e}")))?;

            let mut result = Vec::new();
            for row in rows {
                result.push(row_to_meta(&row)?);
            }
            Ok(result)
        })
    }

    pub fn delete_session_path(&self, path: &Path) -> Result<()> {
        let path = path.to_string_lossy().to_string();
        self.with_lock(|conn| {
            init_schema(conn)?;

            conn.execute_raw("BEGIN IMMEDIATE")
                .map_err(|e| Error::session(format!("BEGIN failed: {e}")))?;

            let result = (|| -> Result<()> {
                conn.execute_sync("DELETE FROM sessions WHERE path=?1", &[Value::Text(path)])
                    .map_err(|e| Error::session(format!("Delete failed: {e}")))?;

                conn.execute_sync(
                    "INSERT INTO meta (key,value) VALUES ('last_sync_epoch_ms', ?1)
                     ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                    &[Value::Text(current_epoch_ms())],
                )
                .map_err(|e| Error::session(format!("Meta update failed: {e}")))?;
                Ok(())
            })();

            match result {
                Ok(()) => {
                    conn.execute_raw("COMMIT")
                        .map_err(|e| Error::session(format!("COMMIT failed: {e}")))?;
                    Ok(())
                }
                Err(e) => {
                    let _ = conn.execute_raw("ROLLBACK");
                    Err(e)
                }
            }
        })
    }

    pub fn reindex_all(&self) -> Result<()> {
        let sessions_root = self.sessions_root();
        if !sessions_root.exists() {
            return Ok(());
        }

        let mut metas = Vec::new();
        for entry in walk_sessions(sessions_root) {
            let Ok(path) = entry else { continue };
            if let Ok(meta) = build_meta_from_file(&path) {
                metas.push(meta);
            }
        }

        self.with_lock(|conn| {
            init_schema(conn)?;

            conn.execute_raw("BEGIN IMMEDIATE")
                .map_err(|e| Error::session(format!("BEGIN failed: {e}")))?;

            let result = (|| -> Result<()> {
                conn.execute_sync("DELETE FROM sessions", &[])
                    .map_err(|e| Error::session(format!("Delete failed: {e}")))?;

                for meta in metas {
                    let message_count = sqlite_i64_from_u64("message_count", meta.message_count)?;
                    let size_bytes = sqlite_i64_from_u64("size_bytes", meta.size_bytes)?;
                    conn.execute_sync(
                        "INSERT INTO sessions (path,id,cwd,timestamp,message_count,last_modified_ms,size_bytes,name)
                         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                        &[
                            Value::Text(meta.path),
                            Value::Text(meta.id),
                            Value::Text(meta.cwd),
                            Value::Text(meta.timestamp),
                            Value::BigInt(message_count),
                            Value::BigInt(meta.last_modified_ms),
                            Value::BigInt(size_bytes),
                            meta.name.map_or(Value::Null, Value::Text),
                        ],
                    ).map_err(|e| Error::session(format!("Insert failed: {e}")))?;
                }

                conn.execute_sync(
                    "INSERT INTO meta (key,value) VALUES ('last_sync_epoch_ms', ?1)
                     ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                    &[Value::Text(current_epoch_ms())],
                ).map_err(|e| Error::session(format!("Meta update failed: {e}")))?;

                Ok(())
            })();

            match result {
                Ok(()) => {
                    conn.execute_raw("COMMIT")
                        .map_err(|e| Error::session(format!("COMMIT failed: {e}")))?;
                    Ok(())
                }
                Err(e) => {
                    let _ = conn.execute_raw("ROLLBACK");
                    Err(e)
                }
            }
        })
    }

    /// Check whether the on-disk index is stale enough to reindex.
    pub fn should_reindex(&self, max_age: Duration) -> bool {
        if !self.db_path.exists() {
            return true;
        }
        // Prefer the persisted sync epoch over the main SQLite file mtime.
        // In WAL mode, recent writes can live in the sidecar files while the
        // base database timestamp stays old enough to look stale.
        if let Ok(Some(last_sync_epoch_ms)) = self.with_lock(load_last_sync_epoch_ms) {
            return epoch_ms_is_stale(last_sync_epoch_ms, max_age);
        }
        let Ok(meta) = fs::metadata(&self.db_path) else {
            return true;
        };
        let Ok(modified) = meta.modified() else {
            return true;
        };
        let age = SystemTime::now()
            .duration_since(modified)
            .unwrap_or_default();
        age > max_age
    }

    /// Reindex the session database if the index is stale.
    pub fn reindex_if_stale(&self, max_age: Duration) -> Result<bool> {
        if !self.should_reindex(max_age) {
            return Ok(false);
        }
        self.reindex_all()?;
        Ok(true)
    }

    fn with_lock<T>(&self, f: impl FnOnce(&SqliteConnection) -> Result<T>) -> Result<T> {
        if let Some(parent) = self.db_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let lock_file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&self.lock_path)?;
        let _lock = lock_file_guard(&lock_file, Duration::from_secs(5))?;

        let config = SqliteConfig::file(self.db_path.to_string_lossy())
            .flags(OpenFlags::create_read_write())
            .busy_timeout(5000);

        let conn = SqliteConnection::open(&config)
            .map_err(|e| Error::session(format!("SQLite open: {e}")))?;

        // Set pragmas for performance
        conn.execute_raw("PRAGMA journal_mode = WAL")
            .map_err(|e| Error::session(format!("PRAGMA journal_mode: {e}")))?;
        conn.execute_raw("PRAGMA synchronous = NORMAL")
            .map_err(|e| Error::session(format!("PRAGMA synchronous: {e}")))?;
        conn.execute_raw("PRAGMA wal_autocheckpoint = 1000")
            .map_err(|e| Error::session(format!("PRAGMA wal_autocheckpoint: {e}")))?;
        conn.execute_raw("PRAGMA foreign_keys = ON")
            .map_err(|e| Error::session(format!("PRAGMA foreign_keys: {e}")))?;

        f(&conn)
    }

    fn sessions_root(&self) -> &Path {
        self.db_path.parent().unwrap_or_else(|| Path::new("."))
    }
}

impl Default for SessionIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Queue (currently immediate) index update for a persisted session snapshot.
///
/// Callers use this helper from save paths where index freshness is
/// best-effort and must not fail the underlying session write.
pub(crate) fn enqueue_session_index_snapshot_update(
    sessions_root: &Path,
    path: &Path,
    header: &SessionHeader,
    message_count: u64,
    name: Option<String>,
) {
    if let Err(err) = SessionIndex::for_sessions_root(sessions_root).index_session_snapshot(
        path,
        header,
        message_count,
        name,
    ) {
        tracing::warn!(
            sessions_root = %sessions_root.display(),
            path = %path.display(),
            error = %err,
            "Failed to update session index snapshot"
        );
    }
}

fn init_schema(conn: &SqliteConnection) -> Result<()> {
    conn.execute_raw(
        "CREATE TABLE IF NOT EXISTS sessions (
            path TEXT PRIMARY KEY,
            id TEXT NOT NULL,
            cwd TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            message_count INTEGER NOT NULL,
            last_modified_ms INTEGER NOT NULL,
            size_bytes INTEGER NOT NULL,
            name TEXT
        )",
    )
    .map_err(|e| Error::session(format!("Create sessions table: {e}")))?;

    conn.execute_raw(
        "CREATE TABLE IF NOT EXISTS meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
    )
    .map_err(|e| Error::session(format!("Create meta table: {e}")))?;

    Ok(())
}

fn sqlite_i64_from_u64(field: &str, value: u64) -> Result<i64> {
    i64::try_from(value)
        .map_err(|_| Error::session(format!("{field} exceeds SQLite INTEGER range: {value}")))
}

fn sqlite_u64_from_i64(field: &str, value: i64) -> Result<u64> {
    u64::try_from(value).map_err(|_| {
        Error::session(format!(
            "{field} must be non-negative in session index: {value}"
        ))
    })
}

fn row_to_meta(row: &sqlmodel_core::Row) -> Result<SessionMeta> {
    let message_count = row
        .get_named::<i64>("message_count")
        .map_err(|e| Error::session(format!("get message_count: {e}")))?;
    let size_bytes = row
        .get_named::<i64>("size_bytes")
        .map_err(|e| Error::session(format!("get size_bytes: {e}")))?;

    Ok(SessionMeta {
        path: row
            .get_named("path")
            .map_err(|e| Error::session(format!("get path: {e}")))?,
        id: row
            .get_named("id")
            .map_err(|e| Error::session(format!("get id: {e}")))?,
        cwd: row
            .get_named("cwd")
            .map_err(|e| Error::session(format!("get cwd: {e}")))?,
        timestamp: row
            .get_named("timestamp")
            .map_err(|e| Error::session(format!("get timestamp: {e}")))?,
        message_count: sqlite_u64_from_i64("message_count", message_count)?,
        last_modified_ms: row
            .get_named("last_modified_ms")
            .map_err(|e| Error::session(format!("get last_modified_ms: {e}")))?,
        size_bytes: sqlite_u64_from_i64("size_bytes", size_bytes)?,
        name: row
            .get_named::<Option<String>>("name")
            .map_err(|e| Error::session(format!("get name: {e}")))?,
    })
}

fn build_meta(
    path: &Path,
    header: &SessionHeader,
    entries: &[SessionEntry],
) -> Result<SessionMeta> {
    header
        .validate()
        .map_err(|reason| Error::session(format!("Invalid session header: {reason}")))?;
    let (message_count, name) = session_stats(entries);
    let (last_modified_ms, size_bytes) = session_file_stats(path)?;
    Ok(SessionMeta {
        path: path.display().to_string(),
        id: header.id.clone(),
        cwd: header.cwd.clone(),
        timestamp: header.timestamp.clone(),
        message_count,
        last_modified_ms,
        size_bytes,
        name,
    })
}

pub(crate) fn build_meta_from_file(path: &Path) -> Result<SessionMeta> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("jsonl") => build_meta_from_jsonl(path),
        #[cfg(feature = "sqlite-sessions")]
        Some("sqlite") => build_meta_from_sqlite(path),
        _ => build_meta_from_jsonl(path),
    }
}

#[derive(Deserialize)]
struct PartialEntry {
    #[serde(default)]
    r#type: String,
    #[serde(default)]
    name: Option<String>,
}

fn build_meta_from_jsonl(path: &Path) -> Result<SessionMeta> {
    let file = File::open(path)
        .map_err(|err| Error::session(format!("Read session file {}: {err}", path.display())))?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let header_line = lines
        .next()
        .ok_or_else(|| Error::session(format!("Empty session file {}", path.display())))?
        .map_err(|err| Error::session(format!("Read session header {}: {err}", path.display())))?;

    let header: SessionHeader = serde_json::from_str(&header_line)
        .map_err(|err| Error::session(format!("Parse session header {}: {err}", path.display())))?;
    header.validate().map_err(|reason| {
        Error::session(format!(
            "Invalid session header {}: {reason}",
            path.display()
        ))
    })?;

    let mut message_count = 0u64;
    let mut name = None;

    for line in lines {
        let line = line.map_err(|err| {
            Error::session(format!("Read session entry line {}: {err}", path.display()))
        })?;
        if let Ok(entry) = serde_json::from_str::<PartialEntry>(&line) {
            match entry.r#type.as_str() {
                "message" => message_count += 1,
                "session_info" => {
                    if entry.name.is_some() {
                        name = entry.name;
                    }
                }
                _ => {}
            }
        }
    }

    let meta = fs::metadata(path)?;
    let size_bytes = meta.len();
    let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    let millis = modified
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let last_modified_ms = i64::try_from(millis).unwrap_or(i64::MAX);

    Ok(SessionMeta {
        path: path.display().to_string(),
        id: header.id,
        cwd: header.cwd,
        timestamp: header.timestamp,
        message_count,
        last_modified_ms,
        size_bytes,
        name,
    })
}

#[cfg(feature = "sqlite-sessions")]
fn build_meta_from_sqlite(path: &Path) -> Result<SessionMeta> {
    let meta = futures::executor::block_on(crate::session_sqlite::load_session_meta(path))?;
    let header = meta.header;
    header.validate().map_err(|reason| {
        Error::session(format!(
            "Invalid session header {}: {reason}",
            path.display()
        ))
    })?;
    let (last_modified_ms, size_bytes) = session_file_stats(path)?;

    Ok(SessionMeta {
        path: path.display().to_string(),
        id: header.id,
        cwd: header.cwd,
        timestamp: header.timestamp,
        message_count: meta.message_count,
        last_modified_ms,
        size_bytes,
        name: meta.name,
    })
}

fn session_stats<T>(entries: &[T]) -> (u64, Option<String>)
where
    T: Borrow<SessionEntry>,
{
    let mut message_count = 0u64;
    let mut name = None;
    for entry in entries {
        match entry.borrow() {
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

#[cfg(feature = "sqlite-sessions")]
fn sqlite_auxiliary_paths(path: &Path) -> [PathBuf; 2] {
    ["-wal", "-shm"].map(|suffix| {
        let mut candidate = path.as_os_str().to_os_string();
        candidate.push(suffix);
        PathBuf::from(candidate)
    })
}

pub(crate) fn session_file_stats(path: &Path) -> Result<(i64, u64)> {
    let meta = fs::metadata(path)?;
    #[cfg(feature = "sqlite-sessions")]
    let (size, modified) = {
        let mut size = meta.len();
        let mut modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);

        if path.extension().and_then(|ext| ext.to_str()) == Some("sqlite") {
            for auxiliary_path in sqlite_auxiliary_paths(path) {
                let Ok(aux_meta) = fs::metadata(&auxiliary_path) else {
                    continue;
                };
                size = size.saturating_add(aux_meta.len());
                let aux_modified = aux_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                if aux_modified > modified {
                    modified = aux_modified;
                }
            }
        }

        (size, modified)
    };

    #[cfg(not(feature = "sqlite-sessions"))]
    let (size, modified) = (
        meta.len(),
        meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
    );

    let millis = modified
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let ms = i64::try_from(millis).unwrap_or(i64::MAX);
    Ok((ms, size))
}

fn is_session_file_path(path: &Path) -> bool {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if name.starts_with("session-index.") {
            return false;
        }
    }
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("jsonl") => true,
        #[cfg(feature = "sqlite-sessions")]
        Some("sqlite") => true,
        _ => false,
    }
}

pub(crate) fn walk_sessions(root: &Path) -> Vec<std::io::Result<PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let Ok(file_type) = entry.file_type() else {
                    continue;
                };

                if file_type.is_dir() {
                    stack.push(path);
                } else if file_type.is_symlink() {
                    // Allow symlinks to files, but skip symlinked directories to avoid cycles
                    if let Ok(meta) = fs::metadata(&path) {
                        if meta.is_file() && is_session_file_path(&path) {
                            out.push(Ok(path));
                        }
                    }
                } else if is_session_file_path(&path) {
                    out.push(Ok(path));
                }
            }
        }
    }
    out
}

fn current_epoch_ms() -> String {
    chrono::Utc::now().timestamp_millis().to_string()
}

fn current_epoch_ms_i64() -> i64 {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    i64::try_from(millis).unwrap_or(i64::MAX)
}

fn epoch_ms_is_stale(epoch_ms: i64, max_age: Duration) -> bool {
    let age_ms = current_epoch_ms_i64().saturating_sub(epoch_ms);
    u128::try_from(age_ms).unwrap_or(u128::MAX) > max_age.as_millis()
}

fn load_last_sync_epoch_ms(conn: &SqliteConnection) -> Result<Option<i64>> {
    let rows = conn
        .query_sync(
            "SELECT value FROM meta WHERE key='last_sync_epoch_ms' LIMIT 1",
            &[],
        )
        .map_err(|err| Error::session(format!("Query meta failed: {err}")))?;
    let Some(row) = rows.into_iter().next() else {
        return Ok(None);
    };
    let value = row
        .get_named::<String>("value")
        .map_err(|err| Error::session(format!("get meta value: {err}")))?;
    Ok(value.parse::<i64>().ok())
}

fn lock_file_guard(file: &File, timeout: Duration) -> Result<LockGuard<'_>> {
    let start = Instant::now();
    loop {
        if matches!(FileExt::try_lock_exclusive(file), Ok(true)) {
            return Ok(LockGuard { file });
        }

        if start.elapsed() >= timeout {
            return Err(Error::session(
                "Timed out waiting for session index lock".to_string(),
            ));
        }

        std::thread::sleep(Duration::from_millis(50));
    }
}

#[derive(Debug)]
struct LockGuard<'a> {
    file: &'a File,
}

impl Drop for LockGuard<'_> {
    fn drop(&mut self) {
        let _ = FileExt::unlock(self.file);
    }
}

#[cfg(test)]
#[path = "../tests/common/mod.rs"]
mod test_common;

#[cfg(test)]
mod tests {
    use super::*;

    use super::test_common::TestHarness;
    use crate::model::UserContent;
    use crate::session::{EntryBase, MessageEntry, SessionInfoEntry, SessionMessage};
    use pretty_assertions::assert_eq;
    use proptest::prelude::*;
    use proptest::string::string_regex;
    use std::collections::HashMap;
    use std::fs;
    #[cfg(unix)]
    use std::process::Command;
    use std::time::Duration;

    fn write_session_jsonl(path: &Path, header: &SessionHeader, entries: &[SessionEntry]) {
        let mut jsonl = String::new();
        jsonl.push_str(&serde_json::to_string(header).expect("serialize session header"));
        jsonl.push('\n');
        for entry in entries {
            jsonl.push_str(&serde_json::to_string(entry).expect("serialize session entry"));
            jsonl.push('\n');
        }
        fs::write(path, jsonl).expect("write session jsonl");
    }

    fn make_header(id: &str, cwd: &str) -> SessionHeader {
        let mut header = SessionHeader::new();
        header.id = id.to_string();
        header.cwd = cwd.to_string();
        header
    }

    fn make_user_entry(parent_id: Option<String>, id: &str, text: &str) -> SessionEntry {
        SessionEntry::Message(MessageEntry {
            base: EntryBase::new(parent_id, id.to_string()),
            message: SessionMessage::User {
                content: UserContent::Text(text.to_string()),
                timestamp: Some(chrono::Utc::now().timestamp_millis()),
            },
        })
    }

    fn make_session_info_entry(
        parent_id: Option<String>,
        id: &str,
        name: Option<&str>,
    ) -> SessionEntry {
        SessionEntry::SessionInfo(SessionInfoEntry {
            base: EntryBase::new(parent_id, id.to_string()),
            name: name.map(ToString::to_string),
        })
    }

    fn read_meta_last_sync_epoch_ms(index: &SessionIndex) -> String {
        index
            .with_lock(|conn| {
                init_schema(conn)?;
                let rows = conn
                    .query_sync(
                        "SELECT value FROM meta WHERE key='last_sync_epoch_ms' LIMIT 1",
                        &[],
                    )
                    .map_err(|err| Error::session(format!("Query meta failed: {err}")))?;
                let row = rows
                    .into_iter()
                    .next()
                    .ok_or_else(|| Error::session("Missing meta row".to_string()))?;
                row.get_named::<String>("value")
                    .map_err(|err| Error::session(format!("get meta value: {err}")))
            })
            .expect("read meta.last_sync_epoch_ms")
    }

    #[derive(Debug, Clone)]
    struct ArbitraryMetaRow {
        id: String,
        cwd: String,
        timestamp: String,
        message_count: i64,
        last_modified_ms: i64,
        size_bytes: i64,
        name: Option<String>,
    }

    fn ident_strategy() -> impl Strategy<Value = String> {
        string_regex("[a-z0-9_-]{1,16}").expect("valid identifier regex")
    }

    fn cwd_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("cwd-a".to_string()),
            Just("cwd-b".to_string()),
            string_regex("[a-z0-9_./-]{1,20}").expect("valid cwd regex"),
        ]
    }

    fn timestamp_strategy() -> impl Strategy<Value = String> {
        string_regex("[0-9TZ:.-]{10,32}").expect("valid timestamp regex")
    }

    fn optional_name_strategy() -> impl Strategy<Value = Option<String>> {
        prop::option::of(string_regex("[A-Za-z0-9 _.:-]{0,32}").expect("valid name regex"))
    }

    fn arbitrary_meta_row_strategy() -> impl Strategy<Value = ArbitraryMetaRow> {
        (
            ident_strategy(),
            cwd_strategy(),
            timestamp_strategy(),
            any::<i64>(),
            any::<i64>(),
            any::<i64>(),
            optional_name_strategy(),
        )
            .prop_map(
                |(id, cwd, timestamp, message_count, last_modified_ms, size_bytes, name)| {
                    ArbitraryMetaRow {
                        id,
                        cwd,
                        timestamp,
                        message_count,
                        last_modified_ms,
                        size_bytes,
                        name,
                    }
                },
            )
    }

    #[test]
    fn index_session_on_in_memory_session_is_noop() {
        let harness = TestHarness::new("index_session_on_in_memory_session_is_noop");
        let root = harness.temp_path("sessions");
        fs::create_dir_all(&root).expect("create root dir");
        let index = SessionIndex::for_sessions_root(&root);
        let session = Session::in_memory();

        index
            .index_session(&session)
            .expect("index in-memory session");

        harness
            .log()
            .info_ctx("verify", "No index files created", |ctx| {
                ctx.push(("db_path".into(), index.db_path.display().to_string()));
                ctx.push(("lock_path".into(), index.lock_path.display().to_string()));
            });
        assert!(!index.db_path.exists());
        assert!(!index.lock_path.exists());
    }

    #[test]
    fn index_session_inserts_row_and_updates_meta() {
        let harness = TestHarness::new("index_session_inserts_row_and_updates_meta");
        let root = harness.temp_path("sessions");
        fs::create_dir_all(&root).expect("create root dir");
        let index = SessionIndex::for_sessions_root(&root);

        let session_path = harness.temp_path("sessions/project/a.jsonl");
        fs::create_dir_all(session_path.parent().expect("parent")).expect("create session dir");
        fs::write(&session_path, "hello").expect("write session file");

        let mut session = Session::in_memory();
        session.header = make_header("id-a", "cwd-a");
        session.path = Some(session_path.clone());
        session.entries.push(make_user_entry(None, "m1", "hi"));

        index.index_session(&session).expect("index session");

        let sessions = index.list_sessions(Some("cwd-a")).expect("list sessions");
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "id-a");
        assert_eq!(sessions[0].cwd, "cwd-a");
        assert_eq!(sessions[0].message_count, 1);
        assert_eq!(sessions[0].path, session_path.display().to_string());

        let meta_value = read_meta_last_sync_epoch_ms(&index);
        harness
            .log()
            .info_ctx("verify", "meta.last_sync_epoch_ms present", |ctx| {
                ctx.push(("value".into(), meta_value.clone()));
            });
        assert!(
            meta_value.parse::<i64>().is_ok(),
            "Expected meta value to be an integer epoch ms"
        );
    }

    #[test]
    fn index_session_updates_existing_row() {
        let harness = TestHarness::new("index_session_updates_existing_row");
        let root = harness.temp_path("sessions");
        fs::create_dir_all(&root).expect("create root dir");
        let index = SessionIndex::for_sessions_root(&root);

        let session_path = harness.temp_path("sessions/project/update.jsonl");
        fs::create_dir_all(session_path.parent().expect("parent")).expect("create session dir");
        fs::write(&session_path, "first").expect("write session file");

        let mut session = Session::in_memory();
        session.header = make_header("id-update", "cwd-update");
        session.path = Some(session_path.clone());
        session.entries.push(make_user_entry(None, "m1", "hi"));

        index
            .index_session(&session)
            .expect("index session first time");
        let first_meta = index
            .list_sessions(Some("cwd-update"))
            .expect("list sessions")[0]
            .clone();
        let first_sync = read_meta_last_sync_epoch_ms(&index);

        std::thread::sleep(Duration::from_millis(10));
        fs::write(&session_path, "second-longer").expect("rewrite session file");
        session
            .entries
            .push(make_user_entry(Some("m1".to_string()), "m2", "again"));

        index
            .index_session(&session)
            .expect("index session second time");
        let second_meta = index
            .list_sessions(Some("cwd-update"))
            .expect("list sessions")[0]
            .clone();
        let second_sync = read_meta_last_sync_epoch_ms(&index);

        harness.log().info_ctx("verify", "row updated", |ctx| {
            ctx.push((
                "first_message_count".into(),
                first_meta.message_count.to_string(),
            ));
            ctx.push((
                "second_message_count".into(),
                second_meta.message_count.to_string(),
            ));
            ctx.push(("first_size".into(), first_meta.size_bytes.to_string()));
            ctx.push(("second_size".into(), second_meta.size_bytes.to_string()));
            ctx.push(("first_sync".into(), first_sync.clone()));
            ctx.push(("second_sync".into(), second_sync.clone()));
        });

        assert_eq!(second_meta.message_count, 2);
        assert!(second_meta.size_bytes >= first_meta.size_bytes);
        assert!(second_meta.last_modified_ms >= first_meta.last_modified_ms);
        assert!(second_sync.parse::<i64>().unwrap_or(0) >= first_sync.parse::<i64>().unwrap_or(0));
    }

    #[test]
    fn list_sessions_orders_by_last_modified_desc() {
        let harness = TestHarness::new("list_sessions_orders_by_last_modified_desc");
        let root = harness.temp_path("sessions");
        fs::create_dir_all(&root).expect("create root dir");
        let index = SessionIndex::for_sessions_root(&root);

        let path_a = harness.temp_path("sessions/project/a.jsonl");
        fs::create_dir_all(path_a.parent().expect("parent")).expect("create dirs");
        fs::write(&path_a, "a").expect("write file a");

        let mut session_a = Session::in_memory();
        session_a.header = make_header("id-a", "cwd-a");
        session_a.path = Some(path_a);
        session_a.entries.push(make_user_entry(None, "m1", "a"));
        index.index_session(&session_a).expect("index a");

        std::thread::sleep(Duration::from_millis(10));

        let path_b = harness.temp_path("sessions/project/b.jsonl");
        fs::create_dir_all(path_b.parent().expect("parent")).expect("create dirs");
        fs::write(&path_b, "bbbbb").expect("write file b");

        let mut session_b = Session::in_memory();
        session_b.header = make_header("id-b", "cwd-b");
        session_b.path = Some(path_b);
        session_b.entries.push(make_user_entry(None, "m1", "b"));
        index.index_session(&session_b).expect("index b");

        let sessions = index.list_sessions(None).expect("list sessions");
        harness
            .log()
            .info("verify", format!("listed {} sessions", sessions.len()));
        assert!(sessions.len() >= 2);
        assert_eq!(sessions[0].id, "id-b");
        assert_eq!(sessions[1].id, "id-a");
        assert!(sessions[0].last_modified_ms >= sessions[1].last_modified_ms);
    }

    #[test]
    fn list_sessions_filters_by_cwd() {
        let harness = TestHarness::new("list_sessions_filters_by_cwd");
        let root = harness.temp_path("sessions");
        fs::create_dir_all(&root).expect("create root dir");
        let index = SessionIndex::for_sessions_root(&root);

        for (id, cwd) in [("id-a", "cwd-a"), ("id-b", "cwd-b")] {
            let path = harness.temp_path(format!("sessions/project/{id}.jsonl"));
            fs::create_dir_all(path.parent().expect("parent")).expect("create dirs");
            fs::write(&path, id).expect("write session file");

            let mut session = Session::in_memory();
            session.header = make_header(id, cwd);
            session.path = Some(path);
            session.entries.push(make_user_entry(None, "m1", id));
            index.index_session(&session).expect("index session");
        }

        let only_a = index
            .list_sessions(Some("cwd-a"))
            .expect("list sessions cwd-a");
        assert_eq!(only_a.len(), 1);
        assert_eq!(only_a[0].id, "id-a");
    }

    #[test]
    fn reindex_all_is_noop_when_sessions_root_missing() {
        let harness = TestHarness::new("reindex_all_is_noop_when_sessions_root_missing");
        let missing_root = harness.temp_path("does-not-exist");
        let index = SessionIndex::for_sessions_root(&missing_root);

        index.reindex_all().expect("reindex_all");
        assert!(!index.db_path.exists());
        assert!(!index.lock_path.exists());
    }

    #[test]
    fn reindex_all_rebuilds_index_from_disk() {
        let harness = TestHarness::new("reindex_all_rebuilds_index_from_disk");
        let root = harness.temp_path("sessions");
        fs::create_dir_all(&root).expect("create root dir");
        let index = SessionIndex::for_sessions_root(&root);

        let path = harness.temp_path("sessions/project/reindex.jsonl");
        fs::create_dir_all(path.parent().expect("parent")).expect("create dirs");

        let header = make_header("id-reindex", "cwd-reindex");
        let entries = vec![
            make_user_entry(None, "m1", "hello"),
            make_session_info_entry(Some("m1".to_string()), "info1", Some("My Session")),
            make_user_entry(Some("info1".to_string()), "m2", "world"),
        ];
        write_session_jsonl(&path, &header, &entries);

        index.reindex_all().expect("reindex_all");

        let sessions = index
            .list_sessions(Some("cwd-reindex"))
            .expect("list sessions");
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "id-reindex");
        assert_eq!(sessions[0].message_count, 2);
        assert_eq!(sessions[0].name.as_deref(), Some("My Session"));

        let meta_value = read_meta_last_sync_epoch_ms(&index);
        harness.log().info_ctx("verify", "meta updated", |ctx| {
            ctx.push(("value".into(), meta_value.clone()));
        });
        assert!(meta_value.parse::<i64>().unwrap_or(0) > 0);
    }

    #[test]
    fn reindex_all_skips_invalid_jsonl_files() {
        let harness = TestHarness::new("reindex_all_skips_invalid_jsonl_files");
        let root = harness.temp_path("sessions");
        fs::create_dir_all(&root).expect("create root dir");
        let index = SessionIndex::for_sessions_root(&root);

        let good = harness.temp_path("sessions/project/good.jsonl");
        fs::create_dir_all(good.parent().expect("parent")).expect("create dirs");
        let header = make_header("id-good", "cwd-good");
        let entries = vec![make_user_entry(None, "m1", "ok")];
        write_session_jsonl(&good, &header, &entries);

        let bad = harness.temp_path("sessions/project/bad.jsonl");
        fs::write(&bad, "not-json\n{").expect("write bad jsonl");

        index.reindex_all().expect("reindex_all should succeed");
        let sessions = index.list_sessions(None).expect("list sessions");
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "id-good");
    }

    #[test]
    fn build_meta_from_file_returns_session_error_on_invalid_header() {
        let harness =
            TestHarness::new("build_meta_from_file_returns_session_error_on_invalid_header");
        let path = harness.temp_path("bad_header.jsonl");
        fs::write(&path, "not json\n").expect("write bad header");

        let err = build_meta_from_file(&path).expect_err("expected error");
        harness.log().info("verify", format!("error: {err}"));

        assert!(
            matches!(err, Error::Session(ref msg) if msg.contains("Parse session header")),
            "Expected Error::Session containing Parse session header, got {err:?}",
        );
    }

    #[test]
    fn build_meta_from_file_rejects_semantically_invalid_header() {
        let harness = TestHarness::new("build_meta_from_file_rejects_semantically_invalid_header");
        let path = harness.temp_path("bad_semantic_header.jsonl");
        let header = SessionHeader {
            r#type: "note".to_string(),
            id: "bad-id".to_string(),
            cwd: "/tmp".to_string(),
            timestamp: "2026-01-01T00:00:00.000Z".to_string(),
            ..SessionHeader::default()
        };
        write_session_jsonl(&path, &header, &[]);

        let err = build_meta_from_file(&path).expect_err("expected invalid header error");
        harness.log().info("verify", format!("error: {err}"));

        assert!(
            matches!(err, Error::Session(ref msg) if msg.contains("Invalid session header")),
            "Expected Error::Session containing Invalid session header, got {err:?}",
        );
    }

    #[test]
    fn build_meta_from_file_returns_session_error_on_empty_file() {
        let harness = TestHarness::new("build_meta_from_file_returns_session_error_on_empty_file");
        let path = harness.temp_path("empty.jsonl");
        fs::write(&path, "").expect("write empty");

        let err = build_meta_from_file(&path).expect_err("expected error");
        if let Error::Session(msg) = &err {
            harness.log().info("verify", msg.clone());
        }
        assert!(
            matches!(err, Error::Session(ref msg) if msg.contains("Empty session file")),
            "Expected Error::Session containing Empty session file, got {err:?}",
        );
    }

    #[test]
    fn list_sessions_returns_session_error_when_db_path_is_directory() {
        let harness =
            TestHarness::new("list_sessions_returns_session_error_when_db_path_is_directory");
        let root = harness.temp_path("sessions");
        fs::create_dir_all(&root).expect("create root dir");

        let db_dir = root.join("session-index.sqlite");
        fs::create_dir_all(&db_dir).expect("create db dir to force sqlite open failure");

        let index = SessionIndex::for_sessions_root(&root);
        let err = index.list_sessions(None).expect_err("expected error");
        if let Error::Session(msg) = &err {
            harness.log().info("verify", msg.clone());
        }
        assert!(
            matches!(err, Error::Session(ref msg) if msg.contains("SQLite open")),
            "Expected Error::Session containing SQLite open, got {err:?}",
        );
    }

    #[test]
    fn lock_file_guard_prevents_concurrent_access() {
        let harness = TestHarness::new("lock_file_guard_prevents_concurrent_access");
        let path = harness.temp_path("lockfile.lock");
        fs::write(&path, "").expect("create lock file");

        let file1 = File::options()
            .read(true)
            .write(true)
            .open(&path)
            .expect("open file1");
        let file2 = File::options()
            .read(true)
            .write(true)
            .open(&path)
            .expect("open file2");

        let guard1 = lock_file_guard(&file1, Duration::from_millis(50)).expect("acquire lock");
        let err =
            lock_file_guard(&file2, Duration::from_millis(50)).expect_err("expected lock timeout");
        drop(guard1);

        assert!(
            matches!(err, Error::Session(ref msg) if msg.contains("Timed out")),
            "Expected Error::Session containing Timed out, got {err:?}",
        );

        let _guard2 =
            lock_file_guard(&file2, Duration::from_millis(50)).expect("lock after release");
    }

    #[test]
    fn should_reindex_returns_true_when_db_missing() {
        let harness = TestHarness::new("should_reindex_returns_true_when_db_missing");
        let root = harness.temp_path("sessions");
        fs::create_dir_all(&root).expect("create root dir");
        let index = SessionIndex::for_sessions_root(&root);

        assert!(index.should_reindex(Duration::from_secs(60)));
    }

    // ── session_stats ────────────────────────────────────────────────

    #[test]
    fn session_stats_empty_entries() {
        let (count, name) = session_stats::<SessionEntry>(&[]);
        assert_eq!(count, 0);
        assert!(name.is_none());
    }

    #[test]
    fn session_stats_counts_messages_only() {
        let entries = vec![
            make_user_entry(None, "m1", "hello"),
            make_session_info_entry(Some("m1".to_string()), "info1", None),
            make_user_entry(Some("info1".to_string()), "m2", "world"),
        ];
        let (count, name) = session_stats(&entries);
        assert_eq!(count, 2);
        assert!(name.is_none());
    }

    #[test]
    fn session_stats_extracts_last_name() {
        let entries = vec![
            make_session_info_entry(None, "info1", Some("First Name")),
            make_user_entry(Some("info1".to_string()), "m1", "msg"),
            make_session_info_entry(Some("m1".to_string()), "info2", Some("Final Name")),
        ];
        let (count, name) = session_stats(&entries);
        assert_eq!(count, 1);
        assert_eq!(name.as_deref(), Some("Final Name"));
    }

    #[test]
    fn session_stats_name_not_overwritten_by_none() {
        let entries = vec![
            make_session_info_entry(None, "info1", Some("My Session")),
            make_session_info_entry(Some("info1".to_string()), "info2", None),
        ];
        let (_, name) = session_stats(&entries);
        // None doesn't overwrite previous name because of `if info.name.is_some()`
        assert_eq!(name.as_deref(), Some("My Session"));
    }

    // ── file_stats ──────────────────────────────────────────────────

    #[test]
    fn file_stats_returns_size_and_mtime() {
        let harness = TestHarness::new("file_stats_returns_size_and_mtime");
        let path = harness.temp_path("test_file.txt");
        fs::write(&path, "hello world").expect("write");

        let (last_modified_ms, size_bytes) = session_file_stats(&path).expect("file_stats");
        assert_eq!(size_bytes, 11); // "hello world" = 11 bytes
        assert!(last_modified_ms > 0, "Expected positive modification time");
    }

    #[cfg(feature = "sqlite-sessions")]
    #[test]
    fn file_stats_sqlite_includes_wal_and_shm_sizes() {
        let harness = TestHarness::new("file_stats_sqlite_includes_wal_and_shm_sizes");
        let path = harness.temp_path("test_session.sqlite");
        let [wal_path, shm_path] = sqlite_auxiliary_paths(&path);

        fs::write(&path, b"db").expect("write sqlite db");
        fs::write(&wal_path, b"walpayload").expect("write sqlite wal");
        fs::write(&shm_path, b"shm!").expect("write sqlite shm");

        let (_, size_bytes) = session_file_stats(&path).expect("file_stats");
        assert_eq!(size_bytes, 2 + 10 + 4);
    }

    #[cfg(feature = "sqlite-sessions")]
    #[test]
    fn index_session_snapshot_uses_newest_sqlite_sidecar_mtime_and_size() {
        let harness =
            TestHarness::new("index_session_snapshot_uses_newest_sqlite_sidecar_mtime_and_size");
        let root = harness.temp_path("sessions");
        let project_dir = root.join("project");
        fs::create_dir_all(&project_dir).expect("create project dir");

        let path = project_dir.join("test.sqlite");
        let [wal_path, _shm_path] = sqlite_auxiliary_paths(&path);
        fs::write(&path, b"db").expect("write sqlite db");

        let base_millis = fs::metadata(&path)
            .expect("base metadata")
            .modified()
            .expect("base modified")
            .duration_since(UNIX_EPOCH)
            .expect("base since epoch")
            .as_millis();
        std::thread::sleep(Duration::from_millis(1_100));
        fs::write(&wal_path, b"walpayload").expect("write sqlite wal");
        let wal_millis = fs::metadata(&wal_path)
            .expect("wal metadata")
            .modified()
            .expect("wal modified")
            .duration_since(UNIX_EPOCH)
            .expect("wal since epoch")
            .as_millis();

        assert!(
            wal_millis > base_millis,
            "test requires WAL sidecar mtime to be newer than base db mtime"
        );

        let index = SessionIndex::for_sessions_root(&root);
        let header = make_header("sqlite-id", "sqlite-cwd");
        index
            .index_session_snapshot(&path, &header, 3, Some("sqlite session".to_string()))
            .expect("index sqlite snapshot");

        let listed = index
            .list_sessions(Some("sqlite-cwd"))
            .expect("list sqlite session");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].size_bytes, 2 + 10);
        assert_eq!(
            listed[0].last_modified_ms,
            i64::try_from(wal_millis).expect("wal mtime fits in i64")
        );
    }

    #[test]
    fn file_stats_missing_file_returns_error() {
        let err = session_file_stats(Path::new("/nonexistent/file.txt"));
        assert!(err.is_err());
    }

    #[test]
    fn list_sessions_errors_on_negative_message_count() {
        let harness = TestHarness::new("list_sessions_errors_on_negative_message_count");
        let root = harness.temp_path("sessions");
        fs::create_dir_all(&root).expect("create root dir");
        let index = SessionIndex::for_sessions_root(&root);

        index
            .with_lock(|conn| {
                init_schema(conn)?;
                conn.execute_sync(
                    "INSERT INTO sessions (path,id,cwd,timestamp,message_count,last_modified_ms,size_bytes,name)
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                    &[
                        Value::Text("/tmp/negative-message-count.jsonl".to_string()),
                        Value::Text("id-neg".to_string()),
                        Value::Text("cwd-neg".to_string()),
                        Value::Text("2026-01-01T00:00:00Z".to_string()),
                        Value::BigInt(-1),
                        Value::BigInt(1),
                        Value::BigInt(1),
                        Value::Null,
                    ],
                )
                .map_err(|err| Error::session(format!("insert negative row: {err}")))?;
                Ok(())
            })
            .expect("seed negative row");

        let err = index
            .list_sessions(None)
            .expect_err("negative count should error");
        assert!(
            matches!(err, Error::Session(ref msg) if msg.contains("message_count must be non-negative")),
            "unexpected error: {err:?}"
        );
    }

    #[test]
    fn list_sessions_errors_on_negative_size_bytes() {
        let harness = TestHarness::new("list_sessions_errors_on_negative_size_bytes");
        let root = harness.temp_path("sessions");
        fs::create_dir_all(&root).expect("create root dir");
        let index = SessionIndex::for_sessions_root(&root);

        index
            .with_lock(|conn| {
                init_schema(conn)?;
                conn.execute_sync(
                    "INSERT INTO sessions (path,id,cwd,timestamp,message_count,last_modified_ms,size_bytes,name)
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                    &[
                        Value::Text("/tmp/negative-size-bytes.jsonl".to_string()),
                        Value::Text("id-neg".to_string()),
                        Value::Text("cwd-neg".to_string()),
                        Value::Text("2026-01-01T00:00:00Z".to_string()),
                        Value::BigInt(1),
                        Value::BigInt(1),
                        Value::BigInt(-1),
                        Value::Null,
                    ],
                )
                .map_err(|err| Error::session(format!("insert negative row: {err}")))?;
                Ok(())
            })
            .expect("seed negative row");

        let err = index
            .list_sessions(None)
            .expect_err("negative size should error");
        assert!(
            matches!(err, Error::Session(ref msg) if msg.contains("size_bytes must be non-negative")),
            "unexpected error: {err:?}"
        );
    }

    // ── is_session_file_path ────────────────────────────────────────

    #[test]
    fn is_session_file_path_jsonl() {
        assert!(is_session_file_path(Path::new("session.jsonl")));
        assert!(is_session_file_path(Path::new("/foo/bar/test.jsonl")));
    }

    #[test]
    fn is_session_file_path_non_session() {
        assert!(!is_session_file_path(Path::new("session.txt")));
        assert!(!is_session_file_path(Path::new("session.json")));
        assert!(!is_session_file_path(Path::new("session")));
    }

    // ── walk_sessions ───────────────────────────────────────────────

    #[test]
    fn walk_sessions_finds_jsonl_files_recursively() {
        let harness = TestHarness::new("walk_sessions_finds_jsonl_files_recursively");
        let root = harness.temp_path("sessions");
        fs::create_dir_all(root.join("project")).expect("create dirs");

        fs::write(root.join("a.jsonl"), "").expect("write");
        fs::write(root.join("project/b.jsonl"), "").expect("write");
        fs::write(root.join("not_session.txt"), "").expect("write");

        let paths = walk_sessions(&root);
        let ok_paths: Vec<_> = paths
            .into_iter()
            .filter_map(std::result::Result::ok)
            .collect();
        assert_eq!(ok_paths.len(), 2);
        assert!(ok_paths.iter().any(|p| p.ends_with("a.jsonl")));
        assert!(ok_paths.iter().any(|p| p.ends_with("b.jsonl")));
    }

    #[test]
    fn walk_sessions_empty_dir() {
        let harness = TestHarness::new("walk_sessions_empty_dir");
        let root = harness.temp_path("sessions");
        fs::create_dir_all(&root).expect("create dirs");

        let paths = walk_sessions(&root);
        assert!(paths.is_empty());
    }

    #[test]
    fn walk_sessions_nonexistent_dir() {
        let paths = walk_sessions(Path::new("/nonexistent/path"));
        assert!(paths.is_empty());
    }

    // ── current_epoch_ms ────────────────────────────────────────────

    #[test]
    fn current_epoch_ms_is_valid_number() {
        let ms = current_epoch_ms();
        let parsed: i64 = ms.parse().expect("should be valid i64");
        assert!(parsed > 0, "Epoch ms should be positive");
        // Should be after 2020-01-01
        assert!(parsed > 1_577_836_800_000, "Epoch ms should be after 2020");
    }

    // ── delete_session_path ─────────────────────────────────────────

    #[test]
    fn delete_session_path_removes_row() {
        let harness = TestHarness::new("delete_session_path_removes_row");
        let root = harness.temp_path("sessions");
        fs::create_dir_all(&root).expect("create root dir");
        let index = SessionIndex::for_sessions_root(&root);

        let session_path = harness.temp_path("sessions/project/del.jsonl");
        fs::create_dir_all(session_path.parent().expect("parent")).expect("create dirs");
        fs::write(&session_path, "data").expect("write");

        let mut session = Session::in_memory();
        session.header = make_header("id-del", "cwd-del");
        session.path = Some(session_path.clone());
        session.entries.push(make_user_entry(None, "m1", "hi"));
        index.index_session(&session).expect("index session");

        let before = index.list_sessions(None).expect("list before");
        assert_eq!(before.len(), 1);

        index
            .delete_session_path(&session_path)
            .expect("delete session path");

        let after = index.list_sessions(None).expect("list after");
        assert!(after.is_empty());
    }

    #[test]
    fn delete_session_path_noop_when_not_exists() {
        let harness = TestHarness::new("delete_session_path_noop_when_not_exists");
        let root = harness.temp_path("sessions");
        fs::create_dir_all(&root).expect("create root dir");
        let index = SessionIndex::for_sessions_root(&root);

        // Delete a path that was never indexed — should succeed without error
        index
            .delete_session_path(Path::new("/nonexistent/session.jsonl"))
            .expect("delete nonexistent should succeed");
    }

    // ── should_reindex ──────────────────────────────────────────────

    #[test]
    fn should_reindex_returns_false_when_db_is_fresh() {
        let harness = TestHarness::new("should_reindex_returns_false_when_db_is_fresh");
        let root = harness.temp_path("sessions");
        fs::create_dir_all(&root).expect("create root dir");
        let index = SessionIndex::for_sessions_root(&root);

        // Create the db by indexing a session
        let session_path = harness.temp_path("sessions/project/fresh.jsonl");
        fs::create_dir_all(session_path.parent().expect("parent")).expect("create dirs");
        fs::write(&session_path, "data").expect("write");

        let mut session = Session::in_memory();
        session.header = make_header("id-fresh", "cwd-fresh");
        session.path = Some(session_path);
        session.entries.push(make_user_entry(None, "m1", "hi"));
        index.index_session(&session).expect("index session");

        // DB just created — should not need reindex for large max_age
        assert!(!index.should_reindex(Duration::from_secs(3600)));
    }

    #[cfg(unix)]
    #[test]
    fn should_reindex_prefers_meta_timestamp_over_stale_db_mtime() {
        let harness = TestHarness::new("should_reindex_prefers_meta_timestamp_over_stale_db_mtime");
        let root = harness.temp_path("sessions");
        fs::create_dir_all(&root).expect("create root dir");
        let index = SessionIndex::for_sessions_root(&root);

        let session_path = harness.temp_path("sessions/project/fresh-meta.jsonl");
        fs::create_dir_all(session_path.parent().expect("parent")).expect("create dirs");
        fs::write(&session_path, "data").expect("write");

        let mut session = Session::in_memory();
        session.header = make_header("id-fresh-meta", "cwd-fresh-meta");
        session.path = Some(session_path);
        session.entries.push(make_user_entry(None, "m1", "hi"));
        index.index_session(&session).expect("index session");

        let status = Command::new("touch")
            .args([
                "-t",
                "200001010000",
                index.db_path.to_str().expect("utf-8 db path"),
            ])
            .status()
            .expect("run touch");
        assert!(status.success(), "touch should succeed");

        assert!(
            !index.should_reindex(Duration::from_secs(3600)),
            "fresh meta.last_sync_epoch_ms should outrank stale db mtime"
        );
    }

    // ── reindex_if_stale ────────────────────────────────────────────

    #[test]
    fn reindex_if_stale_returns_false_when_fresh() {
        let harness = TestHarness::new("reindex_if_stale_returns_false_when_fresh");
        let root = harness.temp_path("sessions");
        fs::create_dir_all(&root).expect("create root dir");
        let index = SessionIndex::for_sessions_root(&root);

        // Create a session file on disk
        let session_path = harness.temp_path("sessions/project/stale_test.jsonl");
        fs::create_dir_all(session_path.parent().expect("parent")).expect("create dirs");
        let header = make_header("id-stale", "cwd-stale");
        let entries = vec![make_user_entry(None, "m1", "msg")];
        write_session_jsonl(&session_path, &header, &entries);

        // First reindex (no db exists yet)
        let result = index
            .reindex_if_stale(Duration::from_secs(3600))
            .expect("reindex");
        assert!(result, "First reindex should return true (no db)");

        // Second call with large max_age should return false (fresh)
        let result = index
            .reindex_if_stale(Duration::from_secs(3600))
            .expect("reindex");
        assert!(!result, "Second reindex should return false (fresh)");
    }

    #[test]
    fn reindex_if_stale_returns_true_when_stale() {
        let harness = TestHarness::new("reindex_if_stale_returns_true_when_stale");
        let root = harness.temp_path("sessions");
        fs::create_dir_all(&root).expect("create root dir");
        let index = SessionIndex::for_sessions_root(&root);

        // Create a session on disk
        let session_path = harness.temp_path("sessions/project/stale.jsonl");
        fs::create_dir_all(session_path.parent().expect("parent")).expect("create dirs");
        let header = make_header("id-stale2", "cwd-stale2");
        let entries = vec![make_user_entry(None, "m1", "msg")];
        write_session_jsonl(&session_path, &header, &entries);

        // Reindex with zero max_age — always stale
        let result = index.reindex_if_stale(Duration::ZERO).expect("reindex");
        assert!(result, "Should reindex with zero max_age");
    }

    // ── build_meta ──────────────────────────────────────────────────

    #[test]
    fn build_meta_from_file_returns_correct_fields() {
        let harness = TestHarness::new("build_meta_from_file_returns_correct_fields");
        let path = harness.temp_path("test_session.jsonl");
        let header = make_header("id-bm", "cwd-bm");
        let entries = vec![
            make_user_entry(None, "m1", "hello"),
            make_user_entry(Some("m1".to_string()), "m2", "world"),
            make_session_info_entry(Some("m2".to_string()), "info1", Some("Named Session")),
        ];
        write_session_jsonl(&path, &header, &entries);

        let meta = build_meta_from_file(&path).expect("build_meta_from_file");
        assert_eq!(meta.id, "id-bm");
        assert_eq!(meta.cwd, "cwd-bm");
        assert_eq!(meta.message_count, 2);
        assert_eq!(meta.name.as_deref(), Some("Named Session"));
        assert!(meta.size_bytes > 0);
        assert!(meta.last_modified_ms > 0);
        assert!(meta.path.contains("test_session.jsonl"));
    }

    // ── for_sessions_root path construction ─────────────────────────

    #[test]
    fn for_sessions_root_constructs_correct_paths() {
        let root = Path::new("/home/user/.pi/sessions");
        let index = SessionIndex::for_sessions_root(root);
        assert_eq!(
            index.db_path,
            PathBuf::from("/home/user/.pi/sessions/session-index.sqlite")
        );
        assert_eq!(
            index.lock_path,
            PathBuf::from("/home/user/.pi/sessions/session-index.lock")
        );
    }

    // ── sessions_root accessor ──────────────────────────────────────

    #[test]
    fn sessions_root_returns_parent_of_db_path() {
        let root = Path::new("/home/user/.pi/sessions");
        let index = SessionIndex::for_sessions_root(root);
        assert_eq!(index.sessions_root(), root);
    }

    // ── reindex_all clears old rows ─────────────────────────────────

    #[test]
    fn reindex_all_replaces_stale_rows() {
        let harness = TestHarness::new("reindex_all_replaces_stale_rows");
        let root = harness.temp_path("sessions");
        fs::create_dir_all(root.join("project")).expect("create dirs");

        // Index two sessions manually
        let index = SessionIndex::for_sessions_root(&root);

        let path_a = harness.temp_path("sessions/project/a.jsonl");
        let header_a = make_header("id-a", "cwd-a");
        write_session_jsonl(&path_a, &header_a, &[make_user_entry(None, "m1", "a")]);

        let path_b = harness.temp_path("sessions/project/b.jsonl");
        let header_b = make_header("id-b", "cwd-b");
        write_session_jsonl(&path_b, &header_b, &[make_user_entry(None, "m1", "b")]);

        // Index both
        index.reindex_all().expect("reindex_all");
        let all = index.list_sessions(None).expect("list all");
        assert_eq!(all.len(), 2);

        // Now delete one file on disk and reindex
        fs::remove_file(&path_a).expect("remove file");
        index.reindex_all().expect("reindex_all after delete");
        let all = index.list_sessions(None).expect("list after reindex");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, "id-b");
    }

    // ── Session with multiple info entries ───────────────────────────

    #[test]
    fn index_session_with_session_name() {
        let harness = TestHarness::new("index_session_with_session_name");
        let root = harness.temp_path("sessions");
        fs::create_dir_all(&root).expect("create root dir");
        let index = SessionIndex::for_sessions_root(&root);

        let session_path = harness.temp_path("sessions/project/named.jsonl");
        fs::create_dir_all(session_path.parent().expect("parent")).expect("create dirs");
        fs::write(&session_path, "data").expect("write");

        let mut session = Session::in_memory();
        session.header = make_header("id-named", "cwd-named");
        session.path = Some(session_path);
        session.entries.push(make_user_entry(None, "m1", "hi"));
        session.entries.push(make_session_info_entry(
            Some("m1".to_string()),
            "info1",
            Some("My Project"),
        ));

        index.index_session(&session).expect("index session");

        let sessions = index.list_sessions(None).expect("list");
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].name.as_deref(), Some("My Project"));
    }

    #[test]
    fn index_session_update_clears_stale_session_name() {
        let harness = TestHarness::new("index_session_update_clears_stale_session_name");
        let root = harness.temp_path("sessions");
        fs::create_dir_all(&root).expect("create root dir");
        let index = SessionIndex::for_sessions_root(&root);

        let session_path = harness.temp_path("sessions/project/clear-name.jsonl");
        fs::create_dir_all(session_path.parent().expect("parent")).expect("create dirs");
        fs::write(&session_path, "first").expect("write");

        let mut named = Session::in_memory();
        named.header = make_header("id-clear-name", "cwd-clear-name");
        named.path = Some(session_path.clone());
        named.entries.push(make_user_entry(None, "m1", "hi"));
        named.entries.push(make_session_info_entry(
            Some("m1".to_string()),
            "info1",
            Some("My Project"),
        ));

        index.index_session(&named).expect("index named session");
        let first = index.list_sessions(None).expect("list named");
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].name.as_deref(), Some("My Project"));

        std::thread::sleep(Duration::from_millis(10));
        fs::write(&session_path, "second").expect("rewrite");

        let mut unnamed = Session::in_memory();
        unnamed.header = make_header("id-clear-name", "cwd-clear-name");
        unnamed.path = Some(session_path);
        unnamed.entries.push(make_user_entry(None, "m1", "hi"));

        index
            .index_session(&unnamed)
            .expect("index unnamed session");
        let second = index.list_sessions(None).expect("list unnamed");
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].name, None);
    }

    // ── Multiple cwd filtering ──────────────────────────────────────

    #[test]
    fn list_sessions_no_cwd_returns_all() {
        let harness = TestHarness::new("list_sessions_no_cwd_returns_all");
        let root = harness.temp_path("sessions");
        fs::create_dir_all(&root).expect("create root dir");
        let index = SessionIndex::for_sessions_root(&root);

        for (id, cwd) in [("id-x", "cwd-x"), ("id-y", "cwd-y"), ("id-z", "cwd-z")] {
            let path = harness.temp_path(format!("sessions/project/{id}.jsonl"));
            fs::create_dir_all(path.parent().expect("parent")).expect("create dirs");
            fs::write(&path, id).expect("write");

            let mut session = Session::in_memory();
            session.header = make_header(id, cwd);
            session.path = Some(path);
            session.entries.push(make_user_entry(None, "m1", id));
            index.index_session(&session).expect("index session");
        }

        let all = index.list_sessions(None).expect("list all");
        assert_eq!(all.len(), 3);
    }

    // ── build_meta_from_jsonl with entries having parse errors ───────

    #[test]
    fn build_meta_from_jsonl_skips_bad_entry_lines() {
        let harness = TestHarness::new("build_meta_from_jsonl_skips_bad_entry_lines");
        let path = harness.temp_path("mixed.jsonl");

        let header = make_header("id-mixed", "cwd-mixed");
        let good_entry = make_user_entry(None, "m1", "good");
        let mut content = serde_json::to_string(&header).expect("ser header");
        content.push('\n');
        content.push_str(&serde_json::to_string(&good_entry).expect("ser entry"));
        content.push('\n');
        content.push_str("not valid json\n");
        content.push_str(
            &serde_json::to_string(&make_user_entry(Some("m1".to_string()), "m2", "another"))
                .expect("ser entry"),
        );
        content.push('\n');

        fs::write(&path, content).expect("write");

        let meta = build_meta_from_jsonl(&path).expect("build_meta");
        // Bad line is skipped, so we get 2 messages
        assert_eq!(meta.message_count, 2);
    }

    #[test]
    fn build_meta_from_jsonl_errors_on_invalid_utf8_entry_line() {
        let harness = TestHarness::new("build_meta_from_jsonl_errors_on_invalid_utf8_entry_line");
        let path = harness.temp_path("invalid_utf8.jsonl");

        let header = make_header("id-invalid", "cwd-invalid");
        let mut bytes = serde_json::to_vec(&header).expect("serialize header");
        bytes.push(b'\n');
        bytes.extend_from_slice(br#"{"type":"message","message":{"role":"user","content":"ok"}}"#);
        bytes.push(b'\n');
        bytes.extend_from_slice(&[0xFF, 0xFE, b'\n']);

        fs::write(&path, bytes).expect("write");

        let err = build_meta_from_jsonl(&path).expect_err("invalid utf8 should error");
        assert!(
            matches!(err, Error::Session(ref msg) if msg.contains("Read session entry line")),
            "Expected entry line read error, got {err:?}"
        );
    }

    #[test]
    fn index_session_snapshot_rejects_message_count_over_i64_max() {
        let harness = TestHarness::new("index_session_snapshot_rejects_message_count_over_i64_max");
        let root = harness.temp_path("sessions");
        fs::create_dir_all(root.join("project")).expect("create project dir");
        let index = SessionIndex::for_sessions_root(&root);

        let path = root.join("project").join("overflow.jsonl");
        fs::write(&path, "").expect("write session payload");

        let header = make_header("id-overflow", "cwd-overflow");
        let err = index
            .index_session_snapshot(&path, &header, (i64::MAX as u64) + 1, None)
            .expect_err("out-of-range message_count should error");
        assert!(
            matches!(err, Error::Session(ref msg) if msg.contains("message_count exceeds SQLite INTEGER range")),
            "expected out-of-range message_count error, got {err:?}"
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 128, .. ProptestConfig::default() })]

        #[test]
        fn proptest_list_sessions_handles_arbitrary_sql_rows(
            rows in prop::collection::vec(arbitrary_meta_row_strategy(), 1..16)
        ) {
            let harness = TestHarness::new("proptest_list_sessions_handles_arbitrary_sql_rows");
            let root = harness.temp_path("sessions");
            fs::create_dir_all(&root).expect("create root dir");
            let index = SessionIndex::for_sessions_root(&root);

            let expected_by_path: HashMap<String, ArbitraryMetaRow> = rows
                .iter()
                .cloned()
                .enumerate()
                .map(|(idx, row)| (format!("/tmp/pi-session-index-{idx}.jsonl"), row))
                .collect();

            index
                .with_lock(|conn| {
                    init_schema(conn)?;
                    conn.execute_sync("DELETE FROM sessions", &[])
                        .map_err(|err| Error::session(format!("delete sessions: {err}")))?;

                    for (idx, row) in rows.iter().enumerate() {
                        let path = format!("/tmp/pi-session-index-{idx}.jsonl");
                        conn.execute_sync(
                            "INSERT INTO sessions (path,id,cwd,timestamp,message_count,last_modified_ms,size_bytes,name)
                             VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                            &[
                                Value::Text(path),
                                Value::Text(row.id.clone()),
                                Value::Text(row.cwd.clone()),
                                Value::Text(row.timestamp.clone()),
                                Value::BigInt(row.message_count),
                                Value::BigInt(row.last_modified_ms),
                                Value::BigInt(row.size_bytes),
                                row.name.clone().map_or(Value::Null, Value::Text),
                            ],
                        )
                        .map_err(|err| Error::session(format!("insert session row {idx}: {err}")))?;
                    }

                    Ok(())
                })
                .expect("seed session rows");

            let has_invalid_unsigned = rows
                .iter()
                .any(|row| row.message_count < 0 || row.size_bytes < 0);

            let listed = index.list_sessions(None);
            if has_invalid_unsigned {
                prop_assert!(listed.is_err(), "negative message_count/size_bytes should error");
                return Ok(());
            }
            let listed = listed.expect("list all sessions");
            prop_assert_eq!(listed.len(), rows.len());
            for pair in listed.windows(2) {
                prop_assert!(pair[0].last_modified_ms >= pair[1].last_modified_ms);
            }

            for meta in &listed {
                let expected = expected_by_path
                    .get(&meta.path)
                    .expect("expected row should exist");
                prop_assert_eq!(&meta.id, &expected.id);
                prop_assert_eq!(&meta.cwd, &expected.cwd);
                prop_assert_eq!(&meta.timestamp, &expected.timestamp);
                prop_assert_eq!(
                    meta.message_count,
                    u64::try_from(expected.message_count).expect("filtered non-negative count")
                );
                prop_assert_eq!(
                    meta.size_bytes,
                    u64::try_from(expected.size_bytes).expect("filtered non-negative size")
                );
                prop_assert_eq!(&meta.name, &expected.name);
            }

            let filtered = index
                .list_sessions(Some("cwd-a"))
                .expect("list cwd-a sessions");
            let expected_filtered = rows.iter().filter(|row| row.cwd == "cwd-a").count();
            prop_assert_eq!(filtered.len(), expected_filtered);
            prop_assert!(filtered.iter().all(|meta| meta.cwd == "cwd-a"));
            for pair in filtered.windows(2) {
                prop_assert!(pair[0].last_modified_ms >= pair[1].last_modified_ms);
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 128, .. ProptestConfig::default() })]

        #[test]
        fn proptest_index_session_snapshot_roundtrip_metadata(
            id in ident_strategy(),
            cwd in cwd_strategy(),
            timestamp in timestamp_strategy(),
            message_count in any::<u64>(),
            name in optional_name_strategy(),
            content in prop::collection::vec(any::<u8>(), 0..256)
        ) {
            let harness = TestHarness::new("proptest_index_session_snapshot_roundtrip_metadata");
            let root = harness.temp_path("sessions");
            fs::create_dir_all(root.join("project")).expect("create project dir");
            let index = SessionIndex::for_sessions_root(&root);

            let path = root.join("project").join(format!("{id}.jsonl"));
            fs::write(&path, &content).expect("write session payload");

            let mut header = make_header(&id, &cwd);
            header.timestamp = timestamp.clone();
            let index_result = index.index_session_snapshot(&path, &header, message_count, name.clone());
            if message_count > i64::MAX as u64 {
                prop_assert!(
                    index_result.is_err(),
                    "expected out-of-range message_count to fail indexing"
                );
            } else {
                index_result.expect("index snapshot");

                let listed = index
                    .list_sessions(Some(&cwd))
                    .expect("list sessions for cwd");
                prop_assert_eq!(listed.len(), 1);

                let meta = &listed[0];
                let expected_count = message_count;
                prop_assert_eq!(&meta.id, &id);
                prop_assert_eq!(&meta.cwd, &cwd);
                prop_assert_eq!(&meta.timestamp, &timestamp);
                prop_assert_eq!(&meta.path, &path.display().to_string());
                prop_assert_eq!(meta.message_count, expected_count);
                prop_assert_eq!(meta.size_bytes, content.len() as u64);
                prop_assert_eq!(&meta.name, &name);
                prop_assert!(meta.last_modified_ms >= 0);

                let other_cwd = index
                    .list_sessions(Some("definitely-not-this-cwd"))
                    .expect("list sessions for unmatched cwd");
                prop_assert!(other_cwd.is_empty());
            }
        }
    }
}
