# PRD: Hermes Autonomous Operator Upgrade (2026-04-23)

## Objective
Evolve the existing Hermes fork into a production operator for GitHub + Vercel + Infisical + Twilio voice with strong observability and deployability on Hostinger/Coolify.

## In-Scope Functional Outcomes
- GitHub: inspect repos/issues/PRs/workflows, trigger workflows safely, webhook ingest and verification, audit trail.
- Vercel: inspect projects/deployments/logs, diagnose common failures, redeploy.
- Infisical: bootstrap auth + managed secret sync + health checks.
- Twilio Voice: inbound/outbound voice to agent loop, transcript persistence, STT/TTS flow.
- Dashboard: operator health across repo/deploy/secret/voice/knowledge/job planes.
- Knowledge plane: ingest/index/search with source attribution.

## Non-Goals
- Faking live integrations without credentials.
- Replacing Hermes core loop.

## Acceptance Criteria
1. Required adapters implemented and tested.
2. CI checks cover new critical behaviors.
3. Deployment runbooks validated against real target environment.
4. Auditability for every external action path.

## Current Program Status
- Discovery and planning artifacts are in place.
- Full implementation remains open; external credentials/access are required for final green E2E validation.
