# Stage 04 — Generate Site

## Purpose

Convert the design system into working site code.
Output: a self-contained directory of HTML/CSS/JS that runs without a build step.

## Input

From Stage 03:
- `runs/current/tokens.json`
- `runs/current/archetype_selection.json`
- `runs/current/component_specs.json`
- `runs/current/brief.json`

## Process

### Step 1: Scaffold

Create output directory: `runs/current/site/`

```
runs/current/site/
├── index.html
├── styles/
│   ├── tokens.css        ← CSS custom properties from tokens.json
│   ├── reset.css
│   └── main.css
├── scripts/
│   ├── main.js           ← GSAP ScrollTrigger setup
│   └── animations.js     ← per-section animation timelines
└── assets/
    └── (placeholder images / SVGs)
```

### Step 2: Generate tokens.css

Convert `tokens.json` → CSS custom properties:
```css
:root {
  --color-bg: #hex;
  --color-accent: #hex;
  --font-heading: 'Font Name', sans-serif;
  --ease-default: cubic-bezier(0.25, 0.46, 0.45, 0.94);
  /* etc */
}
```

### Step 3: Generate index.html

- Semantic HTML sections matching brief structure
- Each section gets `data-section="name"` attribute for GSAP targeting
- No inline styles — all via CSS custom properties
- GSAP loaded from CDN (pinned version): `https://cdnjs.cloudflare.com/ajax/libs/gsap/3.12.5/gsap.min.js`
- ScrollTrigger plugin loaded same way

### Step 4: Generate animations.js

For each component spec in `component_specs.json`:
- Create a GSAP ScrollTrigger timeline
- Apply `scrub` for scroll-linked motion
- Apply spring easing for user-initiated interactions
- Register each animation as a bead:

```javascript
// bead: hero-animation-v1
gsap.timeline({
  scrollTrigger: {
    trigger: "[data-section='hero']",
    start: "top top",
    end: "bottom top",
    scrub: 1,
  }
})
```

### Step 5: Beads Registration

Every generated file version = one bead.
Write `runs/current/beads/stage_04/manifest.json` listing all beads.

### Step 6: Static Validation

- HTML validates (no unclosed tags)
- CSS has no undefined custom properties
- JS has no syntax errors (`node --check`)
- All GSAP targets (`data-section`) exist in HTML

## Output

- `runs/current/site/` — Complete site
- `runs/current/beads/stage_04/manifest.json` — Bead manifest
- `runs/current/receipts/stage_04.json`

## Gate

**PASS** if:
- All sections from brief are implemented
- Zero syntax errors in HTML, CSS, JS
- All GSAP targets resolve to real DOM elements
- Bead manifest written

**BLOCK** if any syntax error or missing section.

## Next Stage

→ `stages/05_judge_panel/CONTEXT.md`
