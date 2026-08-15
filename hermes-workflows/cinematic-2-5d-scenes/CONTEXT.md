# Cinematic 2.5D Scenes — ICM Workflow

Outcome: convert one approved still into a mobile-first cinematic web scene that communicates its narrative job without requiring video.

Hermes orchestrates. A builder implements. A fresh critic verifies. Production/deployment remains separately governed.

## Stage 00 — DISCOVER
### Input
Owner request, target scene, source repo/project.
### Process
Read the current project router/context, canon, neighboring scenes, deployment guardrails, and available assets. Determine the scene's single narrative job and prohibited reveals.
### Output
`runs/<run-id>/00_discover/SCENE-BRIEF.md`
### Gate
One sentence answers: **What must the audience understand/feel by the end of this scene?**
### Receipt
Source files read + current commit/ref.

## Stage 01 — SOURCE
### Input
Scene brief.
### Process
Inventory approved/user-owned images first. If none, find rights-clear real photography from authoritative sources. Use generated imagery only when the fictional requirement cannot be captured truthfully. Record source URL, creator/agency, license/usage note, credit, acquisition date, and whether local archival is required.
### Output
`SOURCE.md`, `source-manifest.json`
### Gate
`APPROVED | HOLD`. Unknown rights = HOLD.
### Receipt
Traceable URL/license/credit or owner provenance.

## Stage 02 — ARCHITECT
### Input
Approved source + scene brief.
### Process
Choose depth mode A/B/C/D. Define mobile crop, desktop crop, semantic HTML story content, progressive-enhancement ladder, reduced-motion behavior, performance budget, scroll phases, and fallback behavior.
### Output
`scene-spec.yaml`, `scene-plan.md`
### Gate
The base image + HTML alone must still communicate the story.
### Receipt
Explicit layer plan and acceptance tests.

## Stage 03 — MAP
### Input
Scene spec.
### Process
Map visual layers and their jobs. Typical map:
- base photographic plane;
- background/sky/horizon layer;
- midground geography/architecture;
- foreground object/atmosphere;
- Canvas/SVG atmosphere;
- HTML/SVG HUD;
- transition cue.
For each layer specify movement range, opacity/filter, phase, z-order, fallback, and reason.
### Output
`layer-map.json`, `scroll-map.json`
### Gate
Every moving layer has a narrative or depth reason.
### Receipt
No orphan animation.

## Stage 04 — BUILD
### Input
Approved maps.
### Process
Builder implements the smallest judgeable version. Prefer semantic HTML + CSS transforms/masks. Add Canvas/SVG only where it creates meaningful atmosphere. No WebGL unless explicitly approved. Keep source URLs/credits out of code comments only; surface required attribution appropriately.
### Output
Working local/isolated preview + implementation diff.
### Gate
No production deployment. Local/isolated preview only.
### Receipt
Build command/test result + changed files.

## Stage 05 — JUDGE / GAUNTLET
### Input
Builder preview + scene spec.
### Process
Fresh critic tests the target mobile viewport first, then desktop, reduced motion, slow-network/fallback behavior, and story comprehension. Compare against a named reference or exact internal bar. Pick WIN/LOSE, not a vague score. Return the single largest gap.
### Output
`GAUNTLET.md`
### Gate
Builder cannot approve itself.
### Receipt
Viewport(s), observations, comparison, biggest gap.

## Stage 06 — REPAIR
### Input
Single biggest gap.
### Process
Change one meaningful variable. Do not add unrelated polish. Re-run the same test.
### Output
Repair diff + updated `GAUNTLET.md`
### Gate
Same metric/bar improves without breaking canon/mobile/fallback.
### Receipt
Before/after evidence.

## Stage 07 — QC
### Input
Gauntlet winner.
### Process
Verify:
- no horizontal overflow at 390×844;
- correct crop at 390×844, 430×932, tablet, desktop;
- 44px touch targets;
- `prefers-reduced-motion` coherent;
- source image failure leaves semantic fallback;
- no required video/WebGL;
- transform/opacity animations preferred;
- current + next scene preloading plan only;
- no unbounded `requestAnimationFrame` work when offscreen;
- Canvas pauses when scene not visible;
- text contrast/safe areas;
- no accidental canon reveal;
- attribution present where required.
### Output
`QC.md`
### Gate
No critical mobile, canon, rights, or accessibility failure.
### Receipt
Pass/fail table with evidence.

## Stage 08 — PACKAGE
### Input
QC-approved scene.
### Process
Package source manifest, scene spec, implementation, prompts if any, debug notes, screenshots/preview links, rollback instructions, and the next-scene handoff.
### Output
`HANDOFF.md`, `delivery/`
### Gate
A fresh agent can reproduce the scene without conversation history.
### Receipt
Commit/ref + artifact paths.

## Stage 09 — PROMOTE
Promotion is outside this workflow's automatic authority. Production merge/deploy/domain changes require the project's infrastructure policy and explicit owner approval.

## Scene-spec minimum fields
```yaml
scene_id: UNKNOWN
project: UNKNOWN
narrative_job: UNKNOWN
must_reveal: []
must_not_reveal: []
source:
  type: canonical|owner|rights-cleared-real|licensed|generated|fallback
  url: UNKNOWN
  rights: UNKNOWN
  credit: UNKNOWN
mobile:
  primary_viewport: 390x844
  crop_anchor: center
  reduced_motion: required
performance:
  video_required: false
  webgl_required: false
  preload: current_and_next_only
motion:
  depth_mode: A
  phases: []
acceptance: []
human_gate: production_promotion
```

## Stop conditions
Stop and return to owner when:
- rights are unknown;
- canon conflicts with requested visual;
- the only solution requires paid generation not approved;
- the scene cannot survive without video/WebGL but those are prohibited;
- production deployment would be triggered by a normal code write;
- required source asset is unavailable;
- the builder cannot reproduce the state from filesystem artifacts.
