# Integrator Migration and Compatibility Playbook (DROPIN-163 / bd-2sx56)

Generated: 2026-02-15

## Purpose

This playbook gives downstream teams a practical, low-risk path to migrate from TypeScript Pi (`pi-mono`) to `pi_agent_rust` and verify compatibility without needing internal project context.

Use this when you need to:
- replace an existing `pi` integration with Rust Pi,
- validate that automation/scripts still behave correctly,
- document a go/no-go decision with reproducible evidence.

## Compatibility Contract Inputs

Before migration, pin these artifacts as your source of truth:

- Baseline snapshot: `docs/dropin-upstream-baseline.json`
- Surface inventory: `docs/dropin-feature-inventory-matrix.json`
- Gap ledger: `docs/dropin-parity-gap-ledger.json`
- Certification gates: `docs/dropin-certification-contract.json`
- Certification verdict artifact (strict claim gate): `docs/dropin-certification-verdict.json`
- Current parity status snapshot (informational): `docs/parity-certification.json`

If your required workflow maps to an open `critical`/`high` gap in `docs/dropin-parity-gap-ledger.json`, treat migration as blocked until that gap is closed or explicitly waived for your environment.
If `docs/dropin-certification-verdict.json` is missing or not `overall_verdict = CERTIFIED`, treat strict drop-in replacement messaging as blocked.

## Migration Outcomes

A migration is complete only when all are true:

1. Rust Pi is installed and callable through the intended command (`pi` or `pi-rust`).
2. Required execution surfaces pass validation (interactive, print, JSON mode, RPC, SDK where used).
3. Provider/auth/config behavior matches your production expectations.
4. Evidence artifacts are stored so another engineer can reproduce the same result.

## Phase 0: Pre-Migration Inventory

Record your current TypeScript Pi usage footprint:

- Invocation surfaces used:
  - interactive
  - print text/json
  - RPC
  - SDK embedding
- CLI flags/subcommands your automation depends on
- Providers/models used in production
- Session persistence expectations (resume behavior, session directory usage)
- Extension usage (tools, commands, capability prompts)

Minimum capture template:

```text
Current pi version:
Execution surfaces in use:
Required flags/subcommands:
Provider + model matrix:
Env vars used:
Extension dependencies:
Session storage expectations:
```

## Phase 1: Install and Command Strategy

Use one of these rollout options:

1. Canonical replacement (preferred): Rust Pi becomes `pi`, legacy preserved as `legacy-pi`.
2. Side-by-side canary: keep TypeScript `pi`, install Rust as `pi-rust`.

Verification commands:

```bash
command -v pi
pi --version
pi --help >/dev/null

# If side-by-side migration is used
command -v legacy-pi || true
command -v pi-rust || true
```

## Phase 2: Configuration and Credential Migration

Move settings and secrets deliberately; do not rely on implicit defaults.

### 2.1 Settings

Review and reconcile:
- `~/.pi/agent/settings.json`
- project-level `.pi/settings.json`

Key parity-sensitive areas:
- default provider/model/thinking level
- queue modes (`steeringMode`, `followUpMode`)
- compaction/retry knobs
- extension policy and repair policy
- terminal/image behavior

### 2.2 Credentials

Validate all provider credentials required by your workflows (for example `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `GOOGLE_API_KEY`, `AZURE_OPENAI_API_KEY`, `COHERE_API_KEY`, plus any OpenAI-compatible provider keys).

Reference:
- `docs/provider-auth-crosswalk.json`
- `docs/provider-auth-troubleshooting.md`

## Phase 3: Surface-by-Surface Compatibility Validation

Run only the checks relevant to your integration footprint.

### 3.1 CLI and Interactive

```bash
pi --list-models >/dev/null
pi config >/dev/null
pi --model claude-sonnet-4-20250514 -p "ping"
```

Validate:
- expected flags parse successfully,
- expected subcommands exist (`install`, `remove`, `update`, `list`, `config`),
- interactive slash commands used by your team are present/working.

### 3.2 Print and JSON Mode

```bash
printf 'Hello\n' | pi -p
printf 'Hello\n' | pi --mode json
```

Validate:
- stdout framing and exit codes match expectations,
- JSON event envelopes parse in existing tooling,
- no downstream parser breaks on event ordering or field names.

### 3.3 RPC Mode

Smoke-check line-delimited JSON protocol:

```bash
pi --mode rpc
```

Then send at least:
- `prompt`
- `get_state`
- `follow_up`
- `abort`
- `compact` (if used by clients)

Validate:
- command handling semantics,
- event order consistency,
- tool and extension UI events if your client depends on them.

Reference:
- `docs/rpc.md`

### 3.4 SDK (Only for Embedded Integrations)

If embedding Pi as a library surface, run the SDK migration checks in:
- `docs/sdk.md`
- `docs/dropin-sdk-contract.json`

Do not claim SDK drop-in compatibility unless your usage scenario passes those checks.

## Phase 4: Session and Persistence Validation

Validate behavior for your actual session workflows:

```bash
pi --continue
pi --session <path-to-known-session>
```

Checks:
- resume selects expected project session,
- message history and branching semantics are preserved,
- session index behavior is acceptable for your usage.

Reference:
- `docs/session.md`
- `docs/tree.md`

## Phase 5: Extension and Tooling Validation (If Applicable)

If you rely on extensions, validate:
- extension load/discovery,
- capability prompt behavior,
- required hostcalls (`tool/http/session/ui`),
- policy behavior (`safe`/`balanced`/`permissive`) for your deployment mode.

References:
- `EXTENSIONS.md`
- `docs/extension-architecture.md`
- `docs/capability-prompts.md`

## Phase 6: CI Evidence and Go/No-Go Gate

Before promoting to production, capture and store:

- command transcript of migration checks,
- machine-readable test/log artifacts from your CI run,
- explicit pass/fail against each required surface,
- unresolved parity risks (if any) with owner and mitigation.

Recommended gate policy:
- Block rollout if any required surface fails.
- Block rollout if an unresolved `critical` parity gap affects your workflow.
- Block strict drop-in replacement messaging unless `docs/dropin-certification-verdict.json` exists and reports `overall_verdict = CERTIFIED`.
- Require sign-off that references the artifact set used for the decision.

## Rollback Plan

If compatibility fails in canary or production:

1. Switch command aliasing back to legacy (`legacy-pi` or TypeScript `pi`).
2. Restore prior config snapshot.
3. Record failing command/event transcript.
4. Map failure to a parity gap entry (or create one) before retrying migration.

## Fast Checklist

```text
[ ] Baseline/gap/certification artifacts reviewed (including drop-in verdict)
[ ] Required surfaces identified
[ ] Install strategy selected (canonical vs canary)
[ ] Config + credential migration completed
[ ] CLI/print/JSON/RPC/SDK checks run as applicable
[ ] Session behavior validated
[ ] Extension behavior validated (if used)
[ ] Evidence captured and archived
[ ] Go/No-Go decision documented with rollback path
```

## Related References

- `docs/dropin-upstream-baseline.json`
- `docs/dropin-feature-inventory-matrix.json`
- `docs/dropin-parity-gap-ledger.json`
- `docs/dropin-certification-contract.json`
- `docs/dropin-certification-verdict.json`
- `docs/parity-certification.json`
- `docs/rpc.md`
- `docs/session.md`
- `docs/sdk.md`
- `docs/providers.md`
