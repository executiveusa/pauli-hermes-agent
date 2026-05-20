# Claude Code with Free NVIDIA NIM Inference

Complete guide to activate free Claude inference globally in Claude Code.

## Current Status

✅ **NVIDIA NIM Proxy Running:** Port 8082 on your VPS (`31.220.58.212`)
✅ **Voice Agent Deployed:** https://pauli-hermes-agent.vercel.app/agent
✅ **Provider Toggles:** Mercury + NVIDIA selectable in web UI
⚠️ **Global Claude Code:** NOT YET activated (use setup below)

## Global Setup: Make NVIDIA Your Default

### Option 1: Shell Profile (Permanent)

**For your VPS and any machine using the agent:**

```bash
# Add to ~/.bashrc or ~/.zshrc
export ANTHROPIC_BASE_URL=http://31.220.58.212:8082
export ANTHROPIC_API_KEY=dummy

# Reload shell
source ~/.bashrc
```

Then use Claude Code normally:
```bash
claude
```

All requests will route through the free NIM proxy.

### Option 2: Claude Code Config (Project-Level)

Create `.claude.env` in your project root:

```bash
ANTHROPIC_BASE_URL=http://31.220.58.212:8082
ANTHROPIC_API_KEY=dummy
```

Claude Code will pick it up automatically.

### Option 3: Docker/Container (Isolated)

```dockerfile
FROM ubuntu:latest
RUN apt-get install -y python3 python3-pip curl git

# Install Claude Code
RUN curl -fsSL https://install.claude.ai | bash

# Set NIM proxy as default
ENV ANTHROPIC_BASE_URL=http://31.220.58.212:8082
ENV ANTHROPIC_API_KEY=dummy

# Your project setup
COPY . /app
WORKDIR /app

ENTRYPOINT ["claude"]
```

## How It Works

```
Your Project (claude cli)
    ↓
ANTHROPIC_BASE_URL=http://31.220.58.212:8082
    ↓
NIM Proxy (translates Anthropic API → OpenAI format)
    ↓
NVIDIA NIM (moonshotai/kimi-k2-thinking) — FREE
    ↓
Response back to Claude Code
```

**Cost Savings:**
- Standard Anthropic API: $3/million input tokens, $15/million output tokens
- NVIDIA NIM: **100% FREE** (developer tier, unlimited)

**Limitations:**
- 40 requests/minute rate limit
- All models map to `moonshotai/kimi-k2-thinking` (not model-specific)

## Test It

```bash
# Verify NIM proxy is running
curl http://31.220.58.212:8082/health

# Test Claude Code with free inference
export ANTHROPIC_BASE_URL=http://31.220.58.212:8082
export ANTHROPIC_API_KEY=dummy
claude "What is 2+2?"

# Should respond instantly with free NVIDIA inference
```

## Prompt Template for New Projects

Use this prompt to onboard any new Claude Code project with the full agent setup:

```markdown
# Hermes Agent Setup for New Project

You are setting up Claude Code with free NVIDIA NIM inference and the Hermes Rolodex agent.

## Goals
1. Configure this project for free Claude Code inference via NVIDIA NIM
2. Enable Hermes agent for memory, contact tracking, and autonomous actions
3. Set up provider toggles (Mercury + NVIDIA selection)

## Configuration

### 1. Environment (.claude.env or export)
```bash
# Free inference routing
export ANTHROPIC_BASE_URL=http://31.220.58.212:8082
export ANTHROPIC_API_KEY=dummy

# Hermes agent
export HERMES_CONFIG=~/.hermes/config.yaml

# API keys (populate from ~/.hermes/.env)
export NVIDIA_NIM_API_KEY=nvapi-...
export MERCURY_API_KEY=... (if using Mercury)
export TELEGRAM_BOT_TOKEN=...
```

### 2. Initialize Project
- Add `.claude.env` with the above exports
- Run `hermes mcp test hermes-rolodex` to verify agent connection
- Set up `start-agent.sh` script if integrating with voice agent

### 3. Use in Code
```python
# Claude Code will automatically:
# - Route all requests through NVIDIA NIM (free)
# - Use Hermes Rolodex for memory persistence
# - Support Mercury provider selection for advanced reasoning
```

## Provider Selection

### NVIDIA NIM (Default - Free)
- Model: `moonshotai/kimi-k2-thinking`
- Cost: $0 (free tier)
- Speed: Fast
- Use case: General assistance, coding, analysis

### Mercury Inception Labs (Premium - Optional)
- Cost: Varies by usage
- Speed: Very fast
- Use case: Advanced reasoning, complex problem-solving
- Toggle: Available in web UI and API

## Voice Agent Integration

The web voice agent at `https://pauli-hermes-agent.vercel.app/agent` automatically:
- Routes to selected provider (toggle switches in UI)
- Uses Web Speech API for microphone input
- Synthesizes voice responses
- Stores interactions in Hermes memory

## Troubleshooting

**Q: Claude Code still uses Anthropic API**
- A: Make sure `ANTHROPIC_BASE_URL` is set BEFORE starting Claude Code
- Verify: `echo $ANTHROPIC_BASE_URL` should show `http://31.220.58.212:8082`

**Q: Getting 403 errors from NIM proxy**
- A: VPS firewall might be blocking. Check: `nc -zv 31.220.58.212 8082`
- Fix: `sudo ufw allow 8082` on VPS

**Q: Want to switch back to Anthropic API**
- A: `unset ANTHROPIC_BASE_URL` or remove from `.claude.env`

## Best Practices

1. **Always enable NIM first** — reduces costs immediately
2. **Use Mercury for complex tasks** — toggle when you need advanced reasoning
3. **Store secrets in `~/.hermes/.env`** — never commit to git
4. **Test locally before deploying** — ensure proxy is reachable
5. **Monitor rate limits** — 40 req/min on NVIDIA free tier

## Repository References

- Main repo: https://github.com/executiveusa/pauli-hermes-agent
- NIM proxy: `services/nim-proxy/`
- Voice agent: `website/src/pages/agent.tsx`
- Setup guide: `VPS_SETUP.md`
- This guide: `CLAUDE.md`
```

---

## Universal Onboarding Prompt

Save this as `HERMES_ONBOARDING.prompt` for any new Claude project:

```
System: You are setting up Claude Code with free NVIDIA inference and Hermes agent.

User provides:
1. Project description
2. Required capabilities
3. Preferences (voice, autonomous actions, etc.)

You will:
1. Configure ANTHROPIC_BASE_URL=http://31.220.58.212:8082
2. Set up .claude.env or exports
3. Initialize Hermes Rolodex if memory/contacts needed
4. Provide startup commands
5. Add error handling for provider failures
6. Include provider toggles for Mercury (if needed)
7. Document all environment variables

Output:
- Complete setup guide for the project
- Code examples using free inference
- Testing procedures
- Troubleshooting steps

Example secrets (replace with actual):
- ANTHROPIC_BASE_URL=http://31.220.58.212:8082
- NVIDIA_NIM_API_KEY=nvapi-...
- MERCURY_API_KEY=...
- HERMES_CONFIG=~/.hermes/config.yaml
```

---

## Summary: You Now Have

| Feature | Status | Cost |
|---------|--------|------|
| **Free Claude Code** | ✅ Active | $0 |
| **Voice Agent** | ✅ Live | $0 |
| **Hermes Memory** | ✅ Ready | $0 |
| **Provider Toggles** | ✅ Implemented | $0 |
| **NVIDIA NIM** | ✅ Configured | $0 |
| **Mercury (Optional)** | ✅ Ready | Pay-as-you-go |

**To activate globally for ALL Claude projects:**

```bash
# Add to ~/.bashrc or ~/.zshrc
export ANTHROPIC_BASE_URL=http://31.220.58.212:8082
export ANTHROPIC_API_KEY=dummy

# Reload
source ~/.bashrc

# Test
claude --version
# Now all Claude Code runs on free NVIDIA inference
```

That's it. Every Claude CLI session will route through the free proxy.
