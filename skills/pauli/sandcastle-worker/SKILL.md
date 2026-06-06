# Sandcastle Worker Skill

**Name:** `pauli-sandcastle-worker`  
**Description:** Use Sandcastle to run coding agents in isolated branches/sandboxes with live observability, commits, logs, tests, and approval gates.  
**Version:** 1.0.0

## Triggers

- "sandbox this"
- "run in sandbox"
- "safe coding task"
- "fix repo"
- "repair tests"
- "fix Vercel"
- "edit UI"
- "create PR"
- "run worker"
- "assign bead"
- "observe agent"

## When to Use

- Any task that modifies code
- Implementing features
- Fixing bugs
- Repairing failing tests
- Fixing Vercel builds
- Editing UI components
- Creating pull requests
- Running autonomous agents

## When NOT to Use

- Reading-only operations
- Querying documentation
- Analysis without code changes

## Required Tools

- Sandcastle runner
- Git provider (for branching)
- Test framework
- CI/CD system (optional)

## Required Environment

```
SANDCASTLE_ENABLED=true
SANDCASTLE_DEFAULT_PROVIDER=auto
SANDCASTLE_ALLOW_NO_SANDBOX=false
VERCEL_SANDBOX_ENABLED=true|false
DOCKER_SANDBOX_ENABLED=true|false
PODMAN_SANDBOX_ENABLED=true|false
```

## Safety Gates

✅ **Default isolation:** All code runs in sandboxes  
✅ **No direct host execution** by default  
✅ **Sandcastle required** for all code-changing beads  
✅ **Branch isolation:** Every run creates a feature branch  
✅ **Test gating:** Tests must pass before merge consideration  
✅ **Human approval:** All merges require human review  
✅ **Deploy gating:** All deployments require approval  
✅ **Paid model gating:** Premium inference requires approval  
✅ **Destructive action blocking:** No destructive ops without approval  
✅ **Secret redaction:** API keys redacted from logs  

## Workflow

1. **Create bead** with `execution: { mode: sandbox }`
2. **Select provider** (Vercel → Docker → Podman)
3. **Create branch** with pattern `agent/{beadId}-{slug}`
4. **Execute prompt** inside sandbox
5. **Stream events** to UI (agent activity, commits, tests)
6. **Run tests** (if configured)
7. **Request approval** if required
8. **Merge on approval** or **discard**

## Output Contract

Returns:

```typescript
{
  run: SandcastleRun,
  events: SandcastleEvent[],
  success: boolean
}
```

## Failure Modes

| Mode | Recovery |
|------|----------|
| Provider unavailable | Fall back to next provider |
| Tests fail | Agent can fix or task blocked |
| Approval denied | Discard sandbox, branch cleaned up |
| Timeout | Sandbox stopped, work can be retried |
| Secret found in logs | Redacted automatically |

## Tests

### Unit Tests
- Provider selection prefers Vercel then Docker then Podman
- No-sandbox mode blocked by default
- Branch strategy always creates branch
- Events emitted correctly
- Secrets redacted before logging
- Status transitions valid

### Integration Tests
- Full run lifecycle (queue → complete)
- Approval gates work
- Events persisted
- Commits captured
- Changed files tracked

## Examples

### Simple Task
```
Bead: Implement dark mode toggle
Execution: sandbox
Result: Branch created, 3 files changed, tests pass, awaiting approval
```

### Fix Tests
```
Bead: Repair failing auth tests
Execution: sandbox with tests_required=true
Result: Agent diagnoses, fixes 2 tests, 1 new test added, all pass
```

### Vercel Repair
```
Bead: Fix Vercel build error
Execution: sandbox
Result: Branch created, issue found in next.config, fixed, tests pass
```
