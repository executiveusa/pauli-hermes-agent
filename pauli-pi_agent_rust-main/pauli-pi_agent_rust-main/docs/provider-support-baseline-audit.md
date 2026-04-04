# Provider Support Baseline Audit (`bd-3uqg.11.1`)

Generated at (UTC): `2026-02-13T04:48:33Z`

Machine-readable artifact: `docs/provider-baseline-audit.json`

## Summary

- Upstream union providers: **90**
- Matrix rows (including explicit user aliases): **92**
- Pi canonical providers in metadata: **87**

### Current Pi Status Counts

| Status | Count |
|---|---:|
| `alias->native-implemented` | 4 |
| `alias->oai-compatible-preset` | 3 |
| `native-adapter-required-unimplemented` | 2 |
| `native-implemented` | 8 |
| `oai-compatible-preset` | 75 |

### Risk Counts

| Risk | Count |
|---|---:|
| `high` | 7 |
| `low` | 14 |
| `medium` | 71 |

## User-Requested Provider Resolution

| Provider | Canonical | Current status | Target status | Risk |
|---|---|---|---|---|
| `alibaba` | `alibaba` | `oai-compatible-preset` | `promote-to-provider-specific-runtime-path-and-complete-test-doc-evidence` | `high` |
| `cerebras` | `cerebras` | `oai-compatible-preset` | `promote-to-provider-specific-runtime-path-and-complete-test-doc-evidence` | `high` |
| `groq` | `groq` | `oai-compatible-preset` | `promote-to-provider-specific-runtime-path-and-complete-test-doc-evidence` | `high` |
| `kimi` | `moonshotai` | `alias->oai-compatible-preset` | `promote-to-provider-specific-runtime-path-and-complete-test-doc-evidence` | `high` |
| `moonshotai` | `moonshotai` | `oai-compatible-preset` | `promote-to-provider-specific-runtime-path-and-complete-test-doc-evidence` | `high` |
| `openrouter` | `openrouter` | `oai-compatible-preset` | `promote-to-provider-specific-runtime-path-and-complete-test-doc-evidence` | `high` |
| `qwen` | `alibaba` | `alias->oai-compatible-preset` | `promote-to-provider-specific-runtime-path-and-complete-test-doc-evidence` | `high` |

## Execution Guidance

- Use this matrix as the source of truth for `bd-3uqg.11` provider-gap execution.
- Prioritize `high` risk rows, then `medium` rows that block parity completeness.
