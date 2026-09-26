# Hermes Agent Activation Protocol

## How Hermes Auto-Activates This Skill

When Hermes receives a request that asks what people are currently saying, trending, or reacting to on social/news platforms, it automatically:

1. **Detects request type** — Scans for "last 30 days", "trending", "what's hot", "what are people saying about", "social listening" + a topic
2. **Routes to workflow** — Loads `skills/research/last30days/SKILL.md` and follows its engine-invocation contract
3. **Runs the engine** — Executes `scripts/last30days.py` against Reddit, X, YouTube, TikTok, Hacker News, Polymarket, GitHub, and the web
4. **Synthesizes results** — Produces a badge-led, law-governed report with the engine's pass-through footer (never raw evidence dumps)

## Detection Pattern

Hermes triggers this skill when the request contains:

**Keyword combinations:**
- `last30days` / `/last30days` (explicit invocation)
- `trending` + no specific tool named (global or domain trending)
- `what's hot` / `what's exploding` / `what's blowing up` + a topic or domain
- `what are people saying about` / `what's the community saying about` + a topic
- `research the last 30 days of` / `social listening on` + a topic
- `content angles for` / `podcast angle` / `X article angle` + a topic (content-creation framing)

**Examples that activate:**
```
✓ "/last30days nvidia earnings reaction"
✓ "what's trending in AI agents right now?"
✓ "what are people saying about the new iPhone on Reddit and X?"
✓ "give me content angles for a podcast about vector databases"
✗ "tell me about NVIDIA" (no recency/social-listening signal — use general research)
✗ "search my notes for nvidia" (not social/web listening)
```

## Skill Metadata

```json
{
  "last30days": {
    "triggers": [
      "/last30days",
      "last 30 days of",
      "trending",
      "what's hot right now",
      "what are people saying about",
      "social listening on",
      "content angles for",
      "podcast angle",
      "X article angle"
    ],
    "entry_point": "/last30days [topic]",
    "activation_style": "automatic-on-keyword"
  }
}
```

## Direct Invocation

```
/last30days <topic>
/last30days trending
/last30days --trending
/last30days what's exploding in <domain>?
```

## Fallback / Manual Activation

```bash
python3 skills/research/last30days/scripts/last30days.py "<topic>" --emit=compact
```

## What Hermes Knows

✓ Knows where the skill lives (`skills/research/last30days/`)
✓ Knows how to trigger it (`/last30days` or auto via keywords)
✓ Knows the SKILL.md is a strict output contract (LAWS 1-11) — badge, no invented titles, no `##` headers outside COMPARISON, engine footer pass-through, community-voice weaving, discovery three-command protocol
✓ Knows the engine is the source of truth — WebSearch alone is a supplement, never a substitute
✓ Knows results can be saved to a local research library (`LAST30DAYS_MEMORY_DIR`, default `~/Documents/Last30Days`) for later reuse
