## Stratified Extension Sampling Matrix

This matrix defines a deterministic, stratified selection policy for the expanded extension corpus
using:
- `docs/EXTENSION_POPULARITY_CRITERIA.md` (scoring + gates),
- `docs/extension-candidate-pool.json` (raw candidates), and
- `docs/extension-individual-enumeration.json` (individual extension coverage stats).

It is intentionally **tiered**:
- `Tier-0`: official baseline (all official pi-mono extensions),
- `Tier-1`: must-pass corpus (**>= 200** unmodified extensions),
- `Tier-2`: additional long-tail/stretch coverage.

This matrix is not a replacement for the frozen 16-extension runtime sample in
`docs/extension-sample.json`; that file remains the compact parity harness baseline.

> **Note:** Tags below are **inferred** from README/descriptions. A static scan should validate and adjust before final selection.

---

## 1) Sampling Axes & Quotas (Tiered Corpus)

### 1.1 Selection Size Targets

| Tier | Target |
|---|---|
| `tier-0` | Include all official pi-mono extensions |
| `tier-1` | Select **>= 200** extensions (must-pass) |
| `tier-2` | Select all additional eligible long-tail entries |

### 1.2 Quota Formula (deterministic + feasible)

For each bucket:

`required(bucket) = min(available(bucket), ceil(tier1_target * ratio(bucket)))`

Where:
- `tier1_target = 200`
- `available(bucket)` is computed from the latest candidate snapshot
- ratios below encode desired behavioral coverage

### 1.3 Tier-1 Source-Tier Minimums (absolute)

Using `docs/extension-individual-enumeration.json` availability snapshot (`total = 214`):

| Source Tier | Available | Tier-1 Minimum |
|---|---:|---:|
| `official-pi-mono` | 60 | 60 |
| `community` | 55 | 50 |
| `third-party-github` | 59 | 50 |
| `npm-registry` | 37 | 37 |
| `agents-mikeastock` | 3 | 3 |

These minima sum to **200**.

### 1.4 Tier-1 Behavior-Bucket Targets

Behavior targets are quota-governed and capped by availability:

| Bucket | Ratio Target | Availability Source |
|---|---:|---|
| `event_hook` | 0.40 | `by_capability.event_hook` |
| `registerTool` | 0.25 | `by_capability.registerTool` |
| `registerShortcut` | 0.07 | `by_capability.registerShortcut` |
| `registerFlag` | 0.04 | `by_capability.registerFlag` |
| `registerProvider` | 0.02 | `by_capability.registerProvider` |
| `exec_api` | 0.12 | `by_capability.exec_api` |
| `session_api` | 0.05 | `by_capability.session_api` |
| `ui_header` / `ui_overlay` | 0.25 combined | `by_capability.ui_header`, `by_capability.ui_overlay` |

If `available(bucket) < ceil(200 * ratio)`, mark that bucket as `availability_limited=true`
in the selection output and include the shortfall count.

---

## 2) Candidate Tag Mapping (All Candidates)

**Legend:**  
Interaction tags = `tool_only`, `slash_command`, `event_hook`, `ui_integration`, `provider`, `input_transform`  
Capabilities = `read`, `write`, `exec`, `http`, `env`  
Runtime = `legacy-js`, `multi-file`, `pkg-with-deps`, `provider-ext`, `gist`, `pi-package`  
I/O = `fs-heavy`, `network-heavy`, `ui-centric`, `cpu-heavy`, `os-heavy`

### A) pi‑mono example extensions

| Candidate | Runtime | Interaction | Capabilities | Complexity | I/O |
|---|---|---|---|---|---|
| `permission-gate.ts` | legacy-js | event_hook, ui_integration | exec, env | medium | ui-centric |
| `protected-paths.ts` | legacy-js | event_hook | write, read | small | fs-heavy |
| `confirm-destructive.ts` | legacy-js | slash_command, ui_integration | env | small | ui-centric |
| `dirty-repo-guard.ts` | legacy-js | event_hook | exec | small | fs-heavy |
| `sandbox/` | multi-file | event_hook | exec | large | os-heavy |

| Candidate | Runtime | Interaction | Capabilities | Complexity | I/O |
|---|---|---|---|---|---|
| `todo.ts` | legacy-js | tool_only, slash_command, ui_integration | write, read | medium | fs-heavy |
| `hello.ts` | legacy-js | tool_only | env | small | ui-centric |
| `question.ts` | legacy-js | tool_only, ui_integration | env | small | ui-centric |
| `questionnaire.ts` | legacy-js | tool_only, ui_integration | env | medium | ui-centric |
| `tool-override.ts` | legacy-js | event_hook, tool_only | read, write | medium | fs-heavy |
| `truncated-tool.ts` | legacy-js | tool_only | exec | medium | fs-heavy |
| `antigravity-image-gen.ts` | legacy-js | tool_only | http, write | medium | network-heavy |
| `ssh.ts` | legacy-js | tool_only | exec, http | large | network-heavy |
| `subagent/` | multi-file | tool_only | exec | large | cpu-heavy |

| Candidate | Runtime | Interaction | Capabilities | Complexity | I/O |
|---|---|---|---|---|---|
| `preset.ts` | legacy-js | slash_command, ui_integration | env | medium | ui-centric |
| `plan-mode/` | multi-file | slash_command, ui_integration | read | large | ui-centric |
| `tools.ts` | legacy-js | slash_command, ui_integration | env | medium | ui-centric |
| `handoff.ts` | legacy-js | slash_command | write | medium | fs-heavy |
| `qna.ts` | legacy-js | slash_command, ui_integration | env | small | ui-centric |
| `status-line.ts` | legacy-js | ui_integration | env | small | ui-centric |
| `widget-placement.ts` | legacy-js | ui_integration | env | small | ui-centric |
| `model-status.ts` | legacy-js | event_hook, ui_integration | env | small | ui-centric |
| `snake.ts` | legacy-js | ui_integration | env | large | cpu-heavy |
| `space-invaders.ts` | legacy-js | ui_integration | env | large | cpu-heavy |
| `send-user-message.ts` | legacy-js | slash_command | env | small | ui-centric |
| `timed-confirm.ts` | legacy-js | ui_integration | env | small | ui-centric |
| `rpc-demo.ts` | legacy-js | ui_integration | env | medium | ui-centric |
| `modal-editor.ts` | legacy-js | ui_integration | env | large | ui-centric |
| `rainbow-editor.ts` | legacy-js | ui_integration | env | medium | ui-centric |
| `notify.ts` | legacy-js | event_hook, ui_integration | exec | medium | os-heavy |
| `titlebar-spinner.ts` | legacy-js | ui_integration | env | small | ui-centric |
| `summarize.ts` | legacy-js | slash_command, tool_only | http | medium | network-heavy |
| `custom-footer.ts` | legacy-js | ui_integration | env | small | ui-centric |
| `custom-header.ts` | legacy-js | ui_integration | env | small | ui-centric |
| `overlay-test.ts` | legacy-js | ui_integration | env | medium | ui-centric |
| `overlay-qa-tests.ts` | legacy-js | ui_integration | env | large | ui-centric |
| `doom-overlay/` | multi-file | ui_integration | exec? | large | cpu-heavy |
| `shutdown-command.ts` | legacy-js | slash_command | env | small | ui-centric |
| `interactive-shell.ts` | legacy-js | event_hook | exec | medium | os-heavy |
| `inline-bash.ts` | legacy-js | input_transform | exec | medium | os-heavy |
| `bash-spawn-hook.ts` | legacy-js | event_hook | exec | small | os-heavy |
| `input-transform.ts` | legacy-js | event_hook | env | small | ui-centric |
| `system-prompt-header.ts` | legacy-js | event_hook | env | small | ui-centric |

| Candidate | Runtime | Interaction | Capabilities | Complexity | I/O |
|---|---|---|---|---|---|
| `git-checkpoint.ts` | legacy-js | event_hook | exec | medium | fs-heavy |
| `auto-commit-on-exit.ts` | legacy-js | event_hook | exec | medium | fs-heavy |

| Candidate | Runtime | Interaction | Capabilities | Complexity | I/O |
|---|---|---|---|---|---|
| `pirate.ts` | legacy-js | event_hook | env | small | ui-centric |
| `claude-rules.ts` | legacy-js | event_hook | read | medium | fs-heavy |
| `custom-compaction.ts` | legacy-js | event_hook | env | medium | ui-centric |
| `trigger-compact.ts` | legacy-js | slash_command | env | small | ui-centric |

| Candidate | Runtime | Interaction | Capabilities | Complexity | I/O |
|---|---|---|---|---|---|
| `mac-system-theme.ts` | legacy-js | event_hook | env | small | os-heavy |
| `dynamic-resources/` | multi-file | event_hook | read | medium | fs-heavy |
| `message-renderer.ts` | legacy-js | ui_integration | env | medium | ui-centric |
| `event-bus.ts` | legacy-js | event_hook | env | medium | ui-centric |
| `session-name.ts` | legacy-js | event_hook | env | small | ui-centric |
| `bookmark.ts` | legacy-js | event_hook | env | small | ui-centric |

| Candidate | Runtime | Interaction | Capabilities | Complexity | I/O |
|---|---|---|---|---|---|
| `custom-provider-anthropic/` | provider-ext | provider | http | large | network-heavy |
| `custom-provider-gitlab-duo/` | provider-ext | provider | http | large | network-heavy |
| `custom-provider-qwen-cli/` | provider-ext | provider | exec, http | large | network-heavy |

| Candidate | Runtime | Interaction | Capabilities | Complexity | I/O |
|---|---|---|---|---|---|
| `with-deps/` | pkg-with-deps | mixed | read, write | medium | fs-heavy |
| `file-trigger.ts` | legacy-js | event_hook | read | small | fs-heavy |

### B) Repo‑local `.pi/extensions`

| Candidate | Runtime | Interaction | Capabilities | Complexity | I/O |
|---|---|---|---|---|---|
| `.pi/extensions/diff.ts` | legacy-js | slash_command, ui_integration | exec | medium | fs-heavy |
| `.pi/extensions/files.ts` | legacy-js | slash_command, ui_integration | read | small | fs-heavy |
| `.pi/extensions/prompt-url-widget.ts` | legacy-js | ui_integration | http | medium | network-heavy |
| `.pi/extensions/redraws.ts` | legacy-js | ui_integration | env | small | ui-centric |

### C) badlogic gists

| Candidate | Runtime | Interaction | Capabilities | Complexity | I/O |
|---|---|---|---|---|---|
| `review-extension*.ts` | gist | slash_command, ui_integration | write | medium | fs-heavy |
| `diff.ts` | gist | slash_command, ui_integration | exec | medium | fs-heavy |

### D) Community / npm / git packages

| Candidate | Runtime | Interaction | Capabilities | Complexity | I/O |
|---|---|---|---|---|---|
| `agentsbox` | pi-package | tool_only | exec, http | medium | network-heavy |
| `pi-doom` | pi-package | ui_integration | exec | large | cpu-heavy |

---

## 3) How to Apply the Matrix

1. Compute candidate scores with the executable rubric (`src/extension_scoring.rs`) and persist
   ranked output (`pi.ext.scoring.v1`).
2. Apply hard gates (`provenance_pinned`, license redistribution, deterministic scenario,
   unmodified compatibility). Excluded candidates do not count toward quotas.
3. Allocate Tier-0 first (all official pi-mono), then fill Tier-1 to `>=200` using score order.
4. Enforce source-tier minimums and behavior-bucket quotas using the formula in §1.2.
5. When a quota cannot be met, record `availability_limited` and shortfall metadata.
6. Publish a machine-consumable selection artifact with:
   - per-candidate score breakdown,
   - selected tier (`tier-0|tier-1|tier-2|excluded`),
   - quota-satisfaction summary,
   - explicit manual overrides (if any).
