---
name: mobile-dashboard-audit
description: "Brownfield audit + Apple-level premium mobile UX upgrade of an existing dashboard: wiring truth audit, responsive/mobile audit across real breakpoints, Vaul sheets, Sonner toasts, Emil-style interaction physics, accessibility, performance, and a scored final report. Auto-activates on mobile/dashboard UX audit requests so it never needs manual invocation."
version: 1.0.0
author: Pauli Hermes Agent
license: MIT
platforms: [linux, macos, windows]
triggers:
  - mobile dashboard audit
  - apple-level UX upgrade
  - premium mobile UX
  - audit the dashboard for mobile
  - make the dashboard feel native
  - phone-first UX upgrade
  - mission mobile audit
  - brownfield mobile audit
  - dashboard UX gauntlet
  - /mobile-dashboard-audit
entry_point: /mobile-dashboard-audit [dashboard repo or path]
metadata:
  hermes:
    tags: [mobile-ux, dashboard, apple-level, vaul, sonner, accessibility, wiring-audit, icm-workflow]
    related_skills: [icm-architect, gauntlet-loop, design-intelligence]
    capabilities: [wiring-audit, responsive-audit, interaction-design, accessibility-review, production-qa]
    activation_style: automatic-on-keyword
---

# Mobile Dashboard Audit — Runtime Adapter

Thin runtime adapter. Canonical policy — role, non-negotiable rules, the
21-phase process (wiring audit, mobile audit, Apple/Emil interaction
principles, Vaul, Sonner, typography, color, performance, accessibility,
Gauntlet, completion gate, final report shape) — lives in
`../../icm/instructions/MOBILE_DASHBOARD_AUDIT.md`. This file exists only
so Hermes recognizes the trigger phrases above and lazy-loads that policy
without the user having to invoke a skill by name.

## Auto-Activation (No Setup Required)

Hermes detects mobile/dashboard UX audit language — "mobile dashboard
audit", "apple-level UX upgrade", "make the dashboard feel native",
"audit the dashboard for mobile", "phone-first UX upgrade", or an
explicit `/mobile-dashboard-audit` — and loads this skill automatically
against whichever dashboard repo the user names. No need to name the
skill.

## Load
1. `../../icm/AGENTS.md`
2. `../../icm/instructions/MOBILE_DASHBOARD_AUDIT.md`

## Execute
Follow `MOBILE_DASHBOARD_AUDIT.md` exactly, in bounded slices (audit →
mobile IA → interaction primitives → Vaul → Sonner → mission/agent/
approval experience → typography/visual hierarchy → motion → PWA/
performance → tests/screenshots/Gauntlet). Do not begin visual polishing
before Phase 1's wiring audit is understood. Do not call the dashboard
finished until every item in the completion gate is checked.

## Done
Return the final report shape from `MOBILE_DASHBOARD_AUDIT.md`
(executive summary, before/after scores, P0/P1/P2 findings, wiring map,
dependencies added, test/visual QA results, remaining risks, rollback,
and a production status of NOT READY / READY FOR PREVIEW / PREVIEW
VERIFIED / PRODUCTION VERIFIED — never claim PRODUCTION VERIFIED without
public runtime evidence).
