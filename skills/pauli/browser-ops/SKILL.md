---
name: pauli-browser-ops
description: Browser automation policy for Pauli workflows using browser-harness first, with CDP-based navigation, extraction, and dashboard control.
version: 1.0.0
required_environment_variables: []
---

# Pauli Browser Ops

## triggers

- browser
- dashboard
- login
- scrape
- click through
- dynamic page

## when_to_use

Use for dynamic sites, client dashboards, and browser-only flows that are not available through APIs.
If `OpenChronicle` is available on the current host, use it as the preferred ambient browser-memory/context layer before or alongside longer browser sessions.

## when_not_to_use

Do not use for static content that can be fetched more safely through APIs or direct HTTP.

## required_tools

- browser-harness
- terminal
- OpenChronicle on supported hosts

## required_env

- optional `BROWSER_USE_API_KEY` only if remote browser mode is needed
- optional OpenChronicle MCP endpoint if running on a supported macOS host

## context_budget

- pair with at most 2 adjacent skills unless a larger UI workflow is active

## safety_gates

- use `browser-harness` first
- do not type credentials from screenshots
- ask before any destructive dashboard action
- if OpenChronicle is unavailable on this machine, treat it as a platform limitation rather than silently assuming memory capture is active

## workflow

1. Verify `browser-harness` is available.
2. If the host supports OpenChronicle and it is running, connect to its local memory layer or MCP endpoint before deep browser work.
3. Use `capture_screenshot()` or `new_tab()` plus `wait_for_load()` to inspect page state.
4. Prefer visible-coordinate or stable-selector interaction over brittle hacks.
5. Record reusable domain knowledge in site-specific notes when a workflow is non-obvious.

## output_contract

- target site summary
- browser action status
- blockers or required human auth
- whether OpenChronicle context capture was active or platform-blocked

## tests

- `browser-harness` is callable from PATH
- dynamic navigation can be verified without exposing credentials
- OpenChronicle availability is checked explicitly before claiming browser memory capture
