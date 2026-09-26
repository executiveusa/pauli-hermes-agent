# Where's Pauli — ICM Project Router

## Purpose
This is the project-specific operating context for `executiveusa/wherespauli`.

Hermes should load this folder whenever the owner asks to work on Where's Pauli, Pauli Pass, Case File 001, the 12-shot Seattle scroll world, the Living Story Engine, or the reusable Interactive Avatar World Engine.

This folder does **not** duplicate reusable skills. It routes the project into the correct shared workflows and preserves project-specific canon, visual laws, product boundaries, and current execution order.

---

# 1. I — INTERPRET

Before acting, identify which Where's Pauli layer the request belongs to:

1. **STORY AUTHORITY** — canon, episode facts, reveals, clues, character knowledge.
2. **PLAYER AUTHORITY** — what a player has discovered, chosen, inferred, missed, replayed, or earned.
3. **MEMBERSHIP AUTHORITY** — Pauli Pass, founding provenance, access, invites, rewards.
4. **SCROLL WORLD** — 12-scene cinematic Seattle experience and its mobile behavior.
5. **LIVING STORY** — Case File 001, branching, evidence, theories, character response boundaries.
6. **MEDIA** — locked stills, living stills, 2.5D, optional video, provider-neutral generation.
7. **INFRASTRUCTURE** — hosting, database, deployment, auth, analytics, VPS/Coolify.

Never collapse these authorities into one mutable state object.

If a request touches more than one layer, declare the boundary before building.

---

# 2. C — CONTEXT TO LOAD

## Canonical project repo
- Repo: `executiveusa/wherespauli`
- Default branch: `main`
- Last verified project head at this workflow creation: `35d278e327872fdf29c9ecdeefbb4c5e830fa032`
- That commit contains the 12-part scroll-world MVP and Pauli Pass signup framing.

Always re-check current repo state before consequential work. Do not assume this SHA remains current.

## Shared Hermes workflows/skills
Load only as needed:

### Scroll/cinematic world
- `hermes-workflows/scroll-world-design/`
- `skills/studio/cinematic-2-5d-scenes/`
- `hermes-workflows/cinematic-2-5d-scenes/`
- Reference mechanics: `oso95/scroll-world`

### Video escalation
- `skills/studio/cinematic-master-editor/`
- `hermes-workflows/cinematic-production/`

### Quality
- `skills/gauntlet-loop/`

### Long-running work
- hardened long-run/subagent harness when a task spans sessions or many assets.

## Project-specific sources
When available in `executiveusa/wherespauli`, prefer:
- `docs/canon/`
- `docs/episodes/`
- `AGENTS.md`
- `CONTEXT.md`
- ICM folders `01_discover` through `06_repair`

Current repo canon outranks this Hermes summary if they disagree.

---

# 3. M — METHOD / EXECUTION ROUTER

## Current product objective
Build Where's Pauli into a mobile-first cinematic Seattle mystery world that:

1. opens with a seamless 12-scene investigation;
2. feels like a live drone/search feed rather than a conventional landing page;
3. transitions into Case File 001 / Living Story interaction;
4. creates a persistent Pauli Pass / Founding Investigator identity after meaningful activation;
5. grants exactly one invite entitlement at the defined activation gate;
6. can later expand into real Seattle locations, rewards, Pauli's Place, and reusable Interactive Avatar Worlds.

The immediate product loop is:

```text
INVITE / ENTRY
      ↓
12-SCENE CINEMATIC SEARCH
      ↓
CASE FILE / INVESTIGATION
      ↓
MEANINGFUL ACTIVATION
      ↓
PAULI PASS / FOUNDING PROVENANCE
      ↓
ONE INVITE
      ↓
RETURN / REPLAY / DEEPER ACCESS
```

---

# 4. STORY LAWS

## Core mystery
The project should continually make the audience ask what is really happening around Pauli.

Pauli is not a conventional mascot or exposition device. Mystery is the engine.

## Canon model
Lock:

```text
CANON TRUTH
    ↓
EVIDENCE
    ↓
PLAYER KNOWLEDGE
    ↓
PLAYER THEORY
```

Choices can change:
- route;
- evidence discovered;
- inventory/state;
- character interactions;
- interpretation;
- access/reward eligibility;
- replay meaning.

Choices do **not** silently rewrite canon.

## Early reveal rule
For the early episode arc, Pauli remains visually unconfirmed until canon explicitly permits a reveal.

For Episode 001, do not add a face, silhouette, foot, coat edge, body fragment, or equivalent accidental reveal if canon forbids it.

## 12-shot grammar
The opening world uses a 12-shot Seattle progression. Treat the current episode shot map as authored story structure, not arbitrary website sections.

Current working sequence:

1. Orbit / Pacific Northwest
2. Seattle region acquisition
3. Downtown descent
4. Waterfront sweep
5. Space Needle / `12` signal event
6. Pioneer Square
7. Food Bank
8. Post Alley
9. Eldorado / positive lead
10. Pauli's Place / target location
11. anomaly / target confirmation / system failure escalation
12. contact without verified Pauli reveal

Re-check episode canon before editing shot 11 or 12.

---

# 5. VISUAL DNA

## Prime law
**The world image is the hero. The drone feed informs.**

Do not turn the experience into a large dashboard, marketing hero, or interface-heavy SaaS screen.

## Image authority
If an image is marked locked/approved by the owner:
- do not regenerate it;
- do not repaint it;
- do not substitute it;
- do not alter story content inside it;
- do not replace it because a new model can make something prettier.

Enhancement should happen around the locked plate through camera, depth, atmosphere, light, overlays, timing, and transitions.

## Default visual treatment
- black and white / monochrome;
- high-contrast noir/surveillance texture;
- imagery dominates;
- typography and HUD remain restrained;
- color pops are evidence or authored anomalies, not decoration.

## Color policy
Default: monochrome.

For the first 12-shot sequence, use at most one strong authored color event unless canon changes. The current preferred event is the Scene 05 blue/green `12` flag already present in the locked artwork.

Do not randomize color inside canon storytelling. Seeded/random surprise color may be tested only in explicitly non-canon experiments.

---

# 6. TYPOGRAPHY / FONT RESEARCH

Typography is an early design decision, not final polish.

When Hermes needs a font or type reference:

1. **Search `fontsinuse.com` first** for real-world typography precedent.
2. Search by:
   - surveillance / intelligence / editorial / noir / civic / Seattle / wayfinding mood;
   - output format (web, film/video, identity, signage, mobile);
   - known typefaces;
   - comparable cultural references.
3. Capture:
   - typeface name;
   - real usage/project;
   - context;
   - why it fits the narrative job;
   - official foundry/source link.
4. Fonts In Use is a **research/precedent source**, not the licensing authority.
5. Verify availability/license at the official foundry/source.
6. If the exact typeface is unavailable or unsuitable, find lawful alternatives via Fontshare, Google Fonts, Open Foundry, Typewolf, or official foundries.
7. Never scrape or redistribute font binaries from Fonts In Use.

For Where's Pauli, typography should generally resolve to a small system:
- drone/HUD mono or technical face;
- restrained display/editorial face only where needed;
- highly legible body/interface fallback.

Do not let typography overpower the imagery.

---

# 7. SCROLL WORLD ENGINE

## Goal
The first 12 scenes should feel like **one continuous investigation**, not 12 stacked image sections.

Use one global scroll clock and one fixed cinematic viewport.

## Rendering modes

### LIVE_STILL — default
Locked still + HTML/CSS/Canvas:
- 2.5D multiplane depth;
- camera push/pull/glide;
- mobile discovery pan;
- rain/fog/steam;
- lamp/window flicker;
- reflection breathing;
- HUD progression;
- signal artifacts.

### HYBRID
LIVE_STILL scenes with selected video connectors/payoffs.

### VIDEO_CHAIN
Scroll-scrubbed video legs in the style inspired by `oso95/scroll-world` when the story truly needs it.

Video is optional. Do not make the experience depend on video merely because it is cinematic.

## Perceptual Seam Lock
Extend scroll-world's seam idea to stills.

A still-to-still seam passes only when:
- outgoing and incoming scenes share a continuous camera velocity;
- incoming focal content enters from the expected direction;
- overlap begins after motion is already underway;
- HUD stays stable across the seam;
- blur/signal veil is short and motivated;
- reversing scroll remains coherent;
- there are no blank frames.

The user should feel that their thumb moved one camera through one world.

## Mobile Discovery Pan
Do not simply center-crop locked landscape images on phones.

Every scene may declare a mobile pan/focal route so scrolling reveals the full wide composition over time.

This is a cinematic behavior, not a responsive-design afterthought.

## Liveness grammar
Use one or two meaningful effects per scene unless canon requires more.

Examples:
- Food Bank → rain reflection + fluorescent/window flicker.
- Post Alley → rain + lamp/steam movement.
- Eldorado → reflection/glint + tracking lock.
- Pauli's Place → rain + lamp instability/signal corruption.

Reject decorative motion that has no attention or story purpose.

---

# 8. 2.5D / LIVING STILL RULES

Use the cheapest technique that wins the shot:

1. Layered still — same image duplicated/masked into depth bands.
2. Cutout multiplane — foreground/midground/background separated.
3. Depth-map displacement — only if Mode 1/2 cannot achieve the result and performance permits.
4. True 3D — rare escalation for spatial interaction that cannot be expressed otherwise.
5. Video — rare payoff or connector, not wallpaper.

Base still + semantic HTML must tell the story even if all advanced enhancement fails.

---

# 9. DRONE HUD LAW

The drone UI is **evidence language**.

It should communicate things such as:
- feed number;
- altitude/speed/heading where narratively useful;
- search status;
- target/vehicle match;
- signal anomaly;
- time sync error;
- source clock mismatch;
- jamming;
- subject not confirmed.

HUD should:
- sit mostly at edges;
- remain lightweight;
- avoid giant cards/panels;
- avoid generic cyberpunk/neon UI;
- never obstruct the shot's focal evidence on mobile.

---

# 10. PAULI PASS / PRODUCT BOUNDARIES

Pauli Pass is:
- identity;
- provenance;
- status;
- access;
- invites;
- rewards.

It is **not** crypto, an investment token, or stored monetary value by default.

Keep Membership Authority separate from Story Authority and Player Authority.

Current target activation contract:

```text
Part 12 / meaningful activation completed
      ↓
Pauli Pass claim
      ↓
Founding Investigator provenance
      ↓
Case entitlement
      ↓
EXACTLY ONE invite entitlement
```

Requirements:
- server-verifiable eligibility;
- idempotent claim;
- no duplicate founding provenance;
- no duplicate invite entitlement;
- transaction safety;
- per-user authorization/RLS;
- auditable state transitions;
- no canon mutation.

Invite redemption is a separate Gauntlet slice.

---

# 11. DATABASE / AUTH CAUTION

Where's Pauli historically references legacy Supabase project `ycyylbabttgguqdcllfc`.

The shared Botanic control plane is a different Supabase project and already contains a substantial `pauli` schema with internal operational entities such as organizations, memberships, agents, missions, experiments, audits, world locations, treasury entries, and evidence systems.

Do **not** reuse internal `pauli.memberships` as consumer Pauli Pass membership.

Do not migrate/repoint the legacy Where's Pauli database until the old database is inventoried or formally declared unavailable.

Different Supabase projects have different auth/JWT trust boundaries. Do not assume a consumer token from one project can safely authorize direct writes to another.

Preferred architecture direction:

```text
CONSUMER AUTH
    ↓
SERVER-SIDE PAULI GATEWAY
    ↓
CONTROL PLANE / CONSUMER TABLES
```

Consumer tables should use explicit semantics, e.g. `player_passes`, `pass_entitlements`, `invites`, `story_sessions`, `player_knowledge`, `evidence_discoveries`, `theories`, `reward_entitlements`, `reward_claims`.

---

# 12. MEDIA / GENERATION LAW

Preferred hierarchy:

```text
AUTHORED / LOCKED MEDIA
        ↓
PRE-RENDERED LIKELY BRANCH MEDIA
        ↓
ON-DEMAND GENERATION
```

Generation is progressive enhancement and must not become the sole way to continue the story.

Never make a player stare at meaningless generation/loading.

If media is being generated, give them something narratively useful to do:
- inspect evidence;
- question a witness;
- compare theories;
- decode a clue;
- study the map;
- review the case board.

Paid generation is bounded. Do not reroll blindly.

---

# 13. MOBILE LAW

Mobile is the primary acceptance surface.

Hard test viewports:
- 390 × 844
- 430 × 932

Pass conditions:
- no horizontal overflow;
- no blank seams;
- correct focal subject stays discoverable;
- reverse scrolling works;
- HUD does not cover evidence;
- reduced-motion path remains coherent;
- current + next scene preload strategy is bounded;
- future heavy scenes lazy-load;
- touch interactions remain comfortable;
- no heavy 3D/video requirement for story comprehension.

Prefer CSS transform/opacity and lightweight Canvas/SVG over large continuous WebGL workloads.

---

# 14. GAUNTLET

Every consequential visual/product slice runs the Gauntlet.

1. Name the exact artifact/task.
2. Name a real fetchable/comparable bar.
3. Builder creates the smallest judgeable version.
4. Fresh critic reviews actual output.
5. Compare ours vs bar.
6. Pick WIN or LOSE.
7. Identify the single biggest gap.
8. Repair that gap.
9. Re-test the same bar.
10. Repeat until ours wins or owner stops.

Builder never self-approves.

For scroll mechanics, `oso95/scroll-world` is a reference for continuous-world behavior, not a template that must be copied literally.

For locked visual scenes, the locked source image itself is also a bar: enhancements must preserve composition, story evidence, and mystery.

---

# 15. CURRENT EXECUTION ORDER

Unless the owner changes priority:

1. Keep Where's Pauli current-state context accurate.
2. Build/lock the Scene 01–10 continuous scroll using locked images.
3. Run mobile Gauntlet on depth, seam invisibility, focal pan, and liveness.
4. Lock Scene 01–10 when they pass.
5. Build Scene 11 and 12 as the anomaly/contact payoff.
6. Assemble all 12 into the production episode flow.
7. Complete Pauli Pass activation backend/auth boundary.
8. Implement one-invite entitlement and then separate invite-redemption Gauntlet.
9. Instrument activation, scene, clue, pass, invite, replay behavior.
10. Run a small real-human beta only after the visuals and activation loop are strong.

Do not drift into merch, crypto, large merchant marketplaces, multiple avatar worlds, or broad infrastructure redesign while these core gates remain open.

---

# 16. DEPLOYMENT / INFRASTRUCTURE GUARDRAIL

GitHub source control is not deployment authorization.

For this workflow:
- do not use Vercel as a debugging loop;
- do not create production deployments autonomously;
- do not change domains/DNS autonomously;
- do not add paid integrations autonomously;
- do not start repeated deploy/retry loops;
- local/static/Coolify preview is preferred for visual iteration;
- production promotion requires explicit owner authority.

Vercel is considered optional presentation infrastructure, not a required architectural dependency.

---

# 17. AGENT HANDOFF FORMAT

At the end of any consequential Where's Pauli work, record:

```text
OBJECTIVE
STARTING STATE
COMPLETED
PROOF
FILES / REFS CHANGED
CANON DECISIONS
PRODUCT DECISIONS
FAILED APPROACHES + LESSON
BLOCKERS
GAUNTLET RESULT
MOBILE RESULT
NEXT ACTION
STOP CONDITION
ROLLBACK
LAST VERIFIED COMMIT
```

Current state outranks old conversation history.

---

# 18. COMPLETION TEST

A Where's Pauli task is not complete merely because code or images exist.

Call it complete only when the relevant combination is proven:
- canon-safe;
- visually coherent;
- mobile-safe;
- state/auth-safe;
- measurable;
- reversible;
- independently judged;
- deployable without hidden platform dependence.
