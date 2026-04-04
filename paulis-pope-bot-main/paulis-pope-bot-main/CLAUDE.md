# thepopebot - AI Agent NPM Package

This document explains the thepopebot codebase for AI assistants working on this package.

## What is thepopebot?

thepopebot is an **NPM package** for creating custom autonomous AI agents. Users install it via `npx thepopebot init`, which scaffolds a Next.js project. It features a two-layer architecture: a Next.js Event Handler for orchestration (webhooks, Telegram chat, cron scheduling) and a Docker Agent for autonomous task execution via the Pi coding agent.

## Package vs. Templates — Where Code Goes

All event handler logic, API routes, library code, and core functionality lives in the **npm package** (`api/`, `lib/`, `config/`, `bin/`). This is what users import when they `import ... from 'thepopebot/...'`.

The `templates/` directory contains **only files that get scaffolded into user projects** via `npx thepopebot init`. Templates are for user-editable configuration and thin wiring — things users are expected to customize or override. Never add core logic to templates.

**When adding or modifying event handler code, always put it in the package itself (e.g., `api/`, `lib/`), not in `templates/`.** Templates should only contain:
- Configuration files users edit (`config/SOUL.md`, `config/CRONS.json`, etc.)
- Thin Next.js wiring (`next.config.mjs`, `instrumentation.js`, catch-all route)
- GitHub Actions workflows
- Docker files
- CLAUDE.md files for AI assistant context in user projects

## Two-Layer Architecture

```
┌───────────────────────────────────────────────────────────────────────┐
│                                                                       │
│  ┌─────────────────┐         ┌─────────────────┐                     │
│  │  Event Handler  │ ──1──►  │     GitHub      │                     │
│  │  (creates job)  │         │ (job/* branch)  │                     │
│  └────────▲────────┘         └────────┬────────┘                     │
│           │                           │                              │
│           │                           2 (triggers run-job.yml)       │
│           │                           │                              │
│           │                           ▼                              │
│           │                  ┌─────────────────┐                     │
│           │                  │  Docker Agent   │                     │
│           │                  │  (runs Pi, PRs) │                     │
│           │                  └────────┬────────┘                     │
│           │                           │                              │
│           │                           3 (creates PR)                 │
│           │                           │                              │
│           │                           ▼                              │
│           │                  ┌─────────────────┐                     │
│           │                  │     GitHub      │                     │
│           │                  │   (PR opened)   │                     │
│           │                  └────────┬────────┘                     │
│           │                           │                              │
│           │                           4a (rebuild-event-handler.yml) │
│           │                           4b (auto-merge.yml)            │
│           │                           │                              │
│           5 (Telegram notification)   │                              │
│           └───────────────────────────┘                              │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘
```

## Directory Structure

```
/
├── api/                        # Next.js route handlers (exported as thepopebot/api)
│   └── index.js                # GET/POST handlers for all /api/* routes
├── lib/                        # Core implementation
│   ├── actions.js              # Shared action executor (agent, command, webhook)
│   ├── cron.js                 # Cron scheduler (loads CRONS.json)
│   ├── triggers.js             # Webhook trigger middleware (loads TRIGGERS.json)
│   ├── paths.js                # Central path resolver (resolves from user's project root)
│   ├── ai/                     # LLM integration (chat, streaming, agent, model, tools)
│   ├── auth/                   # NextAuth config, helpers, middleware, server actions
│   ├── channels/               # Channel adapters (base class, Telegram, factory)
│   ├── chat/                   # Chat route handler, server actions, React UI components
│   ├── db/                     # SQLite via Drizzle (schema, users, chats, api-keys, notifications)
│   ├── tools/                  # Job creation, GitHub API, Telegram, OpenAI Whisper
│   └── utils/
│       └── render-md.js        # Markdown {{include}} processor
├── config/
│   ├── index.js                # withThepopebot() Next.js config wrapper
│   └── instrumentation.js      # Server startup hook (loads .env, starts crons)
├── bin/
│   └── cli.js                  # CLI: init, setup, setup-telegram, reset, diff
├── setup/                      # Interactive setup wizard
├── templates/                  # Files scaffolded to user projects by `thepopebot init`
│   ├── app/                    # Next.js app (pages, API routes, components)
│   ├── .github/workflows/      # GitHub Actions (auto-merge, build-image, deploy, run-job, notify)
│   ├── docker/                 # Docker files for job and event-handler containers
│   ├── pi-skills/              # Git submodule — Pi agent skills (brave-search, browser-tools, etc.)
│   └── config/                 # Agent config (SOUL, EVENT_HANDLER, CRONS, TRIGGERS, etc.)
├── docs/                       # Extended documentation
└── package.json                # NPM package definition
```

## Key Files

| File | Purpose |
|------|---------|
| `api/index.js` | Next.js GET/POST route handlers for all `/api/*` endpoints |
| `lib/paths.js` | Central path resolver — all paths resolve from user's `process.cwd()` |
| `lib/actions.js` | Shared action dispatcher for agent/command/webhook actions |
| `lib/cron.js` | Cron scheduler — loads `config/CRONS.json` at server start |
| `lib/triggers.js` | Trigger middleware — loads `config/TRIGGERS.json` |
| `lib/utils/render-md.js` | Markdown include and variable processor (`{{filepath}}`, `{{datetime}}`, `{{skills}}`) |
| `config/index.js` | `withThepopebot()` Next.js config wrapper |
| `config/instrumentation.js` | `register()` server startup hook (loads .env, validates AUTH_SECRET, initializes database, starts crons) |
| `bin/cli.js` | CLI entry point (`thepopebot init`, `setup`, `reset`, `diff`, `set-agent-secret`, `set-agent-llm-secret`, `set-var`) |
| `lib/ai/index.js` | Chat, streaming, and job summary functions |
| `lib/ai/agent.js` | LangGraph agent with SQLite checkpointing and tool use |
| `lib/channels/base.js` | Channel adapter base class (normalize messages across platforms) |
| `lib/db/index.js` | Database initialization — SQLite via Drizzle ORM |
| `lib/db/api-keys.js` | Database-backed API key management (SHA-256 hashed, timing-safe verify) |
| `templates/docker/job/Dockerfile` | Builds the job agent container (Node.js 22, Puppeteer, Pi) — scaffolded to user projects |
| `templates/docker/job/entrypoint.sh` | Container startup — clones repo, runs agent, commits results — scaffolded to user projects |

## NPM Package Exports

| Import | Module | Purpose |
|--------|--------|---------|
| `thepopebot/api` | `api/index.js` | `GET` and `POST` route handlers — re-exported by the user's catch-all route |
| `thepopebot/config` | `config/index.js` | `withThepopebot()` — wraps the user's Next.js config to mark server-only packages as external |
| `thepopebot/instrumentation` | `config/instrumentation.js` | `register()` — Next.js instrumentation hook that loads `.env` and starts cron jobs on server start |
| `thepopebot/auth` | `lib/auth/index.js` | Auth helpers (`auth()`, `getPageAuthState()`) |
| `thepopebot/auth/actions` | `lib/auth/actions.js` | Server action for admin setup (`setupAdmin()`) |
| `thepopebot/chat` | `lib/chat/components/index.js` | Chat UI components |
| `thepopebot/chat/actions` | `lib/chat/actions.js` | Server actions for chats, notifications, and swarm |
| `thepopebot/chat/api` | `lib/chat/api.js` | Dedicated chat streaming route handler (session auth) |
| `thepopebot/db` | `lib/db/index.js` | Database access |
| `thepopebot/middleware` | `lib/auth/middleware.js` | Auth middleware |

### Column Naming Convention

Drizzle schema uses camelCase JS property names mapped to snake_case SQL columns.
Example: `createdAt: integer('created_at')` — use `createdAt` in JS, SQL column is `created_at`.

## CLI Commands

| Command | Description |
|---------|-------------|
| `thepopebot init` | Scaffold a new project — copies templates, creates `package.json`, runs `npm install` |
| `thepopebot setup` | Run interactive setup wizard (API keys, GitHub secrets, Telegram bot) |
| `thepopebot setup-telegram` | Reconfigure Telegram webhook only |
| `thepopebot reset [file]` | Restore a template file to package default (or list all available templates) |
| `thepopebot diff [file]` | Show differences between project files and package templates |
| `thepopebot reset-auth` | Regenerate AUTH_SECRET (invalidates all sessions) |
| `thepopebot set-agent-secret <KEY> [VALUE]` | Set a GitHub secret with `AGENT_` prefix and update `.env` |
| `thepopebot set-agent-llm-secret <KEY> [VALUE]` | Set a GitHub secret with `AGENT_LLM_` prefix |
| `thepopebot set-var <KEY> [VALUE]` | Set a GitHub repository variable |

## How User Projects Work

When a user runs `npx thepopebot init`, the CLI scaffolds a Next.js project that wires into the package:

1. **`next.config.mjs`** imports `withThepopebot` from `thepopebot/config` — marks server-only dependencies as external so they aren't bundled for the client
2. **`instrumentation.js`** re-exports `register` from `thepopebot/instrumentation` — Next.js calls this on server start to load `.env`, validate AUTH_SECRET, initialize the database, and start cron jobs
3. **`app/api/[...thepopebot]/route.js`** re-exports `GET` and `POST` from `thepopebot/api` — catch-all route that handles all `/api/*` requests
4. **`middleware.js`** re-exports auth middleware from `thepopebot/middleware` — protects all routes except `/login` and `/api`
5. **`app/api/auth/[...nextauth]/route.js`** re-exports from `thepopebot/auth` — handles NextAuth login/session routes

The user's project contains only configuration files (`config/`, `.env`, `.github/workflows/`) and the thin Next.js wiring. All core logic lives in the npm package.

## Web Interface

thepopebot includes a full web interface for managing the agent, accessible after login at `APP_URL`.

### Pages

| Page | Route | Purpose |
|------|-------|---------|
| Chat | `/` | AI chat with streaming responses, file uploads (images, PDFs, text) |
| Chat History | `/chats` | Browse past conversations grouped by date |
| Individual Chat | `/chat/[chatId]` | Resume a specific conversation |
| Crons | `/settings/crons` | View scheduled jobs from CRONS.json |
| Triggers | `/settings/triggers` | View webhook triggers from TRIGGERS.json |
| Swarm | `/swarm` | Monitor active/completed agent jobs with cancel/rerun controls |
| Notifications | `/notifications` | Job completion alerts with unread badges |
| Settings | `/settings/secrets` | Generate and manage API keys |
| Login | `/login` | Authentication (first visit shows admin setup form) |

### Architecture

Page components live in the package (`lib/chat/components/`) and are exported via `thepopebot/chat`. Template pages in `templates/app/` are thin wrappers that import these components. The UI uses Tailwind CSS with CSS variables for light/dark theming.

Server actions in `lib/chat/actions.js` handle all browser-to-server mutations (chat CRUD, notifications, API keys, swarm control) using the `requireAuth()` session pattern.

## Authentication

The web interface uses NextAuth v5 with a Credentials provider (email/password). Sessions use JWT stored in httpOnly cookies.

- **First-time setup**: If no users exist, `/login` shows a setup form to create the admin account
- **`requireAuth()`**: All server actions validate the session before executing
- **API routes**: Use `x-api-key` header with database-backed keys (not session auth)
- **`AUTH_SECRET`**: Required env var for session encryption, auto-generated by setup wizard

## Database

thepopebot uses SQLite (via Drizzle ORM) for all persistence. The database is stored at `data/thepopebot.sqlite` (override with `DATABASE_PATH` env var) and is initialized automatically on first server start.

### Tables

| Table | Purpose |
|-------|---------|
| `users` | Admin accounts (email, bcrypt password hash, role) |
| `chats` | Chat sessions (user_id, title, timestamps) |
| `messages` | Chat messages (chat_id, role, content) |
| `notifications` | Job completion notifications |
| `subscriptions` | Channel subscriptions |
| `settings` | Key-value configuration store (also stores API keys) |

## Database Migrations

This project uses Drizzle Kit for database schema management. **All schema changes MUST go through the migration workflow.**

### Rules
- **NEVER** write raw `CREATE TABLE`, `ALTER TABLE`, or any DDL SQL manually
- **NEVER** modify `initDatabase()` to add schema changes
- **ALWAYS** make schema changes by editing `lib/db/schema.js` then running `npm run db:generate`

### Workflow
1. Edit `lib/db/schema.js` (add/modify tables or columns using Drizzle's schema builders)
2. Run `npm run db:generate` to create a versioned migration file in `drizzle/`
3. Review the generated SQL in `drizzle/XXXX_*.sql`
4. Commit both the schema change and the migration file
5. Migrations auto-apply on server startup via `migrate()` in `initDatabase()`

### Key Files
- `lib/db/schema.js` — Single source of truth for the database schema
- `drizzle/` — Generated migration SQL files (committed to repo, shipped with package)
- `drizzle.config.js` — Drizzle Kit config (used by `db:generate`)
- `lib/db/index.js` — `initDatabase()` calls `migrate()` to apply pending migrations

## Event Handler Layer

The Event Handler is a Next.js API route handler (`api/index.js`) that provides orchestration capabilities:

### Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/create-job` | POST | Generic webhook for job creation (requires x-api-key header) |
| `/api/telegram/webhook` | POST | Telegram bot webhook for conversational interface |
| `/api/telegram/register` | POST | Register Telegram webhook URL |
| `/api/github/webhook` | POST | Receives notifications from GitHub Actions (notify-pr-complete.yml, notify-job-failed.yml) |
| `/api/jobs/status` | GET | Check status of a running job |
| `/api/ping` | GET | Health check (public — no API key required) |

### Security: /api vs Server Actions

**`/api` routes are for external callers only.** They authenticate via `x-api-key` header (database-backed API keys managed through the admin UI) or their own webhook secrets (Telegram, GitHub). Never add session/cookie auth to `/api` routes.

**Browser UI uses Server Actions.** All authenticated browser-to-server calls (data fetching, mutations) MUST use Next.js Server Actions (`'use server'` functions in `lib/chat/actions.js` or `lib/auth/actions.js`), not `/api` fetch calls. Server Actions use the `requireAuth()` pattern which validates the session cookie internally.

**The one exception is chat streaming.** The AI SDK's `DefaultChatTransport` requires an HTTP endpoint for streaming responses. Chat has its own dedicated route handler at `lib/chat/api.js` (mapped to `/stream/chat` via `templates/app/stream/chat/route.js`) with its own session auth. This lives outside `/api` entirely, so it doesn't hit the catch-all `api/index.js`.

| Caller | Mechanism | Auth | Location |
|--------|-----------|------|----------|
| External (cURL, GitHub Actions, Telegram) | `/api` route handler | `x-api-key` header or webhook secret | `api/index.js` |
| Browser UI (data/mutations) | Server Action | `requireAuth()` session check | `lib/chat/actions.js`, `lib/auth/actions.js` |
| Browser UI (chat streaming) | Dedicated route handler | `auth()` session check | `lib/chat/api.js` |

### Components

- **`api/index.js`** — Next.js route handlers (GET/POST) with auth and trigger middleware
- **`lib/cron.js`** — Loads CRONS.json and schedules jobs using node-cron
- **`lib/triggers.js`** — Loads TRIGGERS.json and fires actions when watched paths are hit
- **`lib/ai/`** — LangGraph agent with multi-provider LLM support and tool use
- **`lib/channels/`** — Channel adapter pattern for Telegram (and future channels)
- **`lib/tools/`** — Job creation, GitHub API, Telegram, and OpenAI utilities

### Action Types: `agent`, `command`, and `webhook`

Both cron jobs and webhook triggers use the same shared dispatch system (`lib/actions.js`). Every action has a `type` field — `"agent"` (default), `"command"`, or `"webhook"`.

| | `agent` | `command` | `webhook` |
|---|---------|-----------|--------|
| **Uses LLM** | Yes — spins up Pi in a Docker container | No — runs a shell command directly | No — makes an HTTP request |
| **Thinking** | Can reason, make decisions, write code | No thinking, just executes | No thinking, just sends a request |
| **Runtime** | Minutes to hours (full agent lifecycle) | Milliseconds to seconds | Milliseconds to seconds |
| **Cost** | LLM API calls + GitHub Actions minutes | Free (runs on event handler) | Free (runs on event handler) |

If the task needs to *think*, use `agent`. If it just needs to *do*, use `command`. If it needs to *call an external service*, use `webhook`.

#### Type: `agent` (default)

Creates a full Docker Agent job via `createJob()`. This pushes a `job/*` branch to GitHub, which triggers `run-job.yml` to spin up the Docker container with Pi. The `job` string is passed directly as-is to the LLM as its task prompt (written to `logs/<JOB_ID>/job.md` on the job branch).

#### Type: `command`

Runs a shell command directly on the event handler server. No Docker container, no GitHub branch, no LLM. Each system has its own working directory for scripts (in the user's project root):
- **Crons**: `cron/`
- **Triggers**: `triggers/`

#### Type: `webhook`

Makes an HTTP request to an external URL. No Docker container, no LLM. Useful for forwarding webhooks, calling external APIs, or pinging health endpoints. `GET` requests skip the body; `POST` (default) sends `{ ...vars }` if no incoming data, or `{ ...vars, data: <incoming payload> }` when triggered by a webhook.

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `url` | yes | — | Target URL |
| `method` | no | `"POST"` | `"GET"` or `"POST"` |
| `headers` | no | `{}` | Outgoing request headers |
| `vars` | no | `{}` | Extra key/value pairs merged into outgoing body |

### Cron Jobs

Defined in `config/CRONS.json`, loaded by `lib/cron.js` at server startup via `node-cron`. Each entry has `name`, `schedule` (cron expression), `type` (`agent`/`command`/`webhook`), and the corresponding action fields (`job`, `command`, or `url`/`method`/`headers`/`vars`). Set `enabled: false` to disable.

### Webhook Triggers

Defined in `config/TRIGGERS.json`, loaded by `lib/triggers.js`. Each trigger watches an endpoint path (`watch_path`) and fires an array of actions (fire-and-forget, after auth, before route handler). Actions use the same `type`/`job`/`command`/`url` fields as cron jobs. Action `job` and `command` strings support template tokens: `{{body}}`, `{{body.field}}`, `{{query}}`, `{{query.field}}`, `{{headers}}`, `{{headers.field}}`.

### Environment Variables (Event Handler)

| Variable | Description | Required |
|----------|-------------|----------|
| `APP_URL` | Public URL for webhooks, Telegram, and Traefik hostname | Yes |
| `AUTH_SECRET` | Secret for NextAuth session encryption (auto-generated by setup) | Yes |
| `GH_TOKEN` | GitHub PAT for creating branches/files | Yes |
| `GH_OWNER` | GitHub repository owner | Yes |
| `GH_REPO` | GitHub repository name | Yes |
| `TELEGRAM_BOT_TOKEN` | Telegram bot token from BotFather | For Telegram |
| `TELEGRAM_WEBHOOK_SECRET` | Secret for validating Telegram webhooks | No |
| `TELEGRAM_VERIFICATION` | Verification code for getting chat ID | For Telegram setup |
| `TELEGRAM_CHAT_ID` | Default Telegram chat ID for notifications | For Telegram |
| `GH_WEBHOOK_SECRET` | Secret for GitHub Actions webhook auth | For notifications |
| `LLM_PROVIDER` | LLM provider: `anthropic`, `openai`, `google`, or `custom` (default: `anthropic`). `custom` uses OpenAI-compatible endpoints (e.g., Ollama) | No |
| `LLM_MODEL` | LLM model name override (provider-specific default if unset) | No |
| `LLM_MAX_TOKENS` | Max tokens override for LLM responses (default: 4096) | No |
| `ANTHROPIC_API_KEY` | API key for Anthropic provider | For anthropic provider |
| `OPENAI_API_KEY` | API key for OpenAI provider / Whisper voice transcription | For openai provider or voice |
| `OPENAI_BASE_URL` | Custom OpenAI-compatible base URL (e.g., `http://localhost:11434/v1` for Ollama) | For custom provider |
| `GOOGLE_API_KEY` | API key for Google provider | For google provider |
| `CUSTOM_API_KEY` | API key for custom OpenAI-compatible provider | For custom provider |
| `AUTH_TRUST_HOST` | Trust host header behind reverse proxy (set `true` for Docker/Traefik) | For reverse proxy |
| `DATABASE_PATH` | Override SQLite database location (default: `data/thepopebot.sqlite`) | No |

## Docker Agent Layer

The Dockerfile (`templates/docker/job/Dockerfile`, scaffolded to `docker/job/Dockerfile` in user projects) creates a container with:
- **Node.js 22** (Bookworm slim)
- **Pi coding agent** (`@mariozechner/pi-coding-agent`)
- **Chrome/Chromium dependencies** (shared libs for headless browser; actual Chrome binary installed at runtime by browser-tools skill via Puppeteer)
- **Git + GitHub CLI** (for repository operations)

Pi skills (brave-search, browser-tools, etc.) are **not baked into the Docker image**. They live in the user's repo under `pi-skills/` and are symlinked into `.pi/skills/`. The entrypoint runs `npm install` for each symlinked skill at container startup to compile native dependencies for Linux.

### Runtime Flow (entrypoint.sh)

Key phases: export secrets as env vars → clone repo branch → install skill dependencies → optionally start headless Chrome → build SYSTEM.md from `config/SOUL.md` + `config/AGENT.md` → run Pi with job.md prompt → save session log to `logs/{JOB_ID}/` → commit and create PR.

### Environment Variables (Docker Agent)

| Variable | Description | Required |
|----------|-------------|----------|
| `REPO_URL` | Git repository URL to clone | Yes |
| `BRANCH` | Branch to clone and work on (e.g., job/uuid) | Yes |
| `SECRETS` | JSON with protected credentials (GH_TOKEN, ANTHROPIC_API_KEY, etc.) — built at runtime from `AGENT_*` GitHub secrets, filtered from LLM | Yes |
| `LLM_SECRETS` | JSON with credentials the LLM can access (browser logins, skill API keys) — built at runtime from `AGENT_LLM_*` GitHub secrets | No |
| `LLM_PROVIDER` | LLM provider for the Pi agent (`anthropic`, `openai`, `google`) | No (default: `anthropic`) |
| `LLM_MODEL` | LLM model name for the Pi agent | No (provider default) |

## Deployment

See [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) for full production deployment details (Docker Compose, Traefik, volume mounts, event handler image, self-hosted runner).

## GitHub Actions

Workflows are scaffolded from `templates/.github/workflows/` into user projects. Read the workflow files directly for implementation details.

- **rebuild-event-handler.yml** — Triggers on push to `main`. Fast path (build + PM2 reload) or Docker restart path if the thepopebot package version changed.
- **upgrade-event-handler.yml** — Manual `workflow_dispatch`. Creates a PR to upgrade the thepopebot package in an isolated clone.
- **build-image.yml** — Builds and pushes the job Docker image to GHCR when `docker/job/**` changes (only if `JOB_IMAGE_URL` is a `ghcr.io/` URL).
- **run-job.yml** — Triggers on `job/*` branch creation. Runs the Docker agent container.
- **auto-merge.yml** — Squash-merges job PRs if `AUTO_MERGE` is not `"false"` and all changed files fall within `ALLOWED_PATHS` prefixes (default: `/logs`).
- **notify-pr-complete.yml** — Fires after `auto-merge.yml`. Gathers job data and sends notification payload to event handler.
- **notify-job-failed.yml** — Fires when `run-job.yml` fails. Sends failure notification to event handler.

### GitHub Secrets Required

Individual GitHub secrets use a prefix-based naming convention:

| Prefix | Purpose | Filtered from LLM? | Example |
|--------|---------|---------------------|---------|
| `AGENT_` | Protected secrets for the Docker agent container | Yes (env-sanitizer) | `AGENT_GH_TOKEN`, `AGENT_ANTHROPIC_API_KEY` |
| `AGENT_LLM_` | LLM-accessible secrets for the Docker agent container | No | `AGENT_LLM_BRAVE_API_KEY` |
| *(none)* | Workflow-only secrets (never passed to container) | N/A | `GH_WEBHOOK_SECRET` |

### GitHub Repository Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `APP_URL` | Public URL for the event handler (e.g., `https://mybot.example.com`) | — |
| `AUTO_MERGE` | Set to `false` to disable auto-merge of job PRs | Enabled (any value except `false`) |
| `ALLOWED_PATHS` | Comma-separated path prefixes (e.g., `/logs`). Use `/` for all paths. | `/logs` |
| `JOB_IMAGE_URL` | Full Docker image path for the job agent (e.g., `ghcr.io/myorg/mybot`). GHCR URLs trigger automatic builds via `build-image.yml`. Non-GHCR URLs (e.g., `docker.io/user/mybot`) are pulled directly. | Not set (uses `stephengpope/thepopebot:job-${THEPOPEBOT_VERSION}`) |
| `EVENT_HANDLER_IMAGE_URL` | Docker image path for the event handler | Not set (uses `stephengpope/thepopebot-event-handler:latest`) |
| `RUNS_ON` | GitHub Actions runner label (e.g., `self-hosted` for docker-compose runner) | `ubuntu-latest` |
| `LLM_PROVIDER` | LLM provider for the Pi agent (`anthropic`, `openai`, `google`) | Not set (default: `anthropic`) |
| `LLM_MODEL` | LLM model name for the Pi agent (e.g., `claude-sonnet-4-5-20250929`) | Not set (provider default) |

## How Credentials Work

GitHub secrets with `AGENT_*` and `AGENT_LLM_*` prefixes are collected by `run-job.yml` into `SECRETS` and `LLM_SECRETS` JSON objects (prefix stripped). The container exports these as env vars. `AGENT_*` secrets are filtered from the LLM's bash via `env-sanitizer`; `AGENT_LLM_*` secrets are not filtered.

## Customization Points

Users create their agent project with `npx thepopebot init` then `npm run setup`. The setup wizard handles API keys, GitHub secrets/variables, and Telegram bot configuration. Users customize their agent by editing:

- **config/SOUL.md** — Personality, identity, and values (who the agent is)
- **config/EVENT_HANDLER.md** — Event handler system prompt
- **config/JOB_SUMMARY.md** — Prompt for summarizing completed jobs
- **config/HEARTBEAT.md** — Self-monitoring behavior
- **config/AGENT.md** — Agent runtime environment
- **config/PI_SKILL_GUIDE.md** — Guide for Pi agent skills (referenced by agent tools)
- **config/CRONS.json** — Scheduled job definitions
- **config/TRIGGERS.json** — Webhook trigger definitions
- **pi-skills/** — All available Pi agent skills (scaffolded from package submodule)
- **.pi/skills/** — Symlinks to active skills (e.g., `.pi/skills/brave-search` → `../../pi-skills/brave-search`)
- **cron/** and **triggers/** — Shell scripts for command-type actions

## Session Logs

Each job gets its own directory at `logs/{JOB_ID}/` containing both the job description (`job.md`) and session logs (`.jsonl`). This directory can be used to resume sessions for follow-up tasks via the `--session-dir` flag.

## Markdown File Includes

Markdown files in `config/` support includes and built-in variables, powered by `lib/utils/render-md.js`.

**File includes**: `{{ filepath.md }}` — resolves relative to project root, recursive with circular detection. Missing files are left as-is.

**Built-in variables**:
- `{{datetime}}` — Current ISO timestamp
- `{{skills}}` — Bullet list of active skill descriptions from `.pi/skills/*/SKILL.md` frontmatter

Currently used by the Event Handler to load EVENT_HANDLER.md as the LLM system prompt.
