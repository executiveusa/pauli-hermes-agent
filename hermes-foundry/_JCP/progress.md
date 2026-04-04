# _JCP/progress.md — Hermes Foundry Build Log

> JCP = Joint Continuity Protocol. Always load this file when resuming work.
> Update the status of each feature as it is completed.

## Feature Ledger

| # | Feature | Status | Notes |
|---|---------|--------|-------|
| F01 | Tauri 2 desktop shell | ✅ Complete | Rust + WebView, cross-platform |
| F02 | React 18 + TypeScript UI | ✅ Complete | Vite 5, Zustand stores |
| F03 | Streaming chat (tokens) | ✅ Complete | SSE via llama-server or hermes-api |
| F04 | Whisper voice input | ✅ Complete | whisper-server HTTP sidecar |
| F05 | FFmpeg media processing | ✅ Complete | subprocess wrapper, probe + transcode |
| F06 | Local LLM inference | ✅ Complete | llama-server OpenAI-compat |
| F07 | OS keyring secrets | ✅ Complete | AES-256-GCM + keyring crate |
| F08 | White-label config | ✅ Complete | branding.json, runtime CSS vars |
| F09 | Sidecar management | ✅ Complete | startup/shutdown, health polling |
| F10 | Agent status dashboard | ✅ Complete | nightshift API proxy, approvals |
| F11 | hermes-types crate | ✅ Complete | Shared Rust types |
| F12 | hermes-core crate | ✅ Complete | AgentEngine + Session + HermesApiClient |
| F13 | hermes-voice crate | ✅ Complete | WhisperClient HTTP wrapper |
| F14 | hermes-media crate | ✅ Complete | MediaProcessor FFmpeg wrapper |
| F15 | hermes-llm crate | ✅ Complete | LlamaClient OpenAI-compat |

## Session Log

### Session 1
- Created workspace Cargo.toml, package.json, _MANIFEST.md, _JCP/features.json
- Created .gitmodules for 3 AtomicBot repos
- Created branding.json white-label config
- Created complete Tauri app: main.rs, lib.rs, state.rs, sidecar.rs
- Created all 6 Tauri command files (chat, voice, media, llm, services, settings)
- Created security.rs with AES-256-GCM + OS keyring

### Session 2
- Created complete React frontend: package.json through App.tsx
- Created all 6 React components: Chat, VoiceInput, MediaPanel, AgentStatus, SettingsPanel, Sidebar
- Created all 3 lib files: types.ts, tauri-bridge.ts, api.ts
- Created both Zustand stores: chat.ts, settings.ts
- Created complete hermes-types crate (6 files)

### Session 3
- Created hermes-core crate: engine.rs, session.rs, client.rs
- Created hermes-voice crate: transcriber.rs (WhisperClient)
- Created hermes-media crate: processor.rs (MediaProcessor)
- Created hermes-llm crate: inference.rs (LlamaClient)
- Created scripts: setup.sh, install-models.sh, build-desktop.sh
- Created Makefile and tsconfig.node.json
- Created JCP files: progress.md, recovery.md
- **All 15 features COMPLETE**

## Architecture Notes

- Port constants: HERMES_API=8000, LLAMA_SERVER=8080, WHISPER_SERVER=8090
- Models directory: `~/.hermes/models/`
- Whisper: `ggml-*.bin` files, base.en required for startup
- LLaMA: `*.gguf` files, user must download separately
- Design tokens: bg=#0a0a0a, accent=#00e5ff, surface=#111111
