.PHONY: setup dev build clean install-models test lint

# ── Top-level targets ────────────────────────────────────────────────────────

setup:
	@bash scripts/setup.sh

install-models:
	@bash scripts/install-models.sh

# ── Development ──────────────────────────────────────────────────────────────

dev:
	@cd apps/desktop && pnpm tauri dev

dev-frontend:
	@cd apps/desktop && pnpm dev

dev-api:
	@PYTHONPATH=$(shell pwd)/.. python -m uvicorn nightshift.api.main:app --reload --port 8000

# ── Build ────────────────────────────────────────────────────────────────────

build:
	@bash scripts/build-desktop.sh

build-debug:
	@bash scripts/build-desktop.sh --debug

# ── Code quality ─────────────────────────────────────────────────────────────

lint:
	@cargo clippy --workspace --all-targets -- -D warnings
	@cd apps/desktop && pnpm lint

fmt:
	@cargo fmt --all
	@cd apps/desktop && pnpm format

test:
	@cargo test --workspace
	@cd apps/desktop && pnpm test

# ── Utility ──────────────────────────────────────────────────────────────────

clean:
	@cargo clean
	@rm -rf apps/desktop/dist apps/desktop/node_modules

deps:
	@cargo fetch
	@cd apps/desktop && pnpm install
