# Scroll Archetypes — Crawford Reference

Derived from @bycrawford ICM workflow analysis.

## The 5 Archetypes

### 1. POV Walk
**Feel:** Camera moves through a 3D scene as user scrolls
**Signature:** Content appears to have depth; user feels like they're moving through space
**GSAP pattern:**
```javascript
gsap.timeline({ scrollTrigger: { trigger: ".scene", scrub: 1 }})
  .to(".bg-layer", { z: -100 })
  .to(".mid-layer", { z: -40 })
  .to(".fg-layer", { z: 0 })
```
**Use when:** Product reveal, immersive brand story, game/entertainment
**Crawford example:** Site walkthroughs that feel like architectural tours

---

### 2. Reveal
**Feel:** Content unveils or emerges as user scrolls down
**Signature:** Elements animate from hidden to visible with directional motion
**GSAP pattern:**
```javascript
gsap.from(".reveal-element", {
  scrollTrigger: { trigger: ".section", start: "top 80%", toggleActions: "play none none reverse" },
  y: 60,
  opacity: 0,
  duration: 0.6,
  stagger: 0.1
})
```
**Use when:** Feature lists, testimonials, content-heavy sections
**Crawford example:** Feature reveals on SaaS landing pages

---

### 3. Drift
**Feel:** Slow, continuous floating — elements move at different rates
**Signature:** Nothing snaps; everything drifts; deeply calming
**GSAP pattern:**
```javascript
gsap.to(".drift-layer", {
  scrollTrigger: { trigger: "body", start: "top top", end: "bottom bottom", scrub: 2 },
  y: "20%",
  ease: "none"
})
```
**Use when:** Luxury brand, wellness, editorial, creative portfolio
**Crawford example:** Atmospheric brand sites where the mood is the message

---

### 4. Momentum
**Feel:** Sections snap with inertia; discrete steps with smooth transitions
**Signature:** Each section is a full-viewport "world"; transitions have character
**GSAP pattern:**
```javascript
ScrollTrigger.create({
  trigger: ".section",
  start: "top top",
  end: "+=100%",
  snap: { snapTo: 1, duration: 0.3, ease: "power2.inOut" },
  pin: true
})
```
**Use when:** Portfolio, slides-style presentation, product comparisons
**Crawford example:** Step-by-step feature demonstrations

---

### 5. Cinematic
**Feel:** Full-viewport scenes unfold like a film; scroll is the timeline
**Signature:** Hero takes full viewport; scroll advances the story; text appears/disappears at precise moments
**GSAP pattern:**
```javascript
const tl = gsap.timeline({
  scrollTrigger: { trigger: ".cinematic-section", start: "top top", end: "+=300%", scrub: 1, pin: true }
})
tl.to(".title", { opacity: 1, y: 0, duration: 0.3 })
  .to(".subtitle", { opacity: 1, y: 0, duration: 0.3 }, "+=0.1")
  .to(".cta", { opacity: 1, scale: 1, duration: 0.2 }, "+=0.2")
  .to(".title", { opacity: 0, duration: 0.2 }, "+=0.5")
```
**Use when:** Product launch, storytelling brand, high-impact hero section
**Crawford example:** The primary pattern used in 3D scroll site builds

---

## Archetype Selection Guide

| If brief says... | Use archetype |
|-----------------|---------------|
| "immersive", "3D", "depth" | POV Walk |
| "feature-rich", "show everything" | Reveal |
| "luxury", "editorial", "calm" | Drift |
| "product demo", "step by step" | Momentum |
| "launch", "cinematic", "hero" | Cinematic |
| unclear | Cinematic (default — highest impact) |

## Combining Archetypes

Sections within one page can use different archetypes.
Rule: never switch archetypes within a single section.
Rule: limit to 3 archetypes per page (too many = incoherence).

## Crawford's ICM Flow (reference)

1. Brief → pick archetype
2. Find or generate hero image (Higgsfield for AI, stock for others)
3. Animate hero with chosen archetype
4. Deploy to Vercel for preview
5. Iterate on micro-interactions
6. Final delivery
