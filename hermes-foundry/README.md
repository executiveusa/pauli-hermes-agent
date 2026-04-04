# Hermes Foundry

**Enterprise AI desktop application** — whisper.cpp × llama.cpp × FFmpeg × Hermes nightshift, unified in a single Tauri v2 installer with white-label support and OS-keyring security.

---

## What It Is

Hermes Foundry ships one click-to-install desktop app that bundles:

| Capability | Powered by |
|-----------|------------|
| Streaming chat (local LLM) | atomic-llama-cpp-turboquant |
| Voice transcription | AtomicBot whisper.cpp |
| Audio/video processing | AtomicBot FFmpeg |
| Agent orchestration | Hermes nightshift (Python FastAPI) |
| Enterprise auth + secrets | OS keyring + AES-256-GCM |
| White-label branding | `branding.json` |

---

## Quick Start

```bash
# 1. Dependencies
make setup

# 2. Download Whisper base model (148MB, required)
make install-models

# 3. Run in dev mode
make dev

# 4. Build installer
make build
```

---

## Architecture

```
hermes-foundry/
├── apps/desktop/              # Tauri v2 desktop app
│   ├── src-tauri/             # Rust shell (sidecar mgmt, IPC, security)
│   │   └── src/commands/      # chat · voice · media · llm · services
│   └── src/                   # React 18 + TypeScript UI
│       ├── components/        # Chat · VoiceInput · MediaPanel · ...
│       ├── stores/            # Zustand (chat · settings)
│       └── lib/               # tauri-bridge · types · api
├── crates/
│   ├── hermes-types/          # Shared Rust types
│   ├── hermes-core/           # Agent engine + HTTP client + sessions
│   ├── hermes-voice/          # WhisperClient (HTTP)
│   ├── hermes-media/          # MediaProcessor (FFmpeg subprocess)
│   └── hermes-llm/            # LlamaClient (OpenAI-compat HTTP)
├── vendor/                    # AtomicBot repos (git submodules)
│   ├── whisper.cpp/
│   ├── atomic-llama-cpp-turboquant/
│   └── FFmpeg/
└── scripts/                   # setup · install-models · build-desktop
```

---

## Sidecar Ports

| Service | Port | Protocol |
|---------|------|----------|
| hermes-api (Python) | 8000 | REST |
| llama-server | 8080 | OpenAI-compat REST |
| whisper-server | 8090 | REST multipart |

---

## White-Label Config

Edit `branding.json` to customize the app for your organization:

```json
{
  "app_name": "Your Agent",
  "colors": { "accent": "#6366f1" },
  "welcome_message": "Hello, I'm your AI assistant."
}
```

Then rebuild with `make build`.

---

## Security

- API keys stored in **OS keyring** (Keychain on macOS, DPAPI on Windows, libsecret on Linux)
- In-memory secrets encrypted with **AES-256-GCM** and **zeroized** on drop
- No keys logged or written to disk in plaintext
- Tauri CSP enforced — no external network calls from the WebView

---

## Models

Place GGUF files in `~/.hermes/models/`:

```
~/.hermes/models/
├── ggml-base.en.bin          ← required (148MB, auto-downloaded)
├── Qwen2.5-7B-Instruct-Q4_K_M.gguf
└── ...
```

---

## Development

```bash
make lint       # cargo clippy + eslint
make fmt        # cargo fmt + prettier
make test       # cargo test + vitest
make clean      # remove build artifacts
```

---

## License

MIT OR Apache-2.0
