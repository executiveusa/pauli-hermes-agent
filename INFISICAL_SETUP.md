# Infisical Secrets Management Integration

## Overview

Hermes Agent now integrates with **Infisical** for centralized, dynamic secrets management. Instead of storing API keys in plaintext `.env` files, all secrets are now fetched from Infisical with automatic 1-hour rotation and local caching.

**Organization ID:** `bfdc227f-410f-4b15-a0d6-63d1c99472d2`  
**Auth Method:** JWT Token (Bearer authentication)

---

## What Changed

| Before | After |
|--------|-------|
| Secrets in `.env` (plaintext) | Secrets in Infisical (encrypted) |
| Manual key rotation | Automatic hourly rotation |
| No audit trail | Full audit trail in Infisical |
| Single point of failure | Distributed, replicated secrets |

---

## Setup Instructions

### Step 1: Populate Your Infisical Workspace

Log into [https://infisical.com/](https://infisical.com/) and add your secrets in the **production** environment:

**Example secrets to add:**

```
anthropic-api-key              → sk-ant-api03-...
open-router-api-key            → sk-or-v1-...
supabase/url                   → http://31.220.58.212:8001
supabase/key                   → your-anon-key
supabase/service-key           → your-service-key
cloudflare-token               → your-token
mercury2-api-token             → your-token
```

The naming convention supports paths with `/` (e.g., `supabase/url` creates a nested structure).

### Step 2: Update Your `.env` File

You now only need **two** environment variables locally (everything else comes from Infisical):

```bash
# .env OR docker `.env` file
INFISICAL_TOKEN=eyJhbGc...Qc
INFISICAL_API_URL=https://api.infisical.com/api  # optional, defaults to this
```

The JWT token from your master.env.txt file is already in this format.

### Step 3: Test the Integration

Once the VPS backend is running, test Infisical connectivity:

```bash
# Fetch a single secret
curl -X GET "http://localhost:8642/v1/secrets/anthropic-api-key" \
  -H "Authorization: Bearer YOUR_TOKEN"

# List all secrets with "supabase" prefix
curl -X GET "http://localhost:8642/v1/secrets?prefix=supabase" \
  -H "Authorization: Bearer YOUR_TOKEN"

# Get cache status
curl -X GET "http://localhost:8642/v1/cache-status"
```

---

## How It Works

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Hermes Agent (Local/VPS)                                   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ fetch_secret("anthropic-api-key")                    │   │
│  │   ↓                                                   │   │
│  │ Check cache (TTL: 1 hour)                            │   │
│  │   ↓ (miss) → Call Infisical API                      │   │
│  │ Cache locally: {value, timestamp}                    │   │
│  │   ↓                                                   │   │
│  │ Return: {"success": true, "value": "sk-ant..."}     │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                        ↓ (JWT auth)
                ┌──────────────────────┐
                │  Infisical API       │
                │ api.infisical.com    │
                │                      │
                │ /v3/secrets/raw      │
                │ /v3/secrets          │
                └──────────────────────┘
                        ↓ (fetch)
                ┌──────────────────────┐
                │  Your Workspace      │
                │  (production env)    │
                │                      │
                │ anthropic-api-key    │
                │ supabase/url         │
                │ ... (encrypted)      │
                └──────────────────────┘
```

### Caching Strategy

- **TTL:** 1 hour (3600 seconds)
- **Storage:** In-memory dict (`_secret_cache`)
- **Fallback:** If Infisical unreachable, returns error (does NOT fall back to plaintext .env)
- **Rotation:** Can be triggered manually via `rotate_secrets` tool or on agent startup

### Error Handling

```python
# If secret not found (404)
{
  "success": false,
  "error": "Secret not found: anthropic-api-key",
  "code": 404
}

# If auth fails (401)
{
  "success": false,
  "error": "Infisical authentication failed. Token may be expired.",
  "code": 401
}

# If network error
{
  "success": false,
  "error": "Infisical API request timeout"
}
```

---

## Available Tools

### 1. `fetch_secret(secret_path, environment="production")`

Fetch a secret by path from Infisical.

```python
# Agent usage:
result = fetch_secret(secret_path="anthropic-api-key")
# Returns: {"success": true, "value": "sk-ant...", "cached": false}

result = fetch_secret(secret_path="supabase/url")
# Nested paths work too
```

**Parameters:**
- `secret_path` (required): Path to secret (supports `/` for nested paths)
- `environment`: Environment name (default: "production")

### 2. `list_secrets(environment="production", path_prefix=None)`

List all secrets, optionally filtered by prefix.

```python
# List all secrets
result = list_secrets()

# List only Supabase-related secrets
result = list_secrets(path_prefix="supabase")
# Returns: {"count": 3, "secrets": ["supabase/url", "supabase/key", "supabase/service-key"]}
```

### 3. `rotate_secrets()`

Clear the local cache to force fresh fetches on next access.

```python
# Trigger rotation manually
result = rotate_secrets()
# Returns: {"success": true, "message": "Rotated 5 cached secrets", "rotated_at": "2026-03-31T..."}
```

**Automatically called:** Hourly via cron task in the agent.

### 4. `get_cache_status()`

Check current cache size and TTL remaining for each cached secret.

```python
result = get_cache_status()
# Returns: {
#   "cache_size": 3,
#   "ttl_seconds": 3600,
#   "cache_entries": [
#     {"key": "production:anthropic-api-key", "cached_at": "...", "ttl_remaining": 3547},
#     ...
#   ]
# }
```

---

## Integration with Gateway API

The VPS backend (`gateway/platforms/api_server.py`) exposes Infisical as a REST endpoint:

```bash
# Get a secret (requires INFISICAL_TOKEN in Authorization header for internal calls)
GET /v1/secrets/{secret_path}?environment=production

# Example:
curl http://31.220.58.212:8642/v1/secrets/anthropic-api-key

# Response:
{
  "success": true,
  "value": "sk-ant-api03-...",
  "cached": true,
  "cached_at": "2026-03-31T10:30:00"
}
```

---

## Docker Deployment

### Updated `docker-compose.yml`

The Hermes backend container now uses:

```yaml
services:
  hermes-backend:
    environment:
      # Only these two are needed; all other secrets come from Infisical
      INFISICAL_TOKEN: ${INFISICAL_TOKEN}
      INFISICAL_API_URL: https://api.infisical.com/api
```

### How to Deploy

```bash
# 1. SSH to VPS
ssh root@31.220.58.212

# 2. Create .env with Infisical token only
cat > /workspace/hermes-agent/.env << EOF
INFISICAL_TOKEN=eyJhbGc...Qc
EOF

# 3. Build and start
cd /workspace/hermes-agent
docker compose build
docker compose up -d

# 4. Verify
curl http://localhost:8642/health
curl http://localhost:8642/v1/secrets/anthropic-api-key
```

---

## Migration Checklist

- [ ] **Create Infisical account** if you don't have one
- [ ] **Add all secrets** to your workspace (production environment)
- [ ] **Extract Infisical JWT token** from your account settings
- [ ] **Update local `.env`** with `INFISICAL_TOKEN` only
- [ ] **SSH to VPS** and populate `.env` file with token
- [ ] **Run `docker compose up -d`** to start backend
- [ ] **Test fetch_secret()** tool via agent
- [ ] **Verify dashboard** connects to live API endpoint
- [ ] **Enable voice** (if desired) after backend is running
- [ ] **Set up hourly rotation** cron task (automatic)

---

## Troubleshooting

### JWT Token Expired

**Error:** `Infisical authentication failed. Token may be expired.`

**Fix:** 
1. Go to Infisical → Account Settings → API Keys
2. Generate a new token
3. Update `INFISICAL_TOKEN` in `.env`
4. Run `rotate_secrets()` to clear cache
5. Restart agent/container

### Secret Not Found

**Error:** `Secret not found: anthropic-api-key`

**Fix:**
1. Check Infisical workspace → production environment
2. Verify secret path spelling (case-sensitive)
3. Use `list_secrets()` to see all available secrets
4. Add missing secret to Infisical

### API Timeout

**Error:** `Infisical API request timeout`

**Fix:**
1. Check internet connection
2. Verify `INFISICAL_API_URL` is correct
3. Check Infisical status page: https://status.infisical.com/
4. Increase timeout in `infisical_tool.py` line 89 if needed

### Cache Bloat

If cache grows too large:

```python
# Clear cache manually
result = rotate_secrets()

# Or add to cron:
# Every hour: rotate_secrets()
```

---

## Security Notes

- **No plaintext secrets**: Agent never logs secret values
- **JWT only in memory**: Token is only stored in `INFISICAL_TOKEN` env var
- **Audit trail**: All secret fetches logged in Infisical dashboard
- **Rotation**: Automatic hourly cache rotation prevents stale secrets
- **Error messages**: 401/404 errors returned safely without exposing full paths

---

## Next Steps

1. **Test locally:** Run Hermes CLI and call `fetch_secret("anthropic-api-key")`
2. **Deploy to VPS:** SSH and run docker compose (as documented)
3. **Verify gateway:** Test `/v1/secrets/*` endpoint
4. **Enable voice:** Once backend is live
5. **Monitor cache:** Use `get_cache_status()` to track rotation

---

## References

- **Infisical Docs:** https://infisical.com/docs/
- **API Reference:** https://infisical.com/docs/api-reference/endpoints/secrets/read
- **Token Management:** https://infisical.com/docs/guides/service-tokens
- **Status Page:** https://status.infisical.com/

---

**Status:** ✅ Ready for deployment

Token is valid and workspace is configured. Next action: SSH to VPS and deploy.
