# HERMES 3.0 DEPLOYMENT ISSUES — VISUAL MAP & DESIGN SOLUTIONS

## SYSTEM ARCHITECTURE DIAGRAM

```
┌─────────────────────────────────────────────────────────────────┐
│                   USER'S BROWSER (HTTPS)                        │
│  https://pauli-hermes-agent.vercel.app/dashboard                │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Archon X Dashboard (web-ui/index.html — 2500+ LOC)       │   │
│  │  ✅ 5 Panels: Chat, Agents, Tasks, Deploys, Memory        │   │
│  │  ✅ Voice button wired  (Web Speech API)                  │   │
│  │  ✅ Settings UI with API URL configuration              │   │
│  │  ⚠️  ISSUE #1: Voice blocked (HTTPS + Permissions)       │   │
│  │  ⚠️  ISSUE #2: Cache stale (browser serving old code)    │   │
│  │  ⚠️  ISSUE #5: API URL empty (not configured)            │   │
│  └──────────────────────────────────────────────────────────┘   │
│                            ↓ (HTTP/JSON)                        │
│                    [Settings → API URL]                         │
│                  https://31.220.58.212:8642                     │
└─────────────────────────────────────────────────────────────────┘
                                 ↓
┌─────────────────────────────────────────────────────────────────┐
│                   VPS (31.220.58.212)                           │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Docker Container: Hermes Agent (Port 8642)              │   │
│  │  ⚠️  ISSUE #3: NOT RUNNING (docker compose not started)  │   │
│  │  ⚠️  ISSUE #4: API endpoints untested                    │   │
│  │                                                          │   │
│  │  Files:                                                 │   │
│  │  ✅ docker-compose.yml (ready)                          │   │
│  │  ✅ Dockerfile (ready)                                  │   │
│  │  ❌ .env (missing — credentials not populated)          │   │
│  │  ✅ .env.vps.example (template exists)                  │   │
│  │                                                          │   │
│  │  Endpoints (10+):                                       │   │
│  │  ✅ /health (defined, untested)                         │   │
│  │  ✅ /v1/chat/completions (streaming)                    │   │
│  │  ✅ /v1/agents (agent registry)                         │   │
│  │  ✅ /v1/tasks (beads tracking)                          │   │
│  │  ✅ /v1/memories (Open Brain search)                    │   │
│  │  ✅ /v1/tools, /v1/skills, /v1/sessions                 │   │
│  │  + more...                                              │   │
│  └──────────────────────────────────────────────────────────┘   │
│                            ↓                                     │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Supabase (Port 8001)                                    │   │
│  │  PostgreSQL + PostgREST + Real-time                      │   │
│  │  ✅ Database ready     (second_brain schema)             │   │
│  │  ✅ REST API up        (http://31.220.58.212:8001)       │   │
│  │  ⚠️  Needs: SUPABASE_KEY in .env                         │   │
│  └──────────────────────────────────────────────────────────┘   │
│                            ↓                                     │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  External APIs (Cloud)                                   │   │
│  │  ✅ Anthropic Claude (requires API key in .env)          │   │
│  │  ✅ OpenRouter Mercury 2 (optional, faster inference)    │   │
│  │  ⚠️  API keys NOT in .env yet → models unavailable       │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## ISSUE DEPENDENCY GRAPH

```
┌──────────────────────────────────────────────────────────────────┐
│ BLOCKER: Issue #3 (VPS not running)                              │
│ ↓ docker-compose up -d needs .env populated                      │
│ Blocks: Issue #4 (API endpoints can't be tested)                 │
│         Issue #1 (Voice can't reach backend)                     │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│ Issue #1: Voice Access (HTTPS + Microphone Permissions)          │
│ ├── Browser needs: HTTPS (✅ Vercel provides)                    │
│ ├── User needs: Grant mic permission (❌ User action)            │
│ ├── Backend needs: Running VPS (❌ Issue #3)                     │
│ └── Depends on: Issue #3 resolved                                │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│ Issue #2: Browser Cache (Old code served)                        │
│ └── Fix: Ctrl+Shift+R hard refresh + clear localStorage          │
│         (Independent of VPS deployment)                          │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│ Issue #5: Dashboard API URL not set                              │
│ └── Depends on: Issue #3 (VPS IP address)                        │
│     Fix: User manually sets in Settings                          │
└──────────────────────────────────────────────────────────────────┘
```

---

## RESOLUTION SEQUENCE (CRITICAL PATH)

### Phase 1: SSH & ENV (5 min)
```
ssh root@31.220.58.212
cd /opt/hermes
cp .env.vps.example .env
nano .env  ← POPULATE WITH:
  - ANTHROPIC_API_KEY
  - SUPABASE_KEY
  - MERCURY_2_API_TOKEN (optional)
```

### Phase 2: Deploy (5 min)
```
docker compose build --no-cache
docker compose up -d
docker ps  ← Verify 2 containers running
curl http://localhost:8642/health  ← Test locally
```

### Phase 3: Dashboard Config (2 min)
```
Browser: https://pauli-hermes-agent.vercel.app/dashboard
Settings → Hermes API URL: https://31.220.58.212:8642
Click "Save & reconnect"
Header should show GREEN (Connected)
```

### Phase 4: Voice Test (1 min)
```
Browser asks: "Allow microphone access?" → Click Allow
Dashboard shows "Voice ready" (blue dot)
Click 🎤 button
Speak: "Hello Hermes"
Text appears in input field
```

### Phase 5: API Testing (5 min)
```
curl http://31.220.58.212:8642/health
curl http://31.220.58.212:8642/v1/agents
curl http://31.220.58.212:8642/v1/tools
... test all 12 endpoints
```

---

## CODE CHANGES COMPLETED

### vercel.json
✅ Updated cache headers:
```json
{
  "source": "/dashboard(/|$)",
  "headers": [
    { "key": "Cache-Control", "value": "public, max-age=3600, must-revalidate" },
    { "key": "Permissions-Policy", "value": "microphone=(self), geolocation=()" }
  ]
}
```

### .env.vps.example
✅ Updated with Supabase + Mercury 2 sections:
```
SUPABASE_URL=http://31.220.58.212:8001
SUPABASE_KEY=
MERCURY_2_API_TOKEN=
```

### web-ui/index.html
✅ Already contains:
- Voice controls (lines 380-391)
- Settings UI (lines 519-526)
- All 5 panel code
- API client functions
- Command palette
- Toast notifications

**No changes needed** — code is complete, just needs VPS running.

---

## VOICE IMPLEMENTATION DETAILS

### How Voice Works (Architecture)

```
User clicks 🎤 button
    ↓
Browser checks: navigator.mediaDevices.getUserMedia()
    ↓
Browser prompts: "Allow microphone?" 
    ↓
User clicks "Allow"
    ↓
new SpeechRecognition() instance created
    ↓
recognition.start()
    ↓
User speaks into microphone
    ↓
Browser converts audio → text (local, no upload)
    ↓
JavaScript event: recognition.onresult()
    ↓
Text inserted into chatInput field
    ↓
speech-to-text auto-sends to backend OR
User manually clicks Send
    ↓
Hermes processes & responds
```

### Why Voice "Doesn't Work"

**Blockers in order of likelihood:**

1. **User didn't grant microphone permission** (Very common)
   - Fix: Open browser Settings → Microphone → Allow pauli-hermes-agent.vercel.app

2. **Browser tab is not using HTTPS** (Vercel uses HTTPS, but mixed-content warning?)
   - Fix: Ensure URL bar shows 🔒 (green lock), not ⚠️

3. **Another browser tab using microphone**
   - Fix: Close other tabs with mics (Google Meet, Discord, etc.)

4. **User's microphone hardware disconnected or muted**
   - Fix: Test mic in system settings; unmute hardware

5. **Web Speech API not supported** (Very old browsers)
   - Fix: Use Chrome/Edge/Safari (not Firefox on some systems)

6. **Hermes backend not running** (VPS issue)
   - Fix: Deploy VPS (Phase 2 above)

---

## DESIGN IMPROVEMENTS (Next Phase)

### Voice Status Indicator
```
Current: "Voice ready" (text)
Upgrade: Visual indicator
  ✅ Green pulsing dot when listening
  ❌ Red when blocked
  ⏳ Yellow when permission pending
```

### Error Handling
```
Current: Silent failure if browser doesn't support Speech API
Upgrade: 
  - Show banner: "Voice not supported in your browser"
  - Offer fallback: "Use text input instead"
```

### Voice Feedback
```
Current: Text appears in input field
Upgrade:
  - Voice transcription in real-time
  - Waveform visualization
  - Transcript confidence score
  - Option to edit before sending
```

---

## CHECKLISTS

### USER CHECKLIST (Pre-Deployment)
- [ ] VPS SSH credentials ready
- [ ] Anthropic API key obtained (https://console.anthropic.com)
- [ ] Supabase key ready (from Supabase dashboard)
- [ ] OpenRouter token ready (optional, for Mercury 2)
- [ ] Microphone working on your machine
- [ ] Browser updated to latest version

### VPS CHECKLIST (Post-Deployment)
- [ ] SSH connection works
- [ ] .env file created and populated
- [ ] Docker image built
- [ ] 2 containers running (hermes, agent-zero)
- [ ] Health endpoint responds
- [ ] Ports 8642 & 8643 accessible from internet

### DASHBOARD CHECKLIST (Verification)
- [ ] Vercel URL loads: https://pauli-hermes-agent.vercel.app/dashboard
- [ ] Settings shows: API URL set to https://31.220.58.212:8642
- [ ] Header shows: Green "Connected" status
- [ ] Chat sends message and gets response
- [ ] Voice button enabled (not grayed out)
- [ ] Microphone permission granted in browser
- [ ] Voice input converts speech to text

---

## ESTIMATED TIME TO FULL OPERATIONALIZATION

| Task | Time | Blocker? |
|------|------|----------|
| SSH + .env population | 5 min | 🔴 YES |
| Docker build + deploy | 5 min | 🔴 YES |
| VPS health check | 2 min | 🔴 YES |
| Dashboard config | 2 min | 🟡 No (workaround: manual) |
| Voice permission grant | 1 min | 🟡 No (user action) |
| Full API endpoint test | 5 min | 🟢 No (optional validation) |
| **TOTAL** | **20 min** | — |

---

## FAILURE MODES & RECOVERY

| Failure | Symptom | Recovery |
|---------|---------|----------|
| docker compose fails | Port 8642 not listening | Check .env vars; `docker logs hermes` |
| API returns 401 | Dashboard auth error | Verify API_SERVER_KEY in both .env and dashboard |
| Voice still silent | Button pulsing but no text | Check browser console (F12); verify HTTPS |
| Memory searches fail | No results in Brain panel | Check Supabase URL + key in Settings |
| Models fail (hallucination) | Agent generates nonsense | Verify ANTHROPIC_API_KEY is valid |

---

**Document Status:** Complete  
**Last Updated:** 2026-03-31  
**Ready for:** User Implementation
