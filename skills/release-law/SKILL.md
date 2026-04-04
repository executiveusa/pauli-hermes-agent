---
name: release-law
description: Release process and deployment checklist
platforms: [cli, telegram, discord, slack, api]
---

# Release Law

## Purpose
Ensures every release is safe, documented, and reversible.

## Rules

1. **Tests pass**: Full test suite must pass (`python -m pytest tests/ -q`). No skipping failures.
2. **Version bump**: Follow semver. Breaking change = major. New feature = minor. Bug fix = patch.
3. **Changelog**: Every release has a changelog entry listing what changed, added, fixed, removed.
4. **Git tag**: Tag the release commit with `v{major}.{minor}.{patch}`.
5. **Rollback plan**: Document how to revert before deploying. Vercel: instant rollback. VPS: `docker compose up -d --force-recreate`.
6. **Health check**: Verify the deployment is healthy within 5 minutes of deploy.
7. **No Friday deploys**: Avoid deploying on Fridays unless it's a critical hotfix.
8. **Env parity**: Staging and production must have the same env vars (values may differ).

## Deployment Checklist
- [ ] All tests pass
- [ ] Version bumped in pyproject.toml
- [ ] CHANGELOG.md updated
- [ ] Git commit tagged
- [ ] Rollback plan documented
- [ ] Health check passes post-deploy
- [ ] Monitoring confirms no error spike
