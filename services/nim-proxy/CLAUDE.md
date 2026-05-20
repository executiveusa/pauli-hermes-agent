# NIM Proxy — Free Claude Code Inference via NVIDIA NIM

This service intercepts all Claude API calls and routes them to `moonshotai/kimi-k2-thinking`
on NVIDIA NIM — which is **free** on the NIM developer tier.

## How it works

```
Claude Code CLI  →  http://localhost:8082  →  NVIDIA NIM API
(Anthropic API format)    (this proxy)      (OpenAI-compatible)
```

The proxy translates Anthropic-format requests to OpenAI format and maps every Claude model
(`claude-opus-*`, `claude-sonnet-*`, `claude-haiku-*`) to `moonshotai/kimi-k2-thinking`.

## Deploy on VPS (31.220.58.212)

```bash
cd /opt/pauli-hermes-agent/services/nim-proxy
cp .env.example .env
# Edit .env — set NVIDIA_NIM_API_KEY
nano .env

# Start (foreground)
./start.sh

# Or run as systemd service
sudo cp nim-proxy.service /etc/systemd/system/
sudo systemctl enable --now nim-proxy
```

## Use in any Claude Code session

Add to `~/.bashrc` or `~/.zshrc` on your VPS (or any machine that can reach the proxy):

```bash
export ANTHROPIC_BASE_URL=http://31.220.58.212:8082
export ANTHROPIC_API_KEY=dummy  # proxy ignores this value
```

Then restart your shell and run `claude` as normal — all inference is free via NIM.

## Use for a single session

```bash
ANTHROPIC_BASE_URL=http://31.220.58.212:8082 ANTHROPIC_API_KEY=dummy claude
```

## Rate limits (NIM free tier)

- 40 requests / 60 seconds (configurable via `NVIDIA_NIM_RATE_LIMIT` env var)
- No token limit on free tier for `kimi-k2-thinking`

## NVIDIA NIM API key

Get a free key at: https://build.nvidia.com  
Current key is in `~/.hermes/.env` as `NVIDIA_NIM_API_KEY`.
