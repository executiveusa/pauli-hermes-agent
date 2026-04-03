use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use foundry_billing::{enforce_budget, UsageEvent};
use foundry_mcp::tool_contracts;
use foundry_policy::requires_approval;
use foundry_router::{chat, default_profiles, LlmChatRequest};
use foundry_runs::RunLedger;
use foundry_setup::inspect_missing_secrets;
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
}

#[derive(Debug, Deserialize)]
struct TaskRequest {
    tenant_id: String,
    owner: String,
    objective: String,
    input: Value,
    approval_gate: String,
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
        .route("/dashboard/bootstrap", get(dashboard_bootstrap))
        .route("/missing-secrets", get(missing_secrets))
        .route("/provider-profiles", get(provider_profiles))
        .route("/tasks", post(create_task))
        .route("/runs/:id", get(get_run))
        .route("/approvals/:id/resolve", post(resolve_approval))
        .route("/repos/catalog", post(catalog_repos))
        .route("/prds/batches", post(generate_prd_batch))
        .route("/prds/batches/:id", get(get_prd_batch))
        .route("/coding-sessions", post(create_coding_session))
        .route("/coding-sessions/:id", get(get_coding_session))
        .route("/browser-runs", post(create_browser_run))
        .route("/browser-runs/:id", get(get_browser_run))
        .route("/subagents", post(create_subagent).get(list_subagents))
        .route("/subagents/:id/stop", post(stop_subagent))
        .route("/appwrite/projects", post(provision_appwrite_project))
        .route("/mcp/contracts", get(mcp_contracts))
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
    Json(json!({"status":"ready"}))
}

async fn version() -> impl IntoResponse {
    Json(json!({"name":"hermes-foundry","version":"0.1.0"}))
}

async fn dashboard_bootstrap() -> impl IntoResponse {
    Json(json!({
        "top_views": ["Today","Pipeline","Clients","Delivery","Cash","Approvals","New World Kids","Knowledge"],
        "home_questions": [
            "what_money_is_coming_in",
            "what_is_blocked",
            "what_needs_approval",
            "what_was_completed",
            "what_is_at_risk"
        ],
        "localization": {
            "en": {"approval_needed":"Approval required"},
            "es-419": {"approval_needed":"Se requiere aprobación"}
        },
        "actions_max_choices": 3,
        "chat_first": true,
        "cards_second": true,
        "raw_traces_default": false
    }))
}

async fn missing_secrets() -> impl IntoResponse {
    Json(json!({"missing": inspect_missing_secrets()}))
}

async fn provider_profiles() -> impl IntoResponse {
    Json(json!({"profiles": default_profiles()}))
}

async fn create_task(State(state): State<AppState>, Json(req): Json<TaskRequest>) -> impl IntoResponse {
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

    let _ = enforce_budget(&req.tenant_id, 0.0, 500.0);
    let _event = UsageEvent {
        tenant_id: req.tenant_id,
        run_id: run_id.clone(),
        task_type: "task".into(),
        provider: "internal".into(),
        model: "n/a".into(),
        tokens_used: 0,
        compute_ms: 0,
        browser_ms: 0,
        worker_ms: 0,
        estimated_cost_usd: 0.0,
        billable_event_type: "run_created".into(),
    };

    Json(json!({"run_id":run_id,"status":rec.run.status,"standard":rec.run.data}))
}

async fn get_run(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let ledger = state.ledger.lock().await;
    match ledger.get_run(&id) {
        Some(rec) => Json(json!({"run": rec})).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiError {
                code: "RUN_NOT_FOUND",
                summary: "Run not found".to_string(),
                detail: format!("No run exists for id {}", id),
            }),
        )
            .into_response(),
    }
}

async fn resolve_approval(Path(id): Path<String>, Json(payload): Json<Value>) -> impl IntoResponse {
    Json(json!({"approval_id":id,"resolved":true,"decision":payload}))
}

async fn catalog_repos(Json(payload): Json<Value>) -> impl IntoResponse {
    Json(json!({
        "catalog_id": format!("cat_{}", Uuid::new_v4().simple()),
        "input": payload,
        "detected": {
            "languages": ["Python", "TypeScript", "Rust"],
            "package_managers": ["pip", "npm", "cargo"],
            "risk_score": 0.34,
            "deployability_score": 0.72
        }
    }))
}

async fn generate_prd_batch(Json(payload): Json<Value>) -> impl IntoResponse {
    Json(json!({"batch_id":format!("prdb_{}",Uuid::new_v4().simple()),"status":"pending_approval","payload":payload}))
}

async fn get_prd_batch(Path(id): Path<String>) -> impl IntoResponse {
    Json(json!({"batch_id":id,"items":[],"status":"pending_approval"}))
}

async fn create_coding_session(Json(payload): Json<Value>) -> impl IntoResponse {
    Json(json!({"coding_session_id":format!("code_{}",Uuid::new_v4().simple()),"status":"queued","payload":payload}))
}

async fn get_coding_session(Path(id): Path<String>) -> impl IntoResponse {
    Json(json!({"coding_session_id":id,"status":"queued"}))
}

async fn create_browser_run(Json(payload): Json<Value>) -> impl IntoResponse {
    let signal = payload.get("signal").and_then(|v| v.as_str()).unwrap_or("");
    let paused = signal.to_lowercase().contains("captcha") || signal.to_lowercase().contains("mfa");
    Json(json!({
        "browser_run_id":format!("browser_{}",Uuid::new_v4().simple()),
        "status": if paused {"paused_for_human"} else {"running"},
        "captcha_policy":"never_solve_only_pause",
        "payload":payload
    }))
}

async fn get_browser_run(Path(id): Path<String>) -> impl IntoResponse {
    Json(json!({"browser_run_id":id,"status":"running"}))
}

async fn create_subagent(State(state): State<AppState>, Json(payload): Json<Value>) -> impl IntoResponse {
    let sid = format!("sub_{}", Uuid::new_v4().simple());
    let mut s = state.subagents.lock().await;
    s.insert(
        sid.clone(),
        json!({"id":sid,"status":"running","payload":payload}),
    );
    Json(json!({"subagent_id":sid,"status":"running"}))
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
    (
        StatusCode::NOT_FOUND,
        Json(ApiError {
            code: "SUBAGENT_NOT_FOUND",
            summary: "Sub-agent not found".to_string(),
            detail: format!("No sub-agent exists for id {}", id),
        }),
    )
        .into_response()
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

#[allow(dead_code)]
fn _approval_gate_for_action(action: &str) -> &'static str {
    if requires_approval(action) {
        "human_required"
    } else {
        "auto"
    }
}
