# Design Laws — Scroll World Design

> **This file is written by the synthesizer subagent during Stage 02.**
> It is a permanent reference — once written, laws do not change without a PR.
> Placeholder laws below are pre-seeded from repo analysis. The synthesizer
> will extend and refine these with graph-derived patterns from the actual scrape.

---

## The Immutable Laws

### Law 01 — Animate Only Transform and Opacity
Never animate `width`, `height`, `top`, `left`, `margin`, or `padding` in a scroll
context. Only `transform` and `opacity` trigger GPU compositing and maintain 60fps.

*Sources: GSAP docs, performance rubric*

---

### Law 02 — Depth Planes Are Non-Negotiable
Every scroll section using a cinematic or POV archetype must have foreground (1.0×),
midground (0.6×), and background (0.3×) planes with distinct parallax multipliers.
Without depth planes, it's just a flat page that scrolls.

*Sources: robonuggets/cinematic-site-components, archetypes.md*

---

### Law 03 — Scrub Must Be Numeric
`ScrollTrigger` `scrub` must be a number (e.g., `1`, `1.5`), never `true`.
A numeric scrub smooths the animation over N seconds; `true` is instant and causes
abrupt motion on fast scrollers.

*Sources: GSAP/ScrollTrigger docs*

---

### Law 04 — Every Design Change Is a Bead
No design change ships without creating a bead first. Beads are atomic, immutable,
and one-click reversible. This is the only way to maintain creative confidence.

*Sources: gastownhall/beads*

---

### Law 05 — Scripts Load at End of Body or with defer
No render-blocking scripts. GSAP and ScrollTrigger load after HTML is parsed.
Place `<script>` tags before `</body>` or add `defer` attribute.

*Sources: performance rubric, web fundamentals*

---

### Law 06 — Tokens Are the Contract
All color, typography, spacing, and motion values live in CSS custom properties
(`--token-name`) defined in `tokens.css`. No magic numbers in component CSS.
If it's not a token, it doesn't exist.

*Sources: atomicdotdev/atomic*

---

### Law 07 — Constraints Create Quality
Fewer choices → more coherent output. Limit: 2 fonts, 5 font sizes, 6 spacing values,
3 animation durations. Expand the system only when the brief explicitly requires it.

*Sources: pbakaus/impeccable*

---

### Law 08 — Above the Fold Earns Attention in 3 Seconds
The hero section communicates: what it is, who it's for, and why they should scroll —
within 3 seconds of the page loading. If it doesn't, the scroll story is wasted.

*Sources: ytx-readings/design-ui-ux, UX rubric*

---

### Law 09 — No Placeholders Ship
No Lorem Ipsum. No `[IMAGE]`. No `TODO`. No `FIXME`. If real content isn't available,
use realistic placeholder text that reads like real content. Slop is slop regardless
of the reason.

*Sources: design rubric, slop test*

---

### Law 10 — Handoffs Are Validated Contracts
Every inter-agent handoff passes through the handoff agent and receives a signature.
No agent proceeds on unsigned data. Trust but verify — verify via schema.

*Sources: Fission-AI/OpenSpec, willseltzer/claude-handoff*

---

### Law 11 — will-change Budget Is 3
`will-change: transform` on more than 3 elements simultaneously causes memory pressure.
Budget strictly. Apply it only to elements that scroll with the viewport (fixed/sticky).

*Sources: GSAP best practices, performance rubric*

---

### Law 12 — The Slop Test Is Final
After all judges PASS, a senior designer review is run as a final check.
If the output would embarrass a senior designer, it does not ship — regardless of
automated scores. Scores measure compliance; the slop test measures craft.

*Sources: design rubric, agent voice guidelines*

---

*Synthesizer: extend this file with laws derived from the knowledge graph.
Add laws 13+ below after Stage 02 completes.*
