---
title: "Hermes Rolodex — Hermes Rolodex™ — AI-powered relationship graph that grows from conversation"
sidebar_label: "Hermes Rolodex"
description: "Hermes Rolodex™ — AI-powered relationship graph that grows from conversation"
---

{/* This page is auto-generated from the skill's SKILL.md by website/scripts/generate-skill-docs.py. Edit the source SKILL.md, not this page. */}

# Hermes Rolodex

Hermes Rolodex™ — AI-powered relationship graph that grows from conversation. Trigger when user says things like: "remember that", "who was", "I just met", "meeting brief", "birthday", "reconnect with", "who do I know in", "that person from", "add to my rolodex", "remind me about", "who introduced".

## Skill metadata

| | |
|---|---|
| Source | Bundled (installed by default) |
| Path | `skills/hermes-rolodex` |
| Version | `1.0.0` |
| Author | Pauli Second Brain™ | Kupuri Media™ |
| Tags | `relationships`, `memory`, `networking`, `rolodex`, `people`, `contacts` |
| Related skills | `note-taking` |

## Reference: full SKILL.md

:::info
The following is the complete skill definition that Hermes loads when this skill is triggered. This is what the agent sees as instructions when the skill is active.
:::

# Hermes Rolodex™

One agent. Every person. Never forgotten.

## When to Activate

Activate this skill when the user:
- Says **"remember that"** (person mention in conversation)
- Asks **"who was"** that person with X quality
- Says **"I just met"** someone new
- Asks for a **"meeting brief"** before a meeting
- Mentions a **birthday** or asks about upcoming events
- Asks to **"reconnect with"** or check on someone
- Asks **"who do I know in"** a city or industry
- Describes someone vaguely: "that tall guy from Berlin with the green coat"
- Says **"add to my rolodex"** or "log this interaction"
- Asks **"who introduced"** people to each other

## Core Workflow

### Adding someone new
1. Call `rolodex_add_person` with name + any context the user provided
2. If user gave a description (context_tags), include them
3. Report: "📇 Added [Name] to Rolodex"

### Logging an interaction
1. Identify person via `rolodex_fuzzy_recall` if ID not known
2. Call `rolodex_add_memory` with the interaction text
3. Report: "📇 [Name] updated."

### Recalling someone
1. Call `rolodex_fuzzy_recall` with the user's natural language description
2. Return the best match with confidence score
3. If multiple results, present top 3 and ask for confirmation

### Pre-meeting brief
1. Call `rolodex_meeting_brief` with person name or ID
2. Return the `narrative` field formatted for easy scanning

### Relationship health check
1. Call `rolodex_fading_check` for overdue contacts
2. For each FADING person, offer to use `rolodex_draft_outreach`

## MCP Tools Reference

| Tool | When to Use |
|------|-------------|
| `rolodex_add_person` | New person mentioned for first time |
| `rolodex_fuzzy_recall` | Vague description, natural language search |
| `rolodex_get_person` | Full profile lookup by name or ID |
| `rolodex_add_memory` | Log any interaction or note |
| `rolodex_set_reminder` | Schedule meeting, birthday, or follow-up |
| `rolodex_meeting_brief` | Before any meeting |
| `rolodex_draft_outreach` | Reconnect with someone |
| `rolodex_fading_check` | Weekly relationship health check |
| `rolodex_upcoming_events` | Morning briefing, week ahead |
| `rolodex_graph_query` | Network traversal queries |
| `rolodex_queue_unknown` | Can't identify person yet — queue for later |

## Strength Labels

- **ACTIVE** (≥0.70) — in regular contact
- **WARM** (0.30–0.70) — occasional contact
- **FADING** (&lt;0.30) — needs reconnecting

Strength decays at `0.95^(days_since_contact / 7)` and boosts +0.1 per logged memory.

## Auto-Index Behavior

When Hermes detects a person mention in any conversation:
1. Check `rolodex_fuzzy_recall` — is this person already in the graph?
2. If yes: optionally log context via `rolodex_add_memory`
3. If no: create a draft entry via `rolodex_add_person` with what was mentioned
4. Append to message: "📇 Rolodex: [Name] updated."

## Installation

Copy this skill to `~/.hermes/skills/hermes-rolodex/SKILL.md` and add the
MCP server block from `config_patch.yaml` to `~/.hermes/config.yaml`.
