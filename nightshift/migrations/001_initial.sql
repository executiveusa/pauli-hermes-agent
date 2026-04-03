-- Agent MAXX — Initial Schema (15 tables)
-- Run this via docker-entrypoint-initdb.d or manually.

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ═══ Repos ═══
CREATE TABLE repos (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    repo_url    TEXT NOT NULL UNIQUE,
    name        TEXT,
    source      TEXT DEFAULT 'github',
    last_scanned_at TIMESTAMPTZ,
    created_at  TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE repo_profiles (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    repo_id     UUID NOT NULL REFERENCES repos(id) ON DELETE CASCADE,
    language    TEXT,
    framework   TEXT,
    package_manager TEXT,
    has_tests   BOOLEAN DEFAULT FALSE,
    has_lint    BOOLEAN DEFAULT FALSE,
    has_build   BOOLEAN DEFAULT FALSE,
    has_docker  BOOLEAN DEFAULT FALSE,
    has_ci      BOOLEAN DEFAULT FALSE,
    test_command  TEXT,
    lint_command  TEXT,
    build_command TEXT,
    readme_summary TEXT,
    stars       INTEGER DEFAULT 0,
    extra       JSONB DEFAULT '{}',
    created_at  TIMESTAMPTZ DEFAULT now(),
    UNIQUE(repo_id)
);

-- ═══ PRDs ═══
CREATE TABLE prd_batches (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    status      TEXT DEFAULT 'pending',
    strategy    TEXT,
    prd_count   INTEGER DEFAULT 0,
    created_at  TIMESTAMPTZ DEFAULT now(),
    completed_at TIMESTAMPTZ
);

CREATE TABLE prds (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    batch_id    UUID REFERENCES prd_batches(id) ON DELETE SET NULL,
    repo_id     UUID REFERENCES repos(id) ON DELETE SET NULL,
    title       TEXT NOT NULL,
    body        TEXT NOT NULL,
    priority    TEXT DEFAULT 'medium',
    status      TEXT DEFAULT 'draft',
    approved_by TEXT,
    created_at  TIMESTAMPTZ DEFAULT now()
);

-- ═══ Runs ═══
CREATE TABLE runs (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    run_type        TEXT,
    status          TEXT DEFAULT 'queued',
    model           TEXT,
    started_at      TIMESTAMPTZ DEFAULT now(),
    finished_at     TIMESTAMPTZ,
    iteration_count INTEGER DEFAULT 0,
    token_usage     JSONB DEFAULT '{}',
    error           TEXT,
    metadata        JSONB DEFAULT '{}'
);

CREATE TABLE run_steps (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    run_id      UUID NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    step_index  INTEGER NOT NULL,
    tool_name   TEXT,
    tool_input  JSONB DEFAULT '{}',
    tool_output TEXT,
    token_usage JSONB DEFAULT '{}',
    duration_ms INTEGER,
    created_at  TIMESTAMPTZ DEFAULT now()
);

-- ═══ Approvals ═══
CREATE TABLE approvals (
    id                UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    run_id            UUID REFERENCES runs(id) ON DELETE CASCADE,
    risk_level        TEXT DEFAULT 'medium',
    proposed_command  TEXT,
    reason            TEXT,
    decision          TEXT,
    decided_by        TEXT,
    decided_at        TIMESTAMPTZ,
    created_at        TIMESTAMPTZ DEFAULT now()
);

-- ═══ Coding Sessions ═══
CREATE TABLE coding_sessions (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    repo_url        TEXT NOT NULL,
    prompt          TEXT,
    task_type       TEXT DEFAULT 'update',
    status          TEXT DEFAULT 'queued',
    branch          TEXT,
    provider_profile TEXT DEFAULT 'balanced',
    error           TEXT,
    created_at      TIMESTAMPTZ DEFAULT now(),
    finished_at     TIMESTAMPTZ
);

CREATE TABLE coding_session_steps (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id  UUID NOT NULL REFERENCES coding_sessions(id) ON DELETE CASCADE,
    step_name   TEXT,
    status      TEXT,
    output      TEXT,
    created_at  TIMESTAMPTZ DEFAULT now()
);

-- ═══ Browser Runs ═══
CREATE TABLE browser_runs (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    start_url       TEXT,
    state           TEXT DEFAULT 'queued',
    steps           JSONB DEFAULT '[]',
    artifact_count  INTEGER DEFAULT 0,
    error           TEXT,
    created_at      TIMESTAMPTZ DEFAULT now(),
    finished_at     TIMESTAMPTZ
);

CREATE TABLE browser_artifacts (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    browser_run_id  UUID NOT NULL REFERENCES browser_runs(id) ON DELETE CASCADE,
    kind            TEXT NOT NULL,
    label           TEXT,
    content         TEXT,
    file_path       TEXT,
    created_at      TIMESTAMPTZ DEFAULT now()
);

-- ═══ Provider Profiles ═══
CREATE TABLE provider_profiles (
    id          TEXT PRIMARY KEY,
    provider    TEXT NOT NULL,
    model       TEXT NOT NULL,
    purpose     TEXT,
    is_active   BOOLEAN DEFAULT TRUE,
    config      JSONB DEFAULT '{}',
    created_at  TIMESTAMPTZ DEFAULT now()
);

-- Seed default provider profiles
INSERT INTO provider_profiles (id, provider, model, purpose) VALUES
  ('fast',     'openrouter', 'nousresearch/hermes-3-llama-3.1-405b', 'Quick tasks'),
  ('balanced', 'anthropic',  'claude-sonnet-4-20250514',          'Default coding'),
  ('powerful', 'anthropic',  'claude-opus-4-20250514',         'Complex reasoning'),
  ('vision',   'google',     'gemini-2.0-flash',                    'Image analysis')
ON CONFLICT (id) DO NOTHING;

-- ═══ Sub-Agents ═══
CREATE TABLE sub_agents (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    label               TEXT,
    repo_scope          TEXT[] NOT NULL,
    tool_scope          TEXT[] NOT NULL,
    time_budget_minutes INTEGER DEFAULT 30,
    status              TEXT DEFAULT 'pending',
    started_at          TIMESTAMPTZ,
    finished_at         TIMESTAMPTZ,
    error               TEXT,
    created_at          TIMESTAMPTZ DEFAULT now()
);

-- ═══ Appwrite Projects ═══
CREATE TABLE appwrite_projects (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name                TEXT NOT NULL,
    status              TEXT DEFAULT 'provisioning',
    endpoint            TEXT,
    project_id          TEXT,
    api_key             TEXT,
    created_at          TIMESTAMPTZ DEFAULT now()
);

-- ═══ Artifacts (generic) ═══
CREATE TABLE artifacts (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    parent_type TEXT NOT NULL,
    parent_id   UUID NOT NULL,
    kind        TEXT NOT NULL,
    label       TEXT,
    content     TEXT,
    file_path   TEXT,
    created_at  TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_artifacts_parent ON artifacts(parent_type, parent_id);
CREATE INDEX idx_runs_status ON runs(status);
CREATE INDEX idx_approvals_pending ON approvals(decision) WHERE decision IS NULL;
CREATE INDEX idx_coding_sessions_status ON coding_sessions(status);
CREATE INDEX idx_browser_runs_state ON browser_runs(state);
