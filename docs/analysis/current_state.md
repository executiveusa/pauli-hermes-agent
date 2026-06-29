# Current State

Bead: `ZTE-20260506-0001`
Date: `2026-05-06`

## Summary

- Clean Hermes workspace is `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main` on branch `zte/20260506-hermes-desktop-hostinger-stack`.
- Dirty root checkout `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT` was intentionally left untouched.
- Hermes Desktop clone is `E:\ACTIVE PROJECTS-PIPELINE\hermes-desktop` on branch `zte/20260506-hermes-desktop-hostinger-stack`.
- Hermes upstream `main` is now 12 commits ahead of `origin/main` / the clean Pauli worktree.
- Hermes API server is reachable locally on `http://127.0.0.1:8642/health`.
- Hermes authenticated chat is blocked by downstream provider auth failure: API responds, but provider returns `401 Missing Authentication header`.
- Hermes Desktop dependency install is incomplete/hung on this machine: packages land partially, but local CLI wrappers and some expected package files are missing, so lint/test/build cannot complete.

## Toolchain

- Python launcher: present (`py`, CPython 3.13.12)
- `uv`: present
- Node.js: present (`v24.13.0`)
- npm: present (`11.6.2`)
- pnpm: present
- GitHub CLI: present
- Docker: missing
- WSL2 kernel: present
- WSL distro: missing

## Environment Notes

- `E:\THE PAULI FILES\.ENV` exists but is too sparse to run Hermes by itself.
- Additional env-like files in the same secrets folder contain provider/deploy keys; a merged local Hermes env was created without printing or committing secret values.
- Hermes profile `pauli` was created, but Windows profile/config resolution behaved inconsistently and continued resolving runtime config paths to the default Hermes home.

## Runtime Notes

- `hermes doctor` passes core package checks and sees OpenRouter connectivity.
- `hermes status` reports the local API provider path as OpenRouter with API key presence.
- `hermes gateway run` opens port `8642`; `GET /health` returns `{"status":"ok","platform":"hermes-agent"}`.
- `POST /v1/chat/completions` reaches Hermes but returns an assistant error string from the provider path: `401 Missing Authentication header`.

## Desktop Notes

- Repo version: `0.3.5`
- Tag at clone head: `v0.3.5` (`2026-05-06`)
- `npm install` / `npm ci --ignore-scripts` both exceeded long timeouts.
- `node_modules` exists, but `.bin` wrappers are missing and some package directories are incomplete from the perspective of their launch paths.

## Decision

Proceeding to blocker report because remaining failures require external credential confirmation and/or environment changes rather than additional safe local code edits.
