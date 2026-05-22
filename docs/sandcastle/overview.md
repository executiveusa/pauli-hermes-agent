# Sandcastle: Safe Execution for AI Agents

## What is Sandcastle?

Sandcastle is the **Safe Execution Room** for Pauli Hermes.

When a coding agent needs to modify your codebase, it runs inside an isolated sandbox—not directly on your system. This means:

- ✅ Code changes are safe and reversible
- ✅ Tests run before merge
- ✅ You review every commit
- ✅ Nothing reaches production without your approval
- ✅ Failed runs can be discarded with no trace

## How It Works

```
Hermes decides what to do
  ↓
Creates a bead (task)
  ↓
Sandcastle creates isolated sandbox
  ↓
Coding agent works safely
  ↓
Tests run
  ↓
You review & approve
  ↓
Changes merge to main
```

## Supported Providers

| Provider | Type | Speed | Cost | Setup |
|----------|------|-------|------|-------|
| **Vercel** | Firecracker microVM | ⚡⚡ Fast | Free | Cloud-based |
| **Docker** | Local container | ⚡ Medium | Free | Requires daemon |
| **Podman** | Rootless container | ⚡ Medium | Free | No daemon needed |

Sandcastle automatically picks the best available provider.

## Default Behaviors

### Code Changes Always Use Sandbox
When a bead modifies code:
- A feature branch is created
- Changes run in isolation
- Nothing touches main/production directly

### Tests Run Automatically
- Tests execute before merge consideration
- Failing tests block completion
- Test results visible in UI

### Approval Required
- All merges require human review
- Deploy requires separate approval
- Paid models require approval

### No Sandbox Not Allowed (By Default)
- Direct host execution is disabled
- Security-first by design
- Can be overridden with explicit approval

## Safety Rules

Hard enforced:
- ❌ No direct host execution by default
- ❌ No merging without approval
- ❌ No deploying without approval
- ❌ No paid models without approval
- ❌ No secret logging
- ✅ All logs redacted for secrets
- ✅ All work is reversible

## For Users

You don't need to understand Sandcastle's internals. The UI shows:

1. **What agent is doing now** (real-time)
2. **What files changed** (visual diff)
3. **What tests ran** (pass/fail)
4. **What commits were made** (reviewable)
5. **Approve** or **discard** with one click

## For Developers

See:
- [Provider Setup](./provider-setup.md)
- [Observability](./observability.md)
- [Approval Policy](./approval-policy.md)
- [Troubleshooting](./troubleshooting.md)
