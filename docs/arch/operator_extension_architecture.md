# Operator Extension Architecture (Draft)

- Preserve Hermes kernel (`run_agent.py` + `model_tools.py`) as stable core.
- Add operator capabilities as extension modules under `tools/operator_*` and optional gateway services.
- Use explicit adapter boundaries for GitHub, Vercel, Infisical, Twilio Voice.
- Persist operator actions to append-only audit store for traceability.
- Surface health and queue telemetry into dashboard APIs.
