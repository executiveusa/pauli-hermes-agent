use chrono::Utc;
use foundry_billing::UsageEvent;
use foundry_core::{TenantScoped, WorkflowEnvelope};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub run: TenantScoped<WorkflowEnvelope>,
    pub kill_switch_engaged: bool,
}

#[derive(Default)]
pub struct RunLedger {
    pub runs: HashMap<String, RunRecord>,
    pub usage: Vec<UsageEvent>,
}

impl RunLedger {
    pub fn create_run(&mut self, id: String, tenant_id: String, owner: String, objective: String, input: serde_json::Value, approval_gate: String) -> RunRecord {
        let now = Utc::now();
        let run = TenantScoped {
            id: id.clone(),
            tenant_id,
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
        let rec = RunRecord { run, kill_switch_engaged: false };
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
            true
        } else {
            false
        }
    }
}
