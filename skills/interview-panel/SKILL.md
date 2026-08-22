---
name: interview-panel
description: "Persona-driven conversational elicitation. When raw material dropped for social-storytelling-ops is too thin to mine (a short voice memo, a vague text), a chosen interviewer persona asks follow-up questions back through the same gateway channel — one at a time, pushing back on vague answers — until the material has actual specificity, then compiles it into a transcript story-miner can work with. Not a fixed script and not a mandatory 20-minute gate: it runs until the material is specific or the ceiling is hit, whichever comes first."
version: 1.0.0
triggers:
  - interview me about
  - interview me
  - grill me on
  - pull my thoughts out on
entry_point: /interview-panel [persona?] <topic or raw material>
metadata:
  hermes:
    tags: [content, elicitation, social-storytelling-ops, gateway, voice]
    related_skills: [social-storytelling-ops, social-drop-factory, agent-reach]
    capabilities: [conversational-loop, persona-switching, transcript-output]
---

# Interview Panel

## Why this exists

`hermes-workflows/social-storytelling-ops` runs on "human first, human last,
AI in the middle": the human supplies the raw words and the final approval;
everything between is AI structuring, drafting, and reviewing. The weak
point in that shape is the first step — if a team member drops three thin
sentences into the gateway, there's nothing for `story-miner` to mine. This
skill is the active elicitation front-end that fixes that: instead of
Hermes drafting content out of thin material (inventing specifics that
weren't given), a persona interviewer asks for the specifics first.

## When to use it

- `story-miner` receives raw material with no concrete story, moment,
  number, or claim in it — route through here before mining, per
  `hermes-workflows/social-storytelling-ops/stages/00_intake/CONTEXT.md`.
- The team member explicitly says "interview me about X" / picks a persona.
- Skip entirely when the dropped material already has a concrete story —
  don't interview someone who already gave you the goods.

## Personas

Six interviewing styles, one file each in `personas/`. Each file is just
the questioning style — what it presses on, and what counts as "answered."
Pick by explicit request, or default to whichever style suits the topic
(a technical/framework topic defaults to `tim-ferriss.md`; a personal/story
topic defaults to `barbara-walters.md`).

- `personas/tim-ferriss.md` — drills into the specific mechanism: what
  exactly did you do, in what order, what would you do differently.
- `personas/joe-rogan.md` — genuine curiosity, "wait, really — how does
  that work?", follows tangents that turn out to be the real story.
- `personas/larry-king.md` — short, direct questions, no editorializing,
  keeps moving, lets the answer breathe instead of stacking questions.
- `personas/howard-stern.md` — asks the question everyone's thinking but
  polite conversation skips; presses on what it actually felt like.
- `personas/michael-barbaro.md` — narrative sequencing: what happened
  first, then what, then what — builds the timeline out loud.
- `personas/barbara-walters.md` — presses on cost and stakes: what did
  this cost you, what were you afraid of, what changed after.

## Flow

1. **Pick persona + topic.** From an explicit request, or from the thin
   raw material `story-miner` bounced back with a reason it was too thin.
2. **Ask one question at a time**, through the same gateway channel the
   raw material arrived on (Telegram/WhatsApp/Slack/etc. — see
   `gateway/platforms/`). Wait for the reply before asking the next
   question. Never batch multiple questions in one message — that's what
   produces the vague, unspecific answers this skill exists to avoid.
3. **Push back on vague answers.** If a reply is generic ("it went well",
   "we learned a lot"), the persona's follow-up asks for the specific
   instance: a number, a name, a moment, a quote. Do not accept a second
   vague answer to the same question without at least one push-back.
4. **Exit conditions**, first one hit wins:
   - the persona judges the material now has enough specificity (at least
     one concrete story/moment, one quotable line, one verifiable claim or
     number) for `story-miner` to build a graph from;
   - the person says "that's enough" / "stop" / equivalent;
   - **30-minute ceiling** from the first question — hard stop regardless
     of completeness. Compile whatever specificity was gathered; do not
     extend past the ceiling even if the material is still thin.
5. **Compile the transcript.** Write one markdown file (see Output below)
   and hand it to `story-miner` exactly like any other intake source per
   `hermes-workflows/social-storytelling-ops/stages/00_intake/CONTEXT.md`.

## Rules

- Never put words in the interviewee's mouth. The persona asks; it does
  not answer for them or paraphrase an answer into something stronger than
  what was said. `story-miner`'s "no invented copy" rule starts here.
- Never run more than one persona on the same raw material in the same
  session — pick one and stay in that voice for the whole interview.
- If the interviewee goes quiet for the platform's normal idle window,
  save partial progress and let intake fall back to whatever material
  exists rather than blocking the run indefinitely.
- This skill produces source material. It does not draft posts, pick a
  platform, or make campaign decisions — that's `campaign-architect` and
  `reel-director` downstream, working from `story-miner`'s output.

## Output

Write `interview-<topic-slug>-<date>.md`:

```markdown
# Interview: <topic>

- Persona: <persona file used>
- Interviewee: <sender id from gateway>
- Platform: <gateway platform>
- Started: <ISO8601>
- Ended: <ISO8601>
- Exit reason: sufficient_specificity | interviewee_stopped | ceiling_reached

## Full Q&A

**Q:** ...
**A:** ...
**Q (follow-up, pushed back on vague answer):** ...
**A:** ...

## Key stories extracted

- ...

## Core insights

- ...

## Quotable moments

> "..."

## Claims needing verification

- ...
```

This file is the handoff to `story-miner` — same evidentiary status as a
directly-provided transcript. `story-miner` still reads it in full; this
skill does not summarize on `story-miner`'s behalf.
