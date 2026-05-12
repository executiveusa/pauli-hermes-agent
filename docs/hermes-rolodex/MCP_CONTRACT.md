# Hermes Rolodex™ — MCP Tool Contract
# AGENT-2 must implement exactly these 11 tools.
# AGENT-3 tests each tool against these contracts.

---

## Tool 1: rolodex_add_person
**Input required:** name (string)
**Input optional:** role, company, email, phone, location, birthday (YYYY-MM-DD),
                    photo_url, notes, context_tags (string[])
**Output required:** `{ id, name, created: true, message }`
**Side effects:** inserts people row, FTS row, birthday event if birthday provided
**Error:** `{ error: "name is required" }` if name missing

---

## Tool 2: rolodex_fuzzy_recall
**Input required:** query (string — natural language description)
**Output required:** `{ results: RecallResult[], query }`

RecallResult fields:
- id, name, role, company, location, strength_label
- confidence (HIGH|MEDIUM|LOW)
- match_layer (fts|context|graph_traversal)
- context_tags, recent_memories, notes_preview

**Behavior:**
- Layer 1: FTS5 search on people_fts
- Layer 2: LIKE search on context_tags, notes, name, role, company, location
- Layer 3: 2-hop graph traversal for relational queries
- 2+ unique word hits upgrade confidence to HIGH
- Returns max 4 results, sorted by confidence desc
- Returns empty results with suggestion field if no match

---

## Tool 3: rolodex_get_person
**Input:** id (string) OR name (string) — one required
**Output:** full person object including connections[], memory_items[], events[]
**Side effect:** applies strength decay before returning (updates DB if delta > 0.01)

---

## Tool 4: rolodex_add_memory
**Input required:** person_id (string), text (string)
**Input optional:** source (MANUAL|HERMES|VOICE|PHOTO|CALENDAR), context (string)
**Output:** `{ id, person_name, text, message }`
**Side effects:**
- inserts memory_items row
- sets last_contact_at = now()
- strength = MIN(1.0, strength + 0.1)
- recalculates strength_label

---

## Tool 5: rolodex_set_reminder
**Input required:** person_id, title, date (YYYY-MM-DD)
**Input optional:** type (BIRTHDAY|MEETING|REMINDER|AMBIENT, default REMINDER)
**Output:** `{ id, person_name, title, date, type, message }`

---

## Tool 6: rolodex_meeting_brief
**Input:** person_id OR name
**Output:**
```
{
  name, role_company, location, strength_label, last_contact,
  how_connected, recent_context[], open_threads, upcoming_events[],
  connection_network[], narrative (formatted markdown string)
}
```

---

## Tool 7: rolodex_draft_outreach
**Input:** person_id OR name; optional tone (warm|formal|casual, default warm)
**Output:** `{ person_name, first_name, last_topic, days_since_contact, strength_label, instructions }`
**Note:** Returns context for Hermes to draft — does NOT draft itself.
Hermes uses the `instructions` field to compose the actual message.

---

## Tool 8: rolodex_fading_check
**Input optional:** days (int, default 30)
**Output:** `{ fading_count, people: FadingPerson[], message }`

FadingPerson: all person fields + last_topic (from most recent memory_item)

**Behavior:** returns strength_label = FADING OR last_contact_at < (now - days)

---

## Tool 9: rolodex_upcoming_events
**Input optional:** days (int, default 30)
**Output:** `{ upcoming_count, window_days, events: EventWithPerson[] }`

EventWithPerson: all event fields + person_name, role, days_until

---

## Tool 10: rolodex_graph_query
**Input required:** query (string)
**Output depends on detected pattern:**
- Pattern "who does X know" → `{ query_type: connections_of, seed_person, connections[], count }`
- Pattern "introduc*" → `{ query_type: top_introducers, results[] }`
- Pattern "who do I know" → `{ query_type: filter_by_context, qualifier, results[], count }`
- Fallback → `{ query_type: unsupported, message }`

---

## Tool 11: rolodex_queue_unknown
**Input required:** description (string)
**Input optional:** session_id (string)
**Output:** `{ id, description, message }`
**Side effect:** inserts unknown_queue row with resolved=0

---

## Implementation Requirements

- Python 3.11+
- Only external dependency: `mcp` (already in pyproject.toml dev deps)
- No secrets hardcoded — reads from env only
- `PRAGMA foreign_keys=ON` on every connection
- All SQL uses parameterized queries — no f-string SQL
- `_init_db()` creates all 6 tables on import
- Each handler: `def _handle_TOOLNAME(conn, args) -> dict`
- `create_server()` returns object with `async def list_tools()` method
- `call_tool` catches all exceptions, returns `{ error: str(e) }` not raises
- Exactly 11 tools registered — no more, no less
