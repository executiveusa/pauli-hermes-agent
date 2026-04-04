# 🚀 HERMES 3.0 QUICK START CARD
**Print this. Follow in order. 20 minutes to full deployment.**

---

## STEP 1: SSH INTO VPS (1 min)
```bash
ssh root@31.220.58.212
```
✅ You should see: `root@hermes-vps:/opt/hermes#`

---

## STEP 2: CREATE .env FILE (2 min)
```bash
cd /opt/hermes
cp .env.vps.example .env
nano .env
```

**FIND & FILL THESE 3 LINES ONLY:**

```bash
ANTHROPIC_API_KEY=sk-ant-v1-[PASTE YOUR KEY HERE]
SUPABASE_KEY=[PASTE FROM SUPABASE DASHBOARD]
MERCURY_2_API_TOKEN=sk-or-v1-[OPTIONAL - OPENROUTER KEY]
```

**SAVE:** Press `Ctrl+X`, then `Y`, then `Enter`

✅ Verify: `cat .env | grep ANTHROPIC_API_KEY` (should show your key)

---

## STEP 3: BUILD & DEPLOY (5 min)
```bash
docker compose build --no-cache
# ⏳ Wait 2-3 minutes...

docker compose up -d
# ✅ You should see: ✔ hermes Started

docker ps
# ✅ Should show 2 containers: hermes + agent-zero
```

---

## STEP 4: TEST API IS RUNNING (1 min)
```bash
curl http://localhost:8642/health
```

✅ Should return JSON with `"status":"ok"`

---

## STEP 5: UPDATE DASHBOARD (2 min)

**From your local machine (NOT SSH):**

1. Open: https://pauli-hermes-agent.vercel.app/dashboard
2. **Hard refresh**: Ctrl+Shift+R (or Cmd+Shift+R on Mac)
3. Click **⚙️ Settings** (bottom-left)
4. Find: **"Hermes API URL"**
5. Enter: `https://31.220.58.212:8642`
6. Click: **"Save & reconnect"**

✅ Header should show **Green dot** (Connected)

---

## STEP 6: TEST CHAT (1 min)

In dashboard, send a test message:
```
"What tools are available?"
```

✅ Should receive response from Hermes

---

## STEP 7: TEST VOICE (1 min)

1. Browser asks: **"Allow microphone?"** → Click **ALLOW**
2. Click **🎤** button (in chat input)
3. Say: **"Hello Hermes"**
4. ✅ Text should appear in input field
5. Click Send

---

## TROUBLESHOOTING

**Header shows "Offline" (red)?**
```bash
# On VPS, check if hermes is running
docker ps | grep hermes

# If not running, check logs
docker logs hermes | tail -20

# Restart
docker compose restart
```

**Voice button greyed out?**
- Check browser console (F12 → Console tab)
- Look for errors about "SpeechRecognition"
- Make sure you're on HTTPS (green lock in address bar)

**Can't SSH?**
- Check you have right IP: `31.220.58.212`
- Check username: `root`
- If key-based auth, use: `ssh -i your-key.pem root@31.220.58.212`

---

## NEXT (After verification)

See full docs:
- **VPS_DEPLOYMENT_MANUAL.md** — Detailed guide with all commands
- **DEPLOYMENT_ISSUES_AUDIT.md** — Why each issue happened
- **DEPLOYMENT_VISUAL_MAP.md** — Architecture + diagrams

---

**Time estimate: 20 minutes**  
**Current status: All code ready, just needs VPS .env + docker start**
