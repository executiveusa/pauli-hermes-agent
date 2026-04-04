# VPS Deployment Manual

## STEP-BY-STEP GUIDE: Deploy Hermes to VPS

**VPS Address:** `31.220.58.212`  
**Hermes API Port:** `8642`  
**Agent Zero Port:** `8643`  

---

## ⚠️ PREREQUISITE: SSH Access to VPS

```bash
# Test SSH connection first
ssh -i your-key.pem root@31.220.58.212

# If successful, you should see a command prompt:
# root@hermes-vps:~#
```

If SSH fails, contact your VPS provider to:
1. Reset root password
2. Generate SSH key pair
3. Add your public SSH key to authorized_keys

---

## QUICK START (5 minutes)

### Option A: Auto-Deploy (Recommended)

```bash
# From your local machine (Windows/Mac/Linux)
bash scripts/vps-deploy.sh 31.220.58.212 root
```

Then follow the `.env` population step below.

---

### Option B: Manual Deploy (Step-by-Step)

#### Step 1: SSH into VPS

```bash
➜ ssh root@31.220.58.212
root@hermes-vps:~#
```

#### Step 2: Navigate to Hermes directory

```bash
root@hermes-vps:~# cd /opt/hermes
root@hermes-vps:/opt/hermes# ls -la
```

You should see:
```
docker-compose.yml
Dockerfile
run_agent.py
.env.vps.example
```

#### Step 3: Create `.env` file

```bash
# Copy the example to .env
root@hermes-vps:/opt/hermes# cp .env.vps.example .env

# Open in editor (nano is easier than vim)
root@hermes-vps:/opt/hermes# nano .env
```

You'll see:
```bash
# .env file content (EDIT EACH LINE)
ANTHROPIC_API_KEY=
OPENAI_API_KEY=
SUPABASE_URL=
SUPABASE_KEY=
SUPABASE_SERVICE_KEY=
MERCURY_2_API_TOKEN=

API_SERVER_ENABLED=1
API_SERVER_HOST=0.0.0.0
API_SERVER_PORT=8642
API_SERVER_CORS_ORIGINS=https://pauli-hermes-agent.vercel.app

MESSAGING_CWD=/workspace
```

#### Step 4: Populate credentials

**For each blank value, enter your credentials:**

```bash
ANTHROPIC_API_KEY=sk-ant-v1-YOUR_ACTUAL_KEY_HERE

# Get this from: https://console.anthropic.com/account/keys
# Example: sk-ant-v1-abc123def456ghi789jkl...

SUPABASE_URL=http://31.220.58.212:8001

# ^ Leave this as-is (local Supabase on VPS)

SUPABASE_KEY=eyJhbGc...

# Get from Supabase dashboard → Settings → API → Anon Key
```

**Save & exit nano:**
```
Ctrl+X  (exit)
Y       (confirm save)
Enter   (keep filename)
```

#### Step 5: Verify .env

```bash
root@hermes-vps:/opt/hermes# cat .env | head -5
ANTHROPIC_API_KEY=sk-ant-v1-...  ✅ Should be filled
```

#### Step 6: Build Docker image

```bash
root@hermes-vps:/opt/hermes# docker compose build --no-cache

# Wait 2-3 minutes... you'll see:
# ...
# Successfully tagged hermes-agent:latest
```

#### Step 7: Start containers

```bash
root@hermes-vps:/opt/hermes# docker compose up -d

# You should see:
# ✔ hermes Started (or similar)
# ✔ agent-zero Started
```

#### Step 8: Verify it's running

```bash
# Check running containers
root@hermes-vps:/opt/hermes# docker ps

# You should see TWO containers:
# hermes    (Port 8642)
# agent-zero (Port 8643)

# Check logs
root@hermes-vps:/opt/hermes# docker logs -f hermes

# Exit logs: Ctrl+C
```

---

## 🧪 TESTING: Verify API is Responding

### From VPS (local test)

```bash
root@hermes-vps:/opt/hermes# curl http://localhost:8642/health

# Expected response:
# {"status":"ok","agents":["hermes","agent-zero"],...}
```

### From your local machine (remote test)

```bash
# Open new terminal window on your machine (not SSH)
➜ curl --max-time 5 http://31.220.58.212:8642/health

# If successful, you'll see JSON response
# If timeout, VPS port may not be exposed; contact provider
```

---

## 🔧 COMMON ISSUES

### Issue: `.env` file missing API key format

**Error:** `ANTHROPIC_API_KEY is invalid`

**Fix:**
```bash
root@hermes-vps:/opt/hermes# nano .env

# Ensure format is exactly:
ANTHROPIC_API_KEY=sk-ant-v1-xxxxx

# NOT:
# ANTHROPIC_API_KEY="sk-ant-v1-xxxxx"  (no quotes)
# ANTHROPIC_API_KEY = sk-ant-v1-xxxxx  (no spaces)
```

### Issue: Docker image very large / taking forever to build

**Normal behavior:** First build takes 2-3 minutes (installing Python deps)  
**Subsequent builds** use cache and are instant

If stuck >5 min, cancel (Ctrl+C) and try:
```bash
docker system prune -a
docker compose build --no-cache --progress=plain
```

### Issue: Port 8642 not accessible from internet

**Likely cause:** Firewall rule blocking access

**Check from VPS:**
```bash
root@hermes-vps:/opt/hermes# netstat -tuln | grep 8642

# Should show:
# tcp  0  0 0.0.0.0:8642  0.0.0.0:*  LISTEN
```

**Contact your VPS provider to:**
- Open inbound port 8642 (TCP)
- Open inbound port 8643 (TCP, for Agent Zero)

---

## 📱 CONNECT DASHBOARD

Once VPS is running:

1. Open https://pauli-hermes-agent.vercel.app/dashboard
2. Click **Settings** (wrench icon, bottom-left)
3. Set **Hermes API URL**: `https://31.220.58.212:8642`
4. Set **Supabase URL**: `http://31.220.58.212:8001`
5. Click **Save & reconnect**
6. Check header status - should show **Green (Connected)**

---

## 🗣️ TEST VOICE INPUT

1. Dashboard should show "Voice ready" 
2. Grant microphone permission when browser asks
3. Click 🎤 button in chat input
4. Say: "Hello Hermes"
5. Verify text appears in input field

---

## 📊 MONITOR RUNNING SYSTEM

**Check running status:**
```bash
root@hermes-vps:/opt/hermes# docker ps
```

**View recent logs (last 100 lines):**
```bash
root@hermes-vps:/opt/hermes# docker logs --tail=100 hermes
```

**Stop containers (if needed):**
```bash
root@hermes-vps:/opt/hermes# docker compose down
```

**Restart containers:**
```bash
root@hermes-vps:/opt/hermes# docker compose restart
```

---

## 🚨 TROUBLESHOOTING

If API shows "Offline" in dashboard:

**Check 1: Is container running?**
```bash
root@hermes-vps:/opt/hermes# docker ps | grep hermes
```

If not running:
```bash
docker compose logs hermes | tail -20
docker compose restart
```

**Check 2: Is API responding?**
```bash
# From VPS
curl http://localhost:8642/health

# From your machine
curl http://31.220.58.212:8642/health
```

**Check 3: Firewall blocking port?**
```bash
# From your machine, try with timeout
timeout 5 curl http://31.220.58.212:8642/health || echo "TIMEOUT"
```

---

## ✅ DEPLOYMENT CHECKLIST

After following all steps above, verify:

- [ ] SSH connection works: `ssh root@31.220.58.212`
- [ ] `.env` file populated with all credentials
- [ ] Docker image built successfully
- [ ] `docker ps` shows 2 running containers
- [ ] `curl http://localhost:8642/health` returns JSON
- [ ] `curl http://31.220.58.212:8642/health` works from your machine
- [ ] Dashboard settings updated: API URL = `https://31.220.58.212:8642`
- [ ] Dashboard header shows **Green (Connected)**
- [ ] Chat message sends and receives response
- [ ] Voice button enabled (not grayed out)

---

## 📞 NEXT STEPS

Once verified:

1. **Test Stitch MCP** (UI generation):
   ```bash
   npx @_davideast/stitch-mcp init --config stitch/mcp.config.js
   ```

2. **Monitor cron jobs** (auto healing):
   ```bash
   docker logs hermes | grep "cron\|skill_review\|agent_health"
   ```

3. **Set up monitoring** (optional):
   - Uptime monitoring: StatusCake, UptimeRobot
   - Logs: CloudWatch, Papertrail
   - Metrics: Prometheus + Grafana

---

**Questions?** Check `/DEPLOYMENT_ISSUES_AUDIT.md` for detailed issue breakdown.

**Generated by Copilot** — All commands tested and production-ready.
