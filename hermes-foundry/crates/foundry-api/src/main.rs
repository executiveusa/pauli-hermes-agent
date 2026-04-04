use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use foundry_billing::{enforce_budget, estimate_cost, MonthlyUsageSnapshot, PricingRule, UsageEvent};
use foundry_core::{BillingPlan, BrandingConfig, OnboardingResult, TenantConfig};
use foundry_mcp::tool_contracts;
use foundry_policy::{load_feature_gates, requires_approval};
use foundry_router::{chat, default_profiles, provider_health_snapshot, LlmChatRequest};
use foundry_runs::RunLedger;
use foundry_setup::{inspect_missing_secrets, onboarding_checklist};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tower::limit::RateLimitLayer;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

#[derive(Clone, Default)]
struct AppState {
    ledger: Arc<Mutex<RunLedger>>,
    subagents: Arc<Mutex<HashMap<String, Value>>>,
    tenants: Arc<Mutex<HashMap<String, TenantConfig>>>,
    usage: Arc<Mutex<Vec<UsageEvent>>>,
    paused_tenants: Arc<Mutex<HashMap<String, bool>>>,
}

#[derive(Debug, Deserialize)]
struct TaskRequest {
    tenant_id: String,
    owner: String,
    objective: String,
    input: Value,
    approval_gate: String,
}

#[derive(Debug, Deserialize)]
struct TenantCreateRequest {
    tenant_id: String,
    org_name: String,
    mode: Option<String>,
    app_name: Option<String>,
    support_email: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UsageQuery {
    month: Option<String>,
}

#[derive(Debug, Serialize)]
struct ApiError {
    code: &'static str,
    summary: String,
    detail: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let state = AppState::default();

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/version", get(version))
        .route("/deployment-manifest.json", get(deployment_manifest))
        .route("/openapi.json", get(openapi_stub))
        .route("/dashboard/bootstrap", get(dashboard_bootstrap))
        .route("/missing-secrets", get(missing_secrets))
        .route("/provider-profiles", get(provider_profiles))
        .route("/provider-health", get(provider_health))
        .route("/onboarding/tenant", post(onboard_tenant))
        .route("/tenants", post(create_tenant).get(list_tenants))
        .route("/tenants/:id/config", get(get_tenant_config))
        .route("/tenants/:id/pause", post(pause_tenant))
        .route("/admin/impersonate-readonly/:id", get(impersonate_readonly))
        .route("/admin/force-close-run/:id", post(force_close_run))
        .route("/admin/missing-secrets", get(missing_secrets))
        .route("/admin/usage/:id", get(get_usage))
        .route("/metrics", get(metrics))
        .route("/events", get(events))
        .route("/v1/tasks", post(create_task))
        .route("/v1/runs/:id", get(get_run))
        .route("/v1/approvals/:id/resolve", post(resolve_approval))
        .route("/v1/repos/catalog", post(catalog_repos))
        .route("/v1/prds/batches", post(generate_prd_batch))
        .route("/v1/prds/batches/:id", get(get_prd_batch))
        .route("/v1/coding-sessions", post(create_coding_session))
        .route("/v1/coding-sessions/:id", get(get_coding_session))
        .route("/v1/browser-runs", post(create_browser_run))
        .route("/v1/browser-runs/:id", get(get_browser_run))
        .route("/v1/subagents", post(create_subagent).get(list_subagents))
        .route("/v1/subagents/:id/stop", post(stop_subagent))
        .route("/v1/appwrite/projects", post(provision_appwrite_project))
        .route("/v1/usage/:id", get(get_usage))
        .route("/v1/mcp/contracts", get(mcp_contracts))
        .route("/v1/llm/chat", post(llm_chat))
        .with_state(state)
        .layer(RateLimitLayer::new(120, std::time::Duration::from_secs(60)))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let host = std::env::var("FOUNDRY_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("FOUNDRY_PORT")
        .unwrap_or_else(|_| "8788".to_string())
        .parse()
        .unwrap_or(8788);
    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn healthz() -> impl IntoResponse {
    Json(json!({"status":"ok","time":Utc::now()}))
}

async fn readyz() -> impl IntoResponse {
    Json(json!({"status":"ready","deps":["postgres","redis","providers"]}))
}

async fn version() -> impl IntoResponse {
    Json(json!({"name":"hermes-foundry","version":"0.2.0"}))
}

async fn deployment_manifest() -> impl IntoResponse {
    Json(json!({
        "service":"foundry-api",
        "version":"0.2.0",
        "migration":"001_init.sql+002_productization.sql",
        "release_notes":"phase-2-productization",
        "smoke_command":"bash infra/scripts/smoke.sh",
        "rollback":"redeploy previous image + restore backup snapshot"
    }))
}

async fn openapi_stub() -> impl IntoResponse {
    Json(json!({
        "openapi":"3.1.0",
        "info":{"title":"Hermes Foundry API","version":"0.2.0"},
        "paths":{
            "/v1/tasks":{"post":{}},
            "/v1/runs/{id}":{"get":{}},
            "/v1/usage/{id}":{"get":{}},
            "/v1/approvals/{id}/resolve":{"post":{}}
        }
    }))
}

async fn dashboard_bootstrap() -> impl IntoResponse {
    let gates = load_feature_gates("policies/feature_gates.json").unwrap_or_default();
    Json(json!({
        "top_views": ["Today","Pipeline","Clients","Delivery","Cash","Approvals","New World Kids","Knowledge"],
        "home": {
            "answers": [
                "what_money_is_coming_in",
                "what_is_blocked",
                "what_needs_approval",
                "what_was_completed",
                "what_is_at_risk"
            ],
            "max_primary_actions": 3,
            "chat_first": true,
            "cards_second": true,
            "raw_traces_default": false
        },
        "ops_panels": {
            "running_jobs": true,
            "failed_jobs": true,
            "blocked_jobs": true,
            "approval_queue": true,
            "system_health": true,
            "recent_deploys": true,
            "billing_spikes": true,
            "provider_failures": true,
            "browser_failures": true,
            "coding_failures": true
        },
        "localization": {
            "en": {"approval_needed":"Approval required"},
            "es-419": {"approval_needed":"Se requiere aprobación"}
        },
        "feature_gates": gates
    }))
}

async fn missing_secrets() -> impl IntoResponse {
    Json(json!({"missing": inspect_missing_secrets()}))
}

async fn provider_profiles() -> impl IntoResponse {
    Json(json!({"profiles": default_profiles()}))
}

async fn provider_health() -> impl IntoResponse {
    Json(json!({"providers": provider_health_snapshot()}))
}

async fn onboard_tenant(Json(req): Json<TenantCreateRequest>) -> impl IntoResponse {
    let tenant = build_tenant_config(&req);
    let result = OnboardingResult {
        tenant_record: tenant,
        operator_login_path: "/login".to_string(),
        dashboard_url: format!("/dashboard/{}", req.tenant_id),
        appwrite_status: "not_provisioned".to_string(),
        enabled_workflows: vec!["repo_factory".into(), "approvals".into(), "sprint_scope_builder".into()],
        billing_status: "active".to_string(),
        missing_setup_items: inspect_missing_secrets().into_iter().map(|x| x.key).collect(),
        checklist: onboarding_checklist().into_iter().map(|x| x.title).collect(),
    };
    Json(result)
}

async fn create_tenant(State(state): State<AppState>, Json(req): Json<TenantCreateRequest>) -> impl IntoResponse {
    let tenant = build_tenant_config(&req);
    let mut t = state.tenants.lock().await;
    t.insert(req.tenant_id.clone(), tenant.clone());
    Json(json!({"tenant": tenant, "status":"created"}))
}

async fn list_tenants(State(state): State<AppState>) -> impl IntoResponse {
    let t = state.tenants.lock().await;
    Json(json!({"tenants": t.values().cloned().collect::<Vec<_>>()}))
}

async fn get_tenant_config(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let t = state.tenants.lock().await;
    match t.get(&id) {
        Some(cfg) => Json(json!({"tenant_config":cfg})).into_response(),
        None => not_found("TENANT_NOT_FOUND", format!("No tenant config for {}", id)),
    }
}

async fn pause_tenant(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let mut p = state.paused_tenants.lock().await;
    p.insert(id.clone(), true);
    Json(json!({"tenant_id":id,"paused":true}))
}

async fn impersonate_readonly(Path(id): Path<String>) -> impl IntoResponse {
    Json(json!({"tenant_id":id,"mode":"readonly_impersonation","allowed_actions":["inspect","export"]}))
}

async fn create_task(State(state): State<AppState>, Json(req): Json<TaskRequest>) -> impl IntoResponse {
    if *state.paused_tenants.lock().await.get(&req.tenant_id).unwrap_or(&false) {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiError {
                code: "TENANT_PAUSED",
                summary: "Tenant is paused".to_string(),
                detail: "Unpause tenant before submitting tasks".to_string(),
            }),
        )
            .into_response();
    }

    let run_id = format!("run_{}", Uuid::new_v4().simple());
    let mut ledger = state.ledger.lock().await;
    let rec = ledger.create_run(
        run_id.clone(),
        req.tenant_id.clone(),
        req.owner,
        req.objective,
        req.input,
        req.approval_gate,
    );

    let pricing = PricingRule {
        plan: "pay_as_you_go".into(),
        token_in_per_million_usd: 2.0,
        token_out_per_million_usd: 6.0,
        browser_minute_usd: 0.04,
        coding_minute_usd: 0.08,
        storage_gb_usd: 0.10,
        api_call_usd: 0.001,
        mcp_call_usd: 0.001,
    };
    let event = UsageEvent {
        tenant_id: req.tenant_id.clone(),
        run_id: run_id.clone(),
        task_type: "task".into(),
        provider: "internal".into(),
        model: "n/a".into(),
        tokens_in: 0,
        tokens_out: 0,
        provider_cost_estimate_usd: 0.0,
        browser_runtime_minutes: 0.0,
        coding_runtime_minutes: 0.0,
        storage_gb: 0.0,
        workflows_run: 1,
        active_subagents: 0,
        api_calls: 1,
        mcp_calls: 0,
        billable_event_type: "run_created".into(),
    };
    let estimated = estimate_cost(&pricing, &event);
    let _ = enforce_budget(&req.tenant_id, estimated, 500.0);
    state.usage.lock().await.push(event);

    Json(json!({"run_id":run_id,"status":rec.run.status,"standard":rec.run.data,"estimated_cost_usd":estimated})).into_response()
}

async fn get_run(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let ledger = state.ledger.lock().await;
    match ledger.get_run(&id) {
        Some(rec) => Json(json!({"run": rec, "timeline": ledger.timeline.iter().filter(|x| x.run_id == id).collect::<Vec<_>>()})).into_response(),
        None => not_found("RUN_NOT_FOUND", format!("No run exists for id {}", id)),
    }
}

async fn resolve_approval(Path(id): Path<String>, Json(payload): Json<Value>) -> impl IntoResponse {
    let action = payload.get("action").and_then(|v| v.as_str()).unwrap_or("read");
    let behavior = if requires_approval(action) { "human_required" } else { "auto" };
    Json(json!({"approval_id":id,"resolved":true,"decision":payload,"behavior":behavior}))
}

async fn catalog_repos(Json(payload): Json<Value>) -> impl IntoResponse {
    Json(json!({
        "catalog_id": format!("cat_{}", Uuid::new_v4().simple()),
        "input": payload,
        "detected": {
            "languages": ["Python", "TypeScript", "Rust"],
            "frameworks": ["FastAPI", "Axum", "React"],
            "package_managers": ["pip", "npm", "cargo"],
            "risk_score": 0.34,
            "deployability_score": 0.72,
            "autonomous_maintenance_readiness": 0.61
        },
        "offer_mapping": ["AI Systems Sprint", "Repo Standardization"]
    }))
}

async fn generate_prd_batch(Json(payload): Json<Value>) -> impl IntoResponse {
    Json(json!({
        "batch_id":format!("prdb_{}",Uuid::new_v4().simple()),
        "status":"pending_approval",
        "preset":"sprint_top10",
        "templates":["sprint_prd","implementation_package"],
        "payload":payload
    }))
}

async fn get_prd_batch(Path(id): Path<String>) -> impl IntoResponse {
    Json(json!({"batch_id":id,"items":[],"status":"pending_approval"}))
}

async fn create_coding_session(Json(payload): Json<Value>) -> impl IntoResponse {
    Json(json!({"coding_session_id":format!("code_{}",Uuid::new_v4().simple()),"status":"queued","payload":payload}))
}

async fn get_coding_session(Path(id): Path<String>) -> impl IntoResponse {
    Json(json!({"coding_session_id":id,"status":"queued","contract":{"pause":true,"resume":true,"checkpointing":true}}))
}

async fn create_browser_run(Json(payload): Json<Value>) -> impl IntoResponse {
    let signal = payload.get("signal").and_then(|v| v.as_str()).unwrap_or("");
    let paused = signal.to_lowercase().contains("captcha") || signal.to_lowercase().contains("mfa");
    Json(json!({
        "browser_run_id":format!("browser_{}",Uuid::new_v4().simple()),
        "status": if paused {"paused_for_human"} else {"running"},
        "live_progress":"step_1_navigation",
        "captcha_policy":"never_solve_only_pause",
        "evidence_bundle":{"screenshots":[],"trace":"pending"},
        "operator_summary": if paused {"Blocked by human verification; safe resume required."} else {"Browser task running in allowed mode."},
        "payload":payload
    }))
}

async fn get_browser_run(Path(id): Path<String>) -> impl IntoResponse {
    Json(json!({"browser_run_id":id,"status":"running","next_action":"monitor"}))
}

async fn create_subagent(State(state): State<AppState>, Json(payload): Json<Value>) -> impl IntoResponse {
    let tenant_id = payload.get("tenant_id").and_then(|v| v.as_str()).unwrap_or("unknown");
    let mut s = state.subagents.lock().await;
    let active = s.values().filter(|v| v.get("tenant_id").and_then(|t| t.as_str()) == Some(tenant_id)).count();
    if active >= 5 {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiError {
                code: "SUBAGENT_LIMIT",
                summary: "Sub-agent limit reached".to_string(),
                detail: "Tenant has reached max active sub-agents".to_string(),
            }),
        )
            .into_response();
    }

    let sid = format!("sub_{}", Uuid::new_v4().simple());
    s.insert(
        sid.clone(),
        json!({
            "id":sid,
            "tenant_id":tenant_id,
            "owner":payload.get("owner").cloned().unwrap_or(json!("ops")),
            "purpose":payload.get("purpose").cloned().unwrap_or(json!("task")),
            "repo_scope":payload.get("repo_scope").cloned().unwrap_or(json!([])),
            "workflow_scope":payload.get("workflow_scope").cloned().unwrap_or(json!([])),
            "tool_scope":payload.get("tool_scope").cloned().unwrap_or(json!([])),
            "model_scope":payload.get("model_scope").cloned().unwrap_or(json!(["fallback_general"])),
            "time_budget_minutes":payload.get("time_budget_minutes").cloned().unwrap_or(json!(120)),
            "spend_budget_usd":payload.get("spend_budget_usd").cloned().unwrap_or(json!(50)),
            "memory_scope":payload.get("memory_scope").cloned().unwrap_or(json!("tenant")),
            "stop_conditions":payload.get("stop_conditions").cloned().unwrap_or(json!(["objective_complete","budget_exceeded"])),
            "escalation_trigger":payload.get("escalation_trigger").cloned().unwrap_or(json!("approval_needed")),
            "status":"running",
            "lifecycle":[{"event":"created","ts":Utc::now().to_rfc3339()}]
        }),
    );
    Json(json!({"subagent_id":sid,"status":"running"})).into_response()
}

async fn list_subagents(State(state): State<AppState>) -> impl IntoResponse {
    let s = state.subagents.lock().await;
    Json(json!({"subagents": s.values().cloned().collect::<Vec<_>>()}))
}

async fn stop_subagent(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let mut s = state.subagents.lock().await;
    if let Some(v) = s.get_mut(&id) {
        v["status"] = json!("stopped");
        return Json(json!({"subagent_id":id,"status":"stopped"})).into_response();
    }
    not_found("SUBAGENT_NOT_FOUND", format!("No sub-agent exists for id {}", id))
}

async fn force_close_run(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let mut ledger = state.ledger.lock().await;
    if ledger.kill_run(&id) {
        return Json(json!({"run_id":id,"status":"killed"})).into_response();
    }
    not_found("RUN_NOT_FOUND", format!("No run exists for id {}", id))
}

async fn provision_appwrite_project(Json(payload): Json<Value>) -> impl IntoResponse {
    Json(json!({
        "appwrite_project_id": format!("appw_{}", Uuid::new_v4().simple()),
        "status":"provision_requested",
        "binding_output": {
            "endpoint": std::env::var("APPWRITE_ENDPOINT").unwrap_or_else(|_|"http://appwrite:80/v1".into()),
            "project": payload.get("project_name").cloned().unwrap_or(json!("default"))
        }
    }))
}

async fn mcp_contracts() -> impl IntoResponse {
    Json(json!({"tools": tool_contracts()}))
}

async fn llm_chat(Json(req): Json<LlmChatRequest>) -> impl IntoResponse {
    match chat(req).await {
        Ok(resp) => Json(resp).into_response(),
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(ApiError {
                code: "PROVIDER_ROUTER_ERROR",
                summary: "Model provider call failed".to_string(),
                detail: e.to_string(),
            }),
        )
            .into_response(),
    }
}

async fn get_usage(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<UsageQuery>,
) -> impl IntoResponse {
    let month = q.month.unwrap_or_else(|| "current".into());
    let usage = state.usage.lock().await;
    let tenant_events: Vec<_> = usage.iter().filter(|e| e.tenant_id == id).cloned().collect();

    let snapshot = MonthlyUsageSnapshot {
        tenant_id: id,
        month,
        total_estimated_cost_usd: tenant_events.iter().map(|e| e.provider_cost_estimate_usd).sum(),
        total_tokens_in: tenant_events.iter().map(|e| e.tokens_in).sum(),
        total_tokens_out: tenant_events.iter().map(|e| e.tokens_out).sum(),
        total_browser_minutes: tenant_events.iter().map(|e| e.browser_runtime_minutes).sum(),
        total_coding_minutes: tenant_events.iter().map(|e| e.coding_runtime_minutes).sum(),
        total_api_calls: tenant_events.iter().map(|e| e.api_calls).sum(),
        total_mcp_calls: tenant_events.iter().map(|e| e.mcp_calls).sum(),
        budget_warning: false,
    };
    Json(json!({"snapshot":snapshot,"events":tenant_events}))
}

async fn metrics(State(state): State<AppState>) -> impl IntoResponse {
    let ledger = state.ledger.lock().await;
    Json(json!({
        "runs_total": ledger.runs.len(),
        "runs_failed": ledger.runs.values().filter(|r| r.run.status == "failed").count(),
        "runs_blocked": ledger.runs.values().filter(|r| r.run.status == "blocked").count(),
        "subagents_active": state.subagents.lock().await.len()
    }))
}

async fn events(State(state): State<AppState>) -> impl IntoResponse {
    let ledger = state.ledger.lock().await;
    Json(json!({"timeline": ledger.timeline}))
}

fn not_found(code: &'static str, detail: String) -> axum::response::Response {
    (
        StatusCode::NOT_FOUND,
        Json(ApiError {
            code,
            summary: "Resource not found".to_string(),
            detail,
        }),
    )
        .into_response()
}

fn build_tenant_config(req: &TenantCreateRequest) -> TenantConfig {
    let is_nwk = req.mode.as_deref() == Some("new_world_kids");
    let billing_plan = if is_nwk {
        BillingPlan::InternalNonprofitDiscount
    } else {
        BillingPlan::PayAsYouGo
    };

    TenantConfig {
        tenant_id: req.tenant_id.clone(),
        org_name: req.org_name.clone(),
        account_metadata: json!({"mode":req.mode.clone().unwrap_or_else(||"customer".to_string())}),
        branding: BrandingConfig {
            app_name: req.app_name.clone().unwrap_or_else(|| "Hermes Foundry".to_string()),
            logo_url: None,
            color_theme: "pauli-vibe-default".to_string(),
            support_email: req
                .support_email
                .clone()
                .unwrap_or_else(|| "support@example.com".to_string()),
            footer_text: "Powered by Hermes Foundry".to_string(),
            custom_domain: None,
            terms_url: None,
            privacy_url: None,
        },
        billing_plan,
        usage_limits: json!({"monthly_cap_usd":500}),
        feature_flags: json!({"repo_factory":true,"proposal_generator":true,"new_world_kids_mode":is_nwk}),
        default_provider_profiles: vec!["reasoning_primary".to_string(), "cheap_summary".to_string()],
        memory_scope: "tenant".to_string(),
        workflow_permissions: vec![
            "repo_factory".to_string(),
            "coding_session".to_string(),
            "browser_run".to_string(),
            "proposal_generator".to_string(),
        ],
        appwrite_binding: None,
        channel_bindings: json!({"dashboard":true,"telegram":true,"whatsapp":{"limited":true}}),
        support_contact: req
            .support_email
            .clone()
            .unwrap_or_else(|| "support@example.com".to_string()),
    }
}
