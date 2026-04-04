# Agent MAXX — Operator Skill

## What This Is

Agent MAXX is an autonomous overnight coding platform. It manages repos, generates PRDs, runs coding sessions, and automates browser tasks via a REST API + MCP interface.

## MCP Connection

```json
{
  "mcpServers": {
    "agent-maxx": {
      "url": "http://localhost:8642/mcp"
    }
  }
}
```

Once connected, you can call all API endpoints as tools directly from your IDE.

## Key Workflows

### 1. Start a Coding Session
```bash
# Create a coding session for a repo
POST /api/coding-sessions
{
  "repo_url": "https://github.com/org/repo",
  "prompt": "Add dark mode toggle to the settings page",
  "task_type": "feature",
  "provider_profile": "balanced"
}
# Returns: { "id": "uuid", "status": "queued" }
# Worker picks it up: clone → analyze → code → test → commit
```

### 2. Monitor Run Status
```bash
GET /api/runs?status=running
GET /api/runs/{id}/steps      # Step-by-step log
GET /api/dashboard/overview   # All active counts
```

### 3. Handle Approvals
Agent MAXX pauses at high-risk commands and waits for human approval:
```bash
GET /api/approvals?status=pending
POST /api/approvals/{id}/decide
{ "decision": "approved", "decided_by": "operator" }
```

### 4. Browser Runs
```bash
POST /api/browser-runs
{
  "start_url": "https://target.com",
  "steps": [
    {"action": "navigate", "url": "https://target.com"},
    {"action": "screenshot", "label": "homepage"},
    {"action": "vision_analyze", "prompt": "Extract all form fields and fill them with test data"}
  ]
}
```

### 5. Vision / Online Test Research
```bash
POST /api/browser-runs
{
  "start_url": "https://quiz-platform.com/test/123",
  "steps": [
    {"action": "vision_analyze", "prompt": "Read every question. For each, provide the correct answer with brief reasoning.", "save_as": "test_answers"}
  ],
  "vision_mode": true
}
# Artifacts saved to /data/artifacts/{run_id}/
```

### 6. Repo Scanning
```bash
POST /api/repos/scan-trigger   # Trigger manual scan
GET /api/repos                 # List all repos + profiles
```

### 7. PRD Generation
```bash
POST /api/prd-batches
{ "strategy": "auto" }
# Agent MAXX selects repos, generates PRDs, queues coding sessions
```

## Provider Profiles

| Profile | Model | Use When |
|---------|-------|---------|
| `balanced` | claude-sonnet-4 | Normal coding tasks |
| `premium_reasoning` | claude-opus-4 | Architecture, complex debugging |
| `low_cost_coding` | claude-sonnet-4 | High-volume bulk tasks |
| `research_browser` | gemini-2.5-flash | Vision, screenshots, web research |
| `openai_gpt4` | gpt-4o | When GPT-4 is preferred |

## Approval Risk Levels

| Level | Means | Default Action |
|-------|-------|---------------|
| `none` | Read-only ops | Auto-approved |
| `low` | Writes to feature branch | Auto-approved |
| `medium` | Merge to main | Ask operator |
| `high` | Deploy / infra changes | Requires approval |
| `critical` | Delete / destructive | Requires approval + confirmation |

## Dashboard Panels

Open http://localhost:3000 to see:
1. Overview stats (repos, runs, approvals, sessions)
2. Setup / missing secrets
3. Repositories
4. Active runs
5. Pending approvals (approve/reject inline)
6. Coding sessions
7. Browser runs
8. PRD batches
9. Provider profiles
10. Sub-agents
11. Appwrite projects
12. Live activity log

## Tips

- All endpoints auto-refresh every 15s in the dashboard
- Telegram alerts fire for completions/failures/approvals (if configured)
- Worker-browser records video + traces for every session (in /data/artifacts)
- MCP at /mcp auto-exposes all routes — no extra config needed
