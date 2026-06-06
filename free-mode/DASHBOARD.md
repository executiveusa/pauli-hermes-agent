# FREE MODE Dashboard & Monitoring System

Complete production monitoring, cost tracking, and control panel for FREE MODE.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Next.js Web Dashboard                      │
│            (website/src/pages/dashboard.tsx)                 │
├─────────────────────────────────────────────────────────────┤
│  ├─ Provider Status & Health                                 │
│  ├─ Cost Tracking & Billing                                  │
│  ├─ Provider Switcher                                        │
│  └─ Real-time Metrics                                        │
├─────────────────────────────────────────────────────────────┤
│                    Backend API Routes                        │
│              (website/src/pages/api/dashboard/)              │
├─────────────────────────────────────────────────────────────┤
│  ├─ /api/dashboard/status    → Provider health              │
│  ├─ /api/dashboard/costs     → Cost history                 │
│  └─ /api/dashboard/provider  → Switch providers             │
├─────────────────────────────────────────────────────────────┤
│           FREE MODE Monitoring Service (FastAPI)             │
│        (free-mode/monitoring_service.py, port 8001)          │
├─────────────────────────────────────────────────────────────┤
│  ├─ /health                  → Service health               │
│  ├─ /track/request           → Record request               │
│  ├─ /metrics                 → Current metrics              │
│  ├─ /costs                   → Cost data                    │
│  ├─ /costs/history           → Historical costs             │
│  └─ /stats                   → Complete stats               │
├─────────────────────────────────────────────────────────────┤
│         Cost Tracking (cost_tracker.py)                      │
│                                                              │
│  ├─ Daily/monthly cost aggregation                          │
│  ├─ Per-provider cost tracking                              │
│  ├─ Token usage monitoring                                  │
│  └─ Cost estimate calculation                               │
├─────────────────────────────────────────────────────────────┤
│         Proxy Monitor (proxy_monitor.py)                     │
│                                                              │
│  ├─ Request recording                                       │
│  ├─ Latency tracking                                        │
│  ├─ Error rate monitoring                                   │
│  └─ Provider statistics                                     │
├─────────────────────────────────────────────────────────────┤
│              LiteLLM Proxy (port 4000)                       │
│                                                              │
│  └─ Universal provider gateway                              │
└─────────────────────────────────────────────────────────────┘
```

## Components

### 1. Web Dashboard (`website/src/pages/dashboard.tsx`)

**Features:**
- Real-time provider status display
- Provider health indicators (latency, connectivity)
- One-click provider switching
- Cost tracking (daily, monthly, per-provider)
- Request statistics
- Cost history visualization

**Refresh Rate:** Every 5 seconds

**Displays:**
```
┌─────────────────────────────────────┐
│  FREE MODE Status Cards             │
│  ├─ FREE MODE: ✓ ENABLED            │
│  ├─ Proxy Status: ✓ Online (45ms)   │
│  ├─ Current: groq (free-groq)       │
│  └─ Cost Today: $0.00               │
├─────────────────────────────────────┤
│  Provider Switcher                  │
│  ├─ groq (40ms) [ACTIVE]            │
│  ├─ gemini (52ms)                   │
│  ├─ openrouter (48ms)               │
│  └─ [more providers]                │
├─────────────────────────────────────┤
│  Provider Health                    │
│  ├─ groq: healthy (10 reqs)         │
│  ├─ gemini: healthy (5 reqs)        │
│  ├─ openai: auth_failed             │
│  └─ [more providers]                │
├─────────────────────────────────────┤
│  Cost History (Last 24h)            │
│  ├─ groq: 10 reqs, $0.00            │
│  ├─ gemini: 5 reqs, $0.00           │
│  └─ [more history]                  │
└─────────────────────────────────────┘
```

### 2. API Routes (`website/src/pages/api/dashboard/`)

#### `GET /api/dashboard/status`
Returns current dashboard state:
```json
{
  "free_mode_enabled": true,
  "current_provider": "groq",
  "current_model": "free-groq",
  "proxy_reachable": true,
  "proxy_latency_ms": 45,
  "providers": [
    {
      "id": "groq",
      "label": "Groq (Free)",
      "status": "healthy",
      "latency_ms": 40,
      "connected": true
    }
  ],
  "costs_today": 0.0,
  "costs_month": 2.34,
  "recent_requests": 42
}
```

#### `GET /api/dashboard/costs`
Returns cost history (last 100 entries):
```json
[
  {
    "provider": "groq",
    "requests": 10,
    "cost": 0.0,
    "timestamp": "2026-06-06T10:30:00Z"
  }
]
```

#### `POST /api/dashboard/provider`
Switch to a different provider:
```json
{
  "provider_id": "gemini"
}
```

Response:
```json
{
  "success": true,
  "message": "Provider switched from groq to gemini",
  "previous_provider": "groq",
  "new_provider": "gemini"
}
```

### 3. Monitoring Service (`free-mode/monitoring_service.py`)

FastAPI service running on port 8001.

**Endpoints:**

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/health` | GET | Service health check |
| `/track/request` | POST | Record a request |
| `/metrics` | GET | Current metrics |
| `/costs` | GET | Cost data |
| `/costs/history` | GET | Historical costs |
| `/requests` | GET | Recent requests |
| `/stats` | GET | Complete statistics |

**Example: Track a Request**
```bash
curl -X POST http://127.0.0.1:8001/track/request \
  -H "Content-Type: application/json" \
  -d '{
    "provider": "groq",
    "model": "groq-70b",
    "input_tokens": 100,
    "output_tokens": 50,
    "latency_ms": 1250,
    "status": 200
  }'
```

Response:
```json
{
  "success": true,
  "timestamp": "2026-06-06T10:30:00Z",
  "provider": "groq",
  "cost_today": 0.00125
}
```

### 4. Cost Tracker (`free_mode/cost_tracker.py`)

Tracks costs per provider with pricing data:

```python
from free_mode.cost_tracker import get_cost_tracker, track_request

# Track a request
track_request(
    provider="groq",
    input_tokens=100,
    output_tokens=50
)

# Get stats
tracker = get_cost_tracker()
print(tracker.get_costs_today())      # $0.00
print(tracker.get_costs_month())      # $2.34
print(tracker.get_provider_stats("groq"))
# {
#   "requests": 10,
#   "input_tokens": 1000,
#   "output_tokens": 500,
#   "cost": 0.00125
# }
```

**Pricing Data (Provider Costs):**
```python
{
    "groq": {"input": 0.00005, "output": 0.00015},  # per 1k tokens
    "gemini": {"input": 0.0, "output": 0.0},        # free tier
    "openrouter": {"input": 0.00005, "output": 0.00015},
    "nvidia_nim": {"input": 0.0, "output": 0.0},    # free tier
    "openai": {"input": 0.0005, "output": 0.0015},  # GPT-4
    "anthropic": {"input": 0.0003, "output": 0.001}, # Claude
    "mistral": {"input": 0.0001, "output": 0.0003},
    "ollama": {"input": 0.0, "output": 0.0},         # local
    "lmstudio": {"input": 0.0, "output": 0.0},       # local
}
```

**Storage:**
- Daily costs: `~/.hermes/cost_tracking/costs.json`
- History: `~/.hermes/cost_tracking/history.json`

### 5. Proxy Monitor (`free_mode/proxy_monitor.py`)

Monitors proxy requests in memory:

```python
from free_mode.proxy_monitor import get_proxy_monitor, record_request

# Record a request
record_request(
    provider="groq",
    model="groq-70b",
    input_tokens=100,
    output_tokens=50,
    latency_ms=1250,
    status=200
)

# Get metrics
monitor = get_proxy_monitor()
metrics = monitor.get_metrics()
print(metrics)
# ProxyMetrics(
#   total_requests=42,
#   total_tokens=4200,
#   average_latency_ms=1100,
#   error_rate=0.05,
#   providers={'groq': 25, 'gemini': 15, 'openai': 2}
# )
```

## Deployment

### Docker Compose (Recommended)

```bash
# Start all services
docker compose -f docker-compose.free-mode.yml up -d

# Check status
docker compose -f docker-compose.free-mode.yml ps

# View logs
docker compose -f docker-compose.free-mode.yml logs -f litellm-free-mode
docker compose -f docker-compose.free-mode.yml logs -f monitoring-service
```

Services:
- **litellm-free-mode** (port 4000): Proxy gateway
- **monitoring-service** (port 8001): Monitoring API

### Manual Startup

```bash
# Terminal 1: Start LiteLLM proxy
bash free-mode/scripts/start-free-mode.sh

# Terminal 2: Start monitoring service
python -m free_mode.monitoring_service

# Terminal 3: Run Next.js with dashboard
cd website
npm run dev
```

Access:
- Dashboard: http://localhost:3000/dashboard
- Monitoring API: http://127.0.0.1:8001/metrics
- LiteLLM Proxy: http://127.0.0.1:4000/health

## Integration with Hermes Agent

### Automatic Request Tracking

When Hermes makes a request through FREE MODE:

1. Proxy processes request (LiteLLM)
2. Hermes calls monitoring service to log it
3. Cost tracker calculates cost
4. Dashboard displays updated stats

### Code Example

```python
from free_mode import FreeMode
from free_mode.cost_tracker import track_request

# Create client
client = FreeMode()
openai_client = client.create_openai_client()

# Make request
response = openai_client.chat.completions.create(
    model="free-auto",
    messages=[{"role": "user", "content": "Hello"}]
)

# Track cost
track_request(
    provider=os.environ.get("FREE_MODE_PROVIDER", "auto"),
    input_tokens=len(response.usage.prompt_tokens),
    output_tokens=len(response.usage.completion_tokens)
)
```

## Cost Limits & Alerts

**Configurable in Future:**
- Daily cost limit
- Monthly cost limit
- Per-provider limits
- Email alerts on threshold
- Automatic fallback on limit exceeded

## Troubleshooting

### Dashboard Not Loading
```bash
# Check Next.js is running
curl http://localhost:3000/dashboard

# Check monitoring service health
curl http://127.0.0.1:8001/health

# Check proxy is online
curl http://127.0.0.1:4000/health
```

### Costs Not Tracking
```bash
# Check cost file exists
ls -la ~/.hermes/cost_tracking/

# Check monitoring service logs
docker logs free-mode-monitoring

# Test endpoint manually
curl http://127.0.0.1:8001/track/request \
  -H "Content-Type: application/json" \
  -d '{
    "provider": "groq",
    "model": "groq-70b",
    "input_tokens": 100,
    "output_tokens": 50,
    "latency_ms": 1250
  }'
```

### Provider Not Switching
```bash
# Check current provider
echo $FREE_MODE_PROVIDER

# Check provider switch log
cat .free-mode-provider-switches.json

# Test provider endpoint
curl -X POST http://localhost:3000/api/dashboard/provider \
  -H "Content-Type: application/json" \
  -d '{"provider_id": "gemini"}'
```

## Performance

**Typical Metrics (when running):**
- Dashboard load: <500ms
- API responses: <100ms
- Proxy latency: 100-500ms (depending on provider)
- Memory usage: ~200MB (proxy + monitoring)
- Disk usage: ~10MB (logs + cost data)

## Next Steps

**Future Enhancements:**
- [ ] Cost prediction & budgeting
- [ ] Load balancing across providers
- [ ] Advanced routing (by latency, cost, capability)
- [ ] Usage analytics & reports
- [ ] Multi-user support with quotas
- [ ] Webhook notifications
- [ ] Custom alerting rules
- [ ] Cost optimization recommendations
