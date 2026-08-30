# Pauli Hermes First Mate Contract

Status: implementation contract
Reference: `kunchenguid/firstmate`
Pauli control plane: `executiveusa/terabithia`
Pauli coding runtime: `executiveusa/pauli-orca-`
Human control tower: `executiveusa/pauli-command-center`

## Role

Hermes is the single captain-facing First Mate for the Pauli autonomous software factory.

Hermes accepts intent, frames missions, delegates through Terabithia, supervises outcomes, requests captain decisions only when necessary, and returns concise evidence-backed receipts.

Hermes is not a replacement for Terabithia and is not a browser UI.

## Proven Firstmate patterns to adopt

Study and adapt the operational patterns in `kunchenguid/firstmate`:

- one liaison / captain model;
- crew dispatch;
- isolated worktree-per-task execution;
- durable on-disk state and restart recovery;
- event-driven supervision that does not spend model tokens while nothing is happening;
- scout versus ship task shapes;
- explicit project/merge authority;
- bounded escalation for real captain decisions;
- away-mode supervision;
- finished PR, local merge or standalone report handoff;
- persistent second-mate style delegation where useful.

## Critical Orca boundary

Firstmate's documented `backend=orca` refers to the macOS Orca app and CLI.

Do not assume it is compatible with `executiveusa/pauli-orca-`.

Inspect the Pauli Orca runtime and build an explicit Pauli adapter. Reuse Firstmate's isolation/supervision semantics, not an unrelated macOS runtime contract.

## Canonical control path

```text
Captain
  +-- ChatGPT
  +-- Pauli Command Center
  +-- Telegram
        |
        v
      Hermes
   (First Mate)
        |
        v
    Terabithia
        |
        +--> Orca
        +--> BARS/TARS
        +--> Pi
        +--> Jarvis
        +--> Lightning/Watcher
        +--> future specialists
        |
        v
 on-demand isolated execution
        |
        v
      GitHub
        |
 tests / build / preview / proof
        |
        v
      receipt
        |
        v
      Captain
```

## Mission shapes

Minimum supported mission modes:

- `scan`: inspect/report; no project mutation.
- `scout`: bounded investigation that may create a standalone report but does not ship project changes.
- `ship`: authorized implementation path with code changes, verification, branch/PR/merge authority and rollback evidence.

Minimum status vocabulary:

- `queued`
- `planning`
- `running`
- `paused`
- `blocked`
- `needs_human`
- `failed`
- `verified`
- `shipped`

## Mission receipt

Hermes should return structured fields rather than relying on prose alone:

```json
{
  "mission_id": "...",
  "task_id": "...",
  "project_id": "...",
  "repo": "owner/name",
  "mode": "scan|scout|ship",
  "worker": "orca|bars|pi|jarvis|...",
  "branch": null,
  "workspace_id": null,
  "status": "queued",
  "model": null,
  "model_tier": null,
  "model_cost_usd": null,
  "compute_cost_usd": null,
  "preview_url": null,
  "latest_verified_sha": null,
  "rollback_sha": null,
  "evidence": []
}
```

## Supervision principles

1. Do not continuously burn model tokens to supervise idle workers.
2. Workers publish durable state/heartbeat/events.
3. A lightweight watcher wakes Hermes only for actionable state transitions.
4. Repeated unchanged blocked/stale state escalates; normal active work remains quiet.
5. Captain-held decisions remain durable until explicitly resolved.
6. Restart/reconnect reconstructs state from durable records, not chat memory.
7. Every failure remains visible; no silent fallback that pretends work succeeded.

## Agent registry

Specialists should register capabilities and health so Hermes can route by capability instead of hard-coded peer calls.

Example capability families:

- `software_engineering`: Orca
- `culture_operator`: BARS/TARS
- `personal_context`: Pi
- `voice_presence`: Jarvis
- `model_routing`: Lightning/OmniRoute
- `watch_health_cost`: Watcher

Terabithia remains the authoritative fleet bus/routing boundary.

## Voice boundary

Hermes receives text/structured intent. Human-facing speech is an edge adapter.

Pauli Command Center uses Rime as primary TTS for Hermes.

Do not send internal agent-to-agent communication through Rime or any speech provider.

## ICM and knowledge

Hermes uses the project registry to find the project. Project ICM describes what the project is and what stage is current. Supabase provides durable indexed mission/knowledge/lesson retrieval. GitHub remains canonical for source code and version-controlled ICM files.

Do not depend on ChatGPT conversational memory as authoritative project state.

## Negative requirements

- Do not build another orchestrator.
- Do not bypass Terabithia.
- Do not give the browser direct shell/root access.
- Do not expose secrets in prompts, logs, receipts or source code.
- Do not give every worker organization-wide credentials.
- Do not confuse Firstmate's macOS Orca backend with Pauli Orca.
- Do not import tmux/zellij/cmux/Treehouse/AWS voice infrastructure merely because Firstmate supports it.
- Do not let a builder self-approve a high-risk production release.
- Do not label a deployment as verified production without independent runtime proof.
- Do not mutate production instructions from a single successful run; improvements must be evaluated/versioned/reversible.

## First proof

From Pauli Command Center, the captain says:

`Hermes, scan <project> and tell me the next production-ready slice. Do not change anything.`

Pass when:

1. Command Center authenticates the owner.
2. Command Center sends the authenticated request to Hermes through the approved ingress adapter.
3. Hermes receives it as the First Mate and creates durable mission state.
4. Hermes delegates the read-only mission to Terabithia.
5. Terabithia routes the mission to the appropriate worker.
6. A worker returns evidence through Terabithia.
7. Hermes returns one structured receipt to Command Center.
8. No direct browser shell execution occurs.

This keeps the canonical ingress boundary consistent: owner-facing clients talk to Hermes; Hermes delegates through Terabithia; Terabithia never needs to route a request back into Hermes before dispatch.

Then prove one bounded `ship` mission through an isolated cloud sandbox before enabling broad autonomous execution.