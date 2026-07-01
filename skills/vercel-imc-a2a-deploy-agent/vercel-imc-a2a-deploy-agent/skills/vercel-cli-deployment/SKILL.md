---
name: vercel-cli-deployment
description: Link, deploy, inspect, and verify Vercel projects using Vercel CLI/API with token-based authentication.
---

# Vercel CLI Deployment

## Use when

- Production main needs redeployment.
- A Vercel project must be linked to a GitHub repo checkout.
- Build logs or deployment status are needed.

## Workflow

1. Confirm `VERCEL_TOKEN` exists.
2. Resolve team scope from `VERCEL_TEAM_SLUG` or `VERCEL_TEAM_ID`.
3. Use `vercel link --repo --yes` when the checkout is not linked.
4. Deploy with `vercel deploy --prod --yes --no-wait`.
5. Inspect with `vercel inspect <url> --wait --logs`.
6. Write deployment and inspection JSON/log files.

## Safe commands

```bash
vercel whoami
vercel teams list --format json
vercel link --repo --yes
vercel deploy --prod --yes --no-wait
vercel inspect <deployment-url> --wait --logs
vercel list --status READY
```

## Forbidden

Do not remove domains, projects, env vars, or deployments.
