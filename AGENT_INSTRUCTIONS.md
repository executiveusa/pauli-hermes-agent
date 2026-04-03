# Agent Instructions for Hermes 3.0
## Using Beads for Persistent Agent Tracking

This document defines the agent workflow for working with the Hermes Agent codebase using **Beads** as the persistent memory system for all task tracking and agent operations.

---

## 🚀 Quick Start: Beads + Hermes

### Installation (One-Time)
```bash
# Install Beads globally (macOS/Linux/Windows)
curl -fsSL https://raw.githubusercontent.com/gastownhall/beads/main/scripts/install.sh | bash

# Or via Homebrew
brew install beads

# Or via npm
npm install -g @beads/bd

# Initialize Beads in the Hermes project (from repo root)
bd init --stealth

# Verify installation
bd ready
```

### Configuration
Beads uses embedded Dolt by default (`.beads/embeddeddolt/`). For multi-agent workflows, enable server mode:

```bash
# Optional: Enable external Dolt server for concurrent writes
bd init --server --server-host 127.0.0.1 --server-port 3307
```

---

## 📋 Agent Workflow

### Phase 1: Understand the Task
1. **User requests work** → Agent reads request and context
2. **Scan for blockers** → `bd ready` lists only unblocked tasks
3. **Create task** → `bd create "<Task>" -p <priority>` (0=critical, 1=high, 2=medium, 3=low)
   - Example: `bd create "Implement Beads integration" -p 0`

### Phase 2: Plan & Organize
1. **Create epic** → `bd create "Hermes 3.0 Production Build" --epic` 
2. **Add subtasks** → Use hierarchical IDs: `bd-a3f8.1`, `bd-a3f8.1.1`
3. **Link dependencies** → `bd dep add <child> <parent>`
   - Example: `bd dep add bd-a1b2 bd-a3f8` (task bd-a1b2 blocked by epic bd-a3f8)

### Phase 3: Claim & Execute
1. **Claim task** → `bd update <id> --claim` (atomically sets assignee + in_progress)
   - Example: `bd update bd-a1b2 --claim`
2. **Execute work** → Implement the feature/fix
3. **Log progress** → `bd update <id> --notes "What was accomplished"`
4. **Commit code** → `git commit -m "feat: <description>"`

### Phase 4: Close & Validate
1. **Mark complete** → `bd update <id> --close "Fixed"` or `--close "Done"`
2. **Link related** → `bd dep add <completed_task> <related_task> --relates-to`
3. **Run final tests** → Verify all checks pass

---

## 🔗 Task Hierarchy & Epic Structure

Hermes 3.0 uses the following epic organization:

```
bd-hermes     (Epic: Hermes 3.0 Production System)
├── bd-hermes.1      (Phase 1: Brain Sovereignty)
├── bd-hermes.2      (Phase 2: Archon X Cockpit)
├── bd-hermes.3      (Phase 3: Token Efficiency & Dual-Agent)
└── bd-hermes.4      (Phase 4: Production Build & Deployment)
```

### Creating Hierarchical Tasks

```bash
# Create epic
bd create "Hermes 3.0 System" --epic -p 0 --id bd-hermes

# Create phase tasks (auto-hierarchical)
bd create "Brain Sovereignty - E: drive protection" -p 1 --parent bd-hermes
# Returns: bd-hermes.1

# Create subtasks
bd create "Implement /graph command" -p 2 --parent bd-hermes.1
# Returns: bd-hermes.1.1
```

---

## 🎯 Required Fields for Agent Tasks

Every task should include:

| Field | Format | Example |
|-------|--------|---------|
| **Title** | 1-liner | "Implement Beads integration in API layer" |
| **Priority** | 0-3 | `-p 1` (high) |
| **Type** | bug, task, feature, chore, epic | `-t feature` |
| **Assignee** | `--assignee <email>` | `--assignee agent@hermes.local` |
| **Description** | Long-form context | `bd update <id> --description "Full description"` |
| **Design Notes** | Architecture/approach | `--design "Use REST API + Supabase"` |
| **Acceptance Criteria** | List of requirements | `--acceptance "All 9 tests pass\nCLI works\nDashboard shows beads"` |

### Example: Create a Feature with Full Context
```bash
bd create "Add Beads dashboard widget" \
  -p 1 \
  -t feature \
  --assignee agent@hermes.local \
  --design "Fetch /v1/beads endpoint, render timeline of agent actions" \
  --acceptance "Widget displays last 20 actions\nFilters by date/agent\nRealtime updates via WebSocket"
```

---

## 🔄 Status Transitions

```
not_started → in_progress → ready_for_review → closed
           ↑________________↑___________________↑
                   (can reopen)
```

**Status Commands:**
```bash
bd update <id> --claim              # → in_progress (atomically claim)
bd update <id> --status ready       # → ready_for_review
bd update <id> --close "Fixed"      # → closed
bd update <id> --reopen             # → in_progress (if was closed)
```

---

## 📊 Agent Commands

### Discover Ready Tasks
```bash
# List all tasks with no open blockers
bd ready

# List ready tasks as JSON (for programmatic access)
bd ready --json

# Filter by assignee
bd ready --assignee agent@hermes.local

# Filter by label/tag
bd ready --label hermes-3.0 --label api-critical
```

### View Task Details
```bash
# Show full task with audit trail
bd show bd-a1b2

# Show all tasks in epic
bd show bd-hermes --tree

# Find tasks by title
bd list --query "Beads integration"
```

### Update & Progress
```bash
# Add notes (incremental, doesn't replace)
bd update bd-a1b2 --notes "Completed schema migration, now wiring API endpoints"

# Change priority
bd update bd-a1b2 -p 0

# Add labels
bd update bd-a1b2 --label "api" --label "database"

# Relate to other task
bd dep add bd-a1b2 bd-a3f8 --relates-to
```

### Time Tracking
```bash
# Log work duration (optional, for metrics)
bd update bd-a1b2 --time-spent "2h 30m"

# Set due date
bd update bd-a1b2 --due "2026-04-15"
```

---

## 💾 Database Operations

### Backup
```bash
# Initialize backup destination
bd backup init /path/to/backup

# Sync to backup
bd backup sync

# Verify backup
bd backup status
```

### Restore (if needed)
```bash
# Restore from backup
bd backup restore --force /path/to/backup/
```

### Export for Reporting
```bash
# Export all tasks as JSON
bd list --all --json > tasks.json

# Export as CSV
bd list --all --csv > tasks.csv

# Export Dolt branch
dolt log -p > audit_trail.patch
```

---

## 🔗 Integration Points

### Hermes CLI
```bash
# Show ready tasks in Hermes CLI
hermes /bd ready

# Create task from Hermes CLI
hermes /bd create "Task title"

# Quick claim from Hermes
hermes /bd claim bd-a1b2
```

### API Endpoints
Hermes API exposes Beads data:

```
GET /v1/beads?limit=20&agent=hermes&status=closed
GET /v1/beads/ready
POST /v1/beads/create
POST /v1/beads/:id/update
GET /v1/beads/:id
POST /v1/beads/:id/close
```

### Dashboard Widget
The Hermes dashboard includes a **Beads Panel** showing:
- Ready tasks (no blockers)
- In-progress tasks
- Recent completions
- Task dependencies (graph)
- Audit trail (who did what, when)

---

## 🚨 Important Rules

### DO
✅ **Claim before working** — `bd update <id> --claim` atomically sets assignee + status  
✅ **Add dependencies** — Link blockers with `bd dep add`  
✅ **Document design** — Use `--design` for architecture decisions  
✅ **Write acceptance criteria** — Clear definition of done  
✅ **Update status regularly** — Keep beads in sync with reality  
✅ **Use hierarchical IDs** — Organize related tasks under epics  
✅ **Close with notes** — Explain what was fixed/implemented  

### DON'T
❌ **Don't create tasks without context** — Include description, design, acceptance  
❌ **Don't forget dependencies** — Link blocking tasks  
❌ **Don't leave tasks orphaned** — Close or reopen, don't abandon  
❌ **Don't ignore `bd ready` output** — That's your priority list  
❌ **Don't use arbitrary IDs** — Use hierarchical structure  

---

## 📝 Example Agent Session

```bash
# Agent starts: Check what's ready
$ bd ready --json
[
  {"id": "bd-hermes.4.2", "title": "Wire Supabase to API layer", "priority": 1},
  {"id": "bd-hermes.4.3", "title": "Add health check endpoint", "priority": 2}
]

# Agent claims first task
$ bd update bd-hermes.4.2 --claim
Updated bd-hermes.4.2 → in_progress (assigned to agent@hermes.local)

# Agent works on task
$ bd update bd-hermes.4.2 --notes "Schema created, now adding migrations"
$ # ... implement code ...
$ git commit -m "feat(api): Add Supabase persistence layer (bd-hermes.4.2)"

# Agent completes task
$ bd update bd-hermes.4.2 --close "Supabase fully integrated\nAll endpoints updated\n9/9 tests passing"
Closed bd-hermes.4.2 → closed (merged design patterns documented)

# Agent checks next ready tasks
$ bd ready
[
  {"id": "bd-hermes.4.3", "title": "Add health check endpoint", "priority": 2},
  {"id": "bd-hermes.4.4", "title": "Deploy to Vercel", "priority": 0}
]
```

---

## 🔧 Troubleshooting

### Beads won't initialize
```bash
# Check if Dolt is installed
dolt version

# Try direct initialization
cd .beads && dolt init && cd ..
```

### Tasks not appearing in `bd ready`
```bash
# Check for blockers
bd show <id> | grep depends_on

# Remove blocker
bd dep remove <id> <blocker_id>
```

### Concurrent write errors (Server Mode)
```bash
# Restart Dolt server
dolt sql-server stop
dolt sql-server < /dev/null &
```

### Restore from corruption
```bash
# Export current state
bd list --all --json > backup.json

# Reset database
rm -rf .beads
bd init

# Re-import
# (Beads doesn't have auto-import yet; manually recreate critical tasks)
```

---

## 📚 Documentation

- **Beads Quickstart**: https://github.com/gastownhall/beads#-quick-start
- **Beads Agent Instructions**: https://github.com/gastownhall/beads/blob/main/AGENT_INSTRUCTIONS.md
- **Copilot Setup**: https://github.com/gastownhall/beads/blob/main/docs/COPILOT_INTEGRATION.md
- **FAQ**: https://github.com/gastownhall/beads/blob/main/docs/FAQ.md

---

## 🎯 Agent Success Metrics

The Hermes agent system is optimized by these metrics:

| Metric | Target | How to Read |
|--------|--------|-----------|
| **Ready Tasks** | K ready → 0 | `bd ready \| wc -l` |
| **Task Closure Rate** | 95%+ | `bd list --status closed \| wc -l` |
| **Avg Time-to-Close** | <4h | `bd list --all \| grep time_spent` |
| **Dependency Depth** | <5 levels | `bd show <epic> --tree` |
| **Blocker Resolution** | 100% | `bd list --status in_progress` should have 0 blockers |

---

**Last Updated**: March 31, 2026  
**Hermes Version**: 3.0  
**Beads Integration**: v0.63+
