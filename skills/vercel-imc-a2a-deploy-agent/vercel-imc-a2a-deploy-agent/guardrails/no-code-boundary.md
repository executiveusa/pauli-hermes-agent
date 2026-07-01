# Mission 001 No-Code Boundary

Mission 001 is a deployment visibility sweep, not a refactor.

## Allowed no-code fixes

| Blocker | Allowed action |
|---|---|
| Wrong Vercel scope | Relink with confirmed project/team. |
| Stale deployment | Redeploy current main. |
| Failed deployment with old cache | Redeploy once with `--force`. |
| Unknown latest deploy | Use `vercel list`, `vercel inspect`, and metadata. |
| Protected preview page | Use configured bypass token only for verification. |
| 404 on old preview URL | Find latest production deployment and verify that URL. |

## Stop and report when code is required

Examples:

- Missing route/page files.
- Broken imports.
- TypeScript errors.
- Build script errors.
- Missing package dependency.
- Framework config incorrect.
- Runtime exception from application code.
- App intentionally requires auth before hero content.
