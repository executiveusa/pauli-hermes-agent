-- Seed data: default cron jobs, skills, and health check targets
-- Applied after RLS policies

-- Default scheduled health checks (insert as cron_results template reference)
-- These define what the health monitor should check periodically.
-- The actual cron scheduling is handled by the Python cron/ module.

-- Seed hermes skills from the built-in set
INSERT INTO skills (name, agent_id, description, source, enabled)
SELECT
    s.name,
    a.id,
    s.description,
    s.source,
    true
FROM (VALUES
    ('web_search', 'Search the web using parallel queries', 'tools/web_tools.py'),
    ('web_extract', 'Extract content from web pages', 'tools/web_tools.py'),
    ('file_read', 'Read file contents', 'tools/file_tools.py'),
    ('file_write', 'Write/edit files', 'tools/file_tools.py'),
    ('file_search', 'Search for files by name/pattern', 'tools/file_tools.py'),
    ('terminal', 'Execute terminal commands', 'tools/terminal_tool.py'),
    ('browser_navigate', 'Navigate browser to URL', 'tools/browser_tool.py'),
    ('execute_code', 'Execute code in sandbox', 'tools/code_execution_tool.py'),
    ('delegate_task', 'Delegate to sub-agent', 'tools/delegate_tool.py'),
    ('mcp_tool', 'Call MCP server tools', 'tools/mcp_tool.py')
) AS s(name, description, source)
CROSS JOIN agents a
WHERE a.name = 'hermes'
ON CONFLICT (name, agent_id) DO NOTHING;
