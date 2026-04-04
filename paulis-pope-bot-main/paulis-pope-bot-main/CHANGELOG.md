# Changelog

## 1.2.57

### Drizzle Kit migrations

Database schema changes are now managed by Drizzle Kit instead of hand-written SQL. The old `initDatabase()` with raw `CREATE TABLE` and `ALTER TABLE` statements has been replaced by `migrate()`, which applies versioned migration files from `drizzle/`. Migrations run automatically on server startup — users upgrading thepopebot get schema changes applied seamlessly without any manual steps.

Migration files ship inside the npm package, so they resolve from `node_modules/thepopebot/drizzle/` at runtime regardless of the user's working directory.

**For package developers:** edit `lib/db/schema.js`, then run `npm run db:generate` to create a new migration file. Never write DDL SQL by hand.

---

## 1.2.x — The NPM Package Release

**Released: February 2026**

thepopebot is now an installable NPM package. Instead of forking a repo and wiring everything together yourself, you run one command and get a fully configured AI agent project. This release replaces the old fork-based architecture entirely.

---

### Install in seconds

Run `npx thepopebot init` and you have a working project. The interactive setup wizard walks you through API keys, GitHub secrets, and Telegram configuration — no more copying `.env.example` files and hunting for documentation. Upgrade later with a single command or let GitHub Actions handle it automatically.

### Web chat interface

Your agent now has a full web app. Chat with streaming responses, browse conversation history grouped by date, and pick up where you left off. Upload images, PDFs, and code files directly in the chat — the AI can see and analyze them. It's your own private ChatGPT-style interface for your agent.

### Choose your LLM

Switch between Anthropic, OpenAI, and Google models by changing two environment variables (`LLM_PROVIDER` and `LLM_MODEL`). No code changes needed. The old architecture was hardcoded to Anthropic — now you pick the model that fits the task and the budget.

### See what your agent is doing

The Swarm page shows every active and completed job in real time. See which tasks are running, cancel jobs that went sideways, and rerun completed ones. No more checking GitHub Actions logs to figure out what your agent is up to.

### Never miss a completed job

In-app notifications with unread badges tell you when jobs finish. Telegram notifications give you the summary on your phone. Every notification includes what the agent did, what files changed, and whether the PR merged — so you know the outcome without opening GitHub.

### Secure API access

API keys are hashed with SHA-256 and verified with timing-safe comparison. Create, rotate, and revoke keys from the settings page. The old single-key-in-`.env` approach is gone — you can now issue separate keys for different integrations and revoke them independently.

### Production deployment built in

`docker compose up` gives you Traefik with automatic HTTPS, PM2 process management, and a self-hosted GitHub Actions runner. The old architecture required you to figure out deployment yourself. Now it's one command with TLS certificates handled automatically via Let's Encrypt.

### Auto-upgrades

When a new version of thepopebot is published, a GitHub Actions workflow can open a PR to upgrade your project. Template files (workflows, Docker configs) are updated automatically. Your customizations in `config/` are never touched. You stay current without manual maintenance.

### Three ways to automate

Cron jobs and webhook triggers now support three action types:

- **Agent** — spin up the full AI agent in a Docker container for tasks that need thinking
- **Command** — run a shell script directly on the server for tasks that just need doing
- **Webhook** — fire an HTTP request to an external service

The old architecture only had agent jobs. Now quick tasks don't burn LLM credits or GitHub Actions minutes.

### Upload files to chat

Drag and drop images, PDFs, and code files into the chat. Images are analyzed with AI vision. PDFs and text files are read and included in the conversation context. Useful for asking your agent about screenshots, error logs, or documents.

### Authentication out of the box

NextAuth v5 with JWT sessions protects the web interface. The first time you visit, you create an admin account — no separate setup step. API routes use key-based auth for external callers; the browser UI uses session cookies via server actions. Two auth paths, each suited to its caller.

### Persistent conversations

All chats are stored in SQLite via Drizzle ORM. Browse history, resume old conversations, and search across past chats. The old architecture wrote JSON files to disk with no way to search or manage them.

### Infrastructure stays current

GitHub Actions workflows, Docker configs, and other infrastructure files are managed by the package. When you upgrade thepopebot, `thepopebot init` scaffolds updated versions of these files. Use `thepopebot diff` to see what changed and `thepopebot reset` to restore any file to the package default.

### Talk to your agent anywhere

A channel adapter pattern normalizes messages across platforms. Web chat and Telegram work today, and the base class makes it straightforward to add new channels. The old architecture was Telegram-only with no abstraction layer.

---

### Breaking changes

This release replaces the old fork-based architecture entirely. The old `event_handler/` Express server is gone, but your configuration files carry over to the new project.

**What's gone:**
- Fork-and-modify workflow — replaced by `npx thepopebot init`
- Express server in `event_handler/` — replaced by Next.js route handlers in the package
- Single `.env` API key — replaced by database-backed key management
- File-based JSON conversation history — replaced by SQLite database
- Anthropic-only LLM support — replaced by multi-provider architecture
- Manual deployment — replaced by Docker Compose with Traefik

**To adopt the new architecture:** Run `npx thepopebot init` in a fresh directory and run the setup wizard. Then copy over your configuration files — `config/SOUL.md`, `config/EVENT_HANDLER.md`, `config/AGENT.md`, `config/CRONS.json`, `config/TRIGGERS.json`, and any custom `.md` files you created. Move your `.pi/skills/` directory and any cron/trigger shell scripts as well. Your agent's personality, scheduled jobs, and skills carry over — only the surrounding infrastructure changes.
