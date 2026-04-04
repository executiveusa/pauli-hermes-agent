# _JCP/recovery.md — Hermes Foundry Resumption Guide

If context is lost or a new session starts, read this file first.

## Current State: ALL 15 FEATURES COMPLETE

The full hermes-foundry scaffold is created. The project is ready for:
1. `pnpm install` — install Node deps
2. `cargo fetch`  — prefetch Rust deps
3. `cargo tauri dev` — run in dev mode
4. `cargo tauri build` — produce installer

## Project Root

```
e:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT\pauli-hermes-agent\hermes-foundry\
```

## Key File Locations

| What | Where |
|------|-------|
| Rust workspace | `Cargo.toml` |
| Tauri entry point | `apps/desktop/src-tauri/src/main.rs` |
| Tauri commands | `apps/desktop/src-tauri/src/commands/` |
| React root | `apps/desktop/src/App.tsx` |
| Chat component | `apps/desktop/src/components/Chat.tsx` |
| Zustand stores | `apps/desktop/src/stores/` |
| Tauri→Rust bridge | `apps/desktop/src/lib/tauri-bridge.ts` |
| Security/keyring | `apps/desktop/src-tauri/src/security.rs` |
| Sidecar manager | `apps/desktop/src-tauri/src/sidecar.rs` |
| White-label config | `branding.json` |
| Feature tracking | `_JCP/features.json` |

## Build Commands

```bash
# First time
make setup
make install-models

# Dev
make dev

# Production build
make build
```

## Sidecar Processes

The Tauri app manages 3 sidecar processes on startup:
1. **hermes-api** (Python FastAPI) — Port 8000
2. **llama-server** (atomic-llama-cpp-turboquant) — Port 8080  
3. **whisper-server** (AtomicBot whisper.cpp) — Port 8090

Sidecars are defined in `tauri.conf.json` `bundle.externalBin` and started/stopped in `src/sidecar.rs`.

## AtomicBot Repos (git submodules)

```
vendor/whisper.cpp/                  AtomicBot-ai/whisper.cpp
vendor/atomic-llama-cpp-turboquant/  AtomicBot-ai/atomic-llama-cpp-turboquant
vendor/FFmpeg/                       AtomicBot-ai/FFmpeg
```

Initialize with: `git submodule update --init --recursive`

## What To Do Next (if scope expands)

1. **Multi-window support** — open separate chat windows via `tauri::WindowBuilder`
2. **System tray** — minimize to tray with notification badge for pending approvals
3. **Auto-update** — `tauri-plugin-updater` with semver
4. **Telemetry opt-in** — aggregate usage metrics for enterprise dashboard
5. **Team sync** — sync conversations across devices via nightshift API
6. **Plugin system** — expose `hermes-core` as a dynamic library for third-party tools
