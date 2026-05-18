# OpenClaude Dashboard Panel — UI Specification

## Overview

The OpenClaude dashboard panel is a sidebar widget on the Hermes dashboard Workers page. It is a **supporting panel**, not a second chat interface. The primary conversation remains the embedded `hermes --tui` terminal.

The panel follows the existing dashboard pattern established by `plugins/example-dashboard/` and the kanban dashboard in `plugins/kanban/dashboard/`.

---

## Workers Page Layout

```
┌──────────────────────────────────────────────────────────────┐
│  Hermes Dashboard                            [nav tabs]       │
│  Chat  |  Workers  |  Sessions  |  Kanban  |  Settings        │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  Workers                                                      │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  OpenClaude Coding Worker                               │ │
│  │                                                         │ │
│  │  Status:   ● Healthy  (gRPC port 50051)                 │ │
│  │  Provider: openrouter                                   │ │
│  │  Model:    meta-llama/llama-3.1-8b-instruct:free        │ │
│  │  Active beads: 0 / 2 max                                │ │
│  │                                                         │ │
│  │  [Activate]  [Stop]  [Restart]  [Healthcheck]           │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  Assign Bead                                            │ │
│  │  Task type: [refactor ▼]                               │ │
│  │  Description: ________________________________________  │ │
│  │  Repo path:   ________________________________________  │ │
│  │  [Dispatch]                                             │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  Recent Beads                                           │ │
│  │  bead_001  refactor  ● success  34.2s  2 files changed  │ │
│  │  bead_002  docs      ● running  ...                     │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  Worker Chat (direct message)                           │ │
│  │  Bead: [bead_001 ▼]   Model: [auto ▼]                   │ │
│  │  ┌─────────────────────────────────────────────────┐   │ │
│  │  │ Message history (streaming)                     │   │ │
│  │  └─────────────────────────────────────────────────┘   │ │
│  │  [   Type a message...                        ] [Send]  │ │
│  └─────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

---

## Status Panel

### Status indicator

| State | Color | Label | Condition |
|---|---|---|---|
| Healthy | Green ● | "Healthy" | Binary found, gRPC port responding |
| Starting | Yellow ● | "Starting…" | Binary found, port not yet open |
| CLI mode | Blue ● | "CLI mode" | Binary found, no gRPC (per-task dispatch) |
| Offline | Red ● | "Offline" | Binary not found or process dead |
| Error | Red ● | "Error" | Last healthcheck returned an error |

### Status refresh

- Polls `GET /api/workers/openclaude/status` every 10 seconds.
- On click of "Healthcheck", immediately re-polls and shows a toast notification.

---

## Activate Button Behavior

1. User clicks "Activate".
2. Button changes to "Starting…" (disabled, spinner).
3. Frontend sends `POST /api/workers/openclaude/start`.
4. Backend runs `scripts/pauli/openclaude/start.sh` as subprocess.
5. Frontend polls `GET /api/workers/openclaude/status` every 2 seconds.
6. When status becomes "Healthy":
   - Button changes to "Active" (disabled).
   - "Stop" and "Restart" buttons become enabled.
7. If status does not become "Healthy" within 30 seconds:
   - Toast: "Worker failed to start. Check logs."
   - Log section auto-expands.

---

## Assign Bead Form

### Fields

| Field | Type | Required | Notes |
|---|---|---|---|
| Task type | Select | Yes | Allowed: refactor, test_repair, frontend, docs, mcp, codegen |
| Description | Textarea | Yes | Plain English description of the coding task |
| Repo path | Text | No | Absolute path; defaults to current project root |
| Allowed files | Textarea | No | One per line; limits worker write access |
| Timeout (seconds) | Number | No | Default 600 |

### Submit behavior

1. Client sends `POST /api/workers/openclaude/assign-bead` with form data.
2. Backend validates task type against allow-list.
3. If denied: immediate error toast. Form remains open.
4. If allowed: bead appears in "Recent Beads" list with status "running".
5. List polls `GET /api/workers/openclaude/tasks` every 5 seconds.
6. On completion: status updates to "success" or "failed". Files changed shown inline.

---

## Worker Chat Widget

### Purpose

Direct message interface for ad-hoc instructions to the worker. Suitable for quick tasks that don't need a full bead spec. Creates a bead internally with `task_type = "codegen"` unless a bead is explicitly selected.

### Controls

- **Bead selector**: Dropdown populated from `GET /api/workers/openclaude/tasks`. "New bead" option creates a fresh bead.
- **Model selector**: Dropdown showing all configured providers + "auto" (cheapest available). Default: auto.
- **Message input**: Single-line or multi-line text input.
- **Send button**: Submits `POST /api/workers/openclaude/chat`.

### Streaming response

The response is streamed via SSE (server-sent events) from `POST /api/workers/openclaude/chat`. The frontend appends tokens to the message history as they arrive. A "stop" button is shown while streaming, which sends `DELETE /api/workers/openclaude/chat/{request_id}`.

---

## API Endpoints (Backend Requirements)

See `pauli/workers/openclaude/API_SPEC.md` for the full contract. The panel requires:

- `GET /api/workers` — to list all workers for the Workers page
- `GET /api/workers/openclaude/status` — status polling
- `POST /api/workers/openclaude/start` — activate button
- `POST /api/workers/openclaude/stop` — stop button
- `POST /api/workers/openclaude/restart` — restart button
- `POST /api/workers/openclaude/healthcheck` — manual healthcheck
- `POST /api/workers/openclaude/assign-bead` — bead form submit
- `POST /api/workers/openclaude/chat` — chat widget
- `GET /api/workers/openclaude/logs` — log stream (auto-expand on error)
- `GET /api/workers/openclaude/tasks` — bead list

---

## Frontend Implementation Notes

- The panel is a React component (`OpenClaudeSidebar.tsx` or `OpenClaudeWorkerPanel.tsx`).
- It uses the same `useFetch` / `usePolling` hooks as the existing kanban dashboard.
- State is local to the component — it does not modify the PTY session or Hermes conversation state.
- Failures in the panel must not affect the embedded TUI terminal pane.
- The chat widget uses SSE via the `useEventSource` hook (create if not already available).
- The panel respects the existing dashboard theme (dark/light via CSS variables).
