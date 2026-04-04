# HERMES FOUNDRY — ALWAYS-LOAD MANIFEST
<!-- Load this file at the start of every agent conversation. It is the single source of truth. -->

## Mission
**Hermes Foundry** is a revenue-first, Rust-native agent operating system.
- Primary users: Social-purpose organizations (nonprofits, impact ventures)
- Go-to-market: AI Systems Sprint (4–6 week productized delivery)
- Commercial layer: White-labeled Agent Backend-as-a-Service (BaaS)
- First internal deployment: Archon X by AtomicBot AI

---

## Tech Stack & Versions

| Layer              | Technology                           | Version  |
|--------------------|--------------------------------------|----------|
| Core shell         | Rust                                 | 1.77+    |
| Desktop app        | Tauri                                | 2.x      |
| Frontend           | React + TypeScript + Vite            | 18 / 5   |
| Local LLM          | llama.cpp (atomic-llama-cpp-turboquant) | latest |
| Voice / STT        | whisper.cpp (AtomicBot-ai fork)      | 1.x      |
| Media pipeline     | FFmpeg (AtomicBot-ai fork)           | 8.x      |
| Agent backend      | FastAPI + PostgreSQL + Redis         | Python 3.11 |
| Client BaaS        | Appwrite                             | 1.4+     |
| Social publishing  | Postiz                               | TBD      |
| Source discovery   | OpenSrc (npm/pip)                    | latest   |
| Package manager    | pnpm (Node) / cargo (Rust)           | 9 / latest |

---

## Source Tree Quick Reference

```
hermes-foundry/
├── Cargo.toml                   Rust workspace
├── package.json                 Node workspace root
├── _MANIFEST.md                 ← THIS FILE (always load)
├── _JCP/                        Job Completion Protocol
│   ├── features.json            Feature registry
│   ├── progress.md              Checkpoint log
│   └── recovery.md              Rollback plan
├── branding.json                White-label config
├── .gitmodules                  AtomicBot vendor submodules
├── apps/
│   └── desktop/                 Tauri desktop app
│       ├── src-tauri/           Rust backend (Tauri commands)
│       └── src/                 React/TypeScript UI
├── crates/
│   ├── hermes-types/            Shared types (no deps)
│   ├── hermes-core/             Agent engine, session, tool dispatch
│   ├── hermes-voice/            whisper.cpp HTTP client wrapper
│   ├── hermes-media/            FFmpeg subprocess wrapper
│   └── hermes-llm/              llama.cpp HTTP client (OpenAI-compat)
├── vendor/                      Git submodules
│   ├── whisper.cpp/             AtomicBot-ai/whisper.cpp
│   ├── FFmpeg/                  AtomicBot-ai/FFmpeg
│   └── atomic-llama-cpp-turboquant/
└── scripts/
    ├── setup.sh                 One-time environment bootstrap
    ├── build-desktop.sh         Build & package Tauri app
    └── install-models.sh        Download GGUF models
```

---

## Architecture — Component Map

```
┌─────────────────────────────────────────────────────────────┐
│               HERMES FOUNDRY DESKTOP APP (Tauri v2)          │
│                                                               │
│  ┌─────────────────────────────────────────────────────┐     │
│  │           React Frontend (WebView)                   │     │
│  │  Chat │ Voice │ Media │ Agent Status │ Settings      │     │
│  └──────────────────┬────────────────────────────────────    │
│                     │  Tauri IPC (invoke)                     │
│  ┌──────────────────▼────────────────────────────────────    │
│  │        Rust Tauri Commands Layer                      │     │
│  │  chat.rs │ voice.rs │ media.rs │ llm.rs │ services.rs│     │
│  └──────────┬──────────┬──────────┬────────────────────      │
│             │          │          │                           │
│   ┌─────────▼──┐ ┌─────▼───┐ ┌───▼──────┐                   │
│   │hermes-core │ │hermes-  │ │hermes-   │  Rust crates       │
│   │(engine,    │ │voice    │ │llm       │                     │
│   │ session)   │ │         │ │          │                     │
│   └─────────┬──┘ └────┬────┘ └──┬───────┘                   │
│             │         │         │                            │
└─────────────┼─────────┼─────────┼────────────────────────────
              │         │         │
  ┌───────────▼───┐ ┌───▼──────┐ ┌▼─────────────┐
  │  Hermes API   │ │ whisper- │ │  llama-server │  Sidecar processes
  │  (nightshift) │ │ server   │ │  (local LLM)  │  started by Tauri
  │ FastAPI:8000  │ │ :8090    │ │  :8080        │
  └───────────────┘ └──────────┘ └───────────────┘
         │                                │
  ┌──────▼─────┐                  ┌───────▼────┐
  │ PostgreSQL │                  │  GGUF       │
  │ + Redis    │                  │  Models     │
  └────────────┘                  │  (~/.hermes/│
                                  │   models/)  │
                                  └────────────┘
```

---

## Current API Contract (hermes-api sidecar)

Base URL: `http://127.0.0.1:8000`

| Method | Endpoint                        | Description              |
|--------|---------------------------------|--------------------------|
| GET    | /health                         | Liveness check           |
| GET    | /api/dashboard/overview         | Stats summary            |
| GET    | /api/repos                      | Repository list          |
| GET    | /api/runs                       | Execution runs           |
| GET    | /api/approvals                  | Pending approvals        |
| GET    | /api/coding-sessions            | Active coding sessions   |
| GET    | /api/browser-runs               | Browser automations      |
| GET    | /api/prd-batches                | PRD batches              |
| GET    | /api/providers                  | LLM provider profiles    |
| GET    | /api/subagents                  | Sub-agent instances      |
| GET    | /api/appwrite-projects          | Appwrite projects        |

## LLM Sidecar API (llama-server)

Base URL: `http://127.0.0.1:8080` — OpenAI-compatible

| Method | Endpoint                        | Description              |
|--------|---------------------------------|--------------------------|
| GET    | /v1/models                      | List loaded models       |
| POST   | /v1/chat/completions            | Chat (stream=true OK)    |
| POST   | /v1/completions                 | Text completion          |
| POST   | /v1/embeddings                  | Embeddings               |
| GET    | /health                         | Server health            |

## Voice Sidecar API (whisper-server)

Base URL: `http://127.0.0.1:8090`

| Method | Endpoint                        | Description              |
|--------|---------------------------------|--------------------------|
| GET    | /models                         | List available models    |
| POST   | /inference                      | Transcribe audio file    |
| WS     | /audio/stream                   | Real-time streaming STT  |

---

## Dashboard UX Rules

1. **Background**: #0a0a0a — near-black, no pure black
2. **Surface**: #111111 borders #222222 — 1px solid
3. **Text primary**: #f0f0f0 — Text secondary: #888888
4. **Accent**: #00e5ff (teal/cyan) — used sparingly for CTAs and active states
5. **Border-radius**: 4px maximum — no pill buttons, no rounded excess
6. **Typography**: System-UI / Inter, 14px base, monospace for code/IDs
7. **No gradients** on interactive elements — flat surfaces only
8. **No glassmorphism** — no backdrop-filter blur
9. **No eyebrow labels** — no ALLCAPS decorative above headings
10. **Streaming text**: append tokens; no re-render flicker
11. **Spacing**: 4px grid — use multiples of 4

---

## Policy & Risk Rules

Located in: `nightshift/packages/shared/policies/`

| Policy File          | Governs                          |
|----------------------|----------------------------------|
| action_policy.json   | Command risk scoring (rm/drop = CRITICAL) |
| browser_policy.json  | Domain allowlist, concurrent sessions |
| subagent_policy.json | Tool restrictions, budget limits |

---

## Coding Standards

- **Rust**: `rustfmt` (default settings), Clippy `#[deny(clippy::pedantic)]` optional
- **TypeScript**: strict, no `any`, ESLint + Prettier
- **Python**: `ruff` linter, `black` formatter, type annotations required
- **Secrets**: Never committed, always loaded from `~/.hermes/.env` or system keyring
- **OWASP**: Input validation at all system boundaries, no SQL concatenation, parameterized queries only
- **Logging**: No sensitive data in logs; mask API keys, tokens, passwords
- **Tests**: Required for all new crates; integration tests for API changes

---

## White-Label Config

`hermes-foundry/branding.json` controls:
- App name, window title, tray icon tooltip
- Color palette overrides
- Default model
- Support URL
- Marketing copy

Never hardcode `"Hermes"` in UI strings — always read from branding config.
