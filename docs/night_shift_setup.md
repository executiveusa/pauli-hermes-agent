# Hermes Night Shift Setup (VPS)

This setup adds a clean-room **Night Shift control plane** for multi-repo orchestration, approval loops, disposable sub-agents, provider routing, and a cockpit-friendly API.

## 1) Start services

```bash
./scripts/start_night_shift.sh
```

## 2) Required secrets

Set these in your VPS environment or `.env` before production use:

- `GITHUB_TOKEN`
- `OPENAI_API_KEY`
- `GEMINI_API_KEY`
- `VENICE_API_KEY`
- `TELEGRAM_BOT_TOKEN`
- `APPWRITE_API_KEY`
- `COMPOSIO_API_KEY`
- `PAPERCLIP_API_KEY`

You can inspect missing values via:

```bash
curl http://localhost:8787/setup/missing-secrets
```

## 3) Smoke test

```bash
./scripts/smoke_night_shift.sh
```

## 4) Key API routes

- `GET /health`
- `GET /setup/missing-secrets`
- `GET /dashboard`
- `POST /repos/scan`
- `POST /prds`
- `POST /v1/llm/chat`
- `POST /approvals`
- `POST /subagents`
- `POST /subagents/{id}/kill` (kill switch)
- `POST /appwrite-projects`
- `GET /provider-profiles`
- `GET /mcp`

## 5) Example usage

### Venice-backed chat

```bash
curl -X POST http://localhost:8787/v1/llm/chat \
  -H 'content-type: application/json' \
  -d '{
    "profile":"venice_primary",
    "messages":[{"role":"user","content":"Draft a PRD for onboarding flow"}],
    "temperature":0.2,
    "max_tokens":800
  }'
```

### Telegram approval flow (via Night Shift approval endpoint)

1. Worker proposes action.
2. Worker calls `POST /approvals`.
3. If response is `required`, send Telegram button for approve/deny.
4. Resume task only after operator response.

### WhatsApp business-only mode

Use WhatsApp for narrow workflows (alerts, approvals, business notifications), not open-ended assistant chat. Keep general AI conversations in dashboard or Telegram channels.

## 6) Spanish UX seed strings (es-419)

Night Shift ships starter strings in `/dashboard` payload:

- `Faltan secretos para encender Hermes Night Shift.`
- `Esta acción requiere aprobación humana.`

Extend these strings in your Pauli-Vibe cockpit frontend.

## 7) Next steps

### Rust migration path

1. Move policy classifier + budget guard into a Rust crate.
2. Migrate coding/browser worker executors to Rust workers with gRPC.
3. Keep FastAPI as orchestration edge until full Rust control-plane replacement.

### Temporal full integration path

1. Replace direct DB status updates with Temporal workflows for runs.
2. Add heartbeats/signals for CAPTCHA/MFA pause/resume.
3. Add queries for cockpit progress timelines and SLA tracking.
4. Store workflow IDs on `runs`, `coding_sessions`, `browser_runs`.
