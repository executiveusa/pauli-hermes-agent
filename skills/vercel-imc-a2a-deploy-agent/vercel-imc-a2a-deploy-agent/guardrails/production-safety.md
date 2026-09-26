# Production Safety Guardrails

## Production deployment gate

A production deployment may run automatically only when all conditions are true:

1. Event source is verified.
2. Repository is identified.
3. Branch is approved by `config/production-policy.json`.
4. Vercel token exists.
5. Vercel project scope is resolved.
6. No source-code edits are required.
7. The run writes all artifacts to `runs/<run-id>/`.

## Forbidden commands

Never run automatically:

```bash
git push --force
git reset --hard origin/main
git clean -xfd
vercel project rm
vercel domains rm
vercel env rm
vercel env add
vercel env update
vercel alias rm
```

## Production rollback

Do not rollback automatically in Mission 001. If production breaks, write a rollback recommendation with exact deployment IDs but require human approval.

## Error handling

- One transient retry is allowed.
- One `--force` redeploy is allowed when build cache or stale deployment is suspected.
- After two failed production deploy attempts, stop and report.
