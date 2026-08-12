# Design Intelligence — Hermes Skill Bundle
## Kupuri Media™ × SYNTHIA™ | Version 2.0

This skill bundle makes Hermes a student of what is actually winning on the web
right now — not what was winning when Claude was trained.

## What it does

Every night at 2am UTC, the `design-intelligence-nightly` cron job scrapes:
- **Awwwards** — top 5 sites per niche, jury commentary, blog posts
- **styles.refero.design** — design token signals per niche (colors, typography, spacing)

It extracts checkable visual mechanisms (not adjectives) and writes structured
feed files to `_feeds/`. These feed files become the bar that the Gauntlet Loop
critic and UDEC scoring pipeline measure every client deliverable against.

## Skill map

| Skill | When to load |
|-------|-------------|
| `SKILL.md` (master) | Any visual build for a known niche |
| `sub-skills/awwwards-scraper/` | On demand or via cron |
| `sub-skills/refero-scraper/` | On demand or via cron |
| `sub-skills/design-critic/` | Gauntlet Loop critic phase |

## Relationship to existing skills

| Existing Skill | Relationship |
|----------------|-------------|
| `popular-web-designs` | Static vocabulary (54 brands). Load alongside for established brand references. |
| `claude-design` | Process driver. Load this for how to build; load `design-intelligence` for what to beat. |
| `pauli/impeccable-design` | Now includes UDEC scoring gate on every polish pass. |
| `pauli/taste-skill` | Now calibrated to live feed data, not training-data taste. |
| `gauntlet-loop` | The critic loop this skill feeds into. |
| `synthia-design-studio` | UDEC framework shared. Same 14 axes, same 8.5 floor. |

## Quality Floor

Nothing leaves Hermes with a UDEC score below 8.5.
MOT or ACC below 7.0 → rebuild. Not negotiate.

Kupuri Media™ × SYNTHIA™ v4.0
