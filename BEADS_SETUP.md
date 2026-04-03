# Beads Integration Setup Guide

**Document Purpose**: Complete setup instructions for integrating Beads task tracking into Hermes 3.0 system.

**Status**: Production-ready (v0.63+)  
**Last Updated**: March 31, 2026  
**Agent Focus**: Every agent task is now tracked and persistent.

---

## Table of Contents

1. [Quick Start (5 minutes)](#quick-start)
2. [Installation (by Platform)](#installation)
3. [Integration Verification](#verification)
4. [Using Beads in Your Agent Work](#agent-workflow)
5. [Advanced Configuration](#configuration)
6. [Troubleshooting](#troubleshooting)

---

## Quick Start

### Step 1: Install Beads CLI (Global, One-Time)

```bash
# macOS / Linux / FreeBSD / Windows (with bash)
curl -fsSL https://raw.githubusercontent.com/gastownhall/beads/main/scripts/install.sh | bash

# Homebrew (macOS/Linux)
brew install beads

# npm (All platforms)
npm install -g @beads/bd

# Verify installation
which bd
bd version
```

**Expected Output:**
```
bd version 0.63.3
```

### Step 2: Initialize Beads in Hermes Workspace

```bash
cd e:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT\pauli-hermes-agent

# Initialize with stealth mode (no git hooks)
bd init --stealth

# Verify initialization
bd ready
```

**Expected Output:**
```bash
$ bd ready
No tasks yet. Use `bd create` to get started.
```

### Step 3: Create Your First Epic

```bash
# Create the Hermes 3.0 epic
bd create "Hermes 3.0 Production System" --epic -p 0 --id bd-hermes \
  --assignee "agent@hermes.local"

# Create first phase
bd create "Phase 1: Brain Sovereignty" -p 1 --parent bd-hermes

# Create a feature task under Phase 1
bd create "Implement /graph command" -p 2 --parent bd-hermes.1

# View your work
bd ready
```

### Step 4: Verify Python Integration

```bash
# From the Hermes workspace directory
python3 -c "from agent.beads_integration import BeadsTracker; t = BeadsTracker(); print(f'✅ Tracker initialized: {len(t.list_all())} tasks')"
```

**Expected Output:**
```
✅ Tracker initialized: 3 tasks
```

---

## Installation

### macOS

```bash
# Install dependencies (one-time)
xcode-select --install
brew install icu4c

# Install Beads
brew install beads

# Verify
bd version
```

### Ubuntu / Debian

```bash
# Build dependencies
sudo apt update
sudo apt install -y build-essential

# Install via Homebrew (Linux)
brew install beads

# Or from source
go install github.com/gastownhall/beads/cmd/bd@latest
```

### Windows (PowerShell)

```powershell
# Install via npm (easiest, requires Node.js)
npm install -g @beads/bd

# Or Via Go (requires Go 1.21+)
go install github.com/gastownhall/beads/cmd/bd@latest

# Verify
bd version
```

### Windows (MinGW / MSYS2)

```bash
# Inside MSYS2 shell
pacman -S mingw-w64-x86_64-gcc
go install github.com/steveyegge/beads/cmd/bd@latest
```

---

## Verification

After installation, run these checks:

```bash
# 1. Verify Beads CLI
bd version
# Expected: bd version 0.63.3 (or higher)

# 2. Verify workspace is initialized
cd e:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT\pauli-hermes-agent
ls -la .beads/
# Expected: embeddeddolt/ directory exists

# 3. Verify Python module
python3 -c "from agent.beads_integration import BeadsTracker; print('✅ Python integration OK')"

# 4. Test task creation
bd create "Test task" -p 2
bd ready
bd close <returned-task-id> "Test"

# 5. Verify API endpoints available (when gateway running)
curl http://localhost:8642/v1/beads/ready
# Expected: JSON array of ready tasks
```

---

## Agent Workflow

This section shows how agents (including you) should use Beads for **every task**.

### Workflow Loop

**User requests work** → **Scan ready tasks** → **Plan** → **Execute** → **Track progress** → **Complete**

```
┌──────────────────────────────────────────────────────────────┐
│ 1. USER REQUESTS WORK                                        │
│    "Implement Supabase integration"                          │
└──────────────────────────────────────────────────────────────┘
                            ↓
┌──────────────────────────────────────────────────────────────┐
│ 2. SCAN READY TASKS                                          │
│    $ bd ready                                                │
│    Returns: Tasks with no blockers                           │
└──────────────────────────────────────────────────────────────┘
                            ↓
┌──────────────────────────────────────────────────────────────┐
│ 3. CREATE TASK (if new request)                              │
│    $ bd create "Implement Supabase integration" \            │
│        -p 1 -t feature --parent bd-hermes.4                  │
│    Returns: bd-hermes.4.2                                    │
└──────────────────────────────────────────────────────────────┘
                            ↓
┌──────────────────────────────────────────────────────────────┐
│ 4. PLAN: Add design, acceptance criteria                     │
│    $ bd update bd-hermes.4.2 \                               │
│        --design "Use REST API + Supabase client" \           │
│        --acceptance "Schema created\nMigrations done\nAPI ok"│
└──────────────────────────────────────────────────────────────┘
                            ↓
┌──────────────────────────────────────────────────────────────┐
│ 5. CLAIM TASK                                                │
│    $ bd update bd-hermes.4.2 --claim                         │
│    (Atomically: sets assignee + in_progress status)          │
└──────────────────────────────────────────────────────────────┘
                            ↓
┌──────────────────────────────────────────────────────────────┐
│ 6. EXECUTE: Code, implement, test                            │
│    $ # ... implement the feature ...                         │
│    $ git commit -m "feat(api): Supabase integration"         │
└──────────────────────────────────────────────────────────────┘
                            ↓
┌──────────────────────────────────────────────────────────────┐
│ 7. TRACK PROGRESS: Update with notes                         │
│    $ bd update bd-hermes.4.2 \                               │
│        --notes "Schema created, now wiring endpoints"        │
│    $ # ... more work ...                                     │
│    $ bd update bd-hermes.4.2 \                               │
│        --notes "API endpoints done, running tests"           │
└──────────────────────────────────────────────────────────────┘
                            ↓
┌──────────────────────────────────────────────────────────────┐
│ 8. COMPLETE: Close task with summary                         │
│    $ bd update bd-hermes.4.2 --close \                       │
│        "Supabase integration complete\n✓ All tests passing\n │
│         ✓ Dashboard connected\n✓ Beads persistence active"   │
└──────────────────────────────────────────────────────────────┘
                            ↓
┌──────────────────────────────────────────────────────────────┐
│ 9. ITERATE: Check next ready tasks                           │
│    $ bd ready                                                │
│    (Repeat workflow)                                         │
└──────────────────────────────────────────────────────────────┘
```

### Example: Full Session

```bash
# 1. See what's ready
$ bd ready
Ready tasks (3):
  bd-hermes.4.2 - Wire Supabase to API layer (priority: high)
  bd-hermes.4.3 - Add health check endpoint (priority: medium)

# 2. Claim first task
$ bd update bd-hermes.4.2 --claim
Updated bd-hermes.4.2 → in_progress

# 3. Implement feature
$ # ... code implementation ...
$ git commit -m "feat(api): Wire Supabase persistence (bd-hermes.4.2)"

# 4. Track progress
$ bd update bd-hermes.4.2 --notes "Schema created, migrations ready"
$ # ... more implementation ...
$ bd update bd-hermes.4.2 --notes "Endpoints connected to Supabase, running tests"

# 5. Close when complete
$ bd update bd-hermes.4.2 --close "Supabase fully integrated, all endpoints tested"

# 6. Next task
$ bd ready
Ready tasks (2):
  bd-hermes.4.3 - Add health check endpoint (priority: medium)

$ bd update bd-hermes.4.3 --claim
Updated bd-hermes.4.3 → in_progress
```

---

## Configuration

### Enable Server Mode for Multi-Agent Workflows

**Default (Embedded)**: Single writer, file-locked. Perfect for local development.  
**Server Mode**: Multi-writer, ideal for VPS with concurrent agents.

```bash
# Enable Dolt server mode
bd init --server --server-host 127.0.0.1 --server-port 3307

# Start Dolt server
dolt sql-server &

# Verify connection
bd ready
```

**Dolt Server Environment Variables:**
```bash
export BEADS_DOLT_SERVER_HOST=127.0.0.1
export BEADS_DOLT_SERVER_PORT=3307
export BEADS_DOLT_SERVER_USER=root
export BEADS_DOLT_PASSWORD=yourpassword
```

### Backup Configuration

```bash
# Initialize backup
bd backup init /path/to/backup

# Regular sync
bd backup sync

# Check status
bd backup status

# Restore from backup (destructive)
bd backup restore --force /path/to/backup
```

### Integration with `.env`

Add to `master.env.organized`:

```bash
# Beads Configuration
BEADS_DIR=.beads
BEADS_DOLT_SERVER_HOST=127.0.0.1
BEADS_DOLT_SERVER_PORT=3307
BEADS_AGENT_ID=agent@hermes.local
BEADS_API_ENABLED=true
BEADS_DASHBOARD_ENABLED=true
```

### Dashboard Widget Configuration

In `web-ui/index.html`, the Beads widget:

```javascript
// Features:
// - Fetch /v1/beads/ready for all unblocked tasks
// - Fetch /v1/beads/stats for summary metrics
// - Display task list with status, priority, assignee
// - Quick claim button: POST /v1/beads/:id/claim
// - Real-time updates via dashboard refresh

// Configuration:
const beadsConfig = {
  apiUrl: "http://localhost:8642",  // Hermes API
  refreshInterval: 30000,             // 30s
  maxTasks: 20,
  showReady: true,
  showInProgress: true,
};
```

---

## Troubleshooting

### Beads Not Installing

```bash
# Check Go installation
go version
# Expected: go version go1.21+ (Windows/Homebrew auto-handles this)

# Check npm alternative
npm install -g @beads/bd
which bd
```

### Tasks Not Appearing in `bd ready`

```bash
# Check for blockers
bd list --all --json | grep -i blocker

# View blockers on specific task
bd show <task_id> | grep -i blocker

# Remove blocker
bd dep remove <task_id> <blocker_id>

# Verify task status
bd show <task_id> | grep -i status
```

### Database Locked (Embedded Mode)

```bash
# Check if another process is using .beads
lsof .beads/embeddeddolt/dolt.lock
# Expected: Nothing (if locked, kill the process)

# Clean and reinit
rm -rf .beads
bd init --stealth
```

### Python Module Not Found

```bash
# Verify path
python3 -c "import sys; print(sys.path)"

# Install in development mode
pip install -e .

# Or add to PYTHONPATH
export PYTHONPATH=/path/to/hermes:$PYTHONPATH
python3 -c "from agent.beads_integration import BeadsTracker"
```

### API Endpoint Not Responding

```bash
# Check gateway is running
curl http://localhost:8642/health
# Expected: {"status": "ok"}

# Check Beads API is registered
curl http://localhost:8642/v1/beads/ready
# Expected: JSON array

# If 404, verify gateway imports beads_api
# In gateway/run.py or main API file:
# from gateway.beads_api import router
# app.include_router(router)
```

### Can't Claim Task

```bash
# Check task status
bd show <task_id> | grep -i status

# If already in_progress, you must be assignee
bd show <task_id> | grep -i assignee

# Look for blockers preventing claim
bd show <task_id> | grep -i blocker

# If blocked, resolve blockers first
bd dep remove <task_id> <blocker_id>
```

---

## What's Next

After Beads is set up:

1. **Create Hermes 3.0 Epic**: Organize all work under a single epic
2. **Migrate Existing Work**: Move any informal plans to tasks
3. **Enable API**: Update gateway to include `/v1/beads/*` endpoints
4. **Dashboard Integration**: Add Beads panel to web-ui
5. **Agent Coordination**: Use `/bd ready` to synchronize multi-agent work
6. **Metrics & Reporting**: Export task data for sprint reviews

---

## Reference Links

- **Beads Repository**: https://github.com/gastownhall/beads
- **Agent Instructions**: See `AGENT_INSTRUCTIONS.md`
- **Quickstart**: https://github.com/gastownhall/beads#-quick-start
- **Documentation**: https://github.com/gastownhall/beads/tree/main/docs
- **FAQ**: https://github.com/gastownhall/beads/blob/main/docs/FAQ.md

---

**End of Setup Guide**
