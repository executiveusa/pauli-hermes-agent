# ═══════════════════════════════════════════
# Agent MAXX — Night Shift Makefile
# ═══════════════════════════════════════════

.PHONY: help setup dev prod down restart logs ps pull clean \
        db-shell redis-cli smoke backup shell-api

COMPOSE=docker compose
ENV_FILE=.env

# ── Default target ───────────────────────────────────────────
help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN{FS=":.*?## "}; {printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2}'

# ── Setup ────────────────────────────────────────────────────
setup: ## First-time setup: copy .env.example, pull images
	@test -f $(ENV_FILE) || (cp .env.example $(ENV_FILE) && echo "✅  Created .env — edit it before running 'make dev'")
	@echo "ℹ️   Edit .env with your API keys, then run: make dev"

bootstrap: ## Pull images + init DB
	$(COMPOSE) pull
	$(COMPOSE) up -d postgres redis
	@echo "⏳  Waiting for Postgres…"
	@sleep 4
	$(COMPOSE) exec postgres psql -U maxx -d maxx_db -f /docker-entrypoint-initdb.d/001_initial.sql 2>/dev/null || true
	@echo "✅  Bootstrap complete"

# ── Dev ─────────────────────────────────────────────────────
dev: ## Start all services in dev mode (build from source)
	$(COMPOSE) up --build -d
	@echo "🐕  Agent MAXX running:"
	@echo "    Dashboard: http://localhost:3000"
	@echo "    API:       http://localhost:8642"
	@echo "    Log:       make logs"

dev-fg: ## Start in foreground (with log streaming)
	$(COMPOSE) up --build

# ── Prod ────────────────────────────────────────────────────
prod: ## Start in production mode (no rebuild)
	$(COMPOSE) up -d
	@echo "🚀  Production started"

pull: ## Pull latest images
	$(COMPOSE) pull

restart: ## Restart all services
	$(COMPOSE) restart

restart-api: ## Restart only hermes-api
	$(COMPOSE) restart hermes-api

# ── Status ──────────────────────────────────────────────────
ps: ## Show service status
	$(COMPOSE) ps

logs: ## Tail all logs
	$(COMPOSE) logs -f --tail=100

logs-api: ## Tail hermes-api logs
	$(COMPOSE) logs -f hermes-api --tail=100

logs-worker: ## Tail worker logs
	$(COMPOSE) logs -f worker-coder worker-browser --tail=100

logs-notifier: ## Tail notifier logs
	$(COMPOSE) logs -f notifier --tail=100

# ── Stop / Clean ─────────────────────────────────────────────
down: ## Stop all services (keep volumes)
	$(COMPOSE) down

clean: ## Stop + remove volumes (DESTRUCTIVE — wipes DB!)
	@read -p "⚠️  This will delete all data. Continue? [y/N] " ans && [ "$$ans" = y ]
	$(COMPOSE) down -v

# ── Database ─────────────────────────────────────────────────
db-shell: ## Open psql shell
	$(COMPOSE) exec postgres psql -U maxx -d maxx_db

db-migrate: ## Run pending SQL migrations
	$(COMPOSE) exec postgres psql -U maxx -d maxx_db -f /migrations/001_initial.sql

backup: ## Backup database to ./backups/
	@mkdir -p backups
	$(COMPOSE) exec postgres pg_dump -U maxx maxx_db > backups/maxx_$(shell date +%Y%m%d_%H%M%S).sql
	@echo "✅  Backup saved to backups/"

# ── Redis ────────────────────────────────────────────────────
redis-cli: ## Open redis-cli
	$(COMPOSE) exec redis redis-cli

# ── Smoke test ──────────────────────────────────────────────
smoke: ## Run smoke tests against running stack
	@bash infra/scripts/smoke-test.sh

# ── Utility shells ──────────────────────────────────────────
shell-api: ## Shell into hermes-api container
	$(COMPOSE) exec hermes-api bash

shell-coder: ## Shell into worker-coder container
	$(COMPOSE) exec worker-coder bash
