use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContract {
    pub name: &'static str,
    pub description: &'static str,
}

pub fn tool_contracts() -> Vec<ToolContract> {
    vec![
        ToolContract { name: "catalog_repos", description: "Catalog repositories and compute risk/deployability." },
        ToolContract { name: "generate_prd_batch", description: "Generate a PRD batch for selected repos." },
        ToolContract { name: "list_prd_batch", description: "List PRD batch status and items." },
        ToolContract { name: "approve_prd", description: "Approve a PRD item for execution." },
        ToolContract { name: "create_coding_session", description: "Start coding session workflow." },
        ToolContract { name: "create_browser_run", description: "Start browser workflow run." },
        ToolContract { name: "pause_browser_run", description: "Pause browser run for intervention." },
        ToolContract { name: "resume_browser_run", description: "Resume browser run after intervention." },
        ToolContract { name: "cancel_browser_run", description: "Cancel browser run." },
        ToolContract { name: "provision_appwrite_project", description: "Provision and bind Appwrite project." },
        ToolContract { name: "list_appwrite_projects", description: "List Appwrite project bindings." },
        ToolContract { name: "create_subagent", description: "Create disposable scoped sub-agent." },
        ToolContract { name: "list_subagents", description: "List sub-agent statuses." },
        ToolContract { name: "stop_subagent", description: "Stop sub-agent immediately." },
        ToolContract { name: "get_run_status", description: "Get run state and summary." },
        ToolContract { name: "list_missing_secrets", description: "List missing setup secrets." },
        ToolContract { name: "get_kpi_snapshot", description: "Return revenue/bottleneck/approvals snapshot." },
        ToolContract { name: "generate_proposal", description: "Generate proposal draft for client." },
        ToolContract { name: "donor_followup_generate", description: "Draft donor follow-up content." },
        ToolContract { name: "grant_research_intake", description: "Collect grant research intake." },
    ]
}
