# Vercel IMC A2A Deploy Agent

A portable, filesystem-orchestrated Vercel deployment agent that can be called by humans, IDE agents, GitHub webhooks, or other agents through an Agent2Agent-compatible surface.

The agent’s first job is to build and protect itself. Its second job is to sweep GitHub/Vercel projects and restore visible production deployments without making application source-code changes.

## Operating thesis

This is not a monolithic bot. It is an Interconnected / Interpretable Methodology Context workspace:

- Numbered folders are stages.
- `CONTEXT.md` files are executable contracts.
- `SKILL.md` files are callable capabilities.
- Scripts do mechanical work and emit JSON.
- Sub-agents receive small, scoped briefs instead of the whole repo.
- Every run leaves auditable files in `runs/`.

## Core mission

When the agent receives a GitHub trigger or A2A task:

1. Confirm the event is allowed.
2. Identify the repo, branch, commit, and Vercel scope.
3. Link or resolve the matching Vercel project.
4. Deploy the current main branch to production when authorized.
5. Wait for the deployment result.
6. Inspect build logs if deployment fails.
7. Open the deployed URL and verify that a real page is visible instead of a 404, auth wall, error screen, or blank shell.
8. Iterate through safe no-code repair actions.
9. Produce a machine-readable report.

## What this agent may do automatically

- Read GitHub webhook payloads.
- Read GitHub repo metadata.
- Clone repositories into a workspace.
- Read project files, configs, build logs, Vercel project metadata, and deployment metadata.
- Run `vercel link`, `vercel deploy --prod`, `vercel inspect --wait --logs`, and safe read-only Vercel commands.
- Run browser/HTTP smoke checks against preview or production URLs.
- Retry deployments with safe flags such as `--force` when prior deploy state may be stale.
- Spawn scoped sub-agents for inventory, logs, deployment triage, browser verification, and reporting.

## What this agent must not do without human approval

- Edit application source code.
- Commit, push, force-push, merge, rebase, or delete branches.
- Delete Vercel projects, domains, deployments, or environment variables.
- Change billing, team ownership, domain DNS, or access control.
- Print secret values.
- Deploy non-main branches to production unless explicitly allowed.
- Bypass deployment protection except with an explicit configured bypass token.

## Quick start

```bash
cp .env.example .env
npm install
npm run check
```

Start a local webhook/A2A receiver:

```bash
npm run serve
```

Run a dry sweep against a single repo:

```bash
node scripts/run-agent-cycle.mjs --repo owner/repo --branch main --dry-run
```

Run a production deploy cycle for a repo already connected to Vercel:

```bash
node scripts/run-agent-cycle.mjs --repo owner/repo --branch main --prod
```

Inventory GitHub repos and Vercel projects:

```bash
node scripts/inventory-github-vercel.mjs --github-owner owner --out runs/inventory.json
```

## Required environment variables

See `.env.example`.

Minimum for real execution:

- `GITHUB_TOKEN`
- `VERCEL_TOKEN`
- `VERCEL_TEAM_ID` or `VERCEL_TEAM_SLUG` when projects live under a team
- `A2A_SHARED_SECRET` for remote agent calls
- `GITHUB_WEBHOOK_SECRET` for webhook verification

## Folder map

```text
.
├── AGENTS.md                         # Universal IDE/agent instructions
├── CLAUDE.md                         # Claude Code entrypoint
├── CONTEXT.md                        # Workspace router
├── a2a/                              # Agent card + A2A protocol surface
├── guardrails/                       # Production safety rules
├── hooks/                            # Webhook payload contracts
├── icm/                              # ICM/IMC layer explanation
├── stages/                           # Numbered execution pipeline
├── subagents/                        # Scoped sub-agent roles
├── skills/                           # Callable capabilities
├── scripts/                          # Mechanical JSON-emitting tools
├── missions/                         # First mission and later repeatable missions
└── runs/                             # Runtime artifacts, ignored except placeholder
```

## IDE usage

Any coding agent should start by reading:

1. `AGENTS.md`
2. `CONTEXT.md`
3. The relevant `stages/*/CONTEXT.md`
4. The specific `skills/*/SKILL.md` it needs

Do not load the entire workspace unless performing architecture review.

## Source references embedded in this kit

- `resources/source-notes/vercel-open-agents.md`
- `resources/source-notes/a2a-protocol.md`
- `resources/source-notes/icm-methodology.md`
- `resources/imported-vercel-skills/`
