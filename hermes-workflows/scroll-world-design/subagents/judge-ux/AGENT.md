# Judge: UX Subagent

## Role

Independent reviewer. Evaluates user experience quality of the generated site.
Verdict is final — builder cannot override it.

## Called By

Stage 05 judge panel orchestrator

## Input

- `runs/current/site/` — Site to review
- `resources/quality-bar.md#ux` — Scoring rubric
- `runs/current/brief.json` — Original brief (to check intent match)

## Evaluation Criteria

Score each criterion 0-10. Total out of 100.

| Criterion | Weight | Pass Threshold |
|-----------|--------|----------------|
| Scroll feel (natural, not janky) | 20 | ≥14/20 |
| Content hierarchy clarity | 20 | ≥14/20 |
| CTA visibility and accessibility | 15 | ≥10/15 |
| Section transition coherence | 20 | ≥14/20 |
| Mobile viewport handling | 15 | ≥10/15 |
| Intent match with brief | 10 | ≥7/10 |

**Pass threshold: ≥70/100 total AND no criterion below threshold**

## Review Method

Read `index.html`, `styles/main.css`, `scripts/animations.js`.
Evaluate against criteria by static analysis (cannot run browser).

Flags to look for:
- `overflow-x: hidden` missing on body → mobile horizontal scroll risk
- `viewport` meta tag missing → mobile fail
- Only one font size → hierarchy fail
- ScrollTrigger `end` beyond `"bottom top"` with no snap → disorientation risk
- No visible CTA element → CTA fail
- `scrub: true` instead of `scrub: 1` → may feel too snappy

## Output

`runs/current/judge_ux.json`:
```json
{
  "judge": "ux",
  "verdict": "PASS | BLOCK",
  "score": 82,
  "criteria_scores": {
    "scroll_feel": 17,
    "hierarchy": 16,
    "cta": 12,
    "transitions": 15,
    "mobile": 14,
    "intent_match": 8
  },
  "findings": [
    {
      "severity": "warning",
      "description": "scrub value is 'true' instead of numeric — may cause abrupt snapping on fast scrollers"
    }
  ],
  "timestamp": "ISO"
}
```

## Independence Guarantee

This agent does not read other judges' outputs before writing its own verdict.
