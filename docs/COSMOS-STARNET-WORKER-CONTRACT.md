# StarNet → Hermes/Cosmos worker-call compatibility — source inspection

**Requested contract:** authenticated `POST` containing JSON `{runId, mission, role, task}`, returning `{ok, summary, artifacts, usd}` as a verified receipt.

## Verified from checked-in source

- `gateway/platforms/api_server.py` declares `POST /v1/runs`, `GET /v1/runs/{run_id}`, and streaming/status/approval routes. The `/v1/runs` handler requires a body `input` field and starts an **asynchronous** run, initially returning a run identifier, rather than synchronously implementing the requested `ok/summary/artifacts/usd` worker contract.
- API calls require authentication through `_check_auth()`. A documented OpenAI-compatible API is **not** the same as the StarNet worker-call schema. StarNet's `docs/SINGLE_AGENT_SERVICE_PROTOCOL.md` describes intended service arrangements, not a successful live receipt.

## Not verified

No live worker-protocol POST was performed. The Surface was offline when this audit occurred. No specific live endpoint, authentication identity, response schema, trace, artifact list, spend measurement, or rollback path has been verified. **Do not claim that Hermes currently answers this contract.**

## Required future proof

1. Verify the actual deployed HTTP interface, configuration and authenticated access without revealing credentials.
2. Independently inspect the StarNet caller's exact request and expected response.
3. If incompatible, design a **separate reviewed, least-privilege adapter** that maps authorized Paperclip task IDs to the Hermes runtime; do not bypass Instinct/Heisenberg/Paperclip queue ownership.
4. Use a harmless pre-authorized `runId` and `mission`, capture the exact authenticated request/response with sensitive fields redacted, and independently verify `ok`, `summary`, `artifacts`, `usd`, idempotence, errors and cost reporting.
5. Do not start a model run, turn on API access or mutate a server without owner approval.

This is a documentation-only source compatibility assessment; no runtime adapter or endpoint was created.
