---
name: emerald-tablets
description: Studio skill — Emerald Tablets.
version: 1.0.0
author: Bambú / Pauli Effect
---

# Emerald Tablets Skill

Loaded for autonomous studio agents. Full source below.

## When to Use
Use when the task matches this skill's domain.

## Source
`C:\Users\execu\Downloads\AI WORKSHOP\MASTER SKILLS BUNDLE\agent-must-read-this\emerald-tablets-SKILL.md`

---

---
name: emerald-tablets-governance
description: >
  The Emerald Tablets™ — prime directive governance layer for every repo in the Pauli
  Effect™ ecosystem (executiveusa GitHub org). Reads as a hard constraint file, not
  documentation: any agent (HERMES™, RALPHY, ARCHITECT, Claude Code, Goose, Cursor,
  Windsurf, or any MCP-capable coding agent) must resolve all seven tablets across
  three tiers before code is written, before a commit is made, and before a merge
  is approved. Use this skill at the start of ANY session in a repo under the
  executiveusa org, before writing code, before reviewing a PR, before generating
  copy or documentation, and before declaring any task complete. Triggers on:
  "build this", "ship this feature", "review this PR", "is this ready to merge",
  "write the docs for this", "what's our quality bar", "audit this repo",
  starting a new repo, or any first action in an unfamiliar repo. This skill
  governs WHETHER something ships — not how it's built (see pike-engineering-discipline
  for that) or how it's visualized (see interactive-artifacts for that). Constitutional
  authority: supersedes deadline pressure, supersedes "good enough for now," supersedes
  individual agent preference. Three-tier cascade: language/quality blocks first,
  architecture blocks second, execution/culture blocks third. A failure at any tier
  halts before the next tier is evaluated.
emerald_tablets: I, II, III, IV, V, VI, VII
quality_floor: 8.5
authority: prime_directive
author: Pauli Effect™
version: "1.0"
---

# Emerald Tablets™ — Prime Directive Governance Skill
## Pauli Ecosystem™ Constitutional Law
### Authority: supersedes all other skills, preferences, and deadline pressure
### Applies to: every repo under executiveusa · every agent · every commit

---

## PRIME DIRECTIVE

This is not a style guide. This is not a suggestion. This is the file that determines
whether output is allowed to exist in the Pauli Effect™ ecosystem.

The Emerald Tablets™ are seven laws, divided into three tiers, that cascade in order.
A failure in Tier 1 means Tier 2 is never evaluated — there is no point checking
architecture on output that hasn't earned the right to be examined for architecture.
A failure in Tier 2 means Tier 3 is never evaluated — there is no point checking
whether execution was clean on a design that violates single responsibility.

This is the structural reason the tablets work where checklists fail. A checklist
lets a strong score on item 9 compensate for a weak score on item 2. The tablets do
not average. They gate.

```
TIER 1 (language + quality)  →  PASS  →  TIER 2 (architecture)  →  PASS  →  TIER 3 (execution + culture)
       ↓ FAIL                                    ↓ FAIL                              ↓ FAIL
   HALT. Fix Tier 1.                      HALT. Fix Tier 2.                  HALT. Fix Tier 3.
   Tier 2 never runs.                     Tier 3 never runs.                 Nothing ships.
```

---

## STEP 0 — MANDATORY INSTALLATION CHECK

Before any agent action in a repo under `executiveusa`, run this check:

```bash
# Confirm the tablets exist at repo root
ls EMERALD_TABLETS.md 2>/dev/null || echo "MISSING — install before proceeding"

# If missing, install immediately from this skill before any other work
cat > EMERALD_TABLETS.md << 'EOF'
[contents of SECTION 1-3 below, condensed — see INSTALLATION section]
EOF
```

If `EMERALD_TABLETS.md` is missing from a repo root, the agent's first action —
before any feature work, before any bug fix, before any commit — is to install it.
A repo without the tablets is a repo with no quality floor, and nothing built in
it can be trusted by the next agent that opens it.

---

## TIER 1 — LANGUAGE AND OUTPUT QUALITY

This tier governs every word and every score before anything else is evaluated.
It exists because vague language is how quality erodes silently. A team that allows
"this seamlessly handles edge cases" never has to prove the edge cases are handled.

### TABLET I — ANTI-HYPE LAW™

**The law**: No marketing language in code, comments, commit messages, documentation,
UI copy, or agent-to-human communication. Every claim must be measurable or it does
not get made.

**Banned words — automatic Tier 1 failure if found**:
```
seamless        robust          innovative      leverage (as verb)
synergy         utilize         revolutionize   transforming
elevating       comprehensive   cutting-edge    state-of-the-art
world-class     game-changing   next-level      best-in-class
```

**Replacement protocol**:
```
"Seamlessly integrates with WhatsApp"
  → "Delivers WhatsApp messages within 800ms at 99.7% success rate, automatic retry on failure"

"Robust error handling"
  → "Catches and logs all exceptions; retries network calls up to 3 times with exponential backoff"

"Innovative AI-powered solution"
  → "Routes requests through Claude Sonnet 4.6 with a 12-agent specialization graph"
```

**Self-check before any commit, PR description, or doc**:
```
[ ] Does every adjective have a number, a measurement, or a specific mechanism behind it?
[ ] If I deleted every banned word from this list, would the sentence still make a claim?
[ ] Could a skeptical engineer verify this claim by reading the code?
```
If any answer is no — rewrite before proceeding. This is a Tier 1 gate. Nothing
downstream gets evaluated until this passes.

---

### TABLET II — QUALITY FLOORS™

**The law**: UDEC scoring applies to every design, system, and output artifact.
The floor is 8.5/10. It is not an average — it is a floor. Auto-iterate until met.
There is no override except explicit human authorization with a documented reason
written into the commit message.

**The cascade**:
```
IF overall UDEC score < 8.5
  THEN identify lowest-scoring axis
  AND read the specific fix pattern for that axis
  AND apply the fix
  AND re-score
  REPEAT until ≥ 8.5 OR 5 iterations exhausted
  IF 5 iterations exhausted without reaching floor:
    ESCALATE to human with full iteration log — do not ship below floor
```

**Hard sub-floors that block regardless of overall score**:
```
Feedback Completeness < 7.0  →  HALT — redesign feedback structure first
Resilience Design < 7.0       →  HALT — redesign resilience before anything else
Secret Safety < 8.0           →  HALT COMPLETELY — rotate compromised secrets
```

**What this means in practice**: a system that scores 9.4 overall but 6.0 on
Resilience Design does not ship. The high score elsewhere does not buy back the
weak axis. This is why the tablets gate instead of average — a system can be
excellent everywhere except the one place it will actually fail in production,
and a single weighted average hides that.

**Where this applies**:
```
Frontend design        → design-workflow-e2e skill, 14-axis UDEC
Backend architecture    → synthia-systems-architect skill, 12-axis Meadows scoring
Interactive artifacts   → interactive-artifacts skill, 10-axis sandbox+quality scoring
Copy and content        → P.A.S.S.™ framework + anti-hype law (this tablet)
```

---

## TIER 2 — ARCHITECTURE

Evaluated only after Tier 1 passes. This tier governs the shape of every file,
every agent, and every repository — not the words inside them, but the structure
that determines whether the words can be trusted to scale.

### TABLET III — TASTE AS DISCIPLINE™

**The law**: Design quality is a technical requirement, not an aesthetic preference.
Poor visual hierarchy produces poor user behavior, which produces poor data, which
produces poor decisions. Taste is upstream of everything else in the product.

**Decision protocol**:
```
IF a design decision is justified only by "I like it" or "it looks modern"
  THEN it has not passed Tablet III
  DO trace the decision to a specific user outcome it improves
  DO cite the UDEC axis it serves (clarity, hierarchy, affordance, etc.)

IF a design decision cannot be traced to a measurable outcome
  THEN treat it as decoration, not architecture
  DO remove it or justify it before shipping
```

**What this blocks**:
```
❌ "This gradient looks nicer" — no traceable outcome, decoration
❌ "I added a carousel for visual interest" — adds cognitive load, fails Krug's laws
❌ Numbered markers (01/02/03) on content that isn't actually sequential
❌ Generic AI-design defaults: cream bg + terracotta accent, near-black + neon accent,
   broadsheet hairline layout — used because they're safe, not because the brief calls for them
```

**What this requires**:
```
✓ Every color choice maps to a semantic role (see interactive-artifacts SECTION 4)
✓ Every structural device (divider, label, numbering) encodes something true about the content
✓ Typography pairing is deliberate per-brief, not a repeated default across all repos
```

---

### TABLET IV — SINGLE RESPONSIBILITY™

**The law**: One agent, one domain, one job. No god classes. No god agents.
Blast radius of any single automated action is capped at 3 services.

**The god-class detector**:
```
IF a service handler, orchestrator, or agent definition exceeds ~300 lines
  OR handles more than one domain of responsibility
  OR its name requires "and" to describe what it does
    (e.g. "AuthAndProjectManager", "HandlesAuthAndBilling")
  THEN it is a god class
  DO decompose into single-responsibility domain services
  DO compose via dependency injection, not inheritance
```

**The blast radius circuit breaker**:
```
IF a single automated action would affect > 3 services simultaneously
  THEN HALT
  DO require an explicit multi-service deploy plan with human sign-off
  DO NOT proceed with auto-merge or auto-deploy
```

**Agent scope table — the canonical HERMES™ roster and what each is NOT allowed to do**:

| Agent | Domain | Forbidden scope creep |
|---|---|---|
| HERMES™ | Orchestration, routing | Never writes business logic itself |
| ARCHITECT | Systems design | Never executes code, only specs it |
| CONCIERGE | Client intake | Never makes pricing decisions |
| LENA™ | Copy, P.A.S.S.™ | Never touches backend logic |
| RALPHY | Execution loop | Never self-approves its own PR |
| MARCO | Revenue routing | Never modifies agent definitions |

**Barrel file ban** (a structural instance of this tablet):
```javascript
// ❌ BANNED — index.ts re-exporting multiple modules creates merge-conflict hotspot
export * from './auth';
export * from './billing';
export * from './users';

// ✓ CORRECT — direct imports, no aggregation point
import { login } from './auth/login';
import { chargeCard } from './billing/charge';
```

---

### TABLET VI — REPO AS PRODUCT™

**The law**: Every repository ships like it is a product, because to the next agent
or human who opens it, it is one. No scratchpad repos. No "clean up later."

**The minimum bar for any repo to exist under `executiveusa`**:
```
[ ] README.md — what this repo is, how to run it, who it's for
[ ] EMERALD_TABLETS.md — this file, at root
[ ] SKILL.md (if agent-operable) — discovery description + instructions
[ ] CI pipeline — tests run on every PR, no exceptions
[ ] ops/reports/ — machine-readable completion logs from every agent run
```

**Decision protocol**:
```
IF a repo is created "just to test something quickly"
  THEN it still gets a README and a CI pipeline
  BECAUSE "quick test" repos are the ones still in production 18 months later
  AND the next agent that opens it has zero context unless this bar is met
```

---

## TIER 3 — EXECUTION AND CULTURE

Evaluated only after Tiers 1 and 2 pass. This tier governs how work actually
gets done and which market it's built for — the last gate before anything ships.

### TABLET V — RALPHY LOOP™

**The law**: write → test → fix → verify → report. No agent declares a task
complete without passing all five stages, in order, with the report stage
producing a machine-readable artifact.

**The five stages, non-negotiable order**:
```
1. WRITE   — implement the change
2. TEST    — run the test suite, not just "looks right"
3. FIX     — address any failures found in TEST
4. VERIFY  — re-run TEST after FIX; confirm green
5. REPORT  — write JSON to ops/reports/ — see format below
```

**What "declaring complete" without this looks like — and why it's banned**:
```
❌ "I've implemented the feature" — no TEST stage evidence
❌ "This should work" — no VERIFY stage evidence
❌ "Done" with no ops/reports/ entry — no REPORT stage, next agent has zero context
```

**Required report format**:
```json
{
  "bead_id": "[TASK-ID]",
  "agent": "[which agent ran this]",
  "stages_completed": ["write", "test", "fix", "verify", "report"],
  "test_result": "pass",
  "iterations_to_pass": 2,
  "timestamp": "2026-06-29T00:00:00Z",
  "zero_context_handoff": true
}
```

**The zero-context handoff test**: a different agent, with no memory of this
session, must be able to read the `ops/reports/` entry and continue the work
without re-deriving what happened. If they can't, REPORT stage failed even if
the code itself is correct.

---

### TABLET VII — LATAM SPECIFICITY™

**The law**: Synthia™ 3.0 and any LATAM-facing product is designed Spanish-first,
not translated from English after the fact. Pricing in MXN. Onboarding via
WhatsApp. Every design decision made inside this constraint from the start.

**This tablet functions differently from the other six** — it is not a quality
gate, it is a market constraint. It does not block bad output; it redirects
output toward the right customer before output is even written.

**Decision protocol**:
```
IF building anything customer-facing for Synthia™ 3.0 / Kupuri Media™
  THEN write the Spanish copy FIRST, not as a translation pass
  AND price in MXN as the primary currency, not USD-converted
  AND design the onboarding flow around WhatsApp as the entry point,
      not email or a web form with WhatsApp bolted on after

IF a design defaults to English-first with Spanish as a locale switch
  THEN it has violated Tablet VII regardless of how polished the English version is
```

**ENTREPRENEUR_LATAM tier copy voice** (per existing P.A.S.S.™ templates):
```
Five voice contexts, all Spanish-native, not translated:
  1. WhatsApp intake
  2. Audit delivery
  3. Proposal
  4. Payment confirmation
  5. Social content
```

**What this does NOT apply to**: Akash Engine client work that is explicitly
US/English-market (e.g. a Seattle-based client retainer). Tablet VII scopes to
LATAM-facing products specifically — applying it universally would itself be a
violation of Tablet IV (single responsibility / right tool for the right scope).

---

## THE FULL CASCADE — WORKED EXAMPLE

To make the gating mechanism concrete, here is how a single feature request
moves through all three tiers.

**Request**: "Add a revenue dashboard widget to Synthia™ Studio."

```
TIER 1 CHECK
  Draft copy: "Seamlessly visualizes your revenue with cutting-edge charts"
  → FAIL — "seamlessly" and "cutting-edge" are banned (Tablet I)
  → Rewrite: "Shows daily revenue by stream — SaaS, Operator, Marketplace —
     updated every 15 minutes from Stripe webhook events"
  → PASS — every claim is specific and verifiable

  UDEC score of the widget spec: 7.8/10 (Feedback Completeness: 6.5)
  → FAIL — sub-floor on Feedback Completeness (Tablet II)
  → Fix: add a "data freshness" indicator and a manual refresh feedback loop
  → Re-score: 8.7/10, Feedback Completeness 7.4
  → PASS

TIER 2 CHECK (Tier 1 passed, now evaluated)
  Does this widget live inside MARCO's revenue-routing domain or does it
  reach into LENA's copy domain to render its own labels?
  → Initial design had the widget generating its own UI copy inline
  → FAIL — violates Tablet IV, MARCO should not own copy generation
  → Fix: widget calls LENA™ for label text, MARCO only supplies the data
  → PASS

  Is this a one-off file or does it meet repo-as-product bar?
  → Repo already has README, EMERALD_TABLETS.md, CI, ops/reports/
  → PASS (Tablet VI)

TIER 3 CHECK (Tiers 1 and 2 passed, now evaluated)
  Ralphy Loop: write → test (Stripe webhook mock) → fix (timezone bug found)
  → verify (re-test green) → report (JSON written to ops/reports/)
  → PASS (Tablet V)

  Is this LATAM-facing (Synthia™ 3.0) or Akash Engine US client work?
  → Synthia™ 3.0 context confirmed
  → Copy was written in Spanish first via LENA™, MXN as primary currency
  → PASS (Tablet VII)

RESULT: ships. Every tier passed in order. Nothing was evaluated out of sequence.
```

---

## SELF-AUDIT CHECKLIST — RUN BEFORE ANY MERGE

```
TIER 1 — LANGUAGE AND QUALITY
  [ ] Zero banned words in code comments, commit messages, UI copy, docs
  [ ] Every adjective/claim traces to a number or named mechanism
  [ ] UDEC overall score ≥ 8.5
  [ ] Feedback Completeness ≥ 7.0 (if systems work)
  [ ] Resilience Design ≥ 7.0 (if systems work)
  [ ] Secret Safety ≥ 8.0 (if any credentials touched)
  → If any box unchecked: HALT. Do not evaluate Tier 2.

TIER 2 — ARCHITECTURE
  [ ] Every design decision traces to a measurable outcome, not "looks nice"
  [ ] No god classes (no handler/agent >~300 lines or multi-domain)
  [ ] No god agents (check against agent scope table)
  [ ] Blast radius ≤ 3 services for any single automated action
  [ ] No barrel files / re-export aggregators in agent-touched code
  [ ] Repo has README, EMERALD_TABLETS.md, SKILL.md (if applicable), CI, ops/reports/
  → If any box unchecked: HALT. Do not evaluate Tier 3.

TIER 3 — EXECUTION AND CULTURE
  [ ] Ralphy Loop: write → test → fix → verify → report, all five stages evidenced
  [ ] ops/reports/ JSON written, zero-context-handoff readable
  [ ] If LATAM-facing: Spanish-first copy, MXN primary pricing, WhatsApp-native onboarding
  [ ] If US/English-market client work: Tablet VII correctly scoped out (not misapplied)
  → If any box unchecked: HALT. Do not ship.

ALL THREE TIERS PASS → ship.
```

---

## INSTALLATION — HOW TO INSTALL THIS IN A NEW REPO

Every new repo under `executiveusa` gets `EMERALD_TABLETS.md` at root as its
first commit, before any feature code. Use this condensed version for the
repo-root file (this skill file itself stays in `skills/emerald-tablets-governance/`
for agent reference; the root file is the enforcement copy agents check for):

```bash
cat > EMERALD_TABLETS.md << 'EOF'
# Emerald Tablets™ — Repo Constitution

This repo is governed by the Emerald Tablets™. Full skill at
skills/emerald-tablets-governance/SKILL.md. Summary:

TIER 1 (blocks first): Tablet I anti-hype law, Tablet II quality floors (UDEC 8.5)
TIER 2 (blocks second): Tablet III taste as discipline, Tablet IV single responsibility,
  Tablet VI repo as product
TIER 3 (blocks third): Tablet V Ralphy Loop, Tablet VII LATAM specificity (if applicable)

No agent merges code that fails any tier. Tiers cascade — a Tier 1 failure means
Tier 2 and 3 are not evaluated. Fix in order.
EOF

git add EMERALD_TABLETS.md
git commit -m "[TABLETS][INIT-001] docs: install Emerald Tablets governance | Tablet VI | repo-as-product floor established"
```

---

## RELATIONSHIP TO OTHER SKILLS

This skill is the prime directive. Other skills operate inside its constraints,
not alongside them:

```
emerald-tablets-governance™    ← THIS FILE — supersedes all others
  ├── pike-engineering-discipline   (HOW code is written — Tier 2/3 detail)
  ├── synthia-systems-architect     (HOW backend systems are designed — Tier 2 detail)
  ├── interactive-artifacts         (HOW UI artifacts are built — Tier 1/2/3 detail)
  ├── design-workflow-e2e           (HOW frontend design is scored — Tier 1/2 detail)
  └── zte-autodeploy                (HOW deploys execute — Tier 3 detail)
```

When a more specific skill's guidance conflicts with a tablet, the tablet wins.
A pike-engineering-discipline gate that says "ship the simple version" does not
override Tablet II's 8.5 floor — it determines HOW to reach 8.5, not WHETHER
8.5 is required.

---

## SUMMARY

Seven tablets. Three tiers. A cascade, not an average.

Tier 1 asks: is this true, and is it good enough? Tier 2 asks: is this built
right? Tier 3 asks: was this actually finished, and was it built for the right
person? Each tier gates the next. A repo without `EMERALD_TABLETS.md` at its
root has no floor, and nothing built in it can be trusted by the next agent
that opens it — human or otherwise.

The car is nice. The engine is what wins. The tablets are the engine's
compression ratio — the constraint that makes the power usable instead of
just loud.

