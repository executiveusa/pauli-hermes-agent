use chrono::Utc;
use foundry_billing::UsageEvent;
use foundry_core::{TenantScoped, WorkflowEnvelope};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunTimelineEvent {
    pub ts: String,
    pub run_id: String,
    pub tenant_id: String,
    pub event: String,
    pub status: String,
    pub summary: String,
    pub next_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub run: TenantScoped<WorkflowEnvelope>,
    pub parent_goal: String,
    pub worker: String,
    pub provider: String,
    pub model: String,
    pub cost_estimate_usd: f64,
    pub artifact_links: Vec<String>,
    pub error_summary: Option<String>,
    pub next_action: String,
    pub kill_switch_engaged: bool,
}

#[derive(Default)]
pub struct RunLedger {
    pub runs: HashMap<String, RunRecord>,
    pub usage: Vec<UsageEvent>,
    pub timeline: Vec<RunTimelineEvent>,
}

impl RunLedger {
    pub fn create_run(
        &mut self,
        id: String,
        tenant_id: String,
        owner: String,
        objective: String,
        input: serde_json::Value,
        approval_gate: String,
    ) -> RunRecord {
        let now = Utc::now();
        let run = TenantScoped {
            id: id.clone(),
            tenant_id: tenant_id.clone(),
            status: "queued".to_string(),
            metadata_json: json!({}),
            created_at: now,
            updated_at: now,
            data: WorkflowEnvelope {
                objective,
                owner,
                input,
                process: json!({"steps": []}),
                output: json!({}),
                metric: json!({"revenue_impact": "tbd"}),
                feedback_loop: "post_run_retrospective".to_string(),
                escalation_trigger: "budget_or_approval_blocked".to_string(),
                approval_gate,
            },
        };
        let rec = RunRecord {
            run,
            parent_goal: "first-stable-recurring-revenue".to_string(),
            worker: "orchestrator".to_string(),
            provider: "internal".to_string(),
            model: "n/a".to_string(),
            cost_estimate_usd: 0.0,
            artifact_links: vec![],
            error_summary: None,
            next_action: "await_worker_assignment".to_string(),
            kill_switch_engaged: false,
        };
        self.timeline.push(RunTimelineEvent {
            ts: now.to_rfc3339(),
            run_id: id.clone(),
            tenant_id,
            event: "run_created".to_string(),
            status: "queued".to_string(),
            summary: "Run created and queued.".to_string(),
            next_action: "assign_worker".to_string(),
        });
        self.runs.insert(id, rec.clone());
        rec
    }

    pub fn get_run(&self, id: &str) -> Option<&RunRecord> {
        self.runs.get(id)
    }

    pub fn kill_run(&mut self, id: &str) -> bool {
        if let Some(rec) = self.runs.get_mut(id) {
            rec.kill_switch_engaged = true;
            rec.run.status = "killed".to_string();
            rec.next_action = "manual_review".to_string();
            self.timeline.push(RunTimelineEvent {
                ts: Utc::now().to_rfc3339(),
                run_id: id.to_string(),
                tenant_id: rec.run.tenant_id.clone(),
                event: "kill_switch".to_string(),
                status: "killed".to_string(),
                summary: "Run stopped by operator.".to_string(),
                next_action: "incident_review".to_string(),
            });
            true
        } else {
            false
        }
    }
}
