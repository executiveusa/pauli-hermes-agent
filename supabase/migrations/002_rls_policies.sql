-- Row Level Security policies for Supabase
-- Applied after 001_core_schema.sql

-- Enable RLS on all tables
ALTER TABLE agents ENABLE ROW LEVEL SECURITY;
ALTER TABLE sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE messages ENABLE ROW LEVEL SECURITY;
ALTER TABLE beads ENABLE ROW LEVEL SECURITY;
ALTER TABLE health_checks ENABLE ROW LEVEL SECURITY;
ALTER TABLE cron_results ENABLE ROW LEVEL SECURITY;
ALTER TABLE agent_mail ENABLE ROW LEVEL SECURITY;
ALTER TABLE skills ENABLE ROW LEVEL SECURITY;
ALTER TABLE tool_usage ENABLE ROW LEVEL SECURITY;
ALTER TABLE file_reservations ENABLE ROW LEVEL SECURITY;

-- Service role has full access (API server uses service key)
-- These policies allow the service_role to do everything
-- Anon/authenticated access is denied by default (no public API)

CREATE POLICY "service_full_access" ON agents FOR ALL USING (auth.role() = 'service_role');
CREATE POLICY "service_full_access" ON sessions FOR ALL USING (auth.role() = 'service_role');
CREATE POLICY "service_full_access" ON messages FOR ALL USING (auth.role() = 'service_role');
CREATE POLICY "service_full_access" ON beads FOR ALL USING (auth.role() = 'service_role');
CREATE POLICY "service_full_access" ON health_checks FOR ALL USING (auth.role() = 'service_role');
CREATE POLICY "service_full_access" ON cron_results FOR ALL USING (auth.role() = 'service_role');
CREATE POLICY "service_full_access" ON agent_mail FOR ALL USING (auth.role() = 'service_role');
CREATE POLICY "service_full_access" ON skills FOR ALL USING (auth.role() = 'service_role');
CREATE POLICY "service_full_access" ON tool_usage FOR ALL USING (auth.role() = 'service_role');
CREATE POLICY "service_full_access" ON file_reservations FOR ALL USING (auth.role() = 'service_role');

-- Read-only access for authenticated dashboard users (optional — enable when dashboard auth is added)
-- CREATE POLICY "dashboard_read" ON sessions FOR SELECT USING (auth.role() = 'authenticated');
-- CREATE POLICY "dashboard_read" ON messages FOR SELECT USING (auth.role() = 'authenticated');
-- CREATE POLICY "dashboard_read" ON beads FOR SELECT USING (auth.role() = 'authenticated');
-- CREATE POLICY "dashboard_read" ON health_checks FOR SELECT USING (auth.role() = 'authenticated');
