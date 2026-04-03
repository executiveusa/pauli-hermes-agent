# Startup Checks & Operational Checklist

## Pre-start
- [ ] `.env` exists and required secrets loaded
- [ ] DB reachable
- [ ] Redis reachable
- [ ] Policy files present in `/policies`

## Start
- [ ] `cargo run -p foundry-api`
- [ ] `/healthz` returns ok
- [ ] `/readyz` returns ready
- [ ] `/deployment-manifest.json` visible

## Post-start
- [ ] Run smoke script
- [ ] Validate dashboard bootstrap
- [ ] Validate onboarding flow
- [ ] Validate usage endpoint

## Incident first response
- [ ] Check `/metrics`
- [ ] Check `/events`
- [ ] Apply relevant runbook in `docs/runbooks/`
