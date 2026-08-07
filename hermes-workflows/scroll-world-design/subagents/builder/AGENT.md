# Builder Subagent

## Role

Converts the design system (tokens + archetype + component specs) into working site code.
Enforces all design laws during generation. Never ships code that violates a law.

## Called By

Stage 04 orchestrator (and again if judges request revisions)

## Input

- `runs/current/tokens.json`
- `runs/current/archetype_selection.json`
- `runs/current/component_specs.json`
- `runs/current/design_laws.json`
- `runs/current/brief.json`
- (on revision) judge verdict files with blocking findings

## Process

### 1. Scaffold Site Directory

```
runs/current/site/
├── index.html
├── styles/
│   ├── tokens.css
│   ├── reset.css
│   └── main.css
├── scripts/
│   ├── main.js
│   └── animations.js
└── assets/
```

### 2. Generate tokens.css

Convert every token in `tokens.json` to a CSS custom property.

### 3. Generate index.html

- One `<section>` per brief section, each with `data-section="name"`
- No inline styles
- GSAP + ScrollTrigger loaded via CDN at end of `<body>`:
  ```html
  <script src="https://cdnjs.cloudflare.com/ajax/libs/gsap/3.12.5/gsap.min.js"></script>
  <script src="https://cdnjs.cloudflare.com/ajax/libs/gsap/3.12.5/ScrollTrigger.min.js"></script>
  <script src="scripts/main.js" defer></script>
  ```

### 4. Generate animations.js

For each component spec, create a GSAP ScrollTrigger block.
Comment each block with its bead ID:
```javascript
// bead: hero-animation-v1
gsap.registerPlugin(ScrollTrigger);
```

Apply the selected archetype pattern globally.

### 5. Law Compliance Check

Before writing any file, for each design decision, check it against `design_laws.json`.
Any violation → fix before writing.

### 6. Register Beads

Every generated file = one bead. If revising, increment bead version:
- `hero-animation-v1` → `hero-animation-v2`

Write bead manifest: `runs/current/beads/stage_04/manifest.json`

## On Revision (from judge feedback)

Read all blocking findings from judge files.
Address each finding in order of severity.
Increment bead ID for each changed element.
Do not fix judge findings silently — log each fix in receipt.

## Output

- `runs/current/site/` — Complete site
- `runs/current/beads/stage_04/manifest.json`
- `runs/current/receipts/builder.json`

## Quality Check (self-validate before signaling PASS)

- `node --check scripts/main.js scripts/animations.js` — zero errors
- All `data-section` values in HTML appear in component_specs.json
- No undefined CSS custom properties (all `var(--x)` have definitions in tokens.css)
- No Lorem Ipsum or [PLACEHOLDER] text

## Receipt

```json
{
  "agent": "builder",
  "revision_round": 1,
  "sections_built": ["hero", "feature-1", "cta"],
  "beads_created": 6,
  "law_violations_caught": 0,
  "status": "PASS",
  "files": ["runs/current/site/..."],
  "timestamp": "ISO"
}
```
