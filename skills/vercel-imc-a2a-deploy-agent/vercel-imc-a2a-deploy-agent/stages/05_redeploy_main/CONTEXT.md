# Stage 05_redeploy_main: Redeploy Main


## Inputs

- Approved repo/branch/commit
- Vercel project link/scope
- `skills/vercel-cli-deployment/SKILL.md`
- `guardrails/production-safety.md`

## Process

1. Clone or use local checkout.
2. Ensure checkout is on approved production branch.
3. Resolve or link Vercel project.
4. Run production deployment.
5. Wait and inspect.
6. Record deployment URL and logs.

## Outputs

- `runs/<run-id>/deploy.json`
- `runs/<run-id>/inspect.json`
- `runs/<run-id>/build.log`

## Gate

Proceed to browser verification only if deployment is ready or provides a URL to inspect.

