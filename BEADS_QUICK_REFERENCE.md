# Beads Quick Reference Card

**Print this and keep it handy while working on Hermes.**

## One-Line Commands

```bash
# Essential workflow
bd ready                                    # See ready tasks
bd create "Task title" -p 1 -t feature     # Create task
bd update bd-x --claim                     # Claim task
bd update bd-x --notes "Did something"     # Add progress
bd update bd-x --close "Done"              # Close task
bd show bd-x                                # View task details

# Searching & filtering
bd list --all --json                       # All tasks (JSON)
bd list --status in_progress               # Only in-progress
bd list --type bug --priority 0            # Bugs + critical only
bd ready --json | jq .[0].id              # First ready task ID

# Dependency management
bd dep add bd-child bd-parent              # Mark as blocker
bd dep remove bd-child bd-parent           # Remove blocker
bd show bd-x | grep blocker                # See blockers

# Management
bd backup init /path                       # Setup backup
bd backup sync                             # Backup now
bd backup status                           # Check backup
```

## Status Reference

| Status | Meaning | When to Use |
|--------|---------|-----------|
| `not_started` | New task, no work yet | When you create a task |
| `in_progress` | Someone is working | After `bd update --claim` |
| `ready_for_review` | Implementation done, needs review | Before closing |
| `closed` | Task complete | When done (use `--close`) |

## Priority Reference

| Priority | Name | Use For |
|----------|------|---------|
| `0` | Critical / P0 | Blocking other work, urgent |
| `1` | High / P1 | Important, needs attention |
| `2` | Medium / P2 | Normal work (default) |
| `3` | Low / P3 | Nice-to-have, can defer |

## Task Template

When creating a complex task:

```bash
bd create "Implement feature X" \
  -p 1 -t feature \
  --assignee agent@hermes.local

# Then add design + acceptance:
bd update bd-abc --design "Use REST API + database Y"
bd update bd-abc --acceptance "Endpoint works\nTests pass\nDocs updated"
```

## Common Workflows

### Starting Your Day
```bash
bd ready | head -5        # See top 5 ready tasks
bd claim <first-id>       # Start with most urgent
# ... implement ...
bd close <id> "Finished"  # When done
```

### Sprint Kickoff
```bash
# Create epic for sprint
bd create "Sprint 15" --epic -p 1

# Move tasks under epic
bd update bd-x --parent bd-sprint15
bd update bd-y --parent bd-sprint15
```

### Multi-Agent Coordination
```bash
# Agent A: What's ready?
bd ready | grep "api"     # Filter by label/title

# Agent B: I'll take this
bd claim bd-api-123

# Agent A: What's Agent B doing?
bd list --assignee agent-b@company.com --status in_progress
```

### Before Code Review
```bash
bd update bd-x --status ready_for_review
git commit -m "feat: xyz (bd-x)"
# Create PR with link to "Closes bd-x"
```

## Environment Setup

```bash
# Add to ~/.bashrc or ~/.zshrc
export BEADS_EDITOR=nano  # your favorite editor

# Add to ~/your-project/.env
BEADS_DIR=.beads
BEADS_AGENT_ID=agent@hermes.local
```

## Keyboard Shortcuts (if using terminal UI)

```
?               Show help
/               Search tasks
n               New task
c               Close task
r               Refresh
q               Quit
```

## Tips & Tricks

```bash
# Get task count by status
bd list --all --json | jq 'group_by(.status) | map({status: .[0].status, count: length})'

# Find oldest task never touched
bd list --all --json | jq 'sort_by(.created_at) | .[0]'

# Export for reporting
bd list --all --json | jq '.[] | [.id, .title, .status, .priority] | @csv' > report.csv

# Find your tasks
bd ready --assignee agent@hermes.local

# Create a sub-epic
bd create "Sub-epic" -p 2 --parent bd-parent-epic

# Bulk close old tasks (DANGEROUS - review first!)
# Don't do this unless you're sure!
```

## Common Mistakes & Fixes

| Mistake | How to Fix |
|---------|-----------|
| "Can't claim task" | Check if it has blockers: `bd show <id>` |
| "Task looks stuck" | Add note: `bd update <id> --notes "Still working"` |
| "Wrong priority" | Update it: `bd update <id> -p 1` |
| "Need to unblock others" | Close your task: `bd close <id> "Done"` |
| "Lost track of task" | Show it: `bd show <id>` |

## File Locations

```
.beads/                       Database (don't touch)
.beads/embeddeddolt/          Embedded Dolt (default mode)
.beads/dolt/                  Dolt server mode (if enabled)
AGENT_INSTRUCTIONS.md         Full workflow guide
BEADS_SETUP.md                Setup & troubleshooting
BEADS_INTEGRATION.md          Integration summary
```

## When Things Go Wrong

```bash
# Can't run `bd`
which bd                      # Not in PATH?
brew install beads            # Reinstall
npm install -g @beads/bd      # Try npm

# Database corrupt
rm -rf .beads
bd init --stealth             # Reinitialize

# Stuck on task
bd update <id> --notes "Unblocked, now proceeding"
bd update <id> --status in_progress

# Want to start over
bd backup init /tmp/backup
bd backup sync
rm -rf .beads
bd init --stealth
```

## Resources

- **Full Docs**: `BEADS_SETUP.md`
- **Agent Instructions**: `AGENT_INSTRUCTIONS.md`
- **Beads Repo**: https://github.com/gastownhall/beads
- **This Card**: `BEADS_QUICK_REFERENCE.md`

---

**Save this file for quick reference!**  
**Print it if you work on Hermes frequently.**
