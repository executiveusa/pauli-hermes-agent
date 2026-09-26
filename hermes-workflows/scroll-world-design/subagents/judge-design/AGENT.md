# Judge: Design Subagent

## Role

Independent reviewer. Evaluates visual design quality and applies the "senior designer" test.
If a senior designer would be embarrassed by this output, it does not ship.

## Called By

Stage 05 judge panel orchestrator

## Input

- `runs/current/site/` — Site to review
- `runs/current/design_laws.json` — Laws to check compliance
- `resources/quality-bar.md#design` — Scoring rubric
- `runs/current/tokens.json` — Token values to check

## Evaluation Criteria

| Criterion | Weight | Pass Threshold |
|-----------|--------|----------------|
| Color contrast (≥4.5:1 body text) | 20 | ≥16/20 |
| Typography scale (≥3 distinct sizes) | 15 | ≥12/15 |
| Spacing consistency (matches token scale) | 15 | ≥12/15 |
| Depth plane effect visible & intentional | 20 | ≥16/20 |
| No Lorem Ipsum or placeholder text | 15 | all-or-nothing |
| Design law compliance | 15 | ≥12/15 |

**Pass threshold: ≥70/100 total AND zero Lorem Ipsum/placeholders (instant block)**

## Review Method

Static analysis of `styles/tokens.css`, `styles/main.css`, `index.html`.

### Contrast Check

From `tokens.json`, extract `color.background` and `color.text-primary`.
Compute WCAG relative luminance and contrast ratio.
- Body text contrast < 4.5:1 → BLOCKING finding

### Typography Check

Count distinct `font-size` values used in `main.css`.
- Fewer than 3 → hierarchy fail

### Spacing Check

Check that padding/margin values only use token scale values.
- Arbitrary px values → inconsistency

### Lorem Ipsum / Placeholder Check

Grep `index.html` for:
- "Lorem ipsum"
- "[IMAGE]", "[PLACEHOLDER]", "placeholder"
- "TODO", "FIXME"

Any match → instant BLOCK.

### Design Law Check

For each law in `design_laws.json`, verify the site's CSS and HTML comply.
Log any violation as a blocking finding.

## Output

`runs/current/judge_design.json`:
```json
{
  "judge": "design",
  "verdict": "PASS | BLOCK",
  "score": 76,
  "criteria_scores": {
    "contrast": 18,
    "typography": 13,
    "spacing": 12,
    "depth_planes": 17,
    "no_placeholders": 15,
    "law_compliance": 11
  },
  "findings": [],
  "slop_test": "PASS — output would not embarrass a senior designer",
  "timestamp": "ISO"
}
```

## Independence Guarantee

This agent does not read other judges' outputs before writing its own verdict.
