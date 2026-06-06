# FREE MODE — Universal Free AI Toggle for Hermes Agent

When **FREE_MODE=true**, all AI inference routes through LiteLLM proxy → free/local providers.
When **FREE_MODE=false**, Hermes uses its original provider configuration unchanged.

## Quick Start

```bash
# 1. Start the LiteLLM proxy
docker compose -f docker-compose.free-mode.yml up -d

# 2. Enable FREE MODE in shell
export FREE_MODE=true
export ANTHROPIC_BASE_URL=http://127.0.0.1:4000
export ANTHROPIC_AUTH_TOKEN=change-me-local-master-key

# 3. Run Hermes agent
hermes "What is 2+2?"
# Responds instantly — routed through local/free provider
```

## How It Works

```
┌─────────────────────────────────────────────────────────┐
│ Hermes Agent (Python)                                   │
│ Original model calls preserved via providers/base.py    │
└─────────┬───────────────────────────────────────────────┘
          │ FREE_MODE=true ?
          ├─ Yes → LiteLLM Proxy (127.0.0.1:4000)
          │        ├─ Try local (Ollama, LM Studio)
          │        ├─ Try free cloud (Groq, Gemini, OpenRouter free)
          │        └─ Fallback to paid (OpenAI, Anthropic)
          │
          └─ No → Original provider (OpenAI, Anthropic, etc.)
```

## Installation

### Prerequisites

- Docker (for LiteLLM proxy)
- One of:
  - Local model server (Ollama, LM Studio)
  - Free API key (Groq, Gemini, OpenRouter)
  - Paid API key (OpenAI, Anthropic)

### Setup

1. **Copy env placeholders:**
   ```bash
   # .env.example already has all FREE_MODE variables
   cp .env.example .env
   ```

2. **Configure at least one provider:**

   **Option A: Local (Ollama)**
   ```bash
   # Install: https://ollama.ai
   # Pull a model: ollama pull qwen2.5-coder:7b
   # Server runs on: http://127.0.0.1:11434
   # (no env config needed — auto-defaults)
   ```

   **Option B: Free Cloud (Groq)**
   ```bash
   # Get key: https://console.groq.com/keys
   export GROQ_API_KEY=gsk_...
   ```

   **Option C: Free Cloud (Google Gemini)**
   ```bash
   # Get key: https://aistudio.google.com/app/apikey
   export GEMINI_API_KEY=AIza...
   ```

3. **Start LiteLLM proxy:**
   ```bash
   docker compose -f docker-compose.free-mode.yml up -d
   
   # Verify health
   curl http://127.0.0.1:4000/health
   # Should return: {"status": "ok"}
   ```

4. **Test FREE MODE:**
   ```bash
   # In your shell
   export FREE_MODE=true
   export ANTHROPIC_BASE_URL=http://127.0.0.1:4000
   export ANTHROPIC_AUTH_TOKEN=change-me-local-master-key
   
   # Run Hermes
   hermes "Hello"
   # Should respond via free provider
   ```

## Configuration

### Environment Variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `FREE_MODE` | `false` | Enable/disable FREE MODE |
| `FREE_MODE_PROXY_BASE_URL` | `http://127.0.0.1:4000` | LiteLLM proxy address |
| `FREE_MODE_PROXY_MASTER_KEY` | `change-me-local-master-key` | Proxy auth key |
| `FREE_MODE_MODEL` | `free-auto` | Model alias for routing |
| `LITELLM_MASTER_KEY` | `change-me-local-master-key` | Proxy master key |
| `LITELLM_CONFIG_PATH` | `free-mode/litellm.config.yaml` | Proxy config file |

### Model Aliases

**Local-first:**
- `free-auto` → Try local, then free cloud
- `free-local` → Ollama/LM Studio only
- `free-code` → Groq (optimized for coding)
- `free-fast` → Groq (fastest free option)

**Cloud-first:**
- `free-cloud` → OpenRouter free model
- `free-reasoning` → OpenRouter with reasoning
- `free-gemini` → Google Gemini
- `free-nvidia-nim` → NVIDIA NIM

**Premium/Paid:**
- `default-auto` → Original provider selection
- `default-openai` → GPT-4o-mini
- `default-anthropic` → Claude Sonnet
- `premium-auto` → OpenRouter premium models

### LiteLLM Config

See `free-mode/litellm.config.yaml` — it defines:
- Model aliases
- Provider routing priority
- Fallback chains
- Request timeouts

To customize:
```yaml
# Edit free-mode/litellm.config.yaml
model_list:
  - model_name: free-auto
    litellm_params:
      model: free-local  # Change to free-groq, free-gemini, etc.
```

Then restart proxy:
```bash
docker compose -f docker-compose.free-mode.yml restart litellm-free-mode
```

## Provider Guide

### Local Providers (Fastest, Zero Cost)

**Ollama**
```bash
# Install: https://ollama.ai
# Start server: ollama serve
# Pull model: ollama pull qwen2.5-coder:7b
# No env config needed (auto-defaults to http://127.0.0.1:11434)
```

**LM Studio**
```bash
# Download: https://lmstudio.ai
# Load a model → Start server
# Server runs on: http://127.0.0.1:1234/v1
# No env config needed (auto-defaults)
```

### Free Cloud Providers

**Groq** (Fastest free tier)
```bash
# Get key: https://console.groq.com/keys
# Max: 40 requests/min
export GROQ_API_KEY=gsk_...
# Model: llama-3.3-70b-versatile
```

**Google Gemini Flash** (Generous free tier)
```bash
# Get key: https://aistudio.google.com/app/apikey
# No rate limit on free tier
export GEMINI_API_KEY=AIza...
# Model: gemini-3.5-flash
```

**OpenRouter (Free Models)**
```bash
# Get key: https://openrouter.ai/keys
# Free models: DeepSeek, Qwen, etc.
export OPENROUTER_API_KEY=sk-or-...
# Model: deepseek/deepseek-chat-v3-0324:free
```

**NVIDIA NIM** (Free for developer tier)
```bash
# Get key: https://build.nvidia.com/
# No official rate limit published
export NVIDIA_NIM_API_KEY=nvapi-...
# Model: meta/llama-3.1-70b-instruct
```

### Paid Providers (Premium)

**OpenAI**
```bash
export OPENAI_API_KEY=sk-...
# Models: gpt-4o, gpt-4o-mini (cheaper)
```

**Anthropic**
```bash
export ANTHROPIC_API_KEY=sk-ant-...
# Models: claude-opus-4.6, claude-sonnet-4-5, claude-haiku-4-5
```

**Others:** DeepSeek, Mistral, Cerebras, Hugging Face, etc.

## Testing

### Test Proxy Health

```bash
curl http://127.0.0.1:4000/health
# Expected: {"status": "ok"}
```

### Test a Single Provider

```bash
# List available models
curl http://127.0.0.1:4000/v1/models \
  -H "Authorization: Bearer change-me-local-master-key"

# Send test request
curl http://127.0.0.1:4000/v1/chat/completions \
  -H "Authorization: Bearer change-me-local-master-key" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "free-auto",
    "messages": [{"role": "user", "content": "Say: free-mode-ok"}],
    "max_tokens": 16
  }'
```

### Test Hermes Integration

```bash
# Enable FREE MODE
export FREE_MODE=true
export ANTHROPIC_BASE_URL=http://127.0.0.1:4000
export ANTHROPIC_AUTH_TOKEN=change-me-local-master-key

# Run agent
hermes "Hello"

# Monitor proxy logs
docker logs -f free-mode-litellm
```

## Cost Breakdown

| Provider | Cost | Speed | Use Case |
|----------|------|-------|----------|
| **Ollama (local)** | $0 | Slow | Dev/testing, no internet needed |
| **Groq** | $0 | Very Fast | Real-time, code generation |
| **Gemini Flash** | $0 | Fast | General tasks, vision-capable |
| **OpenRouter (free)** | $0 | Fast | Diverse model access |
| **NVIDIA NIM** | $0 | Fast | Production workloads |
| **OpenAI (gpt-4o-mini)** | $0.15/M input | Very Fast | Premium quality |
| **Anthropic (Sonnet)** | $3/M input | Fast | Best reasoning |

## Troubleshooting

### Proxy won't start

```bash
# Check Docker
docker ps | grep free-mode-litellm

# View logs
docker logs free-mode-litellm

# Restart
docker compose -f docker-compose.free-mode.yml restart litellm-free-mode
```

### "Connection refused" to 127.0.0.1:4000

```bash
# Check proxy is running
curl http://127.0.0.1:4000/health

# Check port not in use
lsof -i :4000

# Verify .env has correct URL
grep FREE_MODE_PROXY_BASE_URL .env
```

### "Unauthorized" (401) errors

```bash
# Check auth key matches
grep LITELLM_MASTER_KEY .env
grep ANTHROPIC_AUTH_TOKEN .env
# They should match (or use the same value)
```

### Provider returns 401 (auth failed)

```bash
# Check API key is set
echo $GROQ_API_KEY
# Should be non-empty

# Verify key format
# Groq: gsk_...
# Gemini: AIza...
# OpenRouter: sk-or-...

# Test key directly
curl https://api.groq.com/openai/v1/models \
  -H "Authorization: Bearer $GROQ_API_KEY"
```

### Proxy timeout

```bash
# Increase timeout in .env
export FREE_MODE_TIMEOUT_MS=120000  # 2 minutes

# Or edit litellm.config.yaml
router_settings:
  timeout: 120  # seconds
```

## Advanced

### Custom Model Routing

Edit `free-mode/litellm.config.yaml` to add custom aliases:

```yaml
- model_name: my-custom
  litellm_params:
    model: groq/${GROQ_MODEL}
    api_key: os.environ/GROQ_API_KEY
```

Then use:
```bash
export FREE_MODE_MODEL=my-custom
```

### Claude Code Integration

```bash
# Route Claude Code through FREE MODE proxy
export ANTHROPIC_BASE_URL=http://127.0.0.1:4000
export ANTHROPIC_AUTH_TOKEN=change-me-litellm-master-key
export CLAUDE_CODE_ENABLE_GATEWAY_MODEL_DISCOVERY=1

# Run Claude Code
claude
```

### Multi-Machine Proxy

To use the same proxy from a different machine:

```bash
# On proxy machine (edit docker-compose.free-mode.yml)
ports:
  - "0.0.0.0:4000:4000"  # Bind to all interfaces

# On client machine
export FREE_MODE_PROXY_BASE_URL=http://<proxy-machine-ip>:4000
```

⚠️ **Security**: Only do this on trusted networks. The proxy requires the master key.

### Redis Caching

For production deployments, enable Redis caching:

```bash
# Start Redis
docker run -d -p 127.0.0.1:6379:6379 redis:latest

# Set in .env
export LITELLM_REDIS_HOST=127.0.0.1
export LITELLM_REDIS_PORT=6379
```

## Security Notes

1. **Master Key**: Change `LITELLM_MASTER_KEY` in production
2. **No Secrets in Repo**: All real keys go in `.env` (in `.gitignore`)
3. **Local Binding**: Proxy binds to `127.0.0.1` by default (localhost only)
4. **Payload Logging**: Disabled by default; enable only for debugging

## Architecture

```
free-mode/
├── litellm.config.yaml        # Model aliases and routing
├── providers.json             # Provider metadata registry
├── README.md                  # This file
├── SECURITY.md               # Security guidelines
└── scripts/
    ├── start-free-mode.sh     # Start proxy + export env
    ├── test-free-mode.sh      # Run provider tests
    └── test-provider.py       # Single provider test
```

## Next Steps

1. [x] Install and configure a provider (local or free cloud)
2. [x] Start LiteLLM proxy
3. [x] Enable `FREE_MODE=true`
4. [ ] Run `hermes "test"` to verify integration
5. [ ] Configure additional providers for fallback
6. [ ] (Optional) Set up dashboard toggle for UI-based switching
7. [ ] (Optional) Deploy to production with Redis caching

## References

- [LiteLLM Proxy Docs](https://docs.litellm.ai/)
- [Ollama](https://ollama.ai/)
- [Groq Docs](https://console.groq.com/docs)
- [Google Gemini API](https://ai.google.dev/)
- [Hermes Agent](https://github.com/NousResearch/Hermes-Agent)

---

**Questions?** See `free-mode/SECURITY.md` for security concerns or check logs:
```bash
docker logs -f free-mode-litellm
```
