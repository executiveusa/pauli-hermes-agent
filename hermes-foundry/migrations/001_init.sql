CREATE TABLE tenants (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  status TEXT NOT NULL,
  branding_json JSONB NOT NULL DEFAULT '{}'::jsonb,
  metadata_json JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE runs (
  id TEXT PRIMARY KEY,
  tenant_id TEXT NOT NULL REFERENCES tenants(id),
  status TEXT NOT NULL,
  objective TEXT NOT NULL,
  owner TEXT NOT NULL,
  input_json JSONB NOT NULL,
  process_json JSONB NOT NULL,
  output_json JSONB NOT NULL,
  metric_json JSONB NOT NULL,
  feedback_loop TEXT NOT NULL,
  escalation_trigger TEXT NOT NULL,
  approval_gate TEXT NOT NULL,
  metadata_json JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE approvals (
  id TEXT PRIMARY KEY,
  tenant_id TEXT NOT NULL REFERENCES tenants(id),
  run_id TEXT NOT NULL REFERENCES runs(id),
  status TEXT NOT NULL,
  action TEXT NOT NULL,
  reason TEXT NOT NULL,
  metadata_json JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE invoices_or_usage_ledger (
  id TEXT PRIMARY KEY,
  tenant_id TEXT NOT NULL REFERENCES tenants(id),
  run_id TEXT NOT NULL REFERENCES runs(id),
  task_type TEXT NOT NULL,
  provider TEXT NOT NULL,
  model TEXT NOT NULL,
  tokens_used BIGINT NOT NULL DEFAULT 0,
  compute_ms BIGINT NOT NULL DEFAULT 0,
  browser_ms BIGINT NOT NULL DEFAULT 0,
  worker_ms BIGINT NOT NULL DEFAULT 0,
  tool_usage_json JSONB NOT NULL DEFAULT '{}'::jsonb,
  estimated_cost_usd NUMERIC(12,6) NOT NULL DEFAULT 0,
  billable_event_type TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
