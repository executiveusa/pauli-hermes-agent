---
name: awwwards-design-intelligence
description: ICM-governed awards-level website design reasoning skill. Leads with the MIT-licensed claude-skill-awwwards creative-direction framework, then reasons over a curated registry of award-winning and Awwwards-inspired open-source implementations to select niche-appropriate design DNA without copying branded assets. Triggers on website design, redesign, wireframe, landing page, frontend, UI/UX, art direction, web motion, design system, niche template, and awards-level generator tasks.
user-invocable: true
---

# Awwwards Design Intelligence

Use this skill for ANY website/front-end design or redesign work. It is a reasoning layer, not a command to clone a reference site.

## Source of truth hierarchy

1. **Lead framework:** https://github.com/tponscr-debug/claude-skill-awwwards — MIT-licensed creative-direction skill. Its concept-first, typography, color, layout, motion, performance, accessibility, mobile, and vertical-specific principles lead this skill.
2. **Reference corpus:** `references/awwwards-registry.yaml` — implementations and design families used as evidence/examples.
3. **ICM design record:** every project produces an explicit context packet before visual implementation.
4. **Actual target repo/product:** existing constraints and user value outrank aesthetic preference.

Never claim a recreation/clone is the original award-winning production source. Preserve provenance and license information. Learn design grammar; do not copy trademarks, logos, proprietary copy, photography, illustrations, or other branded assets.

## ICM design packet

Before writing frontend code, create or update this reasoning packet in the project context:

```yaml
icm_design:
  identity:
    brand: ""
    audience: []
    niche: ""
    desired_emotion: []
    voice: []
  context:
    business_outcome: ""
    primary_user_job: ""
    conversion_event: ""
    existing_stack: ""
    constraints: []
    accessibility_needs: []
    performance_budget: ""
    content_assets: []
  design_intelligence:
    archetype: ""
    selected_references: []
    rejected_references: []
    typography_direction: ""
    color_direction: ""
    layout_grammar: []
    motion_grammar: []
    interaction_grammar: []
    narrative_grammar: []
  execution:
    wireframe: ""
    reusable_patterns: []
    original_elements_required: []
    mobile_adaptation: ""
    reduced_motion: ""
  proof:
    reference_comparison: []
    usability_checks: []
    accessibility_checks: []
    performance_checks: []
    conversion_checks: []
```

If the host project's canonical ICM schema differs, map these fields into that schema rather than creating a competing ICM system.

## Workflow

### 1. Establish the job
Determine MODE (greenfield/brownfield), measurable OUTCOME, TARGET user/customer, CONSTRAINTS, required PROOF, and COMMERCIAL VALUE. For brownfield work, inspect before changing.

### 2. Define art direction before code
Use the lead framework to decide emotion, industry language, archetype, typography, color, layout rhythm, motion intensity, mobile behavior, accessibility, and performance budget. Generic/template-looking output is a failed result.

### 3. Query the registry by fit
Read `references/awwwards-registry.yaml`. Select 2-5 references based on the problem, not fame. Prefer complementary references: one for narrative/layout, one for motion, one for conversion/content, optionally one for 3D/technical treatment.

### 4. Extract design DNA
For each selected reference identify:
- visual grammar
- typography hierarchy
- section rhythm
- narrative sequence
- motion grammar
- interaction grammar
- conversion grammar
- mobile implications
- performance cost
- reusable implementation patterns
- brand-specific elements that MUST NOT be copied

When multiple independent recreations represent the same site/design family, triangulate them. Shared patterns are stronger evidence of the underlying design engine than implementation-specific details.

### 5. Synthesize; never collage
Create a new design direction appropriate to the target niche. Do not combine recognizable branded sections into a Frankenstein clone. State why each reference was selected and what principle—not asset—is being transferred.

### 6. Wireframe first
Produce information architecture and section-level wireframe before high-fidelity implementation. Every section needs a user job and narrative/conversion role. Remove sections that do not serve the outcome.

### 7. Build static hierarchy before motion
Establish semantic structure, responsive layout, type scale, spacing, color tokens, and content hierarchy first. Add motion only after the static experience works.

### 8. Motion serves meaning
Every animation must answer at least one: What changed? What matters? Where should the user look? Remove decorative motion that weakens usability or performance. Support `prefers-reduced-motion`.

### 9. Judge against real references
For substantial visual work, use the existing `gauntlet-loop` skill with fetchable reference pages/screenshots. Compare desktop and mobile at matched viewports. Builder does not approve its own work.

### 10. Record reusable learning
After verification, update the project ICM design packet with the chosen archetype, successful patterns, rejected patterns, proof, and reusable design DNA. Do not silently promote an unverified pattern into the shared registry.

## Selection heuristics

- **CPG / beverage / cosmetics:** start with Spylt family or PinkDrink for product-centric kinetic storytelling.
- **Social enterprise / nonprofit commerce:** start with Two Good Co.; preserve clarity and mission before spectacle.
- **Creative agency / studio:** Furrow, DIGITALWERK, CRUE Creative.
- **Editorial / publishing / culture:** Miranda/Paper, Capsule.
- **Technology / AI / gaming / immersive:** WebGL portfolio, Three.js portfolio, 3D portfolio references.
- **Photography / hospitality / destination:** image-first Immorial-style storytelling.
- **Developer / consultant / personal brand:** practical portfolio references before immersive 3D unless 3D serves positioning.
- **Long-form narrative / documentary / impact report:** Scrollytelling as infrastructure, then choose a visual family.

These are starting priors, not automatic choices. The target user's job and commercial outcome decide.

## Quality gates

Do not ship merely because it looks impressive. Verify:
- clear first-screen value proposition
- usable navigation and CTA hierarchy
- responsive mobile composition, not desktop shrinkage
- keyboard/focus behavior and semantic HTML
- reduced-motion path
- contrast/readability
- no unnecessary animation libraries
- acceptable loading and animation performance
- original brand expression
- provenance/license record for reused code
- rollback path for brownfield changes

## Reference maintenance

The GitHub Awwwards topic is a discovery feed, not a quality guarantee: https://github.com/topics/awwwards

When adding a source, record: `id`, `repo`, `source_type`, `category`, `stack`, `best_for`, `design_dna`, `agent_note`, `provenance`, and `license_status`. Deduplicate design families. Never mark license as permissive unless verified from the repository.

## Related skills

- `gauntlet-loop` — independent visual quality loop.
- `vibe-client-factory` — outcome-first client delivery and release governance.
- `campaign-factory` — campaign/landing-page delivery when applicable.

## Output contract

For substantial design work return: selected references + reasons, ICM design packet, wireframe/design direction, implementation plan, proof plan, and any provenance/license constraints. Build only after the design decision is explicit.