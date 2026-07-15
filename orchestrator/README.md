# Hermes Orchestrator — Lightweight Dispatcher

The pragmatic orchestrator for The Pauli Effect agent fleet. Receives missions, routes them to the right specialist agent, reports to Bambu via Telegram.

## Quick Start
```bash
cd orchestrator
node hermes.js  # runs on port 4800
```

Open http://localhost:4800 for the dispatch UI.

## Endpoints
- `GET /` — Dispatch UI
- `POST /dispatch` — Submit a mission (auto-routes or manual)
- `GET /missions` — Recent missions
- `GET /health` — Status check

This is a lightweight alternative to the full Nous Research Hermes agent. It uses the same runtime pattern as the other agents and can be replaced by the full Hermes installation later.
