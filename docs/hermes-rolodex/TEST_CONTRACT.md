# Hermes Rolodex™ — A2A Test Contract
# AGENT-3 executes all tests in this file.
# 100% pass required before merge is permitted.
# Test runner: pytest

---

## SUITE 1 — SCHEMA INTEGRITY

T-001: DB initializes without error at configured path
T-002: All 6 tables exist after init (people, people_fts, connections,
        memory_items, person_events, unknown_queue)
T-003: FTS5 extension is available in SQLite runtime
T-004: Foreign key constraints are enforced (PRAGMA foreign_keys=ON)

## SUITE 2 — TOOL: rolodex_add_person

T-010: Add person with name only → returns id and created:true
T-011: Add person with all fields → all fields persisted in DB
T-012: Add person with context_tags → tags stored as JSON array
T-013: Add person with birthday → birthday event created in person_events
T-014: Add person without name → returns { error: "name is required" }
T-015: Add person → FTS row is created (search returns them immediately)

## SUITE 3 — TOOL: rolodex_fuzzy_recall (THE CRITICAL SUITE)

Seed: Marcus Chen (context_tags: ["bonsai trees", "Boston", "Stanford"]),
       Priya Sharma (context_tags: ["red boots", "Austin", "no small talk"]),
       Adaeze Okonkwo (context_tags: ["Sequoia", "board", "Harvard daughter"]),
       Tobias Reinholt (context_tags: ["Berlin", "Figma Config", "green trench coat"])

T-020: query="bonsai trees" → Marcus Chen in results[0], confidence=HIGH
T-021: query="red boots Austin" → Priya Sharma in results, confidence=HIGH
T-022: query="green trench coat Berlin" → Tobias Reinholt in results
T-023: query="Harvard daughter" → Adaeze Okonkwo in results
T-024: query="Sequoia partner board" → Adaeze Okonkwo in results, confidence=HIGH
T-025: query with no match → returns { results: [] } with suggestion field
T-026: query="Boston bonsai Stanford" (3 words) → Marcus Chen, confidence=HIGH
T-027: Graph traversal: add connection Adaeze→Marcus, query "who does Adaeze know"
        → Marcus appears in graph traversal results

## SUITE 4 — TOOL: rolodex_add_memory

T-030: Add memory to existing person → memory_items row created
T-031: Add memory → last_contact_at updated to now
T-032: Add memory → strength += 0.1 (capped at 1.0)
T-033: Add memory to strength=0.65 person → strength_label becomes ACTIVE (≥0.70)
T-034: Add memory to non-existent person_id → returns { error: "Person not found" }

## SUITE 5 — TOOL: rolodex_get_person

T-040: Get by ID → returns full person with connections, memory_items, events
T-041: Get by name (partial match) → returns correct person
T-042: Get person with old last_contact (60 days ago) → strength decayed in response
T-043: Decay > 0.01 → DB is updated with new strength and strength_label
T-044: Get non-existent person → returns { error: "Person not found" }

## SUITE 6 — STRENGTH DECAY ENGINE

T-050: Person with last_contact 7 days ago → strength × 0.95^1 ≈ 0.95x
T-051: Person with last_contact 35 days ago → strength × 0.95^5 ≈ 0.774x
T-052: Person with strength 0.35, last_contact 60 days ago → label becomes FADING
T-053: Person with no last_contact → strength unchanged
T-054: Decayed strength never goes below 0.01

## SUITE 7 — TOOL: rolodex_meeting_brief

T-060: Brief for existing person → narrative field is non-empty string
T-061: Brief includes recent_context from last 3 memory_items
T-062: Brief includes connection_network if connections exist
T-063: Brief for person with no memory items → still returns valid brief structure
T-064: Brief for unknown person → returns { error: "Person not found" }

## SUITE 8 — TOOL: rolodex_upcoming_events

T-070: No events → returns { upcoming_count: 0, events: [] }
T-071: Birthday 7 days from now → appears in results with days_until <= 8
T-072: Event 31 days from now (default window=30) → NOT in results
T-073: Already fired event → NOT in results (fired=1 excluded)
T-074: Multiple people with events → all appear, sorted by date ASC

## SUITE 9 — TOOL: rolodex_fading_check

T-080: Person with strength_label=FADING → appears in results
T-081: Person with last_contact 45 days ago (default threshold=30) → appears
T-082: Person with last_contact 10 days ago and WARM label → NOT in results
T-083: Result includes last_topic from most recent memory_item

## SUITE 10 — TOOL: rolodex_graph_query

T-090: "who does Marcus know" (has connections) → returns connections_of type
T-091: "who introduced the most people" → returns top_introducers type
T-092: "who do I know in Boston" → returns filter_by_context with Boston qualifier
T-093: "who do I know in healthcare" → returns filter_by_context type
T-094: Unknown pattern → returns { query_type: unsupported, message }

## SUITE 11 — MCP INTEGRATION

T-100: server.py starts without import errors
T-101: Server exposes exactly 11 tools via list_tools()
T-102: Each tool name in list matches tool name in _dispatch_tool handler
T-103: call via _dispatch_tool with unknown tool name → returns { error: "Unknown tool: X" }
T-104: call via _dispatch_tool with missing required field → returns { error: ... } not exception

## SUITE 12 — SKILL.MD VALIDATION

T-110: SKILL.md has valid YAML frontmatter (name, description, invocation fields)
T-111: SKILL.md description contains at least 5 trigger phrases
T-112: SKILL.md references all 11 MCP tool names
T-113: SKILL.md is under 150 lines (progressive disclosure — keep it scannable)

## SUITE 13 — SECURITY (EMERALD TABLETS™ MANDATORY)

T-120: No API keys or secrets hardcoded in server.py
T-121: DB_PATH uses ROLODEX_DB_PATH env var OR defaults to ~/.hermes/rolodex.db
T-122: SQL queries use parameterized statements (no f-string SQL)
T-123: FTS query sanitized before use (special chars removed)

---

## PASS CRITERIA

All 50+ tests pass: AGENT-3 signs off → merge permitted
Any failure: AGENT-3 files specific issue → AGENT-2 fixes → AGENT-3 re-runs
Max 3 iterations. Still failing after 3: escalate.
