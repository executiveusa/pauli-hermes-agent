CREATE TABLE brands (
  id TEXT PRIMARY KEY,
  tenant_id TEXT NOT NULL REFERENCES tenants(id),
  app_name TEXT NOT NULL,
  logo_url TEXT,
  color_theme TEXT NOT NULL,
  support_email TEXT NOT NULL,
  footer_text TEXT NOT NULL,
  custom_domain TEXT,
  terms_url TEXT,
  privacy_url TEXT,
  status TEXT NOT NULL DEFAULT 'active',
  metadata_json JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE tenant_configs (
  id TEXT PRIMARY KEY,
  tenant_id TEXT NOT NULL REFERENCES tenants(id),
  billing_plan TEXT NOT NULL,
  usage_limits_json JSONB NOT NULL DEFAULT '{}'::jsonb,
  feature_flags_json JSONB NOT NULL DEFAULT '{}'::jsonb,
  default_provider_profiles_json JSONB NOT NULL DEFAULT '[]'::jsonb,
  memory_scope TEXT NOT NULL DEFAULT 'tenant',
  workflow_permissions_json JSONB NOT NULL DEFAULT '[]'::jsonb,
  appwrite_binding_json JSONB,
  channel_bindings_json JSONB NOT NULL DEFAULT '{}'::jsonb,
  support_contact_json JSONB NOT NULL DEFAULT '{}'::jsonb,
  status TEXT NOT NULL DEFAULT 'active',
  metadata_json JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE pricing_rules (
  id TEXT PRIMARY KEY,
  plan TEXT NOT NULL,
  token_in_per_million_usd NUMERIC(12,6) NOT NULL,
  token_out_per_million_usd NUMERIC(12,6) NOT NULL,
  browser_minute_usd NUMERIC(12,6) NOT NULL,
  coding_minute_usd NUMERIC(12,6) NOT NULL,
  storage_gb_usd NUMERIC(12,6) NOT NULL,
  api_call_usd NUMERIC(12,6) NOT NULL,
  mcp_call_usd NUMERIC(12,6) NOT NULL,
  status TEXT NOT NULL DEFAULT 'active',
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE monthly_usage_snapshots (
  id TEXT PRIMARY KEY,
  tenant_id TEXT NOT NULL REFERENCES tenants(id),
  month TEXT NOT NULL,
  total_estimated_cost_usd NUMERIC(12,6) NOT NULL DEFAULT 0,
  total_tokens_in BIGINT NOT NULL DEFAULT 0,
  total_tokens_out BIGINT NOT NULL DEFAULT 0,
  total_browser_minutes NUMERIC(12,3) NOT NULL DEFAULT 0,
  total_coding_minutes NUMERIC(12,3) NOT NULL DEFAULT 0,
  total_api_calls BIGINT NOT NULL DEFAULT 0,
  total_mcp_calls BIGINT NOT NULL DEFAULT 0,
  budget_warning BOOLEAN NOT NULL DEFAULT FALSE,
  status TEXT NOT NULL DEFAULT 'active',
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE onboarding_runs (
  id TEXT PRIMARY KEY,
  tenant_id TEXT NOT NULL REFERENCES tenants(id),
  mode TEXT NOT NULL,
  checklist_json JSONB NOT NULL,
  missing_setup_items_json JSONB NOT NULL,
  status TEXT NOT NULL DEFAULT 'completed',
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
