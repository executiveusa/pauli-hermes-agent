# 🐕 Agent MAXX — Night Shift

**Autonomous overnight coding platform.** Clone → analyze → code → test → ship — while you sleep.

---

## Architecture

```
Caddy (80/443 + TLS)
  ├── / → hermes-dashboard (landing + cockpit)
  ├── /api/* → hermes-api (FastAPI control plane)
  └── /mcp* → hermes-api (MCP endpoint for IDE clients)

hermes-api
  ├── PostgreSQL (15 tables, async SQLAlchemy)
  ├── Redis (job queues + pub/sub)
  └── → provider-router (LLM proxy)

Workers (Redis queue consumers)
  ├── worker-coder (clone → code → test → commit)
  └── worker-browser (Playwright, pause/resume, vision)

Support
  ├── notifier (Telegram alerts, approval gates)
  └── scheduler (nightly repo scan, PRD generation, digest)
```

## Quick Start

### Prerequisites
- Docker + Docker Compose v2
- API keys for at least one LLM provider

### 1. Clone & configure

```bash
git clone <your-repo>
cd nightshift
make setup          # copies .env.example → .env
```

Edit `.env` and fill in your keys:

```env
ANTHROPIC_API_KEY=sk-ant-...
TELEGRAM_BOT_TOKEN=...
TELEGRAM_CHAT_ID=...
GITHUB_TOKEN=ghp_...
DOMAIN=your.domain.com     # or leave as localhost
```

### 2. Start

```bash
make dev            # builds + starts all services
```

Open: http://localhost:3000

### 3. Useful commands

```bash
make logs           # tail all logs
make logs-api       # just hermes-api
make ps             # service status
make db-shell       # postgres CLI
make smoke          # run smoke tests
make backup         # dump DB to ./backups/
make down           # stop (keep data)
make clean          # stop + wipe (DESTRUCTIVE)
```

---

## Services

| Service | Port | Purpose |
|---------|------|---------|
| hermes-dashboard | 3000 | Landing page + 12-panel cockpit |
| hermes-api | 8642 | FastAPI control plane |
| provider-router | 8080 (internal) | Multi-model LLM proxy |
| worker-coder | — | Coding session runner |
| worker-browser | — | Playwright browser automation |
| notifier | — | Telegram alerts |
| scheduler | — | Nightly cron jobs |
| postgres | 5432 (internal) | Primary database |
| redis | 6379 (internal) | Queues + pub/sub |
| caddy | 80/443 | Reverse proxy + TLS |

---

## API

All endpoints require `X-API-Key` header (set `API_KEY` in `.env`).  
Dev mode: if `API_KEY` is empty, auth is disabled.

```bash
# Overview stats
curl http://localhost:8642/api/dashboard/overview

# List repos
curl http://localhost:8642/api/repos

# Trigger a coding session
curl -X POST http://localhost:8642/api/coding-sessions \
  -H "Content-Type: application/json" \
  -d '{"repo_url":"https://github.com/org/repo","prompt":"Add dark mode","task_type":"feature"}'

# List browser runs
curl http://localhost:8642/api/browser-runs

# MCP endpoint (for IDE clients)
# Connect any MCP client to: http://localhost:8642/mcp
```

Full OpenAPI docs: http://localhost:8642/docs

---

## MCP Integration

Agent MAXX exposes all API routes as MCP tools at `/mcp`.

**VS Code (Claude extension)**:
```json
{
  "mcpServers": {
    "agent-maxx": {
      "url": "http://localhost:8642/mcp"
    }
  }
}
```

**Zed / JetBrains / Cursor**: Add `http://localhost:8642/mcp` as MCP server URL.

---

## Provider Profiles

Agent MAXX routes tasks to the best model per job type:

| Profile | Default Model | Used For |
|---------|--------------|---------|
| `balanced` | claude-sonnet-4 | Default coding |
| `premium_reasoning` | claude-opus-4 | Complex reasoning |
| `low_cost_coding` | claude-sonnet-4 | Bulk tasks |
| `research_browser` | gemini-2.5-flash | Vision + research |
| `openai_gpt4` | gpt-4o | OpenAI tasks |

---

## Telegram Setup

1. Create a bot via [@BotFather](https://t.me/BotFather) → get `TELEGRAM_BOT_TOKEN`
2. Get your chat ID: message [@userinfobot](https://t.me/userinfobot) → get `TELEGRAM_CHAT_ID`
3. Add both to `.env` and restart: `make restart-notifier`

Alerts you receive:
- 🟠 Approval required (with approve/reject buttons via `/api/approvals/{id}/decide`)
- ✅ Run completed
- ❌ Run failed
- 🌐 Browser intervention needed (CAPTCHA/MFA)

---

## Deployment (Hostinger + Coolify)

See [`coolify.json`](coolify.json) for Coolify deployment configuration.

**Quick deploy:**
1. Push this repo to GitHub
2. In Coolify: New Application → Docker Compose → point to this repo's `nightshift/` folder
3. Set environment variables from `.env.example`
4. Set `DOMAIN` to your Hostinger domain
5. Deploy

Caddy handles TLS automatically via Let's Encrypt.

---

## Vision & Test-Taking Research

Worker-browser has a dedicated vision mode for online test research:

```bash
# Start a vision-mode browser run
curl -X POST http://localhost:8642/api/browser-runs \
  -H "Content-Type: application/json" \
  -d '{
    "start_url": "https://quiz-site.com",
    "steps": [{"action": "vision_analyze", "prompt": "Read and answer all questions on this page"}],
    "vision_mode": true
  }'
```

The browser worker:
1. Screenshots each page
2. Sends image to vision LLM (Gemini 2.0 Flash / GPT-4V)
3. Extracts questions and generates answers
4. Logs all interactions as artifacts

---

## Data

All state lives in PostgreSQL (15 tables). Backups:

```bash
make backup         # ./backups/maxx_YYYYMMDD_HHMMSS.sql
```

Restore:
```bash
cat backups/maxx_*.sql | docker compose exec -T postgres psql -U maxx maxx_db
```

---

## Brand

**Agent MAXX** — Night Shift  
License: MIT | Built by Kupuri Media | 2025  
Personality: Agent 006 vibes. Bond's dog. Ships code, never misses.
