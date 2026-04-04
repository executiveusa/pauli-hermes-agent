# Provider Canonical ID + Alias Policy (`bd-3uqg.1.3`)

Generated: `2026-02-10T04:38:00Z`
Depends on: bd-3uqg.1.1 (upstream snapshot), bd-3uqg.1.2 (baseline audit)

## Normalization Algorithm

When a user supplies a provider ID (CLI flag, config, env var), apply these steps in order:

1. **Trim** leading/trailing whitespace
2. **Lowercase** to ASCII lowercase
3. **Hyphenate** replace underscores with hyphens (`amazon_bedrock` -> `amazon-bedrock`)
4. **Alias resolve** lookup in alias table; if found, replace with canonical ID
5. **Passthrough** if not in alias table, use as-is (supports custom/extension providers)

**Rationale**: models.dev uses lowercase-hyphenated for all 87 IDs. Underscore tolerance prevents common user errors.

## Conflict Resolution Rules

1. **models.dev wins**: When all sources agree, use that ID. When sources disagree, prefer models.dev as canonical.
2. **opencode aliases**: opencode IDs that differ from models.dev become aliases (e.g., `gemini` -> `google`, `bedrock` -> `amazon-bedrock`, `copilot` -> `github-copilot`, `vertexai` -> `google-vertex`).
3. **Pi legacy aliases**: Where pi used a non-standard ID, the upstream ID becomes canonical and the pi ID becomes an alias with deprecation notice.
4. **Regional variants**: IDs like `alibaba-cn`, `moonshotai-cn` are distinct canonical IDs, not aliases.
5. **Coding-plan variants**: IDs like `minimax-coding-plan` are distinct canonical IDs.
6. **Extension providers**: Use their extension-declared ID as-is; no alias normalization.

## Deprecation Posture

**Policy**: Soft deprecation with warning log, hard removal after 2 minor versions.

| Deprecated ID | Canonical ID | Reason |
|--------------|-------------|--------|
| `fireworks` | `fireworks-ai` | Pi used `fireworks`, upstream canonical is `fireworks-ai` |
| `azure-openai` | `azure` | Pi used `azure-openai`, models.dev canonical is `azure` |

Both deprecated IDs will continue to work via alias resolution. A deprecation warning will be logged when they are used.

## Alias Lookup Table

| Alias | Canonical ID | Origin |
|-------|-------------|--------|
| `azure-openai` | `azure` | Pi legacy |
| `bedrock` | `amazon-bedrock` | opencode |
| `copilot` | `github-copilot` | opencode |
| `dashscope` | `alibaba` | Pi alias |
| `fireworks` | `fireworks-ai` | Pi legacy |
| `gemini` | `google` | opencode |
| `kimi` | `moonshotai` | Pi alias |
| `moonshot` | `moonshotai` | Pi alias |
| `qwen` | `alibaba` | Pi alias |
| `vertexai` | `google-vertex` | opencode |

Total: 10 aliases mapping to 7 distinct canonical IDs.

## Canonical ID Registry (88 IDs)

Derived from the union of models.dev (87), opencode (11), and codex (3), after deduplication:

| Canonical ID | Has Aliases | Source(s) |
|-------------|------------|-----------|
| 302ai | no | models.dev |
| abacus | no | models.dev |
| aihubmix | no | models.dev |
| alibaba | yes (dashscope, qwen) | models.dev |
| alibaba-cn | no | models.dev |
| amazon-bedrock | yes (bedrock) | models.dev + opencode |
| anthropic | no | all |
| azure | yes (azure-openai) | models.dev |
| azure-cognitive-services | no | models.dev |
| bailing | no | models.dev |
| baseten | no | models.dev |
| berget | no | models.dev |
| cerebras | no | models.dev + opencode |
| chutes | no | models.dev |
| cloudflare-ai-gateway | no | models.dev + opencode |
| cloudflare-workers-ai | no | models.dev + opencode |
| cohere | no | models.dev |
| cortecs | no | models.dev |
| deepinfra | no | models.dev |
| deepseek | no | models.dev |
| fastrouter | no | models.dev |
| fireworks-ai | yes (fireworks) | models.dev |
| firmware | no | models.dev |
| friendli | no | models.dev |
| github-copilot | yes (copilot) | models.dev + opencode |
| github-copilot-enterprise | no | opencode |
| github-models | no | models.dev |
| gitlab | no | models.dev + opencode |
| google | yes (gemini) | models.dev + opencode |
| google-vertex | yes (vertexai) | models.dev + opencode |
| google-vertex-anthropic | no | models.dev |
| groq | no | models.dev + opencode |
| helicone | no | models.dev |
| huggingface | no | models.dev |
| iflowcn | no | models.dev |
| inception | no | models.dev |
| inference | no | models.dev |
| io-net | no | models.dev |
| jiekou | no | models.dev |
| kimi-for-coding | no | models.dev |
| llama | no | models.dev |
| lmstudio | no | models.dev + codex |
| lucidquery | no | models.dev |
| minimax | no | models.dev |
| minimax-cn | no | models.dev |
| minimax-cn-coding-plan | no | models.dev |
| minimax-coding-plan | no | models.dev |
| mistral | no | models.dev |
| moark | no | models.dev |
| modelscope | no | models.dev |
| moonshotai | yes (moonshot, kimi) | models.dev |
| moonshotai-cn | no | models.dev |
| morph | no | models.dev |
| nano-gpt | no | models.dev |
| nebius | no | models.dev |
| nova | no | models.dev |
| novita-ai | no | models.dev |
| nvidia | no | models.dev |
| ollama | no | codex |
| ollama-cloud | no | models.dev |
| openai | no | all |
| opencode | no | models.dev + opencode |
| openrouter | no | models.dev + opencode |
| ovhcloud | no | models.dev |
| perplexity | no | models.dev |
| poe | no | models.dev |
| privatemode-ai | no | models.dev |
| requesty | no | models.dev |
| sap-ai-core | no | models.dev + opencode |
| scaleway | no | models.dev |
| siliconflow | no | models.dev |
| siliconflow-cn | no | models.dev |
| submodel | no | models.dev |
| synthetic | no | models.dev |
| togetherai | no | models.dev |
| upstage | no | models.dev |
| v0 | no | models.dev |
| venice | no | models.dev |
| vercel | no | models.dev + opencode |
| vivgrid | no | models.dev |
| vultr | no | models.dev |
| wandb | no | models.dev |
| xai | no | models.dev + opencode |
| xiaomi | no | models.dev |
| zai | no | models.dev |
| zai-coding-plan | no | models.dev |
| zenmux | no | models.dev + opencode |
| zhipuai | no | models.dev |
| zhipuai-coding-plan | no | models.dev |

## Pi Migration Actions Required

Only 2 IDs need migration (both non-breaking):

| Current Pi ID | Canonical ID | Action | Breaking? |
|--------------|-------------|--------|-----------|
| `azure-openai` | `azure` | Rename in factory + auth; keep `azure-openai` as alias | No |
| `fireworks` | `fireworks-ai` | Rename in ad-hoc defaults + auth; keep `fireworks` as alias | No |

Files affected:
- `src/providers/azure.rs`: `name()` returns `"azure"` instead of `"azure-openai"`
- `src/providers/mod.rs`: Factory match arm `"azure"` instead of `"azure-openai"`
- `src/auth.rs`: `env_keys_for_provider` match arms for both canonical + aliases
- `src/models.rs`: `ad_hoc_provider_defaults` match arms updated

## Implementation Guidance

The normalization function should be added to `src/models.rs` or a new `src/provider_id.rs`:

```rust
fn normalize_provider_id(raw: &str) -> String {
    let normalized = raw.trim().to_ascii_lowercase().replace('_', "-");
    match ALIAS_TABLE.get(normalized.as_str()) {
        Some(canonical) => canonical.to_string(),
        None => normalized,
    }
}
```

This function should be called at all provider ID entry points:
- CLI argument parsing
- Config file loading
- `ad_hoc_model_entry()` lookup
- `env_keys_for_provider()` lookup
- `create_provider()` factory selection
