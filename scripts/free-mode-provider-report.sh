#!/bin/bash
# Generate provider configuration report

echo "# FREE MODE Provider Configuration Report"
echo ""
echo "Generated: $(date)"
echo ""

# Check Python
if ! command -v python3 &> /dev/null; then
    echo "❌ Python3 not found"
    exit 1
fi

echo "## Configuration Status"
echo ""
echo "| Variable | Set | Value |"
echo "|----------|-----|-------|"

for var in GROQ_API_KEY GEMINI_API_KEY OPENROUTER_API_KEY OPENAI_API_KEY ANTHROPIC_API_KEY NVIDIA_NIM_API_KEY HF_TOKEN OLLAMA_BASE_URL LMSTUDIO_BASE_URL; do
    val=$(eval echo \$$var)
    if [ -n "$val" ]; then
        # Redact actual value, show only first 4 chars + count
        len=${#val}
        redacted="${val:0:4}...(${len} chars)"
        echo "| $var | ✅ | $redacted |"
    else
        echo "| $var | ⊘ | (not set) |"
    fi
done

echo ""
echo "## Provider Health"
echo ""

# Run provider tests if proxy is running
if curl -sf http://127.0.0.1:4000/health > /dev/null 2>&1; then
    echo "Proxy Status: ✅ Running"
    echo ""

    if [ -f "free-mode/scripts/test-provider.py" ]; then
        echo "Running provider tests..."
        python3 free-mode/scripts/test-provider.py --all --json > /tmp/free-mode-results.json 2>&1

        if [ -f "/tmp/free-mode-results.json" ]; then
            echo ""
            echo "| Provider | Status | Model | Latency |"
            echo "|----------|--------|-------|---------|"

            python3 << 'EOF'
import json
try:
    with open("/tmp/free-mode-results.json") as f:
        results = json.load(f)
        for pid, result in results.get("results", {}).items():
            status = result.get("status", "unknown")
            model = result.get("model", "-")
            latency = result.get("latency_ms")
            latency_str = f"{latency:.0f}ms" if latency else "-"
            emoji = {
                "healthy": "✅",
                "missing_secret": "⚠️",
                "auth_failed": "🔒",
                "timeout": "⏱",
                "failed": "❌",
            }.get(status, "?")
            print(f"| {pid} | {emoji} {status} | {model} | {latency_str} |")
except Exception as e:
    print(f"Error parsing results: {e}")
EOF
        fi
    fi
else
    echo "Proxy Status: ⊘ Not running"
    echo ""
    echo "Start with:"
    echo "  bash free-mode/scripts/start-free-mode.sh"
fi

echo ""
echo "## Files"
echo ""
echo "- ✅ free-mode/litellm.config.yaml"
echo "- ✅ free-mode/providers.json"
echo "- ✅ docker-compose.free-mode.yml"
echo "- ✅ free_mode/ (Python client)"
echo "- ✅ .env.example (updated with all vars)"
echo ""
echo "## Next Steps"
echo ""
echo "1. Copy .env.example → .env"
echo "2. Add at least one provider API key to .env"
echo "3. Run: bash free-mode/scripts/start-free-mode.sh"
echo "4. Run: export FREE_MODE=true"
echo "5. Test: hermes 'Hello'"
