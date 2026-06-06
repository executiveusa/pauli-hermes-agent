# Hermes Agent VPS Deployment Guide

Complete setup for deploying Hermes Agent on your VPS (`31.220.58.212`) with voice control and free Claude Code inference.

## Prerequisites

- Linux VPS with Python 3.10+
- Internet connection for NVIDIA NIM and API calls
- Domain name (optional, for HTTPS)

## Quick Start

```bash
# SSH to your VPS
ssh root@31.220.58.212

# Clone the repo (or git pull if already cloned)
cd /opt/pauli-hermes-agent
git pull origin main

# Run the complete setup
bash setup-vps.sh
```

## Manual Setup (if setup-vps.sh doesn't work)

### 1. Install Dependencies

```bash
apt-get update && apt-get install -y python3.11 python3-pip curl git
python3 -m pip install --upgrade pip setuptools

cd /opt/pauli-hermes-agent

# Install API server dependencies
pip install fastapi uvicorn pydantic

# Install NIM proxy dependencies
cd services/nim-proxy
pip install -r ../nim-proxy-requirements.txt
cd ../..
```

### 2. Configure Environment Variables

```bash
mkdir -p ~/.hermes
cat > ~/.hermes/.env << 'EOF'
# NVIDIA NIM (free inference)
NVIDIA_NIM_API_KEY=<YOUR_NVIDIA_NIM_API_KEY>

# Telegram bot
TELEGRAM_BOT_TOKEN=<YOUR_TELEGRAM_BOT_TOKEN>

# API Server
API_SERVER_PORT=8642
API_SERVER_HOST=0.0.0.0

# ElevenLabs TTS (optional, for voice responses)
ELEVENLABS_API_KEY=<YOUR_ELEVENLABS_API_KEY>

# Other keys from your setup...
EOF
chmod 600 ~/.hermes/.env
source ~/.hermes/.env
```

### 3. Start the Agent Stack

Option A: **All-in-one startup**
```bash
cd /opt/pauli-hermes-agent
bash start-agent.sh
```

Option B: **Start services individually**

**Terminal 1 - API Server (port 8642):**
```bash
cd /opt/pauli-hermes-agent
python3 api_server.py
```

**Terminal 2 - NIM Proxy (port 8082):**
```bash
cd /opt/pauli-hermes-agent/services/nim-proxy
python3 -m uvicorn server:app --host 0.0.0.0 --port 8082
```

## Access the Agent

### From Your Phone

**Web Voice Agent:**
- URL: `https://pauli-hermes-agent.vercel.app/agent`
- Speak into the microphone button
- Agent responds with synthesized speech
- Works on any browser with Web Speech API support

### From Terminal (Free Claude Code Inference)

On your VPS or any machine that can reach it:

```bash
# Option 1: Add to shell profile
echo 'export ANTHROPIC_BASE_URL=http://31.220.58.212:8082' >> ~/.bashrc
source ~/.bashrc

# Option 2: One-off command
ANTHROPIC_BASE_URL=http://31.220.58.212:8082 claude

# All Claude models will route to free NVIDIA NIM (moonshotai/kimi-k2-thinking)
```

**Note:** Set `ANTHROPIC_API_KEY` to any value (the proxy ignores it):
```bash
export ANTHROPIC_API_KEY=dummy
```

## Service Ports

| Service | Port | Purpose |
|---------|------|---------|
| **Agent API** | 8642 | Web UI voice endpoint, chat API |
| **NIM Proxy** | 8082 | Claude Code free inference gateway |
| **Hermes MCP** | 37777 (stdio) | Rolodex, memory, contact strength |

## Systemd Services (Optional - Auto-Restart on Reboot)

```bash
# Create service for API server
sudo tee /etc/systemd/system/hermes-api.service > /dev/null << 'EOF'
[Unit]
Description=Hermes Agent API Server
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/pauli-hermes-agent
EnvironmentFile=/root/.hermes/.env
ExecStart=/usr/bin/python3 /opt/pauli-hermes-agent/api_server.py
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Create service for NIM proxy
sudo tee /etc/systemd/system/hermes-nim-proxy.service > /dev/null << 'EOF'
[Unit]
Description=NVIDIA NIM Free Inference Proxy
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/pauli-hermes-agent/services/nim-proxy
EnvironmentFile=/root/.hermes/.env
ExecStart=/usr/bin/python3 -m uvicorn server:app --host 0.0.0.0 --port 8082
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable hermes-api hermes-nim-proxy
sudo systemctl start hermes-api hermes-nim-proxy

# Check status
sudo systemctl status hermes-api hermes-nim-proxy
```

## Testing

### Test API Server
```bash
curl http://localhost:8642/health
curl -X POST http://localhost:8642/api/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "remember that Alice is a product manager at Google"}'
```

### Test NIM Proxy
```bash
# Set your VPS IP and test from another machine
ANTHROPIC_BASE_URL=http://31.220.58.212:8082 \
ANTHROPIC_API_KEY=dummy \
curl -X POST http://31.220.58.212:8082/v1/messages \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer dummy" \
  -d '{
    "model": "claude-opus",
    "max_tokens": 100,
    "messages": [{"role": "user", "content": "Say hello"}]
  }'
```

### Test Voice on Phone
1. Open `https://pauli-hermes-agent.vercel.app/agent`
2. Press "Hold to Speak"
3. Say: "remember that I met John Smith"
4. Release the button
5. Wait for API response
6. Agent will speak back: "✅ Noted: remember that I met John Smith. Saving to Hermes memory..."

## Troubleshooting

### NIM Proxy Fails to Start
- **Issue:** `tiktoken` BPE download fails
- **Fix:** On the VPS with real internet (not sandbox), this should download automatically
- **Manual:** Download from `https://openaipublic.blob.core.windows.net/encodings/` and place in `~/.cache/tiktoken/`

### API Server Won't Respond from Phone
- **Check:** Firewall rules allow port 8642 inbound
- **Check:** Server is binding to `0.0.0.0:8642` (not just localhost)
- **Fix:** `ufw allow 8642` on Ubuntu

### Voice Not Working on Phone
- **Browser:** Chrome, Firefox, Safari (must be HTTPS or localhost)
- **Microphone:** Grant permission when prompted
- **Speech Recognition:** Not available in all languages/regions

### Free Inference Not Working
- **Check:** NIM API key is valid (from build.nvidia.com)
- **Check:** ANTHROPIC_BASE_URL is set correctly
- **Check:** NIM proxy is running: `curl http://localhost:8082/health` (if local)

## Monitoring

```bash
# Watch services
watch -n 2 'curl -s http://localhost:8642/health && curl -s http://localhost:8082/health'

# View logs
tail -f ~/.local/share/hermes/api_server.log
tail -f ~/.local/share/hermes/nim-proxy.log

# Check memory usage
ps aux | grep -E "api_server|server.py|nim-proxy"
```

## Security Notes

⚠️ **Port 8642 and 8082 should only be accessible from your networks:**

```bash
# Restrict to your phone's IP (example: 192.168.1.100)
sudo ufw allow from 192.168.1.0/24 to any port 8642
sudo ufw allow from 192.168.1.0/24 to any port 8082

# Or use nginx reverse proxy with auth
```

## Next Steps

1. **Deploy on VPS:** Run `bash start-agent.sh`
2. **Test on Phone:** Open the voice agent URL
3. **Configure Systemd:** Set up auto-restart on reboot
4. **Add Custom Commands:** Extend `api_server.py` with your own agent actions
5. **Integrate Hermes Rolodex:** Connect memory and recall features

## Support

- **NIM Free Tier:** https://build.nvidia.com
- **Claude Code Docs:** https://claude.ai/code
- **Hermes Repo:** https://github.com/executiveusa/pauli-hermes-agent
