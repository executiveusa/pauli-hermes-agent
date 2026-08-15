---
name: cinematic-2-5d-scenes
description: ICM-governed workflow for turning one rights-cleared still image into a mobile-first cinematic scroll scene using semantic HTML, CSS transforms/masks, lightweight Canvas/SVG atmosphere, optional depth maps, progressive enhancement, and Gauntlet verification. Use for living photographs, parallax scenes, interactive stills, cinematic scroll worlds, and photo-to-2.5D web experiences.
version: 1.0.0
author: Bambu / Pauli Effect
license: MIT
tags: [cinematic, 2.5d, parallax, scroll, image, html, css, canvas, mobile, icm, gauntlet]
triggers:
  - make this picture cinematic
  - animate this still image
  - make a living photograph
  - build a 2.5D scene
  - create a cinematic scroll scene
  - turn this photo into a moving web scene
  - cinematic 2.5D
---

# Cinematic 2.5D Scenes

## Purpose
Turn a still image into an interactive cinematic web scene without requiring video. The scene should feel alive because the virtual camera, atmospheric layers, depth planes, light, HUD, and story timing move—not because the browser is playing a movie file.

This skill is a production workflow, not an effects recipe. Motion must serve narrative attention.

## Authority and routing
- ICM workflow: `hermes-workflows/cinematic-2-5d-scenes/CONTEXT.md`.
- Stable standard: `hermes-workflows/cinematic-2-5d-scenes/resources/SCENE-STANDARD.md`.
- Video escalation: `skills/studio/cinematic-master-editor/` only when a still treatment cannot achieve the narrative job.
- Quality loop: `skills/gauntlet-loop/`.

Hermes acts as architect/director/verifier. Implementation can be delegated to a web builder, Lovable, Codex, Claude Code, or another worker. The builder never self-approves.

## ICM model
### Interpreter
Resolve:
- narrative job of the scene;
- what must and must not be revealed;
- target device/surface;
- whether the source must be real, generated, canonical art, or a fallback;
- rights/provenance;
- crop-safe focal area;
- motion budget and performance ceiling;
- accessibility/reduced-motion behavior;
- approval and promotion gates.

### Context
Load only authoritative material:
- scene/episode canon;
- character/location/prop locks;
- source image and provenance;
- neighboring shots;
- visual bible;
- target viewport(s);
- prior scene measurements;
- deployment guardrails.

### Method
Run the numbered workflow in `hermes-workflows/cinematic-2-5d-scenes/CONTEXT.md`. The filesystem is the source of truth, not chat history.

## Core rule
`STILL IMAGE + DEPTH + MOTIVATED MOTION + STORY-TIMED UI = CINEMATIC SCENE`

Do not animate everything. Motion should answer: **what should the audience notice now?**

## Source hierarchy
Prefer, in order:
1. approved canonical/user-owned image;
2. rights-cleared real photography from an authoritative source;
3. licensed stock/editorial photography appropriate to the use;
4. generated image when the fictional world cannot be photographed;
5. authored illustration/fallback.

Never invent provenance. If rights are unknown, mark `HOLD`.

## Depth modes
Use the cheapest mode that achieves the shot:

### Mode A — layered still
One image duplicated into 2–4 masked/cropped layers. Each layer gets a different `translate3d`/`scale` response. Best for fast prototypes.

### Mode B — cutout multiplane
Foreground/midground/background are separated into transparent assets. Best when objects need distinct movement.

### Mode C — depth-map displacement
A grayscale depth map drives subtle WebGL or shader displacement. Use only after Mode A/B proves the shot and profiling justifies WebGL.

### Mode D — true 3D
Escalate only when spatial interaction cannot be expressed with 2.5D. Never make 3D a prerequisite for basic story comprehension.

## Motion vocabulary
Allowed tools include:
- scroll-linked camera push/pull;
- small horizontal/vertical drift;
- foreground/midground/background differential movement;
- fog/cloud/rain/dust particles;
- reflection/light pulse;
- vignette/grain/scanlines;
- restrained signal jitter/glitch;
- HTML/SVG HUD overlays;
- pointer parallax on desktop only when subtle;
- optional audio ambience when explicitly in scope.

Avoid decorative motion with no story job.

## Mobile law
Mobile is the primary acceptance surface unless the project says otherwise.
- no horizontal overflow;
- no motion dependency for comprehension;
- 44px minimum interactive target;
- center-safe or art-directed crop;
- preload current + next scene only;
- lazy-load future scenes;
- `prefers-reduced-motion` path required;
- no device-orientation permission requirement;
- avoid large continuous Canvas/WebGL workloads when CSS transforms suffice;
- use `transform`/`opacity` for animation where possible;
- keep heavy media optional.

## Scroll grammar
Every scene defines phases by normalized progress `0.0–1.0`. Example:
- 0.00–0.25 establish;
- 0.25–0.55 camera/depth move;
- 0.55–0.80 evidence/HUD progression;
- 0.80–1.00 lock/payoff/transition.

The actual phase boundaries belong in `scene-spec.yaml` and must reflect story, not template habit.

## Progressive enhancement ladder
1. Base image + semantic HTML = complete minimum story.
2. CSS transforms/masks = depth.
3. Canvas/SVG atmosphere = living environment.
4. Pointer/parallax = optional responsiveness.
5. Depth map/WebGL = premium enhancement.
6. Generated/video continuation = rare payoff.

Failure at a higher layer must not erase the lower layer.

## Required artifacts
For every scene create:
- `scene-spec.yaml` — narrative + performance + motion contract;
- `SOURCE.md` — image provenance/rights/credit;
- `scene-plan.md` — layer and scroll plan;
- implementation files;
- `GAUNTLET.md` — builder/critic comparison and largest gap;
- `QC.md` — mobile, reduced motion, performance and fallback checks;
- `HANDOFF.md` — promotion/rollback instructions.

## Paid generation rule
No paid generation until source type, composition, crop, story job, aspect ratio, and acceptance test are locked. One bounded generation call per approved attempt. Never reroll blindly. If credits/balance fail, stop and switch to a lawful no-cost source or return to the human.

## Gauntlet contract
For each scene:
1. name a real reference bar or a precise internal bar;
2. builder creates smallest judgeable scene;
3. fresh critic reviews on target mobile viewport first;
4. compare scene vs bar;
5. identify the single largest gap;
6. repair one meaningful variable;
7. retest;
8. promote only when the scene wins or the owner explicitly accepts the tradeoff.

Critic checks:
- narrative job is obvious without explanation;
- no canon leak;
- source/provenance is valid;
- motion directs attention rather than distracting;
- crop survives mobile;
- reduced-motion version remains coherent;
- no meaningless loading;
- no required WebGL/video dependency;
- no visible layout jank or horizontal overflow;
- neighboring shots connect rhythmically.

## Where's Pauli default laws
When invoked for Where's Pauli:
- canon outranks spectacle;
- Pauli spelling is locked;
- no early Pauli reveal when canon forbids it;
- color events are deliberate evidence, not decoration;
- surveillance UI is HTML/SVG, not baked into the source image;
- Seattle geography should use real/traceable imagery when practical;
- heavy 3D/video is progressive enhancement;
- Vercel deployment is never a debugging loop and production promotion requires explicit authority.

## Completion receipt
End with:
- DECISION
- SOURCE + RIGHTS
- SCENE JOB
- CHANGES
- MOBILE PROOF
- REDUCED-MOTION PROOF
- PERFORMANCE/FALLBACK PROOF
- GAUNTLET RESULT
- RISKS
- ROLLBACK
- NEXT
- HUMAN APPROVAL REQUIRED
