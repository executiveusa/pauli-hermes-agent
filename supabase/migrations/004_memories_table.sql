-- Open Brain: Memories table for persistent knowledge storage
-- Used by tools/open_brain_tool.py via Supabase REST API

CREATE TABLE IF NOT EXISTS memories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    collection TEXT NOT NULL DEFAULT 'general',
    tags JSONB DEFAULT '[]'::jsonb,
    links JSONB DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_memories_collection ON memories(collection);
CREATE INDEX IF NOT EXISTS idx_memories_created ON memories(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_memories_tags ON memories USING GIN(tags);

-- Full-text search index on title + content
ALTER TABLE memories ADD COLUMN IF NOT EXISTS fts tsvector
    GENERATED ALWAYS AS (to_tsvector('english', coalesce(title, '') || ' ' || coalesce(content, ''))) STORED;
CREATE INDEX IF NOT EXISTS idx_memories_fts ON memories USING GIN(fts);

-- RLS: allow anon key to read/write memories (single-user self-hosted)
ALTER TABLE memories ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Allow all access to memories"
    ON memories FOR ALL
    USING (true)
    WITH CHECK (true);
