use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEnvelope {
    pub objective: String,
    pub owner: String,
    pub input: Value,
    pub process: Value,
    pub output: Value,
    pub metric: Value,
    pub feedback_loop: String,
    pub escalation_trigger: String,
    pub approval_gate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantScoped<T> {
    pub id: String,
    pub tenant_id: String,
    pub status: String,
    pub metadata_json: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BillingPlan {
    Free,
    PayAsYouGo,
    Sprint,
    ManagedRetainer,
    InternalNonprofitDiscount,
    ManualInvoiceOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandingConfig {
    pub app_name: String,
    pub logo_url: Option<String>,
    pub color_theme: String,
    pub support_email: String,
    pub footer_text: String,
    pub custom_domain: Option<String>,
    pub terms_url: Option<String>,
    pub privacy_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantConfig {
    pub tenant_id: String,
    pub org_name: String,
    pub account_metadata: Value,
    pub branding: BrandingConfig,
    pub billing_plan: BillingPlan,
    pub usage_limits: Value,
    pub feature_flags: Value,
    pub default_provider_profiles: Vec<String>,
    pub memory_scope: String,
    pub workflow_permissions: Vec<String>,
    pub appwrite_binding: Option<Value>,
    pub channel_bindings: Value,
    pub support_contact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingResult {
    pub tenant_record: TenantConfig,
    pub operator_login_path: String,
    pub dashboard_url: String,
    pub appwrite_status: String,
    pub enabled_workflows: Vec<String>,
    pub billing_status: String,
    pub missing_setup_items: Vec<String>,
    pub checklist: Vec<String>,
}
