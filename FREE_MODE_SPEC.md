# FREE MODE — Universal Free AI Toggle

## Concept

A drop-in module for any AI project that adds a **FREE MODE** toggle.
When enabled, all AI inference routes through LiteLLM → NVIDIA NIM (free).
When disabled, the project uses its original provider unchanged.

## Architecture

```
FREE MODE ON
  App → LiteLLM Router → NVIDIA NIM (moonshotai/kimi-k2-thinking) — $0
FREE MODE OFF
  App → LiteLLM Router → Original provider (OpenAI, Anthropic, etc.)
```

## What to build

### 1. LiteLLM config (`litellm_config.yaml`)
- Two model groups: `free` (NIM) and `default` (whatever the project uses)
- Toggle switches between groups at runtime
- Works with any LiteLLM-compatible app via `LITELLM_MODE=free|default`

### 2. Universal drop-in module (`free-mode/index.ts`)
- Wraps any OpenAI/Anthropic SDK call
- Reads `FREE_MODE=true|false` env var
- When `FREE_MODE=true`: overrides `baseURL` → LiteLLM proxy → NIM
- When `FREE_MODE=false`: passes through to original provider
- Zero changes required in existing app code

### 3. UI Toggle component (`free-mode/Toggle.tsx`)
- Works in React, Vue, Svelte
- Shows current provider + cost indicator
- Visual: green "FREE ⚡" vs blue "Premium"
- Persists choice to localStorage

### 4. Universal agent prompt
- A system prompt snippet any AI agent can prepend
- Tells the agent: "When you need free inference or hit token limits,
  switch FREE_MODE=true and use LiteLLM at http://localhost:4000"

### 5. One-command install
```bash
npx add-free-mode
# Adds: litellm_config.yaml, free-mode/ directory, Toggle component
# Patches: existing .env with FREE_MODE=false
# Adds: docker-compose entry for LiteLLM proxy
```

## NIM Proxy Details (pre-configured)
- URL: http://31.220.58.212:8082
- Auth: none
- Model: moonshotai/kimi-k2-thinking (maps to any Claude model name)
- Cost: $0 (NVIDIA free developer tier)
- Rate limit: 40 req/min

## Groq as alternative free provider
- Model: llama-3.3-70b-versatile
- Cost: $0 (Groq free tier)
- Speed: faster than GPT-4o-mini
- Key: set GROQ_API_KEY in .env

## LiteLLM model config
```yaml
model_list:
  - model_name: free
    litellm_params:
      model: openai/moonshotai/kimi-k2-thinking
      api_base: http://31.220.58.212:8082
      api_key: dummy

  - model_name: free-groq
    litellm_params:
      model: groq/llama-3.3-70b-versatile
      api_key: os.environ/GROQ_API_KEY

  - model_name: default
    litellm_params:
      model: gpt-4o-mini
      api_key: os.environ/OPENAI_API_KEY

router_settings:
  routing_strategy: simple-shuffle
```

## TODO (build this later)
- [ ] Create `packages/free-mode/` in this repo
- [ ] Build the npx installer
- [ ] Build the Toggle React component
- [ ] Write the universal agent system prompt
- [ ] Test with LiteLLM Docker image
- [ ] Publish to npm as `free-mode-toggle`
