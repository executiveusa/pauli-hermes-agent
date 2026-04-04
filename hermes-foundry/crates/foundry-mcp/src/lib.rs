use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContract {
    pub name: &'static str,
    pub description: &'static str,
    pub risk_level: &'static str,
    pub tenant_aware: bool,
    pub approval_behavior: &'static str,
    pub input_schema: &'static str,
    pub output_schema: &'static str,
}

pub fn tool_contracts() -> Vec<ToolContract> {
    vec![
        ToolContract { name: "catalog_repos", description: "Catalog repositories and compute risk/deployability.", risk_level: "safe_read", tenant_aware: true, approval_behavior: "auto", input_schema: "{tenant_id,repos[]}", output_schema: "{catalog_id,scorecards[]}" },
        ToolContract { name: "generate_prd_batch", description: "Generate a PRD batch for selected repos.", risk_level: "safe_write", tenant_aware: true, approval_behavior: "approval_required", input_schema: "{tenant_id,catalog_id,preset}", output_schema: "{batch_id,status}" },
        ToolContract { name: "list_prd_batch", description: "List PRD batch status and items.", risk_level: "safe_read", tenant_aware: true, approval_behavior: "auto", input_schema: "{tenant_id,batch_id}", output_schema: "{items[],status}" },
        ToolContract { name: "approve_prd", description: "Approve a PRD item for execution.", risk_level: "sensitive_write", tenant_aware: true, approval_behavior: "human_required", input_schema: "{tenant_id,prd_id,decision}", output_schema: "{status}" },
        ToolContract { name: "create_coding_session", description: "Start coding session workflow.", risk_level: "safe_write", tenant_aware: true, approval_behavior: "policy_based", input_schema: "{tenant_id,repo_id,prd_id}", output_schema: "{coding_session_id,status}" },
        ToolContract { name: "create_browser_run", description: "Start browser workflow run.", risk_level: "safe_write", tenant_aware: true, approval_behavior: "policy_based", input_schema: "{tenant_id,goal,allowed_domains[]}", output_schema: "{browser_run_id,status}" },
        ToolContract { name: "pause_browser_run", description: "Pause browser run for intervention.", risk_level: "safe_write", tenant_aware: true, approval_behavior: "auto", input_schema: "{tenant_id,browser_run_id}", output_schema: "{status}" },
        ToolContract { name: "resume_browser_run", description: "Resume browser run after intervention.", risk_level: "safe_write", tenant_aware: true, approval_behavior: "human_required_if_captcha", input_schema: "{tenant_id,browser_run_id}", output_schema: "{status}" },
        ToolContract { name: "cancel_browser_run", description: "Cancel browser run.", risk_level: "sensitive_write", tenant_aware: true, approval_behavior: "human_required", input_schema: "{tenant_id,browser_run_id}", output_schema: "{status}" },
        ToolContract { name: "provision_appwrite_project", description: "Provision and bind Appwrite project.", risk_level: "sensitive_write", tenant_aware: true, approval_behavior: "human_required", input_schema: "{tenant_id,name,region}", output_schema: "{project_id,status}" },
        ToolContract { name: "list_appwrite_projects", description: "List Appwrite project bindings.", risk_level: "safe_read", tenant_aware: true, approval_behavior: "auto", input_schema: "{tenant_id}", output_schema: "{projects[]}" },
        ToolContract { name: "create_subagent", description: "Create disposable scoped sub-agent.", risk_level: "sensitive_write", tenant_aware: true, approval_behavior: "policy_based", input_schema: "{tenant_id,template,scope}", output_schema: "{subagent_id,status}" },
        ToolContract { name: "list_subagents", description: "List sub-agent statuses.", risk_level: "safe_read", tenant_aware: true, approval_behavior: "auto", input_schema: "{tenant_id}", output_schema: "{subagents[]}" },
        ToolContract { name: "stop_subagent", description: "Stop sub-agent immediately.", risk_level: "sensitive_write", tenant_aware: true, approval_behavior: "human_required", input_schema: "{tenant_id,subagent_id}", output_schema: "{status}" },
        ToolContract { name: "get_run_status", description: "Get run state and summary.", risk_level: "safe_read", tenant_aware: true, approval_behavior: "auto", input_schema: "{tenant_id,run_id}", output_schema: "{status,timeline,next_action}" },
        ToolContract { name: "list_missing_setup_items", description: "List missing setup items/secrets.", risk_level: "safe_read", tenant_aware: true, approval_behavior: "auto", input_schema: "{tenant_id}", output_schema: "{items[]}" },
        ToolContract { name: "get_kpi_snapshot", description: "Return revenue/bottleneck/approvals snapshot.", risk_level: "safe_read", tenant_aware: true, approval_behavior: "auto", input_schema: "{tenant_id}", output_schema: "{kpis}" },
        ToolContract { name: "get_pipeline_snapshot", description: "Return pipeline stage snapshot.", risk_level: "safe_read", tenant_aware: true, approval_behavior: "auto", input_schema: "{tenant_id}", output_schema: "{pipeline}" },
        ToolContract { name: "get_today_summary", description: "Return operator-friendly today summary.", risk_level: "safe_read", tenant_aware: true, approval_behavior: "auto", input_schema: "{tenant_id}", output_schema: "{summary}" },
        ToolContract { name: "get_approvals", description: "Return approvals queue.", risk_level: "safe_read", tenant_aware: true, approval_behavior: "auto", input_schema: "{tenant_id}", output_schema: "{approvals[]}" },
        ToolContract { name: "submit_task", description: "Submit task to foundry.", risk_level: "safe_write", tenant_aware: true, approval_behavior: "policy_based", input_schema: "{tenant_id,objective,input}", output_schema: "{run_id,status}" },
        ToolContract { name: "get_usage_summary", description: "Return billing and usage summary.", risk_level: "safe_read", tenant_aware: true, approval_behavior: "auto", input_schema: "{tenant_id,month}", output_schema: "{usage,costs}" },
        ToolContract { name: "list_enabled_workflows", description: "List workflows enabled by tenant policy.", risk_level: "safe_read", tenant_aware: true, approval_behavior: "auto", input_schema: "{tenant_id}", output_schema: "{workflows[]}" },
    ]
}
