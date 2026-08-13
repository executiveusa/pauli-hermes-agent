# Gauntlet Loop — Design Critic Protocol
## Source: Matt Shumer's Gauntlet Loop, adapted for SYNTHIA™

## The Three Critics

Run these three critics in parallel, each with fresh context and no knowledge
of how the builder worked. All three must pass. Any fail → back to builder.

### Critic 1 — Brief Critic (Sonnet)
Judges against the stated goal only. Does the output do the thing?
Ignore aesthetics entirely. Binary: YES it achieves the brief / NO it does not.
Name the single biggest gap if NO.

### Critic 2 — System Critic (Haiku)
Judges against the design system / brand tokens only.
Does every value (color, font, spacing) match the token spec?
Binary: COMPLIANT / NON-COMPLIANT. List non-compliant values.

### Critic 3 — Craft Critic (Strongest available model)
Judges against `bar.md` mechanisms only. Never reads the code.
Places our rendered output next to the niche top-5 bar with labels stripped.
Which one is better? Which mechanism shows the biggest gap?
Binary: OURS WINS / BAR WINS. Name the single biggest remaining gap.

## Rules
- Critics are harsh. Praise is not useful.
- Critics judge rendered output, never the code.
- Binary verdicts only. Scores drift upward. Verdicts do not.
- All three must pass. Any fail loops.
- No fixed round count. Exit is winning or user stopping.
- The bar must be named, fetchable, and comparable. Vague bars kill loops.
