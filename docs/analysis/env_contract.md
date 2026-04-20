# Environment Contract

## Canonical source
- `.env.example` is the superset contract for runtime and integrations.

## Selected operator-relevant variables
- GitHub: `GITHUB_TOKEN`, `GH_TOKEN`
- Twilio/SMS: `TWILIO_ACCOUNT_SID`, `TWILIO_AUTH_TOKEN`, `TWILIO_PHONE_NUMBER`, `SMS_WEBHOOK_URL`
- Webhook: `WEBHOOK_ENABLED`, `WEBHOOK_PORT`, `WEBHOOK_SECRET`
- Dashboard: `HERMES_DASHBOARD_HOST`, `HERMES_DASHBOARD_PORT` (plus config-driven options)
- Runtime profiles: `HERMES_HOME`, `HERMES_PROFILE`

## Gap for target state
- No documented `INFISICAL_*` variables currently present in `.env.example`.
- No dedicated Vercel deployment operator credentials documented beyond model-provider gateway aliases.

## Snapshot
- Total env keys parsed from `.env.example`: 94
