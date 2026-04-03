# Voice Interface Context
# Load this when: voice input/output, PersonaPlex, Mercury 2

## Voice Stack
| Component | Role | Source |
|-----------|------|--------|
| PersonaPlex (NVIDIA) | Full-duplex speech I/O with persona control | personaplex-main/ |
| Mercury 2 (Inception Labs) | Fast diffusion LLM inference | api.inceptionlabs.ai |
| Moshi architecture | Underlying real-time audio model | personaplex-main/moshi/ |
| Web Speech API | Browser-side voice input fallback | web-ui/index.html |

## PersonaPlex Setup
```bash
cd personaplex-main/personaplex-main
pip install moshi/.
SSL_DIR=$(mktemp -d); python -m moshi.server --ssl "$SSL_DIR"
# Requires: HF_TOKEN, libopus-dev, GPU (or --cpu-offload)
```
Model: `nvidia/personaplex-7b-v1` (HuggingFace, requires license acceptance)

## Mercury 2 API
- Provider: Inception Labs
- API key: `MERCURY_API_KEY` (from Infisical slug: agent-hermes)
- Endpoint: `https://api.inceptionlabs.ai/v1/chat/completions`
- Use for: ultra-fast responses where latency matters (voice loop responses)

## Voice Persona Configuration
- Persona name: Hermes
- Tone: Strategic, direct, warm — mirrors Pauli's communication style
- Languages: EN (default) + CDMX Spanish (when user switches to Spanish)
- Role prompt: Injected from `HERMES.md` → "Dhandho mental model agent"

## Voice Command Dispatch
Voice → PersonaPlex STT → Hermes routing → execute → PersonaPlex TTS → audio response
Wake word or push-to-talk → triggers gateway voice handler
