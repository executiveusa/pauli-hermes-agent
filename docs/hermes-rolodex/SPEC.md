# Hermes Rolodex™ — Feature Spec
## pauli-hermes-agent | Pauli Second Brain™
## SYNTHIA™ 3.0 Self-Score Floor: 8.5/10

---

### What This Feature Is

One AI agent (Hermes) operating a relationship graph. The user never fills
out a form. The graph grows from conversation. The agent watches for person
mentions, extracts context, stores nodes and edges, and makes the user feel
like they have photographic memory for every person they have ever met.

---

### Core Capability Set

#### 1. FUZZY RECALL — natural language → graph traversal → person match
- Input: "that woman with the red boots who hated small talk"
- Output: Priya Sharma, Arkon Labs, Austin TX (confidence: HIGH)
- Method: FTS5 + LIKE context matching + 2-hop graph traversal

#### 2. AUTO-INDEX — silent background operation
- When Hermes detects a person mention in any conversation, it silently
  calls `rolodex_add_memory` or creates a draft node.
- Reports at end of message: "📇 Rolodex: [name] updated."

#### 3. MEETING BRIEF — pre-meeting context surface
- Input: person name or ID
- Output: who they are, last 3 interactions, open threads,
  what not to say, one question to make them feel remembered

#### 4. STRENGTH DECAY — relationship health signal
- Formula: `strength × 0.95^(days_since_contact / 7)`
- Labels: ACTIVE (≥0.70) | WARM (0.30–0.70) | FADING (<0.30)
- FADING triggers proactive re-engagement suggestion

#### 5. GRAPH TRAVERSAL — second-degree network queries
- "Who does Adaeze know I haven't met?"
- "Who introduced the most people in my network?"
- "Who do I know in healthcare?"

---

### Integration Point

This is NOT a separate app. It is a feature of the existing Hermes agent.
- Integration method: MCP server + SKILL.md
- Storage: SQLite at `~/.hermes/rolodex.db` (zero-config, self-creating)
- Gateway: Hermes existing WhatsApp/Telegram/Slack surfaces

---

### Schema Overview

Six tables:
- `people` — core node (name, role, company, location, strength, tags)
- `people_fts` — FTS5 virtual table for full-text search
- `connections` — edges between people (type: introduced_by | met_at | works_with | friend)
- `memory_items` — timestamped memories per person
- `person_events` — birthdays, meetings, reminders
- `unknown_queue` — unresolved person descriptions for later resolution

---

### MCP Tools (11 total)

| Tool | Purpose |
|------|--------|
| `rolodex_add_person` | Add a new person to the graph |
| `rolodex_fuzzy_recall` | Natural language → person match |
| `rolodex_get_person` | Full person profile with connections + memories |
| `rolodex_add_memory` | Log interaction, boost strength |
| `rolodex_set_reminder` | Schedule birthday/meeting/reminder |
| `rolodex_meeting_brief` | Pre-meeting context surface |
| `rolodex_draft_outreach` | Context for re-engagement drafting |
| `rolodex_fading_check` | Who needs reconnecting |
| `rolodex_upcoming_events` | What's coming up in next N days |
| `rolodex_graph_query` | Second-degree network queries |
| `rolodex_queue_unknown` | Queue unresolved person description |

---

### SYNTHIA™ 3.0 Required Score Before Merge

| Axis | Floor |
|------|-------|
| STK (stocks/durability) | ≥8 |
| FLW (flows/inflows/outflows) | ≥8 |
| FBK (feedback loops) | ≥8 |
| DLY (delays) | ≥7 |
| LVR (leverage) | ≥8 |
| RSL (resilience) | ≥8 |
| VIS (visibility) | ≥8 |
| AGT (agent boundary) | ≥9 |
| BLR (blast radius) | ≥8 |
| LRN (learning loop) | ≥8 |
| SEC (security) | 10 |
| DOC (documentation) | ≥9 |
| **OVERALL** | **≥8.5** |

If score < 8.5: AGENT-3 files issues → AGENT-2 fixes → AGENT-3 re-scores.
Loop maximum: 3 iterations. Still failing → escalate to human.

---

### Governed By

Emerald Tablets™ | Kupuri Media™ × Pauli Second Brain™
"One agent. Every person. Never forgotten."
