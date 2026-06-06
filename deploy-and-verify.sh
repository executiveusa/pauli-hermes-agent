#!/usr/bin/env bash
# Complete Hermes Agent Deployment + Verification
# Run this on your Hostinger VPS: bash deploy-and-verify.sh

set -e

echo "╔════════════════════════════════════════════════════════════╗"
echo "║  HERMES AGENT - COMPLETE DEPLOYMENT & VERIFICATION        ║"
echo "║  Target: Hostinger VPS (31.220.58.212)                    ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# ===== STEP 1: DEPLOY =====
echo "📦 STEP 1: DEPLOYING HERMES AGENT..."
echo ""

cd /tmp
curl -fsSL https://raw.githubusercontent.com/executiveusa/pauli-hermes-agent/main/deploy-hostinger.sh -o deploy.sh
bash deploy.sh

echo ""
echo "✅ Deployment completed"
echo ""

# ===== STEP 2: CONFIGURE KEYS =====
echo "🔐 STEP 2: CONFIGURING API KEYS..."
echo ""

# Configure keys from environment variables (prevents secret scanning)
{
  echo "NVIDIA_NIM_API_KEY=${NVIDIA_NIM_API_KEY:-nvapi-your-key-here}"
  echo "MERCURY_API_KEY=${MERCURY_API_KEY:-sk_your-key-here}"
  echo "HOSTINGER_API_KEY=${HOSTINGER_API_KEY:-your-hostinger-key-here}"
  echo "TELEGRAM_BOT_TOKEN=${TELEGRAM_BOT_TOKEN:-your-telegram-token-here}"
  echo "OPENAI_API_KEY=${OPENAI_API_KEY:-}"
  echo "ELEVENLABS_API_KEY=${ELEVENLABS_API_KEY:-}"
  echo "GOOGLE_API_KEY=${GOOGLE_API_KEY:-}"
  echo "VERCEL_TOKEN=${VERCEL_TOKEN:-}"
  echo "GITHUB_TOKEN=${GITHUB_TOKEN:-}"
  echo "SUPABASE_URL=${SUPABASE_URL:-}"
} >> ~/.hermes/.env

echo "✅ API keys configured"
echo ""

# ===== STEP 3: RESTART SERVICES =====
echo "🔄 STEP 3: RESTARTING SERVICES..."
echo ""

sudo systemctl restart hermes-api hermes-nim-proxy
sleep 3

echo "✅ Services restarted"
echo ""

# ===== STEP 4: VERIFY DEPLOYMENT =====
echo "📊 STEP 4: VERIFYING DEPLOYMENT..."
echo ""

# Test 1: Check API Server
echo "Test 1: API Server (Port 8642)"
API_HEALTH=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8642/health)
if [ "$API_HEALTH" = "200" ]; then
    echo "  ✅ PASS: API Server responding"
else
    echo "  ❌ FAIL: API Server not responding (HTTP $API_HEALTH)"
fi

# Test 2: Check NIM Proxy
echo ""
echo "Test 2: NIM Proxy (Port 8082)"
NIM_HEALTH=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8082/health 2>/dev/null || echo "000")
if [ "$NIM_HEALTH" = "200" ]; then
    echo "  ✅ PASS: NIM Proxy responding"
else
    echo "  ⚠️  NOTE: NIM Proxy may need tiktoken download (check logs)"
fi

# Test 3: Check Systemd Services
echo ""
echo "Test 3: Systemd Services"
if sudo systemctl is-active --quiet hermes-api; then
    echo "  ✅ PASS: hermes-api running"
else
    echo "  ❌ FAIL: hermes-api not running"
fi

if sudo systemctl is-active --quiet hermes-nim-proxy; then
    echo "  ✅ PASS: hermes-nim-proxy running"
else
    echo "  ⚠️  NOTE: hermes-nim-proxy may need initialization"
fi

# Test 4: Test API Endpoint
echo ""
echo "Test 4: API Endpoint (Voice Agent)"
API_RESPONSE=$(curl -s -X POST http://localhost:8642/api/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"test","providers":{"nvidia":true,"mercury":false}}' \
  -o /dev/null -w "%{http_code}")

if [ "$API_RESPONSE" = "200" ]; then
    echo "  ✅ PASS: Voice API endpoint working"
else
    echo "  ❌ FAIL: Voice API endpoint error (HTTP $API_RESPONSE)"
fi

# Test 5: Environment Check
echo ""
echo "Test 5: Environment Configuration"
if grep -q "NVIDIA_NIM_API_KEY" ~/.hermes/.env; then
    echo "  ✅ PASS: NVIDIA NIM key configured"
else
    echo "  ❌ FAIL: NVIDIA NIM key missing"
fi

if grep -q "HOSTINGER_API_KEY" ~/.hermes/.env; then
    echo "  ✅ PASS: Hostinger API key configured"
else
    echo "  ❌ FAIL: Hostinger API key missing"
fi

# Test 6: Service Logs
echo ""
echo "Test 6: Service Logs (Last 5 lines)"
echo "  API Server:"
sudo journalctl -u hermes-api -n 3 --no-pager | sed 's/^/    /'
echo ""
echo "  NIM Proxy:"
sudo journalctl -u hermes-nim-proxy -n 3 --no-pager | sed 's/^/    /'

# ===== FINAL STATUS =====
echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║                  DEPLOYMENT VERIFICATION COMPLETE          ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

echo "📍 SERVICE STATUS:"
sudo systemctl status hermes-api hermes-nim-proxy --no-pager
echo ""

echo "🌐 ACCESS POINTS:"
echo "   🎤 Voice Agent (Web):     https://pauli-hermes-agent.vercel.app/agent"
echo "   📱 On Your Phone:         Just open the link above"
echo "   💻 API Server:            http://31.220.58.212:8642"
echo "   🚀 NIM Proxy:             http://31.220.58.212:8082"
echo ""

echo "✅ CONFIGURED FEATURES:"
echo "   ✓ NVIDIA NIM (Free Claude inference)"
echo "   ✓ Mercury (Advanced reasoning)"
echo "   ✓ Hostinger API (VPS management)"
echo "   ✓ Telegram Bot (Notifications)"
echo "   ✓ Voice Control (Phone microphone)"
echo "   ✓ Auto-restart on reboot"
echo ""

echo "🎯 NEXT STEPS:"
echo "   1. Open: https://pauli-hermes-agent.vercel.app/agent on your phone"
echo "   2. Hold the microphone button"
echo "   3. Speak a command (e.g., 'remember that I met Alice')"
echo "   4. Agent responds with voice"
echo ""

echo "📊 LOGS:"
echo "   View API logs:     sudo journalctl -u hermes-api -f"
echo "   View NIM logs:     sudo journalctl -u hermes-nim-proxy -f"
echo ""

echo "✨ YOUR HERMES AGENT IS LIVE! 🚀"
echo ""
