---
name: design-critic
description: >
  Runs the three-critic Gauntlet Loop protocol against a rendered design artifact,
  using the niche bar from _feeds/ as the comparison target. Fresh context for each
  critic. Binary verdicts only. Triggers on "run the design critic", "critique this",
  "does this beat the bar", or automatically at end of any design build session.
version: 1.0.0
author: Kupuri Media™ × Gauntlet Loop (Matt Shumer)
---

# Design Critic

## Protocol

Load `_shared/gauntlet-protocol.md` for the three-critic structure.
Load the relevant `_feeds/{niche}-latest.md` for the bar.
Load the rendered artifact (screenshot or HTML render).

## Pre-flight Check

Before running:
1. Is the bar named, fetchable, and comparable? If not — stop and name it properly.
2. Can the Craft Critic actually see rendered output (not just source code)?
3. Is the feed file present and dated within 7 days? If stale, trigger a new scrape first.

## Run Sequence

1. **Brief Critic** — fresh context, sees only: the stated goal + the rendered output.
   Verdict: PASSES / FAILS. Gap if FAILS.

2. **System Critic** — fresh context, sees only: the design token spec (or brand kit) + the rendered output.
   Verdict: COMPLIANT / NON-COMPLIANT. List violations.

3. **Craft Critic** — fresh context, sees only: `bar.md` mechanisms + side-by-side renders (our output vs bar site).
   Labels stripped. Verdict: OURS WINS / BAR WINS. Single biggest gap.

## Loop Rule

All three must pass. Any failure → back to builder with exactly one instruction:
fix the single biggest gap named by the failing critic. No other changes.

## Exit Condition

All three critics pass in the same round. Not a round count. Not a score.

## UDEC Score on Exit

When all three critics pass, run the UDEC 14-axis score against the final artifact.
Write the scorecard inline. If overall < 8.5 — one more loop. MOT or ACC < 7.0 — rebuild.
