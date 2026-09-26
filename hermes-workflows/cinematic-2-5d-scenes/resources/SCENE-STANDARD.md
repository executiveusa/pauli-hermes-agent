# Cinematic 2.5D Scene Standard

## 1. Narrative before effect
Every scene has one primary job: establish, narrow, reveal evidence, create doubt, escalate, deny a reveal, or transition. If an animation cannot be connected to that job, remove it.

## 2. Image construction
A still may be made dimensional through:
- duplicated image planes with CSS masks/clip paths;
- transparent cutouts;
- foreground atmospheric layers;
- a depth map and subtle displacement when justified;
- separate semantic HUD/UI above the image.

Never bake critical story text into a generated image when HTML can carry it.

## 3. Motion ranges
Default to restrained ranges:
- global camera scale: ~1.00–1.08 per scene;
- layer translation: usually single-digit to low-double-digit pixels on mobile;
- pointer parallax: <=5px unless a specific scene proves otherwise;
- signal jitter: rare, brief, and <=2–3px for restrained surveillance language;
- opacity/filter transitions: smooth and phase-driven.
These are starting ranges, not mandates. Story and profiling decide.

## 4. Performance
- CSS `transform` and `opacity` first.
- Use `will-change` narrowly, not globally.
- Use `IntersectionObserver` to pause offscreen loops.
- Prefer one lightweight Canvas per active scene at most; pause it offscreen.
- Avoid giant PNGs when AVIF/WebP preserves quality.
- Generate responsive widths and art-directed mobile crops for final production.
- Lazy-load scenes beyond the next scene.
- Do not preload all 12 hero scenes on mobile.
- No video autoplay dependency for the base experience.

## 5. Accessibility
- `prefers-reduced-motion` removes continuous drift, parallax and jitter.
- Important information remains readable as semantic text.
- Do not use color alone for a required clue unless the story intentionally makes color an evidence event and a non-color accessible clue exists.
- Provide alt text that describes the source image without leaking mystery/canon the player has not earned.

## 6. Mobile crop law
A source image is not approved until a 390×844 crop has been judged. Record:
- focal anchor;
- safe horizontal band;
- safe vertical band;
- text-safe zones;
- whether a dedicated mobile crop is required.
Desktop beauty cannot rescue a broken mobile crop.

## 7. Provenance
`SOURCE.md` must include:
- original source URL/file;
- owner/creator/agency;
- rights/license note or owner declaration;
- credit requirement;
- date accessed/acquired;
- transformations performed;
- checksum/local archive path when available.
Generated sources additionally record provider/model, prompt, seed/reference when available, date, and cost/attempt count.

## 8. Cost discipline
- Source discovery and composition happen before generation.
- One approved generation attempt at a time.
- Never create repeated paid calls to chase taste without diagnosing the failure.
- A billing/credit failure is a stop condition, not permission to switch to another paid provider silently.

## 9. Gauntlet comparison
Judge a scene at the same viewport and interaction mode as its reference. The critic returns:
- `WIN` or `LOSE`;
- the single largest reason;
- one smallest repair;
- regression risk.
No self-assigned 8/10 score replaces comparison evidence.

## 10. Where's Pauli visual grammar
Default treatment:
- restrained noir/editorial surveillance language;
- mostly neutral/monochrome unless canon calls for a color event;
- rain, mist, glass reflections, scan behavior and telemetry may recur;
- geography and real-world Seattle locations should remain believable;
- glitches increase only when the surveillance system is under pressure;
- Pauli is not visually revealed before canon permits it;
- the player should feel they are moving through evidence, not scrolling through marketing sections.

## 11. Promotion gate
A scene can be merged into a production scroll world only after:
- source/rights PASS;
- canon PASS;
- mobile PASS;
- reduced motion PASS;
- fallback PASS;
- Gauntlet WIN/owner acceptance;
- deployment policy allows the merge without accidental production side effects.
