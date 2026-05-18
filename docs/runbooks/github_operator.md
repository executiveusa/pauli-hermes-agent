# GitHub Operator Runbook (2026-04-23)

## Purpose
Operate authorized GitHub repositories from Hermes with auditable actions.

## Required Inputs (External)
- GitHub token/app credentials with least privilege for target repos/org.
- Target org/repo allowlist.
- Webhook secret and reachable webhook endpoint.

## Runtime Capabilities to Enable
1. Repo metadata sync/index.
2. Issue/PR/workflow inspection.
3. Workflow dispatch trigger with policy checks.
4. Webhook verification + event ingestion.
5. Action audit logging (who/what/when/result).

## Validation Checklist
- Auth handshake succeeds.
- Repo scan returns metadata for allowlisted repos only.
- Workflow dispatch writes audit event and captures result.
- Webhook signature mismatch is rejected.
- Dashboard shows workflow and repo health.

## External Blockers for Full Validation
- Missing GitHub credentials and allowlisted repo identifiers in this environment.
