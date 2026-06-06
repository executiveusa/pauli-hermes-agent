# Operator Extension Architecture (2026-04-23)

## Architectural Principle
Keep Hermes as the kernel and add operator capabilities as isolated extension modules/adapters.

## Target Layering
1. **Kernel (existing)**
   - `run_agent.py`, `model_tools.py`, `toolsets.py`
2. **Operator Adapters (new/expanded)**
   - `tools/operator_github.py`
   - `tools/operator_vercel.py`
   - `tools/operator_infisical.py`
   - `tools/operator_voice_twilio.py`
3. **Workers + Indexers (new/expanded)**
   - Repo sync/index queue workers.
   - Deployment diagnostic workers.
   - Knowledge ingestion/indexing workers.
4. **Control Plane APIs (new/expanded)**
   - Dashboard endpoints for repo/deploy/secret/voice health.
5. **Audit + Observability (new/expanded)**
   - Append-only operator action log.
   - Structured events/metrics for every external action.

## Cross-Cutting Contracts
- Secrets: bootstrap-to-managed separation (Infisical runbook).
- Safety: explicit allowlists/scopes for workflow trigger and deploy actions.
- Explainability: every autonomous action writes an auditable event with actor, target, result.
- Profile safety: all persistent paths use `get_hermes_home()`.

## Delivery Strategy
- Stage capabilities behind config gates/toolsets.
- Land adapter tests before enabling autonomous writes in production.
- Keep dashboard cards read-only first, then progressive action enablement with confirmation controls.
