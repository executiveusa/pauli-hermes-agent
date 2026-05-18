# Vercel Operator Runbook (2026-04-23)

## Purpose
Enable Hermes to inspect and repair Vercel deployment issues.

## Required Inputs (External)
- Vercel API token/team scope.
- Project IDs and expected domain mappings.

## Runtime Capabilities to Enable
1. List projects and deployments.
2. Fetch deployment build logs.
3. Diagnose failures: 404/root dir/output dir/framework/env/monorepo/rewrites/domain indicators.
4. Trigger redeploy/rebuild safely.

## Validation Checklist
- API authentication succeeds.
- Project/deployment listing available for scoped projects.
- Diagnostic engine classifies known failure cases.
- Redeploy action generates audit event + updated deployment status.

## External Blockers for Full Validation
- Missing Vercel credentials, project IDs, and domain references in this environment.
