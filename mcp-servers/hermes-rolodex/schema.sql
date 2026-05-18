-- Hermes Rolodex™ — SQLite Schema with FTS5
-- Auto-executed on first server.py run
-- Storage: ~/.hermes/rolodex.db (override via ROLODEX_DB_PATH env var)

CREATE TABLE IF NOT EXISTS people (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    role            TEXT,
    company         TEXT,
    email           TEXT,
    phone           TEXT,
    location        TEXT,
    birthday        TEXT,           -- ISO: YYYY-MM-DD
    photo_url       TEXT,
    strength        REAL DEFAULT 0.7,
    strength_label  TEXT DEFAULT 'WARM',
    last_contact_at TEXT,           -- ISO datetime
    notes           TEXT,
    context_tags    TEXT DEFAULT '[]',  -- JSON array
    created_at      TEXT DEFAULT (datetime('now')),
    updated_at      TEXT DEFAULT (datetime('now'))
);

-- FTS5 virtual table for full-text search across all recall fields
CREATE VIRTUAL TABLE IF NOT EXISTS people_fts
    USING fts5(
        id UNINDEXED,
        name,
        role,
        company,
        notes,
        context_tags_flat
    );

CREATE TABLE IF NOT EXISTS connections (
    id              TEXT PRIMARY KEY,
    person_a_id     TEXT NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    person_b_id     TEXT NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    connection_type TEXT,
    context         TEXT,
    strength        REAL DEFAULT 0.5,
    created_at      TEXT DEFAULT (datetime('now')),
    UNIQUE(person_a_id, person_b_id)
);

CREATE TABLE IF NOT EXISTS memory_items (
    id          TEXT PRIMARY KEY,
    person_id   TEXT NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    text        TEXT NOT NULL,
    source      TEXT DEFAULT 'HERMES',
    context     TEXT,
    timestamp   TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS person_events (
    id          TEXT PRIMARY KEY,
    person_id   TEXT NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    type        TEXT NOT NULL,
    title       TEXT NOT NULL,
    date        TEXT NOT NULL,
    fired       INTEGER DEFAULT 0,
    fired_at    TEXT,
    created_at  TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS unknown_queue (
    id          TEXT PRIMARY KEY,
    description TEXT NOT NULL,
    session_id  TEXT,
    created_at  TEXT DEFAULT (datetime('now')),
    resolved    INTEGER DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_memory_person   ON memory_items(person_id);
CREATE INDEX IF NOT EXISTS idx_events_date     ON person_events(date, fired);
CREATE INDEX IF NOT EXISTS idx_events_person   ON person_events(person_id);
CREATE INDEX IF NOT EXISTS idx_strength_label  ON people(strength_label);
CREATE INDEX IF NOT EXISTS idx_last_contact    ON people(last_contact_at);
