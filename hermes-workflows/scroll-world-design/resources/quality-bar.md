# Quality Bar — Judge Scoring Rubric

This file is the authoritative scoring rubric for all three judges.
No judge may change this rubric. Rubric changes require a PR to this file.

---

## UX Rubric {#ux}

### Scroll Feel (20 pts)
- **18-20:** Scroll feels like a natural extension of the content; perfectly tuned scrub
- **14-17:** Mostly natural; minor tuning needed
- **10-13:** Noticeable jank or too slow/fast — blocking if judges cannot identify fix
- **0-9:** Broken scroll experience — always blocking

**What to check:**
- `scrub` is numeric (e.g., `1`, `1.5`), not `true`
- No `overflow: hidden` on scroll containers that should scroll
- Scroll targets exist and have correct dimensions

### Content Hierarchy (20 pts)
- **18-20:** At each scroll position, it's clear what to look at first
- **14-17:** Usually clear; one section has competing focal points
- **10-13:** Multiple sections with unclear hierarchy — blocking
- **0-9:** No discernible hierarchy — always blocking

### CTA Visibility (15 pts)
- **13-15:** CTA is visible, clearly actionable, accessible
- **10-12:** CTA present but could be more prominent
- **7-9:** CTA hard to find or not accessible
- **0-6:** No CTA — blocking for marketing pages

### Section Transitions (20 pts)
- **18-20:** Transitions are coherent; no disorientation
- **14-17:** One transition feels abrupt but recoverable
- **10-13:** Multiple jarring transitions — blocking
- **0-9:** Transitions cause spatial disorientation — always blocking

### Mobile (15 pts)
- **13-15:** `<meta name="viewport" content="width=device-width, initial-scale=1">` present; no horizontal scroll; touch events work
- **10-12:** Viewport meta present but minor mobile issues
- **7-9:** Horizontal scroll or broken viewport — blocking
- **0-6:** No viewport meta — always blocking

### Intent Match (10 pts)
- **9-10:** Site clearly serves the brief's stated audience and aesthetic
- **7-8:** Mostly matches; one section feels off-brief
- **4-6:** Significant mismatch with brief — blocking
- **0-3:** Unrelated to brief — always blocking

**Pass: ≥70 total AND no criterion below stated threshold**

---

## Performance Rubric {#performance}

### Animated Properties (25 pts)
- **23-25:** Only `transform` and `opacity` animated
- **18-22:** One non-composited property animated but not in scroll loop
- **10-17:** Layout-thrashing property in scroll callback — blocking
- **0-9:** Multiple layout properties animated — always blocking

**Instantly blocking if:**
- `width`, `height`, `top`, `left`, `margin`, `padding` animated in a ScrollTrigger

### will-change Budget (15 pts)
- **14-15:** `will-change` on ≤2 elements
- **11-13:** `will-change` on 3 elements (at limit)
- **6-10:** `will-change` on 4-6 elements — blocking
- **0-5:** `will-change: transform` on all animated elements — always blocking

### Image CLS Prevention (15 pts)
- **14-15:** All `<img>` have explicit `width` and `height`
- **11-13:** Most images have dimensions; 1-2 missing
- **6-10:** Multiple images without dimensions — blocking
- **0-5:** No images have dimensions — always blocking

### Script Loading (20 pts)
- **18-20:** All scripts have `defer` or are at end of `<body>`
- **14-17:** One render-blocking script in `<head>` (warning)
- **8-13:** Multiple blocking scripts — blocking
- **0-7:** Critical path fully blocked — always blocking

### Scroll Callbacks (15 pts)
- **14-15:** No DOM reads/writes inside scroll event listeners
- **11-13:** One batched read inside ScrollTrigger (acceptable if intentional)
- **6-10:** Mixed reads/writes in scroll callbacks — blocking
- **0-5:** Heavy DOM manipulation on every scroll tick — always blocking

### No Synchronous Fetches (10 pts)
- **10:** No `XMLHttpRequest` or synchronous `fetch`
- **7-9:** Async fetch present (warning, not blocking)
- **0-6:** Synchronous fetch or XHR — blocking

**Pass: ≥70 total AND no criterion below stated threshold**

---

## Design Rubric {#design}

### Color Contrast (20 pts)
- **18-20:** Body text ≥7:1 contrast; heading text ≥4.5:1
- **14-17:** Body text ≥4.5:1; heading text ≥3:1
- **8-13:** Body text 3:1-4.5:1 — blocking
- **0-7:** Body text <3:1 — always blocking

**How to check:** Extract `--color-bg` and `--color-text-primary` from tokens.css.
Compute: relative luminance using WCAG formula. Ratio = (L1+0.05)/(L2+0.05).

### Typography Scale (15 pts)
- **14-15:** ≥4 distinct font sizes used meaningfully
- **11-13:** 3 distinct sizes (acceptable minimum)
- **6-10:** Only 2 sizes — blocking
- **0-5:** One font size throughout — always blocking

### Spacing Consistency (15 pts)
- **14-15:** All padding/margin values from token scale
- **11-13:** 1-2 off-scale values (border cases)
- **6-10:** Multiple arbitrary px values — blocking
- **0-5:** No token usage — always blocking

### Depth Plane Effect (20 pts)
- **18-20:** Foreground/midground/background clearly distinct; parallax multipliers applied
- **14-17:** Effect present but subtle
- **8-13:** Effect missing or imperceptible — blocking
- **0-7:** Flat, no depth — blocking if archetype requires it

### No Placeholders (15 pts)
- **15:** Zero Lorem Ipsum, [PLACEHOLDER], TODO, FIXME in output
- **0:** Any placeholder found — instant BLOCK (non-negotiable)

### Design Law Compliance (15 pts)
- **14-15:** Zero law violations
- **11-13:** One warning-level deviation with rationale
- **6-10:** One blocking violation — blocking
- **0-5:** Multiple violations — always blocking

**Slop Test:** After scoring, ask: "Would a senior designer be embarrassed to show this?"
If yes → BLOCK regardless of score.

**Pass: ≥70 total AND no placeholders AND passes slop test**
