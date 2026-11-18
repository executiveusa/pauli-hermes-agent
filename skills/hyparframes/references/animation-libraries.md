# Animation libraries

The ten JavaScript animation libraries from the HyperFrames demo the owner shared
(YouTube Short, 2026-11-17). All run client-side inside a HyperFrames HTML
composition, so anything they animate renders deterministically into the MP4/WebM
output. Load each from a CDN `<script>` tag in the composition unless the
HyperFrames catalog already ships a block that wraps it.

| Library | What it does | Good for |
| --- | --- | --- |
| Splitting | Splits text into per-character / per-word / per-line spans for staggered animation | Kinetic typography, title cards, caption reveals |
| anime.js | General-purpose keyframe/timeline animation engine for CSS, SVG and JS values | Choreographed UI-style motion, stagger grids, easings |
| Vivus | Animates SVG `stroke-dashoffset` so paths draw themselves | Line-art logos, signature/draw-in effects |
| Rough.js | Renders shapes with a hand-drawn, sketchy style | Sketchy diagrams, casual/organic overlays |
| Flubber | Interpolates between arbitrary SVG path shapes | Logo-to-icon morphs, shape transitions between scenes |
| Zdog | Pseudo-3D engine: flat-shaded 3D models on a 2D canvas | Rotating 3D logos, product marks, playful depth |
| Matter.js | 2D rigid-body physics engine | Falling/colliding objects, physics-driven scene moments |
| Mo.js | Motion-graphics library with bursts, swirls and shape tweens | Bursts/confetti, micro-interactions, impact accents |
| pts | Creative-coding library of points and forms on canvas | Generative backgrounds, particle fields, data-driven visuals |
| css-doodle | Web component that generates CSS art from rules | Procedural textures and patterns, no extra JS logic |

## Choosing

- Text that assembles itself: Splitting (+ anime.js for the motion).
- SVG line drawing: Vivus; hand-drawn look: Rough.js; shape morphing: Flubber.
- Depth without WebGL: Zdog. Real physics: Matter.js.
- Bursts and accents: Mo.js. Generative backdrops: pts or css-doodle.

Keep the GSAP timeline (see [gsap.md](gsap.md)) as the master clock; drive these
libraries from timeline callbacks so scrubbing and frame capture stay in sync.
