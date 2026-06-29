# Environment Contract (Snapshot: 2026-04-23)

## Canonical Source
- `.env.example` is the repository-level baseline for documented runtime env vars.

## Current Variables Detected
- Parsed keys in `.env.example`: **11**.
- Example groups present:
  - Terminal runtime controls (`TERMINAL_*`).
  - Browserbase controls (`BROWSERBASE_*`, `BROWSER_*`).
  - Web/image/tool debug toggles (`*_DEBUG`).

## External Integration Contract Gaps for Target Program
- **GitHub operator**: needs explicit auth/env contract for repo sync, workflow inspection/trigger, webhook verification.
- **Vercel operator**: needs explicit env contract for API auth and project scoping.
- **Infisical**: needs explicit bootstrap credential + runtime secret sync contract.
- **Twilio voice**: needs explicit voice webhook, phone SID/number mapping, speech provider contract.

## Bootstrap vs Managed Secret Policy
- Bootstrap secrets (minimal set needed to authenticate to secret manager) must be documented separately from managed runtime secrets.
- Secret health checks must confirm presence/validity without printing values.

## Immediate Documentation Actions
1. Keep `.env.example` minimal and non-secret.
2. Add integration-specific contracts in runbooks (`docs/runbooks/*`).
3. Add runtime secret validation checks to startup health path once adapters are implemented.
