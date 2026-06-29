# OpenClaude Dashboard Panel Runbook

## Overview

The OpenClaude dashboard panel is a supporting UI element in the Hermes dashboard (accessible via `hermes dashboard`). It is a status panel, not a second chat surface. The primary conversation interface remains the embedded `hermes --tui` terminal. The OpenClaude panel displays worker status, allows starting/stopping the worker, and surfaces bead activity.

See `pauli/dashboard/openclaude/DASHBOARD_SPEC.md` for the full UI specification.

---

## Architecture

The dashboard is served by the FastAPI web server in `hermes_cli/web_server.py`. The OpenClaude panel is a React sidebar widget (similar to the existing `ChatSidebar` pattern). It does **not** implement a separate chat transcript or composer — the embedded TUI handles those.

```
Browser (React)
  ├── ChatPage.tsx           ← embeds hermes --tui via xterm.js PTY
  └── OpenClaudeSidebar.tsx  ← OpenClaude status panel (new widget)
        ├── Status card      ← GET /api/workers/openclaude/status
        ├── Activate button  ← POST /api/workers/openclaude/start
        ├── Bead list        ← GET /api/workers/openclaude/tasks
        └── Chat widget      ← POST /api/workers/openclaude/chat
```

---

## Accessing the Panel

1. Start the Hermes dashboard:

   ```bash
   hermes dashboard
   ```

2. Navigate to `http://localhost:7860` (or your configured port).

3. Open the **Workers** tab in the sidebar.

4. The **OpenClaude** card shows:
   - Status indicator (green = healthy, red = offline, yellow = starting)
   - Current model and provider
   - Active bead count
   - Activate / Stop / Restart buttons

---

## API Endpoints

The panel communicates with these API endpoints (defined in `pauli/workers/openclaude/API_SPEC.md`):

| Endpoint | Purpose |
|---|---|
| `GET /api/workers` | List all registered workers |
| `GET /api/workers/openclaude/status` | Current status of the OpenClaude worker |
| `POST /api/workers/openclaude/start` | Start the worker |
| `POST /api/workers/openclaude/stop` | Stop the worker |
| `POST /api/workers/openclaude/restart` | Restart the worker |
| `POST /api/workers/openclaude/healthcheck` | Trigger a health check |
| `POST /api/workers/openclaude/assign-bead` | Assign a new bead to the worker |
| `POST /api/workers/openclaude/chat` | Send a message to the worker and stream the response |
| `GET /api/workers/openclaude/logs` | Stream recent log lines |
| `GET /api/workers/openclaude/tasks` | List recent beads and their statuses |
| `GET /api/workers/openclaude/changed-files` | List files changed by the last bead |

---

## Activate Button Behavior

When the user clicks "Activate":

1. The frontend sends `POST /api/workers/openclaude/start`.
2. The server calls `scripts/pauli/openclaude/start.sh` as a subprocess.
3. The server polls `GET /api/workers/openclaude/status` every 2 seconds for up to 30 seconds.
4. The status card updates in real time (yellow "starting" → green "healthy" or red "failed").
5. If the start times out, the panel shows an error with a link to the logs.

---

## Chat Widget

The OpenClaude chat widget is a lightweight message interface for sending one-shot instructions to the worker without writing a full bead spec manually. It is **not** the primary Hermes conversation — it is a power-user shortcut.

Usage:
1. Select a bead from the bead selector (or leave blank to create a new bead).
2. Select a model override (or use "auto" for cheapest available).
3. Type an instruction in the message input and press Send.
4. The panel streams the worker's response below the input.

The widget calls `POST /api/workers/openclaude/chat` with `{bead_id, message, model_override}` and streams the response via server-sent events (SSE).

---

## Troubleshooting

### Panel shows "Offline"

1. Check if the worker is running: `scripts/pauli/openclaude/healthcheck.sh`
2. If not running, click "Activate" or run `scripts/pauli/openclaude/start.sh`
3. If the worker is running but the panel still shows "Offline", check that the API server is running and the `GET /api/workers/openclaude/status` endpoint is reachable.

### Activate button spins indefinitely

The worker may have crashed at startup. Check the log:

```bash
tail -50 ~/.hermes/logs/openclaude-worker.log
```

Common cause: missing or malformed `~/.openclaude.json`. Re-run `generate-config.sh`.

### Chat widget returns no response

The worker may be in CLI-stub mode (no persistent process). In this mode, the chat widget dispatches a new subprocess per message and may be slow. Consider starting the worker in gRPC mode for better responsiveness.
