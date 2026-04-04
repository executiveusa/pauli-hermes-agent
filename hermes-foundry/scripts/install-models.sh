#!/usr/bin/env bash
# scripts/install-models.sh — Download Whisper and LLaMA GGUF models
set -euo pipefail

HERMES_HOME="${HERMES_HOME:-$HOME/.hermes}"
MODELS_DIR="$HERMES_HOME/models"
mkdir -p "$MODELS_DIR"

echo "Hermes Foundry — Model Installer"
echo "Models directory: $MODELS_DIR"
echo ""

download_if_missing() {
  local url="$1"
  local dest="$2"
  local name="$3"
  if [ -f "$dest" ]; then
    echo "  ✓ $name (already downloaded)"
  else
    echo "  ↓ $name…"
    curl -L --progress-bar -o "$dest" "$url"
    echo "  ✓ $name"
  fi
}

# ── Whisper Base (required) ───────────────────────────────────────────────────
echo "Whisper models (speech-to-text):"
download_if_missing \
  "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin" \
  "$MODELS_DIR/ggml-base.en.bin" \
  "Whisper Base (English) ~148MB"

echo ""

# ── LLaMA / Qwen GGUF (optional) ─────────────────────────────────────────────
echo "LLM models (local inference):"
echo ""
echo "To use a local LLM, place any GGUF model file in:"
echo "  $MODELS_DIR/"
echo ""
echo "Recommended free models:"
echo "  • Qwen2.5-7B-Instruct-Q4_K_M.gguf (~5GB)"
echo "    https://huggingface.co/bartowski/Qwen2.5-7B-Instruct-GGUF"
echo ""
echo "  • Llama-3.2-3B-Instruct-Q8_0.gguf (~3.4GB)"
echo "    https://huggingface.co/bartowski/Llama-3.2-3B-Instruct-GGUF"
echo ""
echo "  • phi-4-mini-instruct.Q4_K_M.gguf (~2.2GB)"
echo "    https://huggingface.co/bartowski/phi-4-mini-instruct-GGUF"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Model install complete."
echo "Directory contents:"
ls -lh "$MODELS_DIR" 2>/dev/null || echo "(empty)"
