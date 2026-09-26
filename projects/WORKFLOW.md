# 🚀 Interpretable Context Methodology (ICM) Workflow

## Complete End-to-End Automated Project Delivery System

This system implements **Interpretable Context Methodology** for orchestrating AI agent workflows using **folder structure as the coordination mechanism**.

### Philosophy

Instead of complex multi-agent frameworks, this uses:
- 📁 **Numbered folders** for workflow stages
- 📝 **Markdown files** with prompts and context
- 🔄 **One agent** reading the right files at the right time
- 🖥️ **Local scripts** for mechanical work
- 👤 **Human review** at key decision points

---

## 📋 The 10-Stage Workflow

```
User Points at Repo
        ↓
┌─────────────────────────────────────────────────────┐
│ 00-intake: Capture project details                  │
└─────────────────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────────────────┐
│ 01-scan: Analyze repository structure & tech stack  │
│ → repo-analysis.json, SCAN_SUMMARY.md               │
└─────────────────────────────────────────────────────┘
        ↓ [HUMAN REVIEW]
┌─────────────────────────────────────────────────────┐
│ 02-prd: Generate Product Requirements Document      │
│ → PROJECT_PRD.md, REQUIREMENTS.json                 │
└─────────────────────────────────────────────────────┘
        ↓ [HUMAN REVIEW]
┌─────────────────────────────────────────────────────┐
│ 03-design: Validate against design system           │
│ → DESIGN_SPEC.md, IMPLEMENTATION_GUIDE.md           │
└─────────────────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────────────────┐
│ 04-plan: Create detailed implementation plan        │
│ → IMPLEMENTATION_PLAN.md, TASK_CHECKLIST.json       │
└─────────────────────────────────────────────────────┘
        ↓ [HUMAN DECISION: Ready to build?]
┌─────────────────────────────────────────────────────┐
│ 05-develop: SANDCASTLE - Build in isolated sandbox  │
│ → Creates feature branch, writes code, runs tests   │
└─────────────────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────────────────┐
│ 06-test: Verify all tests pass & quality metrics    │
│ → TEST_REPORT.md, METRICS.json                      │
└─────────────────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────────────────┐
│ 07-deploy: Deploy to Vercel production              │
│ → DEPLOYMENT_REPORT.md, DEPLOYMENT.json             │
└─────────────────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────────────────┐
│ 08-verify: Verify deployment & run health checks    │
│ → VERIFICATION_REPORT.md, ERROR_ANALYSIS.json       │
└─────────────────────────────────────────────────────┘
        ↓ [All checks pass?]
┌─────────────────────────────────────────────────────┐
│ 09-production: Merge to main, create release        │
│ → MERGE_REPORT.md, RELEASE.json, RELEASE_NOTES.md  │
└─────────────────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────────────────┐
│ 10-report: Generate final report & notify user      │
│ → PROJECT_COMPLETION_SUMMARY.md, DASHBOARD.html     │
│ → Sends Vercel link to user                         │
└─────────────────────────────────────────────────────┘
        ↓
      ✅ DONE - Live in production!
```

---

## Quick Start

### 1. Set Up Sandcastle

```bash
# Copy Sandcastle config
cp .env.sandcastle.example .env

# Enable Docker (or Vercel)
# Edit .env:
SANDCASTLE_ENABLED=true
DOCKER_SANDBOX_ENABLED=true

# Ensure Docker is running
docker daemon  # in another terminal
```

### 2. Point to a Repository

```bash
# Create input for Stage 00
cat > projects/00-intake/input.json << 'EOF'
{
  "repo_url": "https://github.com/owner/repo",
  "repo_name": "repo",
  "project_name": "My Project",
  "user_request": "Add dark mode toggle to the UI",
  "submitted_at": "2026-07-04T19:00:00Z"
}
EOF
```

### 3. Run the Workflow

```bash
# Start Stage 01: Scan
hermes "Analyze this repository: https://github.com/owner/repo"

# This:
# 1. Reads 00-intake/input.json
# 2. Clones and analyzes the repo
# 3. Generates 01-scan/repo-analysis.json
# 4. Waits for human review
```

### 4. Review and Approve

```bash
# After Stage 01 completes, review:
cat projects/01-scan/SCAN_SUMMARY.md

# Then proceed to Stage 02
hermes "Generate PRD from scan results"
```

### 5. Automate Further Stages

```bash
# After PRD approval, create implementation plan
hermes "Create implementation plan for this project"

# Then launch Sandcastle to build
hermes "sandbox the development work for this project"
  # This runs Stage 05-06 inside isolated sandbox
  # Creates feature branch
  # Runs tests
  # Awaits approval to merge

# After sandbox succeeds, deploy
hermes "deploy to Vercel and verify"
  # Runs Stages 07-08
  # Deploys to Vercel
  # Runs health checks

# Finally, merge to production
hermes "merge to production and create release"
  # Runs Stages 09-10
  # Merges to main
  # Generates final report
  # Sends user the Vercel link
```

---

## File Structure

```
projects/
├── 00-intake/              # Input: repo URL
│   ├── README.md           # What to put here
│   └── input.json          # User provides this
│
├── 01-scan/               # Output: repo analysis
│   ├── AGENT_PROMPT.md    # Agent reads this
│   ├── repo-analysis.json # Generated
│   └── SCAN_SUMMARY.md    # Generated (human reviews)
│
├── 02-prd/                # Output: requirements
│   ├── AGENT_PROMPT.md
│   ├── PROJECT_PRD.md     # Generated (human approves)
│   └── REQUIREMENTS.json
│
├── 03-design/             # Output: design spec
│   ├── AGENT_PROMPT.md
│   ├── DESIGN_SPEC.md     # Generated
│   └── IMPLEMENTATION_GUIDE.md
│
├── 04-plan/               # Output: implementation plan
│   ├── AGENT_PROMPT.md
│   ├── IMPLEMENTATION_PLAN.md # Generated (detailed steps)
│   └── TASK_CHECKLIST.json
│
├── 05-develop/            # Sandcastle development
│   ├── SANDCASTLE_BRIEF.md    # Developer reads this
│   └── (creates feature branch, code, commits)
│
├── 06-test/               # Test verification
│   ├── VERIFICATION_AGENT.md
│   └── TEST_REPORT.md     # Generated
│
├── 07-deploy/             # Vercel deployment
│   ├── DEPLOYMENT_AGENT.md
│   └── DEPLOYMENT_REPORT.md # Generated
│
├── 08-verify/             # Post-deployment checks
│   ├── VERIFICATION_AGENT.md
│   └── VERIFICATION_REPORT.md # Generated
│
├── 09-production/         # Merge to main
│   ├── MERGE_AGENT.md
│   ├── MERGE_REPORT.md    # Generated
│   └── RELEASE_NOTES.md
│
└── 10-report/             # Final report
    ├── FINAL_REPORT_AGENT.md
    ├── PROJECT_COMPLETION_SUMMARY.md
    ├── DASHBOARD.html     # User-friendly dashboard
    └── USER_NOTIFICATION.txt # Email to send user
```

---

## How It Works

### Stage-by-Stage

Each stage follows this pattern:

1. **Agent reads the prompt** (AGENT_PROMPT.md or SANDCASTLE_BRIEF.md)
2. **Agent performs work** (analysis, coding, testing, deployment)
3. **Agent generates outputs** (JSON, markdown, artifacts)
4. **Human reviews** (key decision points)
5. **Move to next stage** (or iterate if issues)

### Example: Stage 01 (Scan)

```
Agent reads: 01-scan/AGENT_PROMPT.md
  ↓
Agent sees: "Clone repo, analyze structure, generate repo-analysis.json"
  ↓
Agent clones repo → explores structure → creates analysis
  ↓
Agent saves: 01-scan/repo-analysis.json
  ↓
Agent saves: 01-scan/SCAN_SUMMARY.md
  ↓
Human reviews SCAN_SUMMARY.md
  ↓
If OK → proceed to Stage 02
If issues → ask agent to re-scan
```

### Example: Stage 05 (Sandcastle Development)

```
Agent reads: 05-develop/SANDCASTLE_BRIEF.md
  ↓
Agent sees: "Clone repo, create feature branch, implement plan"
  ↓
Sandcastle creates isolated environment
  ↓
Agent creates branch: feature/dark-mode-toggle
  ↓
Agent implements each step from IMPLEMENTATION_PLAN.md
  ↓
Agent runs tests: ✅ All pass
  ↓
Agent commits changes
  ↓
Sandcastle awaits approval to merge
```

---

## Key Principles (from ICM Paper)

### 1. Filesystem is the Coordinator
- Numbered folders = workflow stages
- Agent reads files in order
- No framework code needed

### 2. Plain Text as Interface
- Markdown prompts are human-readable
- JSON outputs are machine-parseable
- Anyone can edit the workflow

### 3. Human Review at Key Points
- After analysis (Stage 01)
- After requirements (Stage 02)
- Before building (Stage 04)
- After testing (Stage 06)
- Before production merge (Stage 09)

### 4. Stage Isolation
- Each stage only sees its folder
- Previous stage output → next stage input
- Easy to inspect, modify, test

### 5. Reusability
- Copy entire folder to use workflow again
- Modify prompts for different projects
- Same process, different outcomes

---

## Customization

### Change the Workflow

1. **Add a stage**: Create `11-custom/` folder
2. **Write prompt**: Create `11-custom/AGENT_PROMPT.md`
3. **Define outputs**: Specify JSON/markdown files agent should create
4. **Insert in workflow**: Update this WORKFLOW.md

### Change a Stage

1. **Edit the prompt**: `XX-stage/AGENT_PROMPT.md`
2. **Define new outputs**: Update expected files
3. **Agent adapts**: Uses new instructions automatically

### Use Different Tools

1. **Instead of Vercel**: Update Stage 07-08 prompts
2. **Instead of Sandcastle**: Update Stage 05 process
3. **Different design system**: Update Stage 03

---

## Running the System

### Full Automated Run (Start to Finish)

```bash
# User provides repo
hermes "Run complete workflow for: https://github.com/owner/repo"

# System does all 10 stages automatically
# Pauses for human review at:
# - After scan (review findings)
# - After PRD (approve requirements)
# - Before building (confirm plan)
# - After merge (confirm release)

# Returns: Vercel link + full documentation
```

### Stage-by-Stage Manual Control

```bash
# Stage 01
hermes "scan repository at https://github.com/owner/repo"
# Wait for output, review 01-scan/SCAN_SUMMARY.md

# Stage 02 (when ready)
hermes "generate PRD from stage 01 scan"
# Wait, review 02-prd/PROJECT_PRD.md

# Stage 04 (when approved)
hermes "create implementation plan"
# Review 04-plan/IMPLEMENTATION_PLAN.md

# Stage 05 (when ready to build)
hermes "sandbox the development work"
# Sandcastle builds in isolation
# Wait for notification

# Approve merge in sandbox (when tests pass)

# Stage 07+ (automatic)
# Deploy → Verify → Merge → Report
```

---

## Error Recovery

The workflow is **idempotent** - if a stage fails:

1. **Fix the issue** (code, config, requirements, etc.)
2. **Re-run the stage** (agent starts fresh)
3. **No side effects** - previous stage outputs still valid
4. **No rollback needed** - Sandcastle isolated anyway

Example:
```
Stage 05 fails: Test didn't pass
  → Fix the code
  → Re-run Stage 05
  → Tests now pass
  → Continue to Stage 06 (no problems)
```

---

## Status Tracking

Monitor each stage:

```bash
# Check what's completed
ls projects/*/  # See which stages have outputs

# Review specific stage
cat projects/01-scan/SCAN_SUMMARY.md      # See scan results
cat projects/02-prd/PROJECT_PRD.md        # See requirements
cat projects/07-deploy/DEPLOYMENT.json    # See deployment details

# Monitor real-time
watch -n 5 'ls -la projects/*/*.json'  # See what's being generated
```

---

## Integration with Your System

### Connect to Hermes Memory
- Save project history in Hermes Rolodex
- Track all completed projects
- Reuse components from previous projects

### Connect to Vercel
- Automatic deployment after merge
- Link to Vercel project from DEPLOYMENT.json
- Monitor performance metrics

### Connect to GitHub
- Automatic PR creation
- Link to GitHub repo in DELIVERY_PACKAGE.json
- Merge uses GitHub CLI (gh)

### Connect to Error Tracking
- Errors automatically reported
- Linked to deployment
- Monitored in Stage 08

---

## Cost Analysis

**With this system using Sandcastle on your VPS:**

- Development: **$0** (your VPS)
- Testing: **$0** (local)
- Deployment: **$0** (Vercel free tier) or paid tier
- Analysis/Design/Planning: **$0** (all AI, no manual work)

**Time savings:**
- Manual project: 20-40 hours
- This system: 2-4 hours (mostly review time)
- **Automation savings: 18-36 hours per project**

---

## Next: Getting Started

1. **Set up Sandcastle** → See `../../../SECRETS_SETUP.md`
2. **Configure your VPS** → See docker setup below
3. **Provide a test repo** → Point at any GitHub project
4. **Run Stage 01** → See what the system can do

Ready to give it a try?

---

## Technical Stack

- **Orchestration**: Filesystem (ICM methodology)
- **Development**: Sandcastle + Docker
- **Deployment**: Vercel
- **Testing**: Jest/Vitest (project-dependent)
- **Merging**: GitHub CLI
- **Notifications**: Plain text + HTML dashboard

All coordinated through **folder structure and markdown files** as described in the ICM paper.
