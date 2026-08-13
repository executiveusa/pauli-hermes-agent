# UDEC 14-Axis Design Scoring Framework
## SYNTHIA™ v3.1 | Floor: 8.5/10 | Hard blocks: MOT < 7.0, ACC < 7.0

Score every visual deliverable across these 14 axes before shipping.

| Axis | Code | Weight | What it measures |
|------|------|--------|-----------------|
| Typography | TYP | 10% | Hierarchy, scale, weight, letter-spacing, line-height |
| Color | CLR | 8% | Palette coherence, contrast ratios, accent discipline |
| Spacing | SPC | 8% | Grid consistency, white space ratio, rhythm |
| Hierarchy | HIE | 10% | Visual weight distribution, scan path, primary action clarity |
| Motion | MOT | 8% | Timing, easing, purposefulness, reduced-motion support |
| Accessibility | ACC | 10% | WCAG 2.1 AA contrast, keyboard path, focus states, labels |
| Imagery | IMG | 6% | Quality, consistency, relevance, generation quality |
| Copy | CPY | 8% | P.A.S.S. compliance, specificity, banned word count = 0 |
| Responsiveness | RSP | 8% | Mobile-first, breakpoint coverage, touch targets |
| Component Quality | CMP | 8% | State coverage, error states, empty states |
| Interaction | INT | 6% | Affordance clarity, feedback timing, hover/active states |
| Performance Signal | PRF | 6% | Load indicators, skeleton states, no layout shift |
| Brand Alignment | BRD | 6% | Token adherence, voice consistency, design system fidelity |
| Market Position | MKT | 8% | Competitive calibration against niche top-5 bar |

## Scoring Protocol

For each axis:
- 9–10: Exceeds the niche bar. Others would copy this.
- 8–8.9: Meets the quality floor. Ships.
- 7–7.9: Acceptable. Flag for next iteration.
- Below 7: Fails. Do not ship this axis.
- MOT below 7: Full rebuild of motion layer.
- ACC below 7: Full rebuild of accessibility layer.
- Any axis below 6: Reject immediately. Do not attempt patch.

## Output Format

```
TYP: [X/10] — [one line rationale]
CLR: [X/10] — [one line rationale]
SPC: [X/10] — [one line rationale]
HIE: [X/10] — [one line rationale]
MOT: [X/10] — [one line rationale]
ACC: [X/10] — [one line rationale]
IMG: [X/10] — [one line rationale]
CPY: [X/10] — [one line rationale]
RSP: [X/10] — [one line rationale]
CMP: [X/10] — [one line rationale]
INT: [X/10] — [one line rationale]
PRF: [X/10] — [one line rationale]
BRD: [X/10] — [one line rationale]
MKT: [X/10] — [one line rationale]

OVERALL: [WEIGHTED AVERAGE]/10
VERDICT: SHIP | ITERATE | REBUILD
```
