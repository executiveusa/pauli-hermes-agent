# 00 Intake
Collect client, business/social outcome, audience, campaign cadence, source assets, protected facts, deadlines, platform accounts, budget, consent/privacy constraints, and proof required. Output `run-brief.md`. Gate: do not proceed if source ownership, target account, or success criteria are unclear.

MODE: BROWNFIELD. This is additive intake routing on top of whatever source
collection this stage already does — it does not replace or restructure it.

## Team capture intake

Any raw material a team member drops for this workflow becomes intake
source material through one of three additive routes. In every route the
resulting object is handed to `story-miner` the same way an existing
transcript/footage source already is — `story-miner` does not need a
different code path per source type, only a different upstream producer.

### 1. Voice memos / text dropped into the gateway

Repo-reality check: `gateway/hooks.py` is a lifecycle event system
(`session:start`, `agent:start`, `agent:step`, `command:*`, …) that fires
handlers loaded from `~/.hermes/hooks/<name>/HOOK.yaml` + `handler.py`
(`gateway/builtin_hooks/` ships empty, kept as an extension point). It has
no concept of message content or workflow tags. `gateway/session_context.py`
only manages per-session environment variables. Neither file has an
existing "route this tagged message to a workflow" mechanism — the prompt
that asked for this integration assumed one exists; it doesn't yet, and
this stage does not add gateway-level code to invent one, since every other
`hermes-workflows/*/CONTEXT.md` in this repo routes by documented agent
convention rather than platform code, and a code-level router would be the
one genuinely new ingestion path the brief said to avoid building.

The convention instead: any inbound message on a platform in
`gateway/platforms/` (Telegram, WhatsApp, Slack, etc.) that is explicitly
tagged for this workflow — e.g. a reply/thread anchored to a run, or the
literal tag `#social-storytelling-ops` / `workflow:social-storytelling-ops`
in the message — is treated by Hermes as raw INGEST source material for
this run: forward the message text/voice-transcript and any attachment
pointers straight to `story-miner` as a candidate source, tagged with
sender, platform, and timestamp. Untagged messages are not pulled into this
workflow. If the team wants this enforced in code rather than by agent
convention, that is a `gateway/hooks.py` builtin-hook addition (a handler on
`agent:start` that inspects the tag and sets a session var via
`session_context.set_session_vars`) — out of scope here since it is new
gateway infrastructure, not additive routing onto something that exists.

If the dropped material is a short/thin voice memo — a few sentences, no
concrete story, numbers, or moment — do not hand it to `story-miner` as-is.
Route it through `skills/interview-panel/` first (persona-driven follow-up
questions back through the same gateway channel) so `story-miner` receives
material with actual specificity. See `skills/interview-panel/SKILL.md` for
the elicitation flow and its exit conditions. Skip the interview when the
dropped material already contains a concrete story, moment, or claim.

### 2. YouTube links

Route through `agent-reach` → `youtube-knowledge-extractor` per the handoff
now defined in `skills/agent-reach/references/youtube.md` ("Handoff to
youtube-knowledge-extractor") and
`skills/agent-reach/schemas/transcript-record.json`. The resulting
structured knowledge object (summary, key takeaways, quotable moments,
source transcript) becomes `story-miner` source material with the same
evidentiary weight as a directly-provided transcript — `story-miner` still
reads the full underlying transcript for hooks/tension/proof per
`stages/01_story-map/CONTEXT.md`, the knowledge object is a routing/index
layer on top, not a replacement for reading the source.

### 3. Local footage references

When a team member's raw material references something already shot
("like the shot of the boat leaving the dock"), `story-miner` queries the
existing local footage index before assuming new footage is required:

```bash
python skills/local-footage-studio/scripts/footage_studio.py search \
  --query "<description from the raw material>"
```

A match returns source path and timestamps (see
`skills/local-footage-studio/SKILL.md`, Phase 5 — search) and is added to
the story graph as available evidence for `reel-director` to cut against.
No match means the footage is genuinely new and `00_intake`'s normal
source-collection gate applies.