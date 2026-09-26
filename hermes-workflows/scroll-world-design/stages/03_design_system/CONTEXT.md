# Stage 03 — Design System

## Purpose

Convert design laws into a concrete, machine-executable design system:
style tokens + component specs + animation library selections.

## Input

From Stage 02:
- `runs/current/design_laws.json`
- `runs/current/brief.json` (aesthetic, audience, constraints)
- `runs/current/handoffs/02_to_03.json`

## Process

### Step 1: Derive Style Tokens

From brief aesthetic + laws:

```json
{
  "tokens": {
    "color": {
      "background": "#hex",
      "surface": "#hex",
      "text-primary": "#hex",
      "text-muted": "#hex",
      "accent": "#hex"
    },
    "typography": {
      "heading-font": "font-name | stack",
      "body-font": "font-name | stack",
      "scale": [12, 14, 16, 20, 24, 32, 48, 64, 96]
    },
    "spacing": {
      "base": 8,
      "scale": [4, 8, 16, 24, 32, 48, 64, 96, 128]
    },
    "motion": {
      "easing-default": "cubic-bezier(0.25, 0.46, 0.45, 0.94)",
      "easing-spring": "cubic-bezier(0.34, 1.56, 0.64, 1)",
      "duration-fast": 200,
      "duration-default": 400,
      "duration-slow": 800
    },
    "depth": {
      "planes": 3,
      "parallax-multipliers": [1.0, 0.6, 0.3]
    }
  }
}
```

### Step 2: Select Scroll Archetype

From `resources/archetypes.md`, pick one primary archetype based on brief:
- **POV Walk** — camera moves through scene
- **Reveal** — elements unveil as user scrolls
- **Drift** — slow floating parallax layers
- **Momentum** — snap-based sections with inertia
- **Cinematic** — full-viewport scenes, scroll as time

Record selection in `runs/current/archetype_selection.json`.

### Step 3: Component Spec (OpenSpec format)

For each page section, write a component spec:

```json
{
  "section": "hero",
  "archetype": "POV Walk",
  "animation": {
    "library": "GSAP",
    "trigger": "ScrollTrigger",
    "timeline": "scrub: 1",
    "elements": [
      { "selector": ".hero-bg", "from": {"y": 0}, "to": {"y": "-30%"} },
      { "selector": ".hero-text", "from": {"opacity": 0, "y": 80}, "to": {"opacity": 1, "y": 0} }
    ]
  },
  "beads": {
    "enabled": true,
    "bead_id": "hero-v1"
  }
}
```

Write all component specs to `runs/current/component_specs.json`.

### Step 4: Validate Against Laws

For each component spec, check it doesn't violate any design law.
Any violation → rewrite the spec before proceeding.

## Output

- `runs/current/tokens.json` — Style tokens
- `runs/current/archetype_selection.json` — Chosen archetype
- `runs/current/component_specs.json` — Per-section animation specs
- `runs/current/receipts/stage_03.json`

## Gate

**PASS** if:
- All token groups present (color, typography, spacing, motion, depth)
- Archetype selected and documented
- Component specs written for all sections in brief
- Zero law violations in specs

**BLOCK** if any law violation persists after one revision attempt.

## Beads

Every token set is a bead. Every component spec revision is a bead.
Bead receipts stored in `runs/current/beads/stage_03/`.

## Next Stage

→ `stages/04_generate_site/CONTEXT.md`
