# Reference Repos — Design Laws Sources

All repos are lazy-loaded. Do not clone. Fetch README + 2-3 key files when needed.

## Animation & Motion

### `greensock/GSAP`
**Domain:** Web animation engine
**Core teaching:**
- Use `transform` and `opacity` for all animation — never width/height/top/left
- `ScrollTrigger` with `scrub: 1` (numeric) for scroll-linked motion
- `gsap.registerPlugin(ScrollTrigger)` must run before any ScrollTrigger use
- Timeline `scrub` value controls smoothing: `1` = 1 second lag, `true` = instant
- GPU compositing is automatic on `transform` animations — don't force it with `will-change`

**Key files to read:** `README.md`, any ScrollTrigger docs

---

### `oso95/scroll-world`
**Domain:** Scroll-world technique library
**Core teaching:**
- Scroll binding patterns: viewport-relative vs document-relative progress
- Momentum / inertia on scroll: `lerp()` for smooth following
- Snap points: only use for discrete sections, never on free-scroll narrative
- Passive scroll listeners for performance

**Key files to read:** `README.md`, scroll binding examples

---

## Design Quality

### `pbakaus/impeccable`
**Domain:** Design constraint philosophy
**Core teaching:**
- Constraints are not limitations — they are design tools
- Fewer choices → more coherent output
- Every element earns its place or it's cut
- Polish is not decoration; it's removing everything that isn't essential

**Key files to read:** `README.md`

---

### `emilkowalski/skills`
**Domain:** Interaction quality bar
**Core teaching:**
- Spring physics feel more natural than bezier curves for user-initiated motion
- Duration matters: 200ms for micro, 400ms for standard, 800ms for cinematic
- Hover states must respond in <100ms
- Never animate the same property in two different ways simultaneously

**Key files to read:** `README.md`, interaction examples

---

### `ihlamury/design-skills`
**Domain:** Visual design patterns
**Core teaching:**
- Hierarchy: size → weight → color → position (in that order of power)
- Contrast creates emphasis; insufficient contrast = visual noise
- Rhythm: repeated spacing intervals create coherence
- Negative space is active, not absent

**Key files to read:** `README.md`

---

### `ytx-readings/design-ui-ux`
**Domain:** UI/UX anti-patterns
**Core teaching:**
- Don't use parallax that conflicts with content legibility
- Above-the-fold must earn attention in <3 seconds
- Scroll hijacking (preventing native scroll) destroys trust
- Never animate content that the user is actively reading

**Key files to read:** `README.md`

---

## Cinematic & Scroll Techniques

### `robonuggets/cinematic-site-components`
**Domain:** Cinematic UI components
**Core teaching:**
- Depth planes: foreground (1.0x), midground (0.6x), background (0.3x)
- Camera drift: slow, continuous movement in one direction
- Scene transitions: cut vs dissolve (use cut for energy, dissolve for calm)
- Hero image should fill viewport edge-to-edge on load

**Key files to read:** `README.md`, depth plane component examples

---

## Component Architecture

### `darula-hpp/uigen`
**Domain:** UI generation patterns
**Core teaching:**
- Components take config objects, not prop soup
- Container queries over media queries when possible
- HTML structure drives CSS, not the reverse
- `data-*` attributes for JS targeting, class for CSS styling

**Key files to read:** `README.md`

---

### `atomicdotdev/atomic`
**Domain:** Atomic design system
**Core teaching (installed on user laptop — reference locally):**
- Token → Component → Template → Page hierarchy
- Design tokens are the contract between design and code
- Components are never aware of their layout context
- Templates compose components; pages instantiate templates with real content

---

## Data & Handoffs

### `safishamsi/graphify`
**Domain:** Graph-to-insight pipeline
**Core teaching:**
- Nodes are entities, edges are relationships — never the reverse
- Clustering reveals hidden structure in flat data
- Reason over the graph, not the raw list
- Graph density (edges/nodes ratio) indicates knowledge richness

**Key files to read:** `README.md`, graph builder examples

---

### `Fission-AI/OpenSpec`
**Domain:** Agent handoff specification format
**Core teaching:**
- All inter-agent data must be schema-validated before transit
- `spec_version` field enables forward compatibility
- `gate_status` is part of the spec, not a separate signal
- Receiving agent must not proceed without a valid signed handoff

**Key files to read:** `README.md`, spec schema

---

### `willseltzer/claude-handoff`
**Domain:** Agent chain-of-custody
**Core teaching:**
- Every agent records what it received, what it produced, and what it decided
- Handoff receipts enable debugging and rollback
- No agent should "just know" what came before — it reads the handoff

**Key files to read:** `README.md`

---

## Deployment

### `vercel-labs/opensrc`
**Domain:** Vercel deployment patterns
**Core teaching:**
- `vercel.json` with `outputDirectory` for static sites
- Edge functions for dynamic content without cold starts
- `headers` config for cache-control on assets
- Preview deployments on every PR are the review mechanism

**Key files to read:** `README.md`, `vercel.json` examples

---

## Atomic Rollback

### `gastownhall/beads`
**Domain:** Atomic design change rollback
**Core teaching:**
- Every design change = one bead (atomic unit)
- Beads are immutable once created
- One-click rollback to any prior bead
- Bead manifest is the audit trail

**Key files to read:** `README.md`, bead creation examples
