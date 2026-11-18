---
name: awwwards-design-reference
description: Research Awwwards design references on demand.
---

# Awwwards — optional design reference and motion evidence

**Authority:** Hermes `pauli-design-factory` delegates design governance to `executiveusa/brand-kit-builder-`; Vibe Engineering owns its House and ICM quality gates. This skill is a **research adapter**, not a new design factory, Awwwards affiliation, always-on daemon or permission to use third-party assets commercially.

## Prerequisites

Node >=22.13 and an approved, pinned MCP setup for live tool use. Read source terms; third-party media are not MIT licensed. The skill itself works as reference-only without the MCP. Enable only through `ENABLE_AWWWARDS_MCP=1 bash .hermes/mcp/bootstrap.sh` after package and environment review.

## How to Run

Ask Hermes for Awwwards design references or explicitly select `awwwards-design-reference`. Confirm the resolved skill file and whether the `awwwards` MCP is registered. Do not install anything automatically.

## Quick Reference

Core: `search_sites` → `get_site_details` → `compare_sites` or `create_design_brief`. Optional, authorized browser: `analyze_page_structure` and `verify_build`.

## Procedure

### Load only when

The brief explicitly calls for award-winning sites, Awwwards comparisons, visual DNA, layout band maps, or evidence-based interaction and motion analysis. For ordinary UI work, prefer existing brand guidelines, saved screenshots, Inspo, or the project's present component system. Load **one** specialist reference source at a time; do not inject the entire design skill catalog.

## Provenance

- Upstream: https://github.com/INSANE0777/Awwwards-mcp
- Verified upstream source commit: `0560b94e646db37edcdba5249beeabf8c58ba6c7` (2026-11-16).
- Package manifest declares `awwwards-mcp` version `1.20.0`; the published npm package and server runtime must be independently checked before executing.
- License: MIT for the server code. Awwwards reference content, screenshots, awards, and original creator assets carry separate restrictions; upstream README describes personal/non-commercial inspiration. **Commercial use requires reviewing Awwwards' terms and rights independently.**
- Node >=22.13.0. Core stdio path has no browser instrumentation by default.

## Upstream specialized skill library (load one, not nine)

The author's companion skill repository is https://github.com/INSANE0777/awwwards-skills at verified release `skills-v2026.11.16` (ref SHA `407fc695d261d24c04401b16b6dc4762dd961172`). **Do not auto-install or flatten these nine upstream skills**. Fetch the complete selected skill directory only when the current stage calls for it, following its own `SKILL.md` and linked references:

| Owner skill | Selection trigger |
| --- | --- |
| `awwwards-inspiration` | Discover and compare reference designs before building |
| `awwwards-design-md` | Cited evidence-backed DESIGN.md for an authorized public URL |
| `awwwards-motion-study` | Detailed motion capture and timing reconstruction |
| `awwwards-section-loop` | Verify one page band/section at a time |
| `motion-authoring` | Author verified motion from a manifest |
| `awwwards-setup` | First-run preferences and installation, only after owner opt-in |
| `awwwards-doctor` | Diagnose failed tools, stale index or captures |
| `recreate` | Authorized, non-published study recreation; not a commercial clone |
| `awwwards-atlas` | Experimental Atlas evidence layer; requires separate `AWWWARDS_MCP_ATLAS=1` authorization |

Use `awwwards-inspiration` for the ordinary brief workflow; use `awwwards-motion-study` only for significant motion jobs. The source skill tag pairs with `awwwards-mcp@1.20.0`, which was released upstream. The MCP server is still a separate opt-in runtime, and its package signature/integrity should be checked before running it.

## Step-by-step selection (cheapest useful route)

1. **Inspect:** read the owner brief and Design Factory capabilities. Choose `awwwards-design-reference` only for an explicit award-site question. Confirm rights, authorized target and data exposure before any live capture.
2. **Discover:** check `mcp2cli bake list` and whether an `awwwards` MCP tool is actually registered. If unavailable, use the upstream README as a reference or stop with the exact enablement instruction. Do not claim it is installed because this skill exists.
3. **Research (core):** `search_sites` with `responseMode: "compact"`, choose 1–3 real sites, then `get_site_details` and optionally `compare_sites` or `create_design_brief`. Record source slugs and attribution. Stop when the brief is sufficient.
4. **Deep dive (optional, gated):** only for an authorized URL and sandboxed browser lane, consider `get_motion_dna`, `analyze_page_structure`, `analyze_scroll_map`, `analyze_hover_census`, `capture_live_site`, `record_site_motion`, or `verify_build`. These require a separately reviewed browser-enabled Playwright/ffmpeg installation and scoped network access. Do not inspect private URLs, user sessions, protected network resources, credentials, or script bundles without authorization.
5. **Build through existing workflow:** hand the cited brief to the canonical Design Factory or repository's Vibe W5 Web Design workflow. No pixel-for-pixel cloning, republished screenshots, or wholesale copied code. Preserve the client's visual identity and conversion goal.
6. **Proof and Council:** run mobile/reduced-motion/a11y checks, verify the actual user journey, capture evidence, and dispatch a different-model independent critic. Judge/owner approval remains required before publishing.

## Pitfalls

Do not bulk scrape, republish copyrighted media, enable experimental Atlas, or assume a core npx install supports browser capture. Study recreations are not publication-ready.

## Verification

Verify resolved checkout path, isolated-home skill loading, pinned package and explicit MCP opt-in. Run `pytest -q tests/skills/test_awwwards_design_reference_skill.py tests/agent/test_pauli_skill_router.py`. Check an attributed design brief and separate review before claiming success.

## Local MCP transport (opt-in)

In an authorized Hermes checkout, `.hermes/mcp/bootstrap.sh` registers the `awwwards` MCP only when `ENABLE_AWWWARDS_MCP=1`. That bootstrap checks `mcp2cli` and Node >=22.13, and uses `npx -y awwwards-mcp@1.20.0` as local stdio. Review the package's current publish/version and installation effects before setting the flag. Automatic self-update is not enabled. The npx route omits browser tools; do not silently add Playwright or ffmpeg. Disable by removing the optional bake according to existing `mcp2cli` procedures; no production services are required.

**Do not** install globally, start crawls, modify service units, or spend money simply because a skill was selected. Never put secrets in logs or prompts.

## Return contract

Return: `{status, target, design_goal, sources_with_urls, chosen_reference, tool_version, tools_used, reference_findings, evidence, terms_status, blockers, next_action}`. Distinguish `REFERENCE_ONLY`, `TOOL_CONNECTED`, `BROWSER_VERIFIED`, and `PRODUCTION_VERIFIED`; never claim later states from earlier ones.
