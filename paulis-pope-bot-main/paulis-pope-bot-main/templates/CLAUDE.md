# Templates Directory — Scaffolding Only

This directory contains files that get copied into user projects when they run `npx thepopebot init`. It is **not** where event handler logic, API routes, or core features live.

## Rules

- **NEVER** add event handler code, API route handlers, or core logic here. All of that belongs in the NPM package (`api/`, `lib/`, `config/`, `bin/`).
- Templates exist solely to scaffold a new user's project folder with thin wiring and user-editable configuration.
- Files here may be modified to fix wiring, update configuration defaults, or adjust scaffolding — but never to implement features.

## What belongs here

- **Next.js wiring**: `next.config.mjs`, `instrumentation.js`, catch-all route, middleware — thin re-exports from `thepopebot/*`
- **User-editable config**: `config/SOUL.md`, `config/EVENT_HANDLER.md`, `config/CRONS.json`, `config/TRIGGERS.json`, etc.
- **GitHub Actions workflows**: `.github/workflows/`
- **Docker files**: `docker/`, `docker-compose.yml`
- **UI page shells**: `app/` pages that import components from the package
- **Client components**: `app/components/` for components that must live in the user's project (e.g., `login-form.jsx`, `setup-form.jsx`) because they depend on user-side packages like `next-auth/react`

## What does NOT belong here

- Route handlers with business logic
- Library code (`lib/`)
- Database operations
- LLM/AI integrations
- Tool implementations
- Anything that should be shared across all users via `npm update thepopebot`

If you're adding a feature to the event handler, put it in the package. Templates just wire into it.
