# Workflow Inventory (Snapshot: 2026-04-23)

## GitHub Actions Workflows Detected
- `.github/workflows/contributor-check.yml` — contributor attribution policy checks.
- `.github/workflows/deploy-site.yml` — static/docs site deployment, including Vercel deploy hook usage.
- `.github/workflows/docker-publish.yml` — container build and publish pipeline.
- `.github/workflows/docs-site-checks.yml` — docs quality checks.
- `.github/workflows/nix.yml` — Nix-based validation.
- `.github/workflows/skills-index.yml` — skill index generation automation.
- `.github/workflows/supply-chain-audit.yml` — supply chain review and PR commentary.
- `.github/workflows/tests.yml` — core test matrix entrypoint.

## Observations
- CI foundation is present.
- No dedicated workflow currently validates Infisical integration, Twilio voice paths, or Vercel operator diagnostics as first-class test suites.
- No workflow currently enforces operator capability contract (GitHub/Vercel/Twilio/Infisical) end-to-end.
