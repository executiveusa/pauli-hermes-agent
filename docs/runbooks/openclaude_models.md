# OpenClaude Model Selection Runbook

## Overview

The Flywheel dispatcher selects the cheapest available model/provider for each bead. This document describes the priority order, how to verify which model is being used, and how to override the selection.

---

## Provider Priority Order (Cheapest First)

The dispatcher iterates this list and returns the first provider whose API key (or local service) is available in the environment.

| Priority | Provider | Model (default) | Cost | Requirement |
|---|---|---|---|---|
| 1 | **Ollama** (local) | `qwen2.5-coder:7b` | **Free** | `OLLAMA_HOST` set, or `ollama` in PATH |
| 2 | **OpenRouter free tier** | `meta-llama/llama-3.1-8b-instruct:free` | **Free** | `OPENROUTER_API_KEY` |
| 3 | **Groq** | `llama-3.1-8b-instant` | Very cheap | `GROQ_API_KEY` |
| 4 | **DeepSeek** | `deepseek-coder` | Cheap | `DEEPSEEK_API_KEY` |
| 5 | **OpenAI** (or compatible) | `gpt-4o-mini` | Paid | `OPENAI_API_KEY` |
| 6 | **auto** | (openclaude default) | varies | Fallback when nothing is configured |

---

## Free Model Details

### Ollama (local inference)

Ollama runs models locally. No API key or internet connection required after pulling the model.

```bash
ollama pull qwen2.5-coder:7b   # ~4 GB download, one time
export OLLAMA_HOST=http://localhost:11434
```

Recommended models for coding tasks (by capability vs. size):
- `qwen2.5-coder:7b` — excellent code quality, 4 GB, fast on M1/M2/CUDA
- `qwen2.5-coder:14b` — higher quality, 8 GB
- `codellama:7b` — good for Python/JS
- `deepseek-coder:6.7b` — strong on refactoring

### OpenRouter Free Tier

OpenRouter offers a free tier with rate limits for selected open-source models. These are real hosted models — no local GPU needed.

Free models (as of May 2026, subject to change):
- `meta-llama/llama-3.1-8b-instruct:free`
- `meta-llama/llama-3.2-3b-instruct:free`
- `mistralai/mistral-7b-instruct:free`
- `google/gemma-2-9b-it:free`
- `microsoft/phi-3-mini-128k-instruct:free`

```bash
export OPENROUTER_API_KEY=sk-or-your-key
```

### Groq

Groq offers ultra-fast inference on open-source models with a generous free tier.

Recommended for coding: `llama-3.1-8b-instant` (fast) or `llama-3.3-70b-versatile` (more capable).

```bash
export GROQ_API_KEY=gsk_your-key
```

---

## Verifying Model Selection

```python
from pauli.flywheel.dispatchers.openclaude_dispatcher import (
    load_worker_registry, get_worker_config, select_model
)

registry = load_worker_registry()
worker_config = get_worker_config(registry)
provider, model = select_model(worker_config)
print(f"Selected provider: {provider}")
print(f"Selected model:    {model}")
```

Or check a dispatch result:

```python
result = dispatcher.dispatch(bead)
print(result["provider"])   # e.g. "openrouter"
print(result["model_used"]) # e.g. "meta-llama/llama-3.1-8b-instruct:free"
```

---

## Overriding the Model

### Per-session override via environment variables

```bash
export OPENAI_MODEL=llama-3.3-70b-versatile
export GROQ_API_KEY=gsk_...
```

The dispatcher will pick Groq (cheaper than OpenAI) and use the overridden model name.

### Bead-level override (future)

The bead spec does not currently support a model override field. When this is needed, pass it via:

```python
bead = {
    "task_type": "refactor",
    "description": "...",
    "metadata": {"model_hint": "qwen2.5-coder:14b"},
}
```

The dispatcher will respect `metadata.model_hint` if the selected provider supports the requested model.

---

## Changing the Priority Order

Edit `config/pauli_worker_registry.yaml` under `workers.openclaude.model_priority`. Each entry:

```yaml
model_priority:
  - provider: ollama
    model: qwen2.5-coder:7b
    env_check: OLLAMA_HOST   # must be set (or ollama in PATH)
  - provider: openrouter
    model: meta-llama/llama-3.1-8b-instruct:free
    env_check: OPENROUTER_API_KEY
  # ... add or reorder entries as needed
```

If `env_check` is omitted, the provider is considered always available (useful for Ollama when OLLAMA_HOST is always set in your environment).

---

## Cost Comparison (Approximate, May 2026)

| Provider | Model | Input $/M tokens | Output $/M tokens |
|---|---|---|---|
| Ollama | qwen2.5-coder:7b | **$0** | **$0** |
| OpenRouter free | llama-3.1-8b | **$0** | **$0** |
| Groq | llama-3.1-8b-instant | ~$0.05 | ~$0.08 |
| DeepSeek | deepseek-coder | ~$0.14 | ~$0.28 |
| OpenAI | gpt-4o-mini | ~$0.15 | ~$0.60 |
| OpenAI | gpt-4o | ~$2.50 | ~$10.00 |
| Anthropic | claude-3-5-sonnet | ~$3.00 | ~$15.00 |

For routine coding tasks (refactoring, docs, test repair), Ollama or OpenRouter free tier is almost always sufficient. Reserve paid models for complex multi-file rewrites or tasks that require strong instruction-following.
