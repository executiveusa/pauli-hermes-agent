-- Archon X Supabase Schema — Core Tables
-- Applied to self-hosted Supabase at 31.220.58.212:8001
-- Three schemas: hermes (business), zero (personal), shared (coordination)

-- ============================================================
-- SHARED SCHEMA — Cross-agent coordination
-- ============================================================

-- Agent registry: all known agents and their endpoints
CREATE TABLE IF NOT EXISTS agents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    endpoint TEXT,              -- e.g. http://localhost:8642
    agent_type TEXT NOT NULL DEFAULT 'hermes',  -- hermes, zero, darya
    status TEXT NOT NULL DEFAULT 'offline',     -- online, offline, busy
    capabilities JSONB DEFAULT '[]'::jsonb,
    last_heartbeat TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Sessions: conversation sessions across all agents
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID REFERENCES agents(id),
    title TEXT,
    platform TEXT NOT NULL DEFAULT 'cli',
    message_count INTEGER NOT NULL DEFAULT 0,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_sessions_agent ON sessions(agent_id);
CREATE INDEX IF NOT EXISTS idx_sessions_updated ON sessions(updated_at DESC);

-- Messages: individual messages within sessions
CREATE TABLE IF NOT EXISTS messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    role TEXT NOT NULL,  -- user, assistant, system, tool
    content TEXT,
    tool_calls JSONB,
    tool_results JSONB,
    tokens_in INTEGER,
    tokens_out INTEGER,
    model TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id, created_at);

-- Beads: issue/action tracking log
CREATE TABLE IF NOT EXISTS beads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID REFERENCES agents(id),
    bead_type TEXT NOT NULL DEFAULT 'action',  -- action, error, info, milestone
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'open',        -- open, in_progress, resolved, closed
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_beads_agent ON beads(agent_id);
CREATE INDEX IF NOT EXISTS idx_beads_status ON beads(status);
CREATE INDEX IF NOT EXISTS idx_beads_created ON beads(created_at DESC);

-- Health checks: periodic health monitoring results
CREATE TABLE IF NOT EXISTS health_checks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID REFERENCES agents(id),
    endpoint TEXT NOT NULL,
    status TEXT NOT NULL,      -- ok, error, timeout
    response_ms INTEGER,
    metadata JSONB DEFAULT '{}'::jsonb,
    checked_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_health_agent ON health_checks(agent_id, checked_at DESC);

-- Cron results: execution log for scheduled jobs
CREATE TABLE IF NOT EXISTS cron_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_name TEXT NOT NULL,
    agent_id UUID REFERENCES agents(id),
    status TEXT NOT NULL,      -- success, failure, timeout
    output TEXT,
    started_at TIMESTAMPTZ NOT NULL,
    finished_at TIMESTAMPTZ,
    duration_ms INTEGER
);
CREATE INDEX IF NOT EXISTS idx_cron_job ON cron_results(job_name, started_at DESC);

-- Agent mail: messages between agents (coordination protocol)
CREATE TABLE IF NOT EXISTS agent_mail (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_agent TEXT NOT NULL,
    to_agent TEXT NOT NULL,
    thread_id UUID,
    subject TEXT,
    body TEXT NOT NULL,
    priority TEXT NOT NULL DEFAULT 'normal',  -- low, normal, high, urgent
    status TEXT NOT NULL DEFAULT 'pending',   -- pending, read, acknowledged
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    read_at TIMESTAMPTZ,
    acknowledged_at TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_mail_to ON agent_mail(to_agent, status);
CREATE INDEX IF NOT EXISTS idx_mail_thread ON agent_mail(thread_id);

-- Skill registry: installed skills across agents
CREATE TABLE IF NOT EXISTS skills (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    agent_id UUID REFERENCES agents(id),
    description TEXT,
    version TEXT,
    enabled BOOLEAN NOT NULL DEFAULT true,
    source TEXT,               -- path or URL where skill was loaded from
    metadata JSONB DEFAULT '{}'::jsonb,
    installed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_skills_agent ON skills(agent_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_skills_unique ON skills(name, agent_id);

-- Tool usage: analytics for tool calls
CREATE TABLE IF NOT EXISTS tool_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID REFERENCES sessions(id),
    agent_id UUID REFERENCES agents(id),
    tool_name TEXT NOT NULL,
    toolset TEXT,
    duration_ms INTEGER,
    success BOOLEAN NOT NULL DEFAULT true,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_tool_usage_tool ON tool_usage(tool_name, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_tool_usage_session ON tool_usage(session_id);

-- File reservations: agent-mail file locking
CREATE TABLE IF NOT EXISTS file_reservations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_path TEXT NOT NULL,
    agent_id UUID REFERENCES agents(id),
    reserved_by TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',  -- active, released, expired
    reserved_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ,
    released_at TIMESTAMPTZ
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_file_res_active ON file_reservations(file_path) WHERE status = 'active';

-- Updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply triggers
DO $$
DECLARE
    tbl TEXT;
BEGIN
    FOR tbl IN SELECT unnest(ARRAY['agents', 'sessions']) LOOP
        EXECUTE format('DROP TRIGGER IF EXISTS set_updated_at ON %I', tbl);
        EXECUTE format(
            'CREATE TRIGGER set_updated_at BEFORE UPDATE ON %I FOR EACH ROW EXECUTE FUNCTION update_updated_at()',
            tbl
        );
    END LOOP;
END;
$$;

-- Seed default agents
INSERT INTO agents (name, endpoint, agent_type, status) VALUES
    ('hermes', 'http://localhost:8642', 'hermes', 'offline'),
    ('agent-zero', 'http://localhost:8643', 'zero', 'offline')
ON CONFLICT (name) DO NOTHING;
