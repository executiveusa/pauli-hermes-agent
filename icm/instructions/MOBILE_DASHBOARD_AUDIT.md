# Mobile Dashboard Audit + Apple-Level Premium UX Upgrade

## Inputs
- the existing dashboard repo/app the user names (brownfield — never
  redesign blindly);
- its current baseline (framework, runtime, deployment) — discovered in
  Phase 0, not assumed;
- only context named in this instruction — do not load the whole repo by
  default; discover it deliberately in Phase 0.

## Role

Senior product engineer, mobile UX architect, interaction designer,
accessibility reviewer, and production QA owner for an **existing**
dashboard application. This is a brownfield task, not a rebuild.

The job:
1. understand what the dashboard actually does;
2. map its real backend/data/control wiring;
3. audit the complete desktop + mobile experience;
4. find UX, interaction, responsiveness, accessibility, performance,
   truthfulness, visual, and wiring problems;
5. preserve everything that already works;
6. consolidate duplicate UI patterns;
7. upgrade the product to a polished, phone-first, premium control
   experience;
8. test the result on real responsive breakpoints;
9. produce evidence before calling anything complete.

The result should feel closer to a high-quality native Apple control
application than a generic SaaS admin dashboard.

"Apple-level" does **not** mean copy Apple visually. It means: immediate
response, restrained interface, clear hierarchy, direct manipulation,
excellent typography, spatial consistency, thoughtful motion, accessible
touch targets, excellent mobile ergonomics, trustworthy system state,
minimal cognitive load, premium finishing details, no visual or
interaction slop.

## Reference implementations and skills

Study these before making major interaction decisions. Do not install
every dependency merely because it appears here — study the pattern
first, use the smallest justified dependency set.

**Primary interaction / design references**
1. Emil Kowalski UI/interaction skill — https://emilkowal.ski/skill
2. Emil Kowalski skills repo — https://github.com/emilkowalski/skills
3. **Sonner** — https://github.com/emilkowalski/sonner — transient
   toast feedback (action accepted, save succeeded, mission queued,
   approval completed, deployment started, recoverable failure, retry
   notification, copy confirmation). Never use toasts for information
   that must remain visible.
4. **Vaul** — https://github.com/emilkowalski/vaul — mobile drawers/
   bottom sheets with physical gesture behavior (mobile navigation, agent
   detail sheet, mission details, approval review, filters, command
   options, project info, action sheets). Prefer bottom-sheet behavior on
   phones over desktop-style centered modals.

**Supporting quality/UX references**
5. ADHD usability skill — https://github.com/ayghri/i-have-adhd — reduce
   decision overload, make the primary action obvious, reduce dense
   control walls, separate advanced options from common actions, keep
   current state visible, avoid requiring users to remember hidden state.
6. Unlazy — https://github.com/Leonxlnx/unlazy — loading behavior,
   perceived performance, truthful loading states, remove unnecessary
   waiting, avoid fake progress, respond immediately to user intent.
7. Uncodixfy / anti-generic-AI design —
   https://github.com/cyxzdev/Uncodixfy (also
   https://github.com/executiveusa/pauli-Uncodixfy) — remove generic
   AI-dashboard appearance, eliminate excessive card walls, avoid
   pointless gradients/glows, avoid identical rounded cards everywhere,
   avoid generic hero copy, make information density intentional.
8. Gauntlet Loop — https://github.com/robonuggets/gauntlet-loop — final
   adversarial review loop: value, UX, architecture, correctness,
   security, accessibility, mobile behavior, runtime proof.

**Optional design/UI references when useful**
9. https://github.com/darula-hpp/uigen
10. https://github.com/executiveusa/pauli-taste-skill (if available)
11. https://github.com/DietrichGebert/ponytail
12. https://github.com/michaelshimeles/ralphy

## Non-negotiable rules

Do NOT:
- rebuild the application from scratch without proving that is necessary;
- replace working backend integrations for aesthetic reasons;
- fabricate data, agent status, mission state, or success;
- invent backend APIs because the UI wants them;
- introduce another orchestrator;
- put infrastructure authority directly in the browser;
- expose secrets in client JavaScript;
- expose raw shell execution from a public dashboard;
- create a second scheduler if one already exists server-side;
- create browser-only state for things that require durable backend state;
- hide errors to make the UI look cleaner;
- use animation to disguise latency, or animate everything;
- create card-within-card-within-card layouts;
- add decorative gradients/glow without functional purpose;
- use tiny touch targets or hover-only controls for essential actions;
- depend on desktop behavior for phone usage;
- use modal dialogs for every interaction;
- block the entire interface because one source is degraded;
- erase useful information merely to make the design minimal;
- call a deployment production-ready without runtime proof.

## Process

### Phase 0 — Baseline
Before editing anything, inspect: repository structure, framework,
runtime, package manager, dependencies, frontend entry points, API
routes, auth boundary, state management, data sources, backend services,
database, realtime channels, websockets, background polling, service
worker/PWA setup, existing tests, CI, deployment provider, production
domain if documented, current screenshots if present.

Return: FRAMEWORK, RUNTIME, PACKAGE MANAGER, UI LIBRARIES, AUTH MODEL,
API SURFACE, DATA SOURCES, MOBILE SUPPORT, PWA SUPPORT, CURRENT TESTS,
DEPLOYMENT MODEL.

Do not modify code during baseline discovery unless specifically
authorized.

### Phase 1 — Wiring audit
Map every important UI surface: UI CONTROL → FRONTEND HANDLER → API
ROUTE → BACKEND SERVICE → DATA/EXECUTOR → RESPONSE → UI RESULT.

Examples: dashboard metrics → `/api/dashboard` → control plane → runtime
telemetry. Agent status → `/api/fleet` → heartbeat source → agents.
Approve button → `/api/approvals/:id` → authoritative control plane →
mission state. Voice command → speech input → command API → orchestrator
→ mission or response.

Classify each path: VERIFIED, PARTIAL, BROKEN, DUPLICATE, FAKE/STATIC,
DEAD, UNSAFE, UNKNOWN.

Find: hard-coded status, stale routes, duplicate endpoints, direct
browser infrastructure mutations, inconsistent approval paths,
direct-model bypasses, frontend shadow state, polling duplication,
legacy integrations, unused screens, routes that pretend success,
dangerous mutations without auth, inconsistent response schemas.

Do not start visual polishing until P0 wiring problems are understood.

### Phase 2 — Mobile audit
Audit at minimum 320/360/375/390/414/430/768/1024/1440px, both portrait
and important landscape cases.

- **Navigation** — reachable one-handed; understandable without cryptic
  labels; primary action near the thumb zone; mobile nav doesn't take
  over the entire experience unnecessarily.
- **Touch** — minimum ~44×44 CSS px targets, hit area beyond the visible
  glyph when appropriate, no essential 16px icon-only buttons, no
  hover-dependent actions.
- **Content** — no horizontal scrolling unless content requires it, no
  tiny desktop table squeezed onto a phone, convert dense tables to
  progressive mobile detail layouts, preserve information while changing
  presentation.
- **Forms** — tappable input sizes, appropriate input types, keyboard
  doesn't hide action controls, bottom composer respects safe-area
  inset, errors appear near the affected control.
- **State** — loading, empty, error, degraded, offline, reconnecting,
  queued, working, success, blocked all need deliberate treatments.

### Phase 3 — Apple-level interaction principles
Use the Emil/Apple design skill as a behavior spec.
1. **Response** — feedback starts on pointer/touch down; buttons feel
   pressed immediately (typical `transform: scale(.97-.985)`, ~80–120ms);
   no long 300–500ms button animations.
2. **Direct manipulation** — sheets/drawers/sliders/movable surfaces
   track the pointer/finger 1:1.
3. **Interruptibility** — gesture-driven motion must be interruptible;
   reversible mid-motion; avoid non-interruptible keyframe animations
   for interactive surfaces.
4. **Spring behavior** — critically damped/no-bounce for normal UI
   transitions (damping ≈ 1.0); bounce belongs to momentum gestures
   (damping ≈ .8). Do not bounce every menu/button/card/modal/toast.
5. **Velocity** — sheets/draggable surfaces carry finger velocity into
   resting motion.
6. **Spatial consistency** — dismiss toward where it entered from;
   maintain perceived origin relationships for details.
7. **Materials** — translucency for floating functional layers (mobile
   toolbar, bottom sheet, floating command bar) — don't turn the whole
   product to glass.
8. **Reduced motion** — support `prefers-reduced-motion`, and consider
   `prefers-reduced-transparency`/`prefers-contrast`. Motion accessibility
   is mandatory.

### Phase 4 — Mobile information architecture
Don't put desktop navigation on a phone unchanged. For a control
dashboard, consider phone primary destinations: Home/Command,
Work/Missions, Agents, Needs You/Approvals; secondary (in a Vaul
sheet/drawer): Projects, System, Integrations, Settings. Keep the most
common action obvious. For an AI/agent-control dashboard, the home
screen should answer within a few seconds: What is happening? What
needs me? Is anything broken? What can I tell the system to do?

### Phase 5 — Dashboard home experience
Don't create twelve equal cards — establish hierarchy: **Primary**
(command/search/voice), **Secondary** ("right now": active work, blocked
work, approvals, incidents), **Tertiary** (compact system/fleet
indicators). Use disclosure — the owner shouldn't consume the entire
infrastructure graph every time they open the dashboard. Then drill into
details.

### Phase 6 — Vaul
Use Vaul where the app benefits from mobile sheets: Mission Details
Sheet (title, project, agent, model/provider, elapsed time, branch,
cost, verification, preview, errors, receipt); Agent Sheet (agent, role,
status, current mission, model, last heartbeat, resource/cost summary,
owner-allowed actions); Approval Sheet (what will happen, why approval
is required, affected project, diff/preview/evidence, cost, risk,
rollback, Approve, Reject); Project Sheet (repo, production URL, current
stage, active missions, latest verified SHA, blockers). On desktop these
may become side panels; on phone, bottom sheets.

### Phase 7 — Sonner
Use toasts for transient acknowledgement only:
`toast.success("Mission queued")`, `toast.success("Approval recorded")`,
`toast.error("Hermes is unreachable")`, `toast.loading("Creating
preview…")`, `toast.promise(...)`. Do not display mission progress
solely through toasts — persistent work belongs in the Missions UI. Do
not toast every polling refresh or create toast spam.

### Phase 8 — Typography
Prefer platform/system typography unless the product has an intentional
type system. Use system UI stack, optical hierarchy, tighter tracking on
large headings, readable small labels, generous body leading, tabular
numerals for operational metrics. Avoid giant marketing typography in
operational dashboards, all-caps paragraphs, tiny 8–9px essential text,
excessive bold, seven simultaneous font weights.

### Phase 9 — Color + material
Color must carry meaning: green = healthy/verified/success; amber =
waiting/needs attention/queued; red = error/failed/destructive; neutral
= idle/unknown/unavailable. Never red decoratively; never green for
merely-configured. Distinguish configured / connected / healthy /
verified / production visually and semantically. Translucent surfaces:
keep legibility high, avoid stacked glass layers, provide a
reduced-transparency fallback, use subtle elevation.

### Phase 10 — Loading + perceived performance
Apply Unlazy principles. Remove artificial delays, fake spinners,
blank-screen loading, one giant global loading state. Prefer immediate
shell, cached last-known state when truthful, local skeletons,
optimistic feedback only for safely reversible actions, server-confirmed
completion for consequential actions. If one API fails, show that
section as degraded — do not kill the whole dashboard.

### Phase 11 — ADHD/cognitive-load review
Apply even if the target audience doesn't have ADHD. Ask: what's the one
most important thing on this screen; are six choices shown where two
would do; does the owner need to remember state from another screen;
are advanced controls visible before necessary; is there a clear next
action; can the user tell what the system is doing; is a technical
implementation detail exposed unnecessarily. Use progressive disclosure
— e.g. "Mission / Running / 27 minutes / [View details]" beats showing
every internal field (Mission ID, Container, PID, Provider, Model,
Worktree, Branch, Hash, Queue, Retry, Memory, Context, Token count) at
once. The detail still exists, behind the appropriate disclosure.

### Phase 12 — Agent control experience
If the dashboard controls AI agents, each agent should communicate:
NAME, ROLE, STATE, CURRENT WORK, LAST HEARTBEAT, MODEL/PROVIDER (when
useful), CURRENT PROJECT, COST (when known). States must be explicit:
offline, idle, thinking, working, waiting, blocked, needs_human, failed
— never collapse to online/offline. Never fabricate heartbeat data.
Agent controls route through the authoritative backend; the browser must
never become the orchestrator.

### Phase 13 — Mission experience
Every mission view should eventually support: MISSION, PROJECT,
OBJECTIVE, STATUS, AGENT, WORKER, MODEL, PROVIDER, BRANCH, ELAPSED TIME,
COST, TEST STATUS, BUILD STATUS, PREVIEW, LATEST VERIFIED SHA, ROLLBACK
SHA, FAILURE REASON, NEXT ACTION — scannable without reading logs. Raw
logs behind a details disclosure.

### Phase 14 — Approval experience
Approvals should feel consequential — no tiny inline check/X icons. A
premium approval explains: what will happen, why the system stopped,
what evidence exists, what could go wrong, what the rollback is; then
Approve / Reject. Require explicit confirmation for destructive/high-risk
operations; do not add confirmations to trivial reversible actions.

### Phase 15 — Voice control
If voice exists, it uses the same governed command path as typed input
— never a special ungoverned executor: voice → speech recognition →
normal command request → orchestrator → mission/control plane → receipt.
Voice UI must visibly represent listening/processing/speaking/error and
provide a stop/interrupt control. TTS is presentation, not reasoning.

### Phase 16 — Empty/error states
No generic "Something went wrong." Use actionable truth, e.g.
"Terabithia is unreachable. Project data may be stale.", "Hermes did not
acknowledge this mission. Nothing was executed.", "No approvals are
waiting.", "Agent heartbeat has not been received for 12 minutes.",
"Voice provider is not configured. Text commands still work." Never
imply success where there is none.

### Phase 17 — Premium microinteractions
Apply selectively: press compression, gentle spring sheet behavior,
active navigation movement, focus transitions, state-change highlight,
progress transition, toast acknowledgement, contextual sheet expansion,
smooth list insert/remove, voice listening pulse, success confirmation.
Avoid infinite decorative movement, glowing orbs everywhere, bouncing
cards, elaborate page transitions, animation that delays task
completion. Every animation must answer "what information does this
movement communicate?" — if nothing, remove it.

### Phase 18 — Accessibility
Audit keyboard navigation, screen readers, focus order/visibility, ARIA,
reduced motion, contrast, font scaling, touch target size, safe areas,
orientation, voice alternative, color-independent state indicators.
Critical operations must not rely on color alone (e.g. "● Healthy", not
merely a green dot).

### Phase 19 — Performance
Audit bundle size, dependency cost, image/font loading, layout shift,
unnecessary rerenders, poll intervals, large lists, unnecessary/duplicate
API calls, hydration, service worker caching, mobile CPU, animation
compositing. Prefer `transform`/`opacity` for animation; avoid
layout-heavy animation. Verify any added dependency (Sonner/Vaul/etc.) is
justified and compatible.

### Phase 20 — PWA quality
If intended for phone use, audit manifest, name, icons, maskable icon,
theme color, standalone display, safe areas, service worker, offline
shell, update behavior, installability, touch icons, status bar
behavior. Do not cache API responses blindly — operational data must
remain truthful. Offline shell ≠ offline operational authority.

### Phase 21 — Gauntlet
After implementation, run an adversarial audit: VALUE (easier to
operate?), MOBILE (usable from a phone?), TRUTH (everything shown
corresponds to a real source?), WIRING (every control reaches the
intended backend?), SECURITY (can the browser access something it
shouldn't?), PERFORMANCE (did polish make it slower?), ACCESSIBILITY
(usable with assistive settings?), TASTE (intentionally designed, not
AI-generated?), COMPLEXITY (did dependencies/abstractions earn their
cost?). Fix P0/P1 issues and rerun.

### Test matrix (minimum)
AUTH — unauthenticated privileged routes → 401. SECURITY — direct
shell/infrastructure mutation denied, secrets never returned to
browser. MOBILE — 320/375/390/430/768px, no unintended horizontal
overflow, navigation works, bottom sheets work, keyboard doesn't trap
controls, touch sizing meets requirements. INTERACTION — sheet
open/close/swipe, backdrop behavior, escape behavior, focus restoration,
reduced motion. NOTIFICATIONS — success/error/promise toast, no
duplicate spam. MISSIONS — queued/running/blocked/failed/verified.
AGENTS — online/idle/working/stale heartbeat/offline/failed. APPROVALS
— view/approve/reject/error/confirmation-if-destructive. ERROR STATES —
backend unreachable, partial outage, voice unavailable, database
unavailable. PWA — manifest, service worker, installability, standalone
display, safe-area layout.

### Visual QA
Screenshots for phone (command/home, navigation open, mission sheet,
agent sheet, approval sheet, error state, active voice, landscape if
relevant), tablet, desktop. Review alignment, spacing, type hierarchy,
tap targets, overflow, clipping, safe areas, sticky controls, sheet
height, keyboard overlap, dark/light mode if supported. Do not call
visual QA complete without rendered evidence.

### Implementation strategy — work in bounded slices
1. Audit + truth/wiring fixes
2. Mobile information architecture
3. Interaction primitives
4. Vaul mobile sheet system
5. Sonner notification system
6. Mission / Agent / Approval experience
7. Typography / spacing / visual hierarchy
8. Motion + reduced motion
9. PWA + performance
10. Tests + screenshots + Gauntlet

Each slice reports: DECISION, CHANGES, PROOF, RISKS, ROLLBACK, NEXT.

## Outputs

A final report containing: EXECUTIVE SUMMARY; ORIGINAL DASHBOARD SCORE
(0–100 for mobile usability, interaction quality, visual hierarchy,
accessibility, performance, wiring integrity, system truthfulness,
premium feel); P0/P1 PROBLEMS; P2 IMPROVEMENTS; WIRING MAP; MOBILE UX
FINDINGS; INTERACTION FINDINGS; WHAT WAS REUSED / REMOVED / ADDED;
DEPENDENCIES ADDED (dependency, reason, bundle/maintenance impact, why
native/custom was not preferable); SONNER USAGE; VAUL USAGE;
APPLE/EMIL PRINCIPLES APPLIED; ADHD/COGNITIVE-LOAD IMPROVEMENTS;
ACCESSIBILITY RESULTS; PERFORMANCE RESULTS; TEST RESULTS;
RESPONSIVE/VISUAL QA; REMAINING RISKS; ROLLBACK; FINAL SCORE; PRODUCTION
STATUS.

Production status uses only: NOT READY, READY FOR PREVIEW, PREVIEW
VERIFIED, PRODUCTION VERIFIED. Never claim PRODUCTION VERIFIED without
public runtime evidence.

## Failure and stop conditions
- Do not start visual polishing until Phase 1 wiring problems are
  understood.
- Do not call the dashboard finished until every item in the completion
  gate is checked: repository baseline documented; wiring graph
  completed; no known fake-success behavior; no fabricated operational
  data; mobile navigation passes; phone command flow passes; mission
  details usable on phone; agent controls usable on phone; approvals
  usable on phone; Sonner feedback integrated appropriately; Vaul sheets
  integrated where justified; Emil interaction principles applied;
  reduced-motion behavior verified; accessibility review passes;
  responsive screenshots reviewed; no accidental horizontal overflow;
  build passes; tests pass; runtime/API integration passes; PWA behavior
  verified if applicable; production deployment points to the exact
  tested SHA; production smoke test passes; rollback target recorded;
  final Gauntlet passes.

## Risk tier / human gate
Production deployment, destructive/admin actions, and any claim of
PRODUCTION VERIFIED require human review of the runtime evidence before
being treated as final. Everything through slice 9 (audit → tests) may
run under approved autonomy; production promotion does not.

## Human check
The user reviews the final report (scores, P0/P1 list, screenshots,
dependency list) and the production-status claim before treating the
upgrade as shipped.

## Design standard
The finished experience should feel fast, quiet, direct, trustworthy,
physical, focused, premium, mobile-native. The user should feel like
they're controlling a powerful system, not administering a database. AI
brings the system's power; the interface gives the human control.
