# Agent MAXX — Master Coding Agent Harness

> This file is the canonical reference for any AI coding assistant (Claude, Copilot, Cursor, etc.) working in the `nightshift/` codebase. Read it before making any change.

---

## What This Is

**Agent MAXX Night Shift** is an autonomous VPS-hosted coding platform.  
It runs 24/7, clones repos, generates PRDs, ships code, and automates browser tasks — while the operator sleeps.

Brand: Agent MAXX. Personality: James Bond's dog. Teal/dark design system.  
Deployed via Docker Compose on Hostinger behind Caddy.

---

## Workspace Layout

```
nightshift/
├── apps/
│   ├── hermes-api/          # FastAPI control plane — THE core
│   │   └── app/
│   │       ├── main.py          # All routes mounted here
│   │       ├── config.py        # Settings from env
│   │       ├── db.py            # Async SQLAlchemy engine
│   │       ├── auth.py          # X-API-Key middleware
│   │       ├── models/__init__.py   # ALL 14 SQLAlchemy models
│   │       ├── schemas/__init__.py  # ALL Pydantic v2 schemas
│   │       └── routes/          # 11 route files (inline logic, no services)
│   ├── provider-router/     # Model-agnostic LLM proxy (OpenAI/Gemini/Anthropic)
│   ├── worker-coder/        # Coding session runner (clone→code→test→commit)
│   ├── worker-browser/      # Playwright browser automation (vision support)
│   ├── notifier/            # Telegram alerts + approval watcher
│   ├── scheduler/           # Nightly cron (scan→PRD→digest)
│   └── hermes-dashboard/    # Landing page + 12-panel cockpit
│       └── public/
│           ├── index.html   # Operator cockpit (auto-refreshes every 15s)
│           └── landing.html # 3D Agent MAXX marketing page
├── migrations/
│   └── 001_initial.sql      # 15 tables + provider profile seeds
├── infra/
│   ├── caddy/Caddyfile      # Reverse proxy config
│   └── scripts/             # bootstrap.sh, smoke-test.sh, backup.sh
├── docker-compose.yml       # All 9 services — DO NOT restructure
├── .env.example             # Copy to .env and fill API keys
├── Makefile                 # make dev / make logs / make db-shell
└── README.md
```

---

## Critical Rules

### ① Don't break Docker Compose
`docker-compose.yml` is the source of truth for service topology. Do not rename services, change network names, or alter volume paths. Services communicate via hostname (`hermes-api`, `redis`, `postgres`, etc.).

### ② Models-first discipline  
All database changes: 
1. Add/modify column in `apps/hermes-api/app/models/__init__.py`
2. Add corresponding migration SQL to `migrations/001_initial.sql` or a new `00X_*.sql` file
3. Update matching Pydantic schema in `apps/hermes-api/app/schemas/__init__.py`
4. Update route that serves/accepts the field

Never add a DB column in one place without updating all three.

### ③ Routes are self-contained
Routes in `apps/hermes-api/app/routes/` do DB/Redis work inline. There is no `services/` layer. Keep it that way unless a function is called from 3+ routes.

### ④ All handlers return JSON-serializable dicts
FastAPI auto-serializes. Don't return SQLAlchemy model objects directly — return dicts or call `.model_dump()`.

### ⑤ Auth is opt-in in dev
If `API_KEY` env var is empty, `auth.py` allows all requests. In prod, every route that mutates state should use `Depends(verify_api_key)`.

### ⑥ Prompt caching discipline (for LLM calls made via provider-router)
Don't rebuild system prompts between steps in the same session. Pass `provider_profile_id` consistently.

### ⑦ Brand is AGENT MAXX (not Hermes Night Shift)
Any string "Hermes" in user-facing text, HTML titles, API responses, or documentation should be "Agent MAXX". Internal Python class names (`CodingSession`, `BrowserRun`, etc.) are fine to keep as-is.

---

## Database Entities

| Table | Model Class | Key Fields |
|-------|-------------|------------|
| repos | Repo | repo_url, name, source, last_scanned_at |
| repo_profiles | RepoProfile | language, framework, has_tests, stars |
| prd_batches | PRDBatch | status, strategy, prd_count |
| prds | PRD | title, body, priority, status |
| runs | Run | run_type, status, model, iteration_count |
| run_steps | RunStep | step_index, tool_name, tool_output, duration_ms |
| approvals | Approval | risk_level, proposed_command, decision |
| coding_sessions | CodingSession | repo_url, prompt, task_type, status, branch |
| coding_session_steps | CodingSessionStep | step_name, status, output |
| browser_runs | BrowserRun | start_url, state, steps (JSONB), artifact_count |
| browser_artifacts | BrowserArtifact | kind, label, content, file_path |
| provider_profiles | ProviderProfile | id (TEXT PK), provider, model, purpose |
| sub_agents | SubAgent | repo_scope[], tool_scope[], time_budget_minutes, status |
| appwrite_projects | AppwriteProject | name, status, endpoint, project_id |
| artifacts | Artifact | parent_type, parent_id, kind, content, file_path |

---

## API Surface (11 route files)

| Prefix | File | Key endpoints |
|--------|------|---------------|
| /api/healthz | health.py | GET /healthz, /readyz, /version |
| /api/dashboard | dashboard.py | GET /overview, /missing-secrets |
| /api/repos | repos.py | GET /repos, POST /scan-trigger |
| /api/prd-batches | prds.py | GET/POST /prd-batches, POST /approve |
| /api/runs | runs.py | GET /runs, GET /{id}/steps |
| /api/approvals | approvals.py | GET /approvals, POST /{id}/decide |
| /api/coding-sessions | coding_sessions.py | GET/POST /coding-sessions, POST /{id}/cancel |
| /api/browser-runs | browser_runs.py | GET/POST /browser-runs, POST /{id}/pause, /resume, /cancel |
| /api/providers | providers.py | GET /providers |
| /api/subagents | subagents.py | GET/POST /subagents |
| /api/appwrite-projects | appwrite.py | GET/POST /appwrite-projects |
| /mcp | (auto via fastapi_mcp) | All routes as MCP tools |

---

## Provider Profiles (LLM Routing)

Call `POST http://provider-router:8080/v1/llm/chat` with:

```json
{
  "provider_profile_id": "balanced",
  "messages": [{"role": "user", "content": "..."}],
  "temperature": 0.7,
  "max_tokens": 4096
}
```

Profiles: `balanced`, `premium_reasoning`, `low_cost_coding`, `research_browser`, `gemini_flash`, `openai_gpt4`, `openrouter_balanced`

---

## Queue Channels (Redis)

Workers consume from:
- `queue:coding_sessions` — CodingSession job payloads
- `queue:browser_runs` — BrowserRun job payloads  
- `queue:repo_scan` — Repo scan triggers

Pub/sub channels:
- `events:runs` — run status events
- `events:approvals` — approval requests (notifier listens)
- `browser:interventions` — CAPTCHA/MFA gate events
- `events:digest` — nightly digest trigger

---

## Secrets Required

| Env Var | Required | Purpose |
|---------|----------|---------|
| API_KEY | Soft | Dashboard + API auth |
| DATABASE_URL | Hard | PostgreSQL |
| ANTHROPIC_API_KEY | Soft | Claude models |
| OPENAI_API_KEY | Optional | GPT models |
| GOOGLE_API_KEY | Optional | Gemini vision |
| OPENROUTER_API_KEY | Optional | Multi-model fallback |
| GITHUB_TOKEN | Soft | Repo scanning |
| TELEGRAM_BOT_TOKEN | Optional | Alerts |
| TELEGRAM_CHAT_ID | Optional | Alerts |
| APPWRITE_ENDPOINT | Optional | Appwrite provisioning |

---

## Adding a New Route

1. Create `apps/hermes-api/app/routes/your_thing.py`
2. Mount it in `apps/hermes-api/app/main.py`:
   ```python
   from app.routes.your_thing import router as your_thing_router
   app.include_router(your_thing_router, prefix="/api")
   ```
3. Add necessary model + schema + migration if touching DB.

---

## Vision / Test-Taking Mode

Worker-browser supports a `vision_analyze` step type for online test research. When a step has `action: vision_analyze`:
1. Takes a full-page screenshot
2. Sends to vision LLM (Gemini Flash or GPT-4V via provider-router `research_browser` profile)  
3. Parses question/answer structure from the response
4. Saves as `browser_artifact` with `kind=vision_analysis`

Enable: send browser run with `{"steps": [{"action": "vision_analyze", "prompt": "..."}]}`

---

## Common Tasks

**Run the stack locally:**
```bash
cd nightshift && make dev
```

**See what's running:**
```bash
make ps && make logs
```

**Reset everything (nuclear):**
```bash
make clean && make dev
```

**Connect VS Code / Cursor to MCP:**
Add `http://localhost:8642/mcp` as MCP server — all API routes become tools.
