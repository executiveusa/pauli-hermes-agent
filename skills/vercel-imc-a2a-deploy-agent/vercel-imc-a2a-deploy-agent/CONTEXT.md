# Workspace Context Router

## Workspace role

This workspace defines a Vercel deployment visibility agent that can be invoked by:

- GitHub webhook payloads
- A2A task requests
- CLI calls from any IDE agent
- Human terminal sessions

## Active mission router

| User/task intent | Stage to open | Skill set |
|---|---:|---|
| Another agent asks “who are you / what can you do?” | `stages/00_handshake/CONTEXT.md` | `skills/a2a-router/SKILL.md` |
| GitHub webhook arrives | `stages/01_incoming_event/CONTEXT.md` | `skills/github-webhook-trigger/SKILL.md` |
| Build a repo/project inventory | `stages/02_repo_inventory/CONTEXT.md` | `skills/repo-inventory/SKILL.md` |
| Match GitHub repos to Vercel projects | `stages/03_vercel_project_map/CONTEXT.md` | `skills/repo-inventory/SKILL.md`, `skills/vercel-cli-deployment/SKILL.md` |
| Diagnose deployment or 404 blockers | `stages/04_deployment_triage/CONTEXT.md` | `skills/no-code-repair-loop/SKILL.md` |
| Deploy production main | `stages/05_redeploy_main/CONTEXT.md` | `skills/vercel-cli-deployment/SKILL.md` |
| Confirm deployed page is visible | `stages/06_browser_verification/CONTEXT.md` | `skills/browser-smoke-check/SKILL.md` |
| Iterate with scoped sub-agents | `stages/07_subagent_iteration/CONTEXT.md` | subagent briefs |
| Produce report/handoff | `stages/08_report_and_handoff/CONTEXT.md` | `subagents/report-writer/AGENT.md` |

## Stable references

- `guardrails/production-safety.md`
- `guardrails/secrets.md`
- `guardrails/no-code-boundary.md`
- `hooks/github-webhook-contract.md`
- `a2a/protocol.md`
- `icm/methodology.md`
- `resources/imported-vercel-skills/`

## Run artifact convention

Every run gets an ID:

```text
YYYYMMDD-HHMMSS-<repo-slug>-<short-sha-or-manual>
```

Write files under:

```text
runs/<run-id>/
```

## Default production branch policy

Allowed production branches:

- `main`
- `master` only if configured in `config/production-policy.json`

Non-main branches are preview-only unless the operator explicitly overrides the policy.
