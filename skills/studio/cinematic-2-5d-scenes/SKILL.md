---
name: cinematic-2-5d-scenes
description: ICM-governed workflow for turning one rights-cleared still image into a mobile-first cinematic scroll scene using semantic HTML, CSS transforms/masks, lightweight Canvas/SVG atmosphere, optional depth maps, progressive enhancement, typography research, reusable design assets, and Gauntlet verification. Use for living photographs, parallax scenes, interactive stills, cinematic scroll worlds, photo-to-2.5D web experiences, and premium motion-graphic treatments.
version: 1.1.0
author: Bambu / Pauli Effect
license: MIT
tags: [cinematic, 2.5d, parallax, scroll, image, html, css, canvas, mobile, typography, fonts, lottie, icons, motion-graphics, design-intelligence, icm, gauntlet]
triggers:
  - make this picture cinematic
  - animate this still image
  - make a living photograph
  - build a 2.5D scene
  - create a cinematic scroll scene
  - turn this photo into a moving web scene
  - cinematic 2.5D
  - research fonts for this design
  - find a better font
  - make this design less generic
  - build a motion graphic style
---

# Cinematic 2.5D Scenes

## Purpose
Turn a still image into an interactive cinematic web scene without requiring video. The scene should feel alive because the virtual camera, atmospheric layers, depth planes, light, HUD, typography, iconography, and story timing move—not because the browser is playing a movie file.

This skill is a production workflow, not an effects recipe. Motion must serve narrative attention.

## Authority and routing
- ICM workflow: `hermes-workflows/cinematic-2-5d-scenes/CONTEXT.md`.
- Stable standard: `hermes-workflows/cinematic-2-5d-scenes/resources/SCENE-STANDARD.md`.
- Video escalation: `skills/studio/cinematic-master-editor/` only when a still treatment cannot achieve the narrative job.
- Quality loop: `skills/gauntlet-loop/`.

Hermes acts as architect/director/verifier. Implementation can be delegated to a web builder, Codex, Claude Code, or another worker. The builder never self-approves.

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
- typography and visual-language requirements;
- approval and promotion gates.

### Context
Load only authoritative material:
- scene/episode canon;
- character/location/prop locks;
- source image and provenance;
- neighboring shots;
- visual bible;
- existing brand typography and design tokens;
- target viewport(s);
- prior scene measurements;
- deployment guardrails.

### Method
Run the numbered workflow in `hermes-workflows/cinematic-2-5d-scenes/CONTEXT.md`. The filesystem is the source of truth, not chat history.

## Core rule
`STILL IMAGE + DEPTH + MOTIVATED MOTION + STORY-TIMED UI + DISTINCTIVE TYPE = CINEMATIC SCENE`

Do not animate everything. Motion should answer: **what should the audience notice now?**

---

# Design Intelligence Layer

Generic typography, default iconography, and unrelated animation packs are major signs of low-quality AI design. When a project does not already have a locked design system, Hermes must research the visual language before polishing the artifact.

## Design-intelligence ladder
1. **Real content/data** — never decorate invented facts or numbers.
2. **Typography** — choose a distinctive, evidence-backed type system before final layout polish.
3. **Icon language** — use one coherent family/style when icons are needed.
4. **Motion assets** — selectively use Lottie/SVG/Canvas assets where they serve the narrative.
5. **Imagery** — canonical/user-owned/real imagery first; generation only when needed.
6. **Timed opportunity detection** — for video/source footage, find exact moments where graphics add comprehension, emphasis, surprise, or retention.
7. **Codify the winner** — once a treatment wins the Gauntlet, turn it into reusable tokens/components/templates instead of recreating it from prompts.

## Typography research protocol
Typography is an early design decision, not cleanup.

When the brand/site already exists:
1. inspect the actual site CSS/brand assets for font-family, `@font-face`, weights, tracking, case, line-height, numeric styling, and fallback stack;
2. if the exact font cannot be identified or legally used, research visually and semantically comparable systems;
3. record the chosen type system and its source/license before implementation.

### Fonts In Use — mandatory first stop when searching fonts
When Hermes is asked to **find, research, identify, compare, or improve fonts**, search `https://fontsinuse.com/` first for real-world precedent.

Use it as a design-research archive, not as a font-download source.

Search by:
- topic/industry;
- output format;
- mood/era/style keywords;
- known typeface names;
- comparable brands or cultural references.

Prefer:
- real-world uses;
- Staff Picks;
- examples matching the actual format (Web, Mobile/Tablet, Film/Video, Branding/Identity, Posters/Flyers, etc.);
- examples matching the actual topic (Sports, Technology, Food/Beverage, Local, Entertainment, etc.).

Capture for each useful precedent:
- typeface name;
- use-case/project title;
- topic/format;
- why the typography fits the current job;
- foundry/designer when available;
- official acquisition/source link when available;
- license/availability status.

If the browser/research tool supports extraction, Hermes may scrape **public typography metadata and visible usage information** from Fonts In Use and referenced public pages, subject to site/tool policy and reasonable rate limits.

Never:
- bypass access controls;
- hammer the site;
- scrape or redistribute font binaries;
- copy commercial font files into a repo without authorization;
- assume a font is free because it appears on Fonts In Use;
- infer licensing from a screenshot.

Follow the official foundry/source link and verify the license separately.

If automated scraping is blocked, use normal web search/open and continue; scraping is not a hard dependency.

After Fonts In Use, use these as appropriate for availability, licensing, and alternatives:
- Fontshare;
- Google Fonts;
- Open Foundry;
- Typewolf for additional editorial precedent;
- the official foundry/typeface site.

### Required font-search output
Return 3–5 candidates, not a giant list:

```yaml
TYPE_CANDIDATES:
  - name: "Example Typeface"
    evidence: "Fonts In Use — real project/category"
    role: "display|body|mono|caption"
    why: "specific visual/semantic fit"
    availability: "free|commercial|unknown"
    license_source: "official source or HOLD"
    fallback: "closest lawful fallback"
```

Then define a **type system**, not just a font:
- display face;
- body face;
- optional mono/data/HUD face;
- weights;
- case rules;
- tracking;
- line-height;
- numeric style;
- fallback stack.

For Where's Pauli, typography must feel authored and cinematic but remain subordinate to the imagery. Drone-feed/HUD type is evidence language, not a giant decorative title system.

## Icon system protocol
If icons materially improve comprehension:
- choose one family/style per artifact or campaign;
- verify license and attribution requirements;
- normalize stroke/fill/weight/optical size;
- prefer semantic SVG icons over raster decoration;
- do not mix unrelated icon packs because individual icons look attractive.

For Where's Pauli surveillance UI, custom simple line glyphs/HUD marks are usually preferable to consumer-app icon packs.

## Lottie / reusable animation protocol
Lottie can provide polished motion without rendering full video, but it is optional.

Use it only when:
- the animation has a clear narrative/interface job;
- licensing is verified;
- its visual style matches the established system;
- payload size is reasonable for mobile;
- a CSS/SVG/Canvas/static fallback exists where the animation is important.

Build an **approved animation library** instead of searching from scratch on every scene. Record:
- asset name;
- source;
- license;
- intended use;
- visual style;
- size/performance notes;
- recoloring/modification permissions.

Do not import multiple unrelated animation styles into one experience.

## Motion-opportunity detection from video/transcripts
When the source is a video, interview, reel, narration, or long-form recording:
1. obtain a transcript with word- or phrase-level timestamps when technically available;
2. identify exact moments where visual explanation would increase comprehension, emphasis, surprise, or retention;
3. rank those moments instead of animating every sentence;
4. enrich missing factual/data claims with research when requested;
5. create an `ANIMATION_OPPORTUNITIES` manifest;
6. route each opportunity to the cheapest medium that solves it: typography, chart, icon treatment, Lottie, living still, 2.5D scene, or full video.

```yaml
ANIMATION_OPPORTUNITIES:
  - start: 00:13:26
    end: 00:13:42
    job: "explain|emphasize|compare|reveal"
    recommended_medium: "chart|type|icon|lottie|living-still|video"
    evidence_needed: true
    priority: high
```

This turns motion design into **moment detection + visual routing**, not decoration.

## Real-data rule
When an animation includes statistics, benchmarks, dates, rankings, prices, or factual comparisons:
- use supplied data or research current primary sources;
- never fabricate values to make a graphic look complete;
- store/cite the source in the build receipt;
- if data is uncertain or missing, design around the known information or mark it HOLD.

## Visual DNA
Every approved project should emerge with a reusable visual layer—effectively a visual CSS for future agents.

Record the winning:
- type system;
- spacing scale;
- colors;
- border/radius rules;
- icon family;
- motion curves/durations;
- grain/noise/vignette treatment;
- chart/data grammar;
- image treatment;
- animation-library references;
- do/don't examples.

For Where's Pauli this becomes `PAULI_VISUAL_DNA` and should be reused across scroll scenes, Case Files, Pauli Pass, motion graphics, trailers, and social assets unless canon explicitly calls for a break.

---

## Source hierarchy
Prefer, in order:
1. approved canonical/user-owned image;
2. rights-cleared real photography from an authoritative source;
3. licensed stock/editorial photography appropriate to the use;
4. generated image when the fictional world cannot be photographed;
5. authored illustration/fallback.

Never invent provenance. If rights are unknown, mark `HOLD`.

## Depth modes
Use the cheapest mode that achieves the shot.

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
- optional Lottie/SVG micro-animation when licensed and stylistically coherent;
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
2. Typography + visual tokens = authored identity.
3. CSS transforms/masks = depth.
4. Canvas/SVG atmosphere = living environment.
5. Approved icon/Lottie assets = selective semantic motion.
6. Pointer/parallax = optional responsiveness.
7. Depth map/WebGL = premium enhancement.
8. Generated/video continuation = rare payoff.

Failure at a higher layer must not erase the lower layer.

## Required artifacts
For every scene create:
- `scene-spec.yaml` — narrative + performance + motion contract;
- `SOURCE.md` — image provenance/rights/credit;
- `TYPE.md` — typography evidence, candidates, license/source, and chosen system when typography is in scope;
- `scene-plan.md` — layer and scroll plan;
- implementation files;
- `GAUNTLET.md` — builder/critic comparison and largest gap;
- `QC.md` — mobile, reduced motion, performance and fallback checks;
- `HANDOFF.md` — promotion/rollback instructions.

For larger campaigns/videos also create when applicable:
- `VISUAL-DNA.md`;
- `ANIMATION-OPPORTUNITIES.yaml`;
- `ASSET-LIBRARY.yaml` with icon/Lottie provenance and licenses.

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
- typography looks intentional rather than like a generic AI default;
- font license/source is known;
- motion directs attention rather than distracting;
- icon/Lottie assets use one coherent visual language;
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
- typography stays minimal and subordinate to the drone feed/world imagery;
- when new fonts are needed, research Fonts In Use first for real-world precedent, then verify the actual license/source separately;
- heavy 3D/video is progressive enhancement;
- Vercel deployment is never a debugging loop and production promotion requires explicit authority.

## Completion receipt
End with:
- DECISION
- SOURCE + RIGHTS
- TYPE SYSTEM + FONT EVIDENCE
- SCENE JOB
- CHANGES
- MOBILE PROOF
- REDUCED-MOTION PROOF
- PERFORMANCE/FALLBACK PROOF
- ASSET/LICENSE PROOF
- GAUNTLET RESULT
- RISKS
- ROLLBACK
- NEXT
- HUMAN APPROVAL REQUIRED
