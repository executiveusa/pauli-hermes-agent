# FREE MODE Security Guidelines

## Threat Model

### Scope: What We Protect

- **API Keys**: Prevent leakage to logs, PRs, or unsecured proxies
- **Subscription Cost**: Ensure no charged requests sneak through FREE MODE
- **Admin Routes**: Prevent unauthorized proxy admin access
- **Request Payloads**: Disable logging of user input unless explicitly enabled

### Out of Scope

- Protection against compromised local machine
- Protection against network eavesdropping (use HTTPS in production)
- Protection against malicious upstream providers

## Key Security Decisions

### 1. Local-Only Proxy by Default

```bash
# Binds to 127.0.0.1 by default — only accessible from this machine
LITELLM_HOST=127.0.0.1
LITELLM_PORT=4000

# To expose to other machines:
# LITELLM_HOST=0.0.0.0  # ⚠️ Requires authentication
```

**Why**: Prevents accidental exposure of proxy to internet before authentication is set up.

### 2. Master Key Authentication

```bash
# Every request must include this key
LITELLM_MASTER_KEY=change-me-local-master-key  # ⚠️ CHANGE THIS

# Header: Authorization: Bearer {LITELLM_MASTER_KEY}
```

**Why**: Prevents unauthorized calls through the proxy.

**Action Required**: Change from default before exposing proxy to untrusted networks.

### 3. No Secrets in Repo

```bash
# ✅ Good: .gitignore blocks .env
echo ".env" >> .gitignore

# ✅ Good: Placeholder only, no real value
export OPENAI_API_KEY=  # Empty by default

# ❌ Bad: Never commit real keys
# export OPENAI_API_KEY=sk-...  # NEVER
```

**Why**: Prevents accidental secret leaks in version control.

**Enforcement**: All `.env.example` vars are placeholders only. Use `.env` (not tracked) for secrets.

### 4. Request Payload Logging Disabled by Default

```bash
# Disabled by default — only enable for debugging
FREE_MODE_DEBUG_PAYLOADS=false

# When enabled:
# - Request bodies are logged
# - User input and prompts are visible in logs
# - Sensitive data may appear in Docker logs
# ⚠️ Only enable in development, never in production
```

**Why**: Prevents user prompts and sensitive data leaking into logs.

### 5. Admin UI Disabled by Default

```bash
# LiteLLM admin routes (e.g., /model/info) are disabled
FREE_MODE_ADMIN_ENABLED=false

# When enabled:
# - Only bind to 127.0.0.1 (localhost)
# - Require separate authentication
FREE_MODE_ADMIN_PORT=4001
```

**Why**: Prevents enumeration of configured keys and model info via admin API.

## Secrets Checklist

Before deploying FREE MODE to production:

- [ ] Change `LITELLM_MASTER_KEY` from default
- [ ] Set `.env` in `.gitignore` (not version-controlled)
- [ ] Never paste real API keys into `.env.example`
- [ ] Verify `FREE_MODE_DEBUG_PAYLOADS=false`
- [ ] Verify `FREE_MODE_ADMIN_ENABLED=false`
- [ ] Set `LITELLM_TELEMETRY=false` (don't send requests to LiteLLM service)
- [ ] Use HTTPS for remote proxy (not covered here)

## Provider-Specific Risks

### OpenAI

```bash
# Risk: If OPENAI_API_KEY leaks, attacker can charge to your account
# Mitigation:
# 1. Use API key with spend limits (https://platform.openai.com/account/billing/limits)
# 2. Monitor usage (https://platform.openai.com/account/billing/overview)
# 3. Rotate key if exposed: https://platform.openai.com/account/api-keys
```

### Anthropic

```bash
# Risk: If ANTHROPIC_API_KEY leaks, attacker can charge to your account
# Mitigation: Same as OpenAI — enable spend limits and monitor
```

### Free Tier Providers (Groq, Gemini, etc.)

```bash
# Risk: Rate limiting (e.g., Groq 40 req/min)
# Mitigation: Proxy will queue requests; rate limiting is per API key
```

### Local Models (Ollama, LM Studio)

```bash
# Risk: Model files are stored locally — ensure /disk is not shared
# No API key leakage possible (runs locally)
```

## Environment Setup

### Recommended: .env File (Not Tracked)

```bash
# .env (in .gitignore)
FREE_MODE=true
FREE_MODE_PROXY_BASE_URL=http://127.0.0.1:4000
LITELLM_MASTER_KEY=your-random-key-here-32-chars-min
OPENAI_API_KEY=sk-...  # Real secret
GROQ_API_KEY=gsk-...   # Real secret
```

### Alternative: Environment Variables

```bash
# Load from secure store or secrets manager
# NOT recommended: export in shell history or in code
```

### Never

```bash
# ❌ DO NOT: Commit .env
git add .env  # Will be rejected by pre-commit hook

# ❌ DO NOT: Put secrets in .env.example
FREE_MODE_PROXY_MASTER_KEY=sk-real-key-12345  # Never!

# ❌ DO NOT: Log payloads in production
FREE_MODE_DEBUG_PAYLOADS=true  # Only dev!
```

## Rotation

### Master Key

```bash
# When to rotate:
# - Quarterly (best practice)
# - If exposed or suspected exposure
# - When an admin leaves the team

# Steps:
# 1. Generate new key
openssl rand -hex 16  # 32-char key

# 2. Update in .env
export LITELLM_MASTER_KEY=newkey123456789abcdef

# 3. Restart proxy
docker compose -f docker-compose.free-mode.yml restart litellm-free-mode

# 4. Update any clients pointing to this proxy
export ANTHROPIC_AUTH_TOKEN=$LITELLM_MASTER_KEY
```

### Provider API Keys

```bash
# For each provider, follow their key rotation guidance:

# Groq: https://console.groq.com/keys → Delete old, create new
# Gemini: https://aistudio.google.com/app/apikey → Delete old, create new
# OpenRouter: https://openrouter.ai/keys → Delete old, create new

# After rotating:
# 1. Update .env
export GROQ_API_KEY=newgskkey...

# 2. Restart proxy
docker compose -f docker-compose.free-mode.yml restart litellm-free-mode

# 3. Verify health
curl http://127.0.0.1:4000/health
```

## Disabling FREE MODE

### Temporary

```bash
# Unset or set to false
unset FREE_MODE
# OR
export FREE_MODE=false

# Hermes will revert to original provider config
```

### Permanent (Remove System)

```bash
# Delete proxy container
docker compose -f docker-compose.free-mode.yml down

# Delete FREE MODE files (optional)
rm -rf free-mode/
rm docker-compose.free-mode.yml

# Restore original .env (remove FREE_MODE vars)
# Edit .env and delete all FREE_MODE_* and LITELLM_* lines
```

## Audit

### Check What's Running

```bash
# See active proxy
docker ps | grep litellm

# See configured keys (⚠️ shows actual keys!)
docker exec free-mode-litellm cat /app/free-mode/litellm.config.yaml | grep -i "api_key"

# See environment vars in proxy
docker exec free-mode-litellm env | grep -E "^(FREE_MODE|LITELLM|OPENAI|GROQ|GEMINI)"
```

### Monitor Requests

```bash
# View recent requests
docker logs free-mode-litellm | grep -i "request\|completion"

# Enable detailed logging
export LITELLM_LOG=DEBUG
docker compose -f docker-compose.free-mode.yml restart litellm-free-mode
docker logs -f free-mode-litellm

# Disable detailed logging in production
export LITELLM_LOG=INFO
```

### Billing Alerts

Set up alerts for paid providers:

- **OpenAI**: https://platform.openai.com/account/billing/alerts
- **Anthropic**: Account dashboard → Usage alerts
- **AWS**: https://console.aws.amazon.com/billing
- **Google Cloud**: https://cloud.google.com/billing

Example threshold: Alert if spend > $10/day

## Compliance Notes

### PII / User Data

```bash
# If handling PII (user names, emails, etc.):
# 1. Disable payload logging (default)
# 2. Use local providers when possible (Ollama, LM Studio)
# 3. Never send PII to free-tier services without consent
# 4. Check each provider's privacy policy

# Example: GDPR-compliant setup
# - Use local Ollama (data stays local)
# - No free cloud providers (avoid 3rd party processing)
# - Set FREE_MODE_DEBUG_PAYLOADS=false (no logs)
```

### Open Source License

Hermes Agent is MIT licensed. FREE MODE module is also MIT licensed.

When using free-tier providers, check their Terms of Service:
- **Groq**: Free tier has service terms
- **Gemini**: Free tier has daily quota
- **OpenRouter**: Free models use model licenses

## Incident Response

### If API Key Leaked

1. **Immediately rotate the key**:
   ```bash
   # Provider's dashboard → delete key → create new key
   # Update .env with new key
   # Restart proxy
   ```

2. **Check for unauthorized usage**:
   ```bash
   # View recent requests to that API
   # Check billing for unexpected charges
   # Contact provider if major leak
   ```

3. **Log incident**:
   - Record when key was exposed
   - How many requests before rotation
   - Whether costs incurred

### If Proxy Compromised

1. **Disconnect proxy from network**:
   ```bash
   docker compose -f docker-compose.free-mode.yml down
   ```

2. **Rotate all secrets**:
   - `LITELLM_MASTER_KEY`
   - All provider API keys
   - Regenerate strong values

3. **Audit logs**:
   ```bash
   docker logs free-mode-litellm | tail -1000
   ```

## Questions?

- **General**: See `free-mode/README.md`
- **Troubleshooting**: See `free-mode/README.md#Troubleshooting`
- **LiteLLM**: https://docs.litellm.ai/
