#!/usr/bin/env bash
# Automated API Key Configuration for Hermes Agent
# Sets all required API keys in ~/.hermes/.env

echo "🔐 Setting up API keys..."

# Create .env if it doesn't exist
mkdir -p ~/.hermes

# Set all keys using environment variables (prevents secret scanning)
{
  echo "# Hermes Agent - Complete Configuration"
  echo "# Generated: $(date)"
  echo ""
  echo "# ===== CORE INFERENCE ====="
  echo "# NVIDIA NIM - Free Claude Code Inference (40 req/min, unlimited tokens)"
  echo "NVIDIA_NIM_API_KEY=${NVIDIA_NIM_API_KEY:-nvapi-your-key-here}"
  echo ""
  echo "# Mercury Inception Labs - Advanced reasoning (premium)"
  echo "MERCURY_API_KEY=${MERCURY_API_KEY:-sk_your-key-here}"
  echo ""
  echo "# ===== HOSTINGER VPS MANAGEMENT ====="
  echo "# Hostinger API for VPS and domain management"
  echo "HOSTINGER_API_KEY=${HOSTINGER_API_KEY:-your-hostinger-key-here}"
  echo ""
  echo "# ===== COMMUNICATION ====="
  echo "# Telegram Bot Token (from @BotFather on Telegram)"
  echo "TELEGRAM_BOT_TOKEN=${TELEGRAM_BOT_TOKEN:-your-telegram-token-here}"
  echo ""
  echo "# ===== API SERVER CONFIG ====="
  echo "# Agent API Server (voice agent backend)"
  echo "API_SERVER_PORT=8642"
  echo "API_SERVER_HOST=0.0.0.0"
  echo "API_SERVER_CORS_ORIGINS=https://pauli-hermes-agent.vercel.app,http://localhost:3000,http://localhost:8642"
  echo ""
  echo "# ===== OPTIONAL: OTHER SERVICES ====="
  echo "# OpenAI API (for fallback inference)"
  echo "OPENAI_API_KEY=${OPENAI_API_KEY:-}"
  echo ""
  echo "# ElevenLabs - Text-to-speech for voice responses"
  echo "ELEVENLABS_API_KEY=${ELEVENLABS_API_KEY:-}"
  echo ""
  echo "# Google API - For search and other services"
  echo "GOOGLE_API_KEY=${GOOGLE_API_KEY:-}"
  echo ""
  echo "# Vercel - For deployment and CI/CD"
  echo "VERCEL_TOKEN=${VERCEL_TOKEN:-}"
  echo ""
  echo "# GitHub - For repo access and CI/CD"
  echo "GITHUB_TOKEN=${GITHUB_TOKEN:-}"
  echo ""
  echo "# Supabase - Vector DB and authentication"
  echo "SUPABASE_URL=${SUPABASE_URL:-}"
  echo ""
  echo "# ===== HERMES CONFIG ====="
  echo "HERMES_CONFIG=~/.hermes/config.yaml"
} > ~/.hermes/.env

chmod 600 ~/.hermes/.env

echo "✅ API keys configured in ~/.hermes/.env"
echo ""
echo "🔑 Keys set:"
echo "   ✓ NVIDIA NIM (free inference)"
echo "   ✓ Mercury (advanced reasoning)"
echo "   ✓ Hostinger (VPS management)"
echo "   ✓ Telegram (bot notifications)"
echo "   ✓ OpenAI (fallback)"
echo "   ✓ ElevenLabs (voice TTS)"
echo "   ✓ All other services"
echo ""
echo "🔄 Restarting Hermes services..."
sudo systemctl restart hermes-api hermes-nim-proxy
sleep 2

echo ""
echo "✅ Services restarted!"
echo ""
echo "📍 Check status:"
sudo systemctl status hermes-api hermes-nim-proxy
echo ""
echo "🎉 Ready to use! Open https://pauli-hermes-agent.vercel.app/agent on your phone"
