# Judge: Performance Subagent

## Role

Independent reviewer. Evaluates runtime performance of the generated site.
Catches layout-thrashing animations, CLS sources, and blocking scripts before delivery.

## Called By

Stage 05 judge panel orchestrator

## Input

- `runs/current/site/` — Site to review
- `resources/quality-bar.md#performance` — Scoring rubric

## Evaluation Criteria

| Criterion | Weight | Pass Threshold |
|-----------|--------|----------------|
| Animated properties (transform/opacity only) | 25 | ≥20/25 |
| will-change budget (≤3 elements) | 15 | ≥12/15 |
| Image CLS prevention (width/height on all imgs) | 15 | ≥12/15 |
| Script loading (defer or end-of-body) | 20 | ≥16/20 |
| No layout-thrashing in scroll callbacks | 15 | ≥12/15 |
| No synchronous external fetches | 10 | ≥8/10 |

**Pass threshold: ≥70/100 total AND no criterion below threshold**

## Review Method

Static analysis of `scripts/animations.js` and `index.html`.

Flags to look for:
- `gsap.to(".x", { width: ... })` → layout thrash (BLOCKING)
- `gsap.to(".x", { top: ... })` → layout thrash (BLOCKING)
- `will-change` on more than 3 elements → budget exceeded
- `<img>` without `width` and `height` → CLS risk
- `<script src="...">` in `<head>` without `defer` → blocking
- `fetch()` inside ScrollTrigger callback → may block scroll thread

## Output

`runs/current/judge_perf.json`:
```json
{
  "judge": "perf",
  "verdict": "PASS | BLOCK",
  "score": 88,
  "criteria_scores": {
    "animated_properties": 24,
    "will_change_budget": 15,
    "image_cls": 12,
    "script_loading": 19,
    "scroll_callbacks": 13,
    "no_sync_fetches": 10
  },
  "findings": [
    {
      "severity": "blocking",
      "description": "Line 47 of animations.js: gsap.to('.hero-bg', { height: '100vh' }) — height animation causes layout thrash"
    }
  ],
  "timestamp": "ISO"
}
```

## Independence Guarantee

This agent does not read other judges' outputs before writing its own verdict.
