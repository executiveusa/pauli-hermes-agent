"""
Hermes Rolodex™ — A2A Test Suite
AGENT-3 Guardian executes this file. 100% pass required before merge.

Run: pytest tests/hermes-rolodex/ -v --tb=short
"""

import asyncio
import json
import os
import re
import sqlite3
import sys
import uuid
from datetime import datetime, timedelta
from pathlib import Path

import pytest
import yaml

sys.path.insert(0, str(Path(__file__).parent.parent.parent /
                        "mcp-servers" / "hermes-rolodex"))

from server import (  # noqa: E402
    _apply_decay,
    _dispatch_tool,
    _handle_add_memory,
    _handle_add_person,
    _handle_fading_check,
    _handle_fuzzy_recall,
    _handle_get_person,
    _handle_graph_query,
    _handle_meeting_brief,
    _handle_queue_unknown,
    _handle_set_reminder,
    _handle_upcoming_events,
    _init_db,
    _strength_label,
    create_server,
)


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------

@pytest.fixture
def conn(tmp_path):
    db_path = str(tmp_path / "test_rolodex.db")
    os.environ["ROLODEX_DB_PATH"] = db_path
    c = _init_db()
    yield c
    c.close()
    os.environ.pop("ROLODEX_DB_PATH", None)


@pytest.fixture
def seeded_conn(conn):
    """Four canonical test people + Adaeze→Marcus connection."""
    people = [
        {"name": "Marcus Chen",
         "context_tags": ["bonsai trees", "Boston", "Stanford", "warm intro only"],
         "location": "Boston, MA"},
        {"name": "Priya Sharma",
         "context_tags": ["red boots", "Austin", "no small talk", "Arkon Labs"],
         "company": "Arkon Labs", "location": "Austin, TX"},
        {"name": "Adaeze Okonkwo",
         "context_tags": ["Sequoia", "board", "Harvard daughter", "SF Summit"],
         "company": "Sequoia Capital"},
        {"name": "Tobias Reinholt",
         "context_tags": ["Berlin", "Figma Config", "green trench coat", "Braun watch"],
         "location": "Berlin, Germany"},
    ]
    ids: dict[str, str] = {}
    for p in people:
        r = _handle_add_person(conn, p)
        assert "id" in r, f"add_person failed: {r}"
        ids[p["name"]] = r["id"]

    conn.execute(
        "INSERT INTO connections (id, person_a_id, person_b_id, connection_type)"
        " VALUES (?,?,?,'met_at')",
        [str(uuid.uuid4()), ids["Adaeze Okonkwo"], ids["Marcus Chen"]],
    )
    conn.commit()
    return conn, ids


# ---------------------------------------------------------------------------
# SUITE 1 — Schema Integrity
# ---------------------------------------------------------------------------

def test_t001_db_initializes(tmp_path):
    os.environ["ROLODEX_DB_PATH"] = str(tmp_path / "init_test.db")
    c = _init_db()
    assert c is not None
    c.close()


def test_t002_all_tables_exist(conn):
    tables = {r[0] for r in conn.execute(
        "SELECT name FROM sqlite_master WHERE type='table'"
    ).fetchall()}
    for expected in ("people", "people_fts", "connections",
                     "memory_items", "person_events", "unknown_queue"):
        assert expected in tables, f"Missing table: {expected}"


def test_t003_fts5_available(conn):
    rows = conn.execute(
        "SELECT * FROM people_fts WHERE people_fts MATCH 'test'"
    ).fetchall()
    assert isinstance(rows, list)


def test_t004_foreign_keys_enforced(conn):
    with pytest.raises((sqlite3.IntegrityError, sqlite3.OperationalError)):
        conn.execute(
            "INSERT INTO memory_items (id, person_id, text) VALUES (?,?,?)",
            [str(uuid.uuid4()), "nonexistent-uuid", "fk test"],
        )
        conn.commit()


# ---------------------------------------------------------------------------
# SUITE 2 — rolodex_add_person
# ---------------------------------------------------------------------------

def test_t010_add_name_only(conn):
    r = _handle_add_person(conn, {"name": "Alice"})
    assert r.get("created") is True
    assert "id" in r
    assert r["name"] == "Alice"


def test_t011_add_all_fields(conn):
    payload = {
        "name": "Bob Barker", "role": "CEO", "company": "Acme",
        "email": "bob@acme.com", "phone": "555-1234",
        "location": "NYC", "notes": "Met at demo day",
    }
    r = _handle_add_person(conn, payload)
    assert r.get("created") is True
    row = conn.execute("SELECT * FROM people WHERE id=?", [r["id"]]).fetchone()
    assert row["company"] == "Acme"
    assert row["email"] == "bob@acme.com"
    assert row["notes"] == "Met at demo day"


def test_t012_add_with_context_tags(conn):
    r = _handle_add_person(conn, {"name": "Tagger",
                                   "context_tags": ["AI", "Stanford", "VC"]})
    row = conn.execute("SELECT context_tags FROM people WHERE id=?",
                       [r["id"]]).fetchone()
    tags = json.loads(row["context_tags"])
    assert "AI" in tags
    assert "Stanford" in tags


def test_t013_birthday_creates_event(conn):
    r = _handle_add_person(conn, {"name": "Birthday Bob",
                                   "birthday": "1990-03-15"})
    ev = conn.execute(
        "SELECT * FROM person_events WHERE person_id=? AND type='BIRTHDAY'",
        [r["id"]],
    ).fetchone()
    assert ev is not None
    assert ev["date"] == "1990-03-15"


def test_t014_missing_name_returns_error(conn):
    r = _handle_add_person(conn, {})
    assert "error" in r
    assert "name" in r["error"].lower()


def test_t015_add_person_fts_row_created(conn):
    r = _handle_add_person(conn, {"name": "FTSUser",
                                   "context_tags": ["uniquetoken123"]})
    fts_rows = conn.execute(
        "SELECT id FROM people_fts WHERE people_fts MATCH 'uniquetoken123'"
    ).fetchall()
    assert any(row["id"] == r["id"] for row in fts_rows)


# ---------------------------------------------------------------------------
# SUITE 3 — rolodex_fuzzy_recall
# ---------------------------------------------------------------------------

def test_t020_bonsai_trees(seeded_conn):
    conn, ids = seeded_conn
    r = _handle_fuzzy_recall(conn, {"query": "bonsai trees"})
    assert r["results"], "Should return at least one result"
    assert r["results"][0]["name"] == "Marcus Chen"
    assert r["results"][0]["confidence"] == "HIGH"


def test_t021_red_boots_austin(seeded_conn):
    conn, ids = seeded_conn
    r = _handle_fuzzy_recall(conn, {"query": "red boots Austin"})
    names = [x["name"] for x in r["results"]]
    assert "Priya Sharma" in names
    priya = next(x for x in r["results"] if x["name"] == "Priya Sharma")
    assert priya["confidence"] == "HIGH"


def test_t022_green_trench_coat_berlin(seeded_conn):
    conn, ids = seeded_conn
    r = _handle_fuzzy_recall(conn, {"query": "green trench coat Berlin"})
    names = [x["name"] for x in r["results"]]
    assert "Tobias Reinholt" in names


def test_t023_harvard_daughter(seeded_conn):
    conn, ids = seeded_conn
    r = _handle_fuzzy_recall(conn, {"query": "Harvard daughter"})
    names = [x["name"] for x in r["results"]]
    assert "Adaeze Okonkwo" in names


def test_t024_sequoia_partner_board(seeded_conn):
    conn, ids = seeded_conn
    r = _handle_fuzzy_recall(conn, {"query": "Sequoia partner board"})
    names = [x["name"] for x in r["results"]]
    assert "Adaeze Okonkwo" in names
    adaeze = next(x for x in r["results"] if x["name"] == "Adaeze Okonkwo")
    assert adaeze["confidence"] == "HIGH"


def test_t025_no_match_returns_empty_with_suggestion(conn):
    r = _handle_fuzzy_recall(conn, {"query": "xyzzy_nonexistent_person_99"})
    assert r["results"] == []
    assert "suggestion" in r


def test_t026_three_word_query_confidence_high(seeded_conn):
    conn, ids = seeded_conn
    r = _handle_fuzzy_recall(conn, {"query": "Boston bonsai Stanford"})
    assert r["results"], "Should return Marcus"
    marcus = next((x for x in r["results"] if x["name"] == "Marcus Chen"), None)
    assert marcus is not None
    assert marcus["confidence"] == "HIGH"


def test_t027_graph_traversal_adaeze_knows_marcus(seeded_conn):
    conn, ids = seeded_conn
    r = _handle_fuzzy_recall(conn, {"query": "who does Adaeze know"})
    names = [x["name"] for x in r["results"]]
    assert "Marcus Chen" in names, f"Expected Marcus in graph results, got: {names}"


# ---------------------------------------------------------------------------
# SUITE 4 — rolodex_add_memory
# ---------------------------------------------------------------------------

def test_t030_add_memory_creates_row(conn):
    p = _handle_add_person(conn, {"name": "Memo Person"})
    r = _handle_add_memory(conn, {"person_id": p["id"], "text": "Met for coffee"})
    assert "id" in r
    row = conn.execute("SELECT * FROM memory_items WHERE id=?",
                       [r["id"]]).fetchone()
    assert row is not None
    assert row["text"] == "Met for coffee"


def test_t031_add_memory_updates_last_contact(conn):
    p = _handle_add_person(conn, {"name": "Contactee"})
    _handle_add_memory(conn, {"person_id": p["id"], "text": "Had a call"})
    row = conn.execute("SELECT last_contact_at FROM people WHERE id=?",
                       [p["id"]]).fetchone()
    assert row["last_contact_at"] is not None


def test_t032_add_memory_boosts_strength(conn):
    p = _handle_add_person(conn, {"name": "Strengthener"})
    conn.execute("UPDATE people SET strength=0.5 WHERE id=?", [p["id"]])
    conn.commit()
    _handle_add_memory(conn, {"person_id": p["id"], "text": "Lunch meeting"})
    row = conn.execute("SELECT strength FROM people WHERE id=?",
                       [p["id"]]).fetchone()
    assert abs(row["strength"] - 0.6) < 0.001


def test_t033_add_memory_upgrades_label_to_active(conn):
    p = _handle_add_person(conn, {"name": "Almost Active"})
    conn.execute("UPDATE people SET strength=0.65, strength_label='WARM'"
                 " WHERE id=?", [p["id"]])
    conn.commit()
    _handle_add_memory(conn, {"person_id": p["id"], "text": "Pushed to active"})
    row = conn.execute("SELECT strength, strength_label FROM people WHERE id=?",
                       [p["id"]]).fetchone()
    assert row["strength"] >= 0.70
    assert row["strength_label"] == "ACTIVE"


def test_t034_add_memory_nonexistent_person(conn):
    r = _handle_add_memory(conn, {"person_id": "no-such-id", "text": "Ghost"})
    assert "error" in r
    assert "not found" in r["error"].lower()


# ---------------------------------------------------------------------------
# SUITE 5 — rolodex_get_person
# ---------------------------------------------------------------------------

def test_t040_get_by_id_returns_full_profile(conn):
    p = _handle_add_person(conn, {"name": "Full Profile"})
    _handle_add_memory(conn, {"person_id": p["id"], "text": "Test mem"})
    r = _handle_get_person(conn, {"id": p["id"]})
    assert r.get("name") == "Full Profile"
    assert "connections" in r
    assert "memory_items" in r
    assert "events" in r


def test_t041_get_by_name_partial_match(conn):
    _handle_add_person(conn, {"name": "Alexandrina Petrova"})
    r = _handle_get_person(conn, {"name": "Alexandrina"})
    assert r.get("name") == "Alexandrina Petrova"


def test_t042_get_decays_old_contact(conn):
    p = _handle_add_person(conn, {"name": "Old Contact"})
    old = (datetime.now() - timedelta(days=60)).isoformat()
    conn.execute("UPDATE people SET last_contact_at=?, strength=0.7 WHERE id=?",
                 [old, p["id"]])
    conn.commit()
    r = _handle_get_person(conn, {"id": p["id"]})
    assert r["strength"] < 0.65


def test_t043_decay_updates_db(conn):
    p = _handle_add_person(conn, {"name": "DB Decay"})
    old = (datetime.now() - timedelta(days=60)).isoformat()
    conn.execute("UPDATE people SET last_contact_at=?, strength=0.7 WHERE id=?",
                 [old, p["id"]])
    conn.commit()
    _handle_get_person(conn, {"id": p["id"]})
    row = conn.execute("SELECT strength FROM people WHERE id=?",
                       [p["id"]]).fetchone()
    assert row["strength"] < 0.65


def test_t044_get_nonexistent_person(conn):
    r = _handle_get_person(conn, {"id": "ghost-id-9999"})
    assert "error" in r


# ---------------------------------------------------------------------------
# SUITE 6 — Strength Decay Engine
# ---------------------------------------------------------------------------

def _person_with_contact(conn, name, days_ago, initial_strength=0.7):
    p = _handle_add_person(conn, {"name": name})
    last = (datetime.now() - timedelta(days=days_ago)).isoformat()
    conn.execute(
        "UPDATE people SET last_contact_at=?, strength=?, strength_label=?"
        " WHERE id=?",
        [last, initial_strength, _strength_label(initial_strength), p["id"]],
    )
    conn.commit()
    return p["id"]


def test_t050_decay_7_days(conn):
    pid = _person_with_contact(conn, "Decay7", 7, 0.7)
    r = _handle_get_person(conn, {"id": pid})
    expected = 0.7 * (0.95 ** 1)
    assert abs(r["strength"] - expected) < 0.01


def test_t051_decay_35_days(conn):
    pid = _person_with_contact(conn, "Decay35", 35, 0.7)
    r = _handle_get_person(conn, {"id": pid})
    expected = 0.7 * (0.95 ** 5)
    assert abs(r["strength"] - expected) < 0.01


def test_t052_fading_label_after_60_days(conn):
    pid = _person_with_contact(conn, "Fading60", 60, 0.35)
    r = _handle_get_person(conn, {"id": pid})
    assert r["strength_label"] == "FADING"


def test_t053_no_last_contact_no_decay(conn):
    p = _handle_add_person(conn, {"name": "No Contact"})
    conn.execute("UPDATE people SET strength=0.7, last_contact_at=NULL WHERE id=?",
                 [p["id"]])
    conn.commit()
    r = _handle_get_person(conn, {"id": p["id"]})
    assert abs(r.get("strength", 0.7) - 0.7) < 0.001


def test_t054_decay_floor_is_001(conn):
    pid = _person_with_contact(conn, "Floor", 3650, 0.01)
    r = _handle_get_person(conn, {"id": pid})
    assert r["strength"] >= 0.01


# ---------------------------------------------------------------------------
# SUITE 7 — rolodex_meeting_brief
# ---------------------------------------------------------------------------

def test_t060_brief_has_narrative(conn):
    p = _handle_add_person(conn, {"name": "Brief Person", "role": "CTO"})
    r = _handle_meeting_brief(conn, {"id": p["id"]})
    assert r.get("narrative"), "narrative should be non-empty"
    assert isinstance(r["narrative"], str)
    assert len(r["narrative"]) > 10


def test_t061_brief_includes_recent_context(conn):
    p = _handle_add_person(conn, {"name": "Context Person"})
    for i in range(3):
        _handle_add_memory(conn, {"person_id": p["id"],
                                   "text": f"Memory {i}"})
    r = _handle_meeting_brief(conn, {"id": p["id"]})
    assert len(r["recent_context"]) >= 1
    assert any("Memory" in m for m in r["recent_context"])


def test_t062_brief_includes_connection_network(seeded_conn):
    conn, ids = seeded_conn
    r = _handle_meeting_brief(conn, {"id": ids["Adaeze Okonkwo"]})
    assert len(r["connection_network"]) >= 1


def test_t063_brief_no_memories_still_valid(conn):
    p = _handle_add_person(conn, {"name": "Quiet Person"})
    r = _handle_meeting_brief(conn, {"id": p["id"]})
    assert "name" in r
    assert "narrative" in r
    assert r["recent_context"] == []


def test_t064_brief_unknown_person(conn):
    r = _handle_meeting_brief(conn, {"id": "unknown-id"})
    assert "error" in r


# ---------------------------------------------------------------------------
# SUITE 8 — rolodex_upcoming_events
# ---------------------------------------------------------------------------

def test_t070_no_events(conn):
    r = _handle_upcoming_events(conn, {})
    assert r["upcoming_count"] == 0
    assert r["events"] == []


def test_t071_birthday_7_days_out(conn):
    p = _handle_add_person(conn, {"name": "Birthday Guy"})
    future = (datetime.now().date() + timedelta(days=7)).isoformat()
    conn.execute(
        "INSERT INTO person_events (id, person_id, type, title, date)"
        " VALUES (?,?,'BIRTHDAY','His Birthday',?)",
        [str(uuid.uuid4()), p["id"], future],
    )
    conn.commit()
    r = _handle_upcoming_events(conn, {})
    assert r["upcoming_count"] >= 1
    event = next((e for e in r["events"] if e["person_name"] == "Birthday Guy"), None)
    assert event is not None
    assert event["days_until"] <= 8


def test_t072_event_31_days_out_not_in_default_window(conn):
    p = _handle_add_person(conn, {"name": "Far Future"})
    future = (datetime.now().date() + timedelta(days=31)).isoformat()
    conn.execute(
        "INSERT INTO person_events (id, person_id, type, title, date)"
        " VALUES (?,?,'REMINDER','Far Out',?)",
        [str(uuid.uuid4()), p["id"], future],
    )
    conn.commit()
    r = _handle_upcoming_events(conn, {})
    names = [e.get("person_name") for e in r["events"]]
    assert "Far Future" not in names


def test_t073_fired_event_excluded(conn):
    p = _handle_add_person(conn, {"name": "Fired Event Person"})
    soon = (datetime.now().date() + timedelta(days=3)).isoformat()
    conn.execute(
        "INSERT INTO person_events (id, person_id, type, title, date, fired)"
        " VALUES (?,?,'REMINDER','Already Fired',?,1)",
        [str(uuid.uuid4()), p["id"], soon],
    )
    conn.commit()
    r = _handle_upcoming_events(conn, {})
    names = [e.get("person_name") for e in r["events"]]
    assert "Fired Event Person" not in names


def test_t074_multiple_events_sorted_asc(conn):
    p1 = _handle_add_person(conn, {"name": "Zara"})
    p2 = _handle_add_person(conn, {"name": "Aaron"})
    d1 = (datetime.now().date() + timedelta(days=10)).isoformat()
    d2 = (datetime.now().date() + timedelta(days=5)).isoformat()
    conn.executemany(
        "INSERT INTO person_events (id, person_id, type, title, date)"
        " VALUES (?,?,'REMINDER','Test',?)",
        [(str(uuid.uuid4()), p1["id"], d1),
         (str(uuid.uuid4()), p2["id"], d2)],
    )
    conn.commit()
    r = _handle_upcoming_events(conn, {})
    assert r["upcoming_count"] >= 2
    dates = [e["date"] for e in r["events"]]
    assert dates == sorted(dates)


# ---------------------------------------------------------------------------
# SUITE 9 — rolodex_fading_check
# ---------------------------------------------------------------------------

def test_t080_fading_label_person_appears(conn):
    p = _handle_add_person(conn, {"name": "Fader"})
    conn.execute("UPDATE people SET strength=0.1, strength_label='FADING'"
                 " WHERE id=?", [p["id"]])
    conn.commit()
    r = _handle_fading_check(conn, {})
    names = [x["name"] for x in r["people"]]
    assert "Fader" in names


def test_t081_old_contact_appears(conn):
    p = _handle_add_person(conn, {"name": "Old Friend"})
    old = (datetime.now() - timedelta(days=45)).isoformat()
    conn.execute("UPDATE people SET last_contact_at=? WHERE id=?",
                 [old, p["id"]])
    conn.commit()
    r = _handle_fading_check(conn, {})
    names = [x["name"] for x in r["people"]]
    assert "Old Friend" in names


def test_t082_recent_contact_not_in_results(conn):
    p = _handle_add_person(conn, {"name": "Active Friend"})
    recent = (datetime.now() - timedelta(days=10)).isoformat()
    conn.execute(
        "UPDATE people SET last_contact_at=?, strength=0.8,"
        " strength_label='ACTIVE' WHERE id=?",
        [recent, p["id"]],
    )
    conn.commit()
    r = _handle_fading_check(conn, {})
    names = [x["name"] for x in r["people"]]
    assert "Active Friend" not in names


def test_t083_includes_last_topic(conn):
    p = _handle_add_person(conn, {"name": "Topic Fader"})
    conn.execute("UPDATE people SET strength=0.1, strength_label='FADING'"
                 " WHERE id=?", [p["id"]])
    conn.execute(
        "INSERT INTO memory_items (id, person_id, text, timestamp)"
        " VALUES (?,?,?,?)",
        [str(uuid.uuid4()), p["id"], "Last topic was bonsai",
         datetime.now().isoformat()],
    )
    conn.commit()
    r = _handle_fading_check(conn, {})
    fader = next((x for x in r["people"] if x["name"] == "Topic Fader"), None)
    assert fader is not None
    assert fader.get("last_topic") == "Last topic was bonsai"


# ---------------------------------------------------------------------------
# SUITE 10 — rolodex_graph_query
# ---------------------------------------------------------------------------

def test_t090_who_does_marcus_know(seeded_conn):
    conn, ids = seeded_conn
    r = _handle_graph_query(conn, {"query": "who does Marcus know"})
    assert r["query_type"] == "connections_of"
    assert r["seed_person"] == "Marcus Chen"
    names = [c["name"] for c in r["connections"]]
    assert "Adaeze Okonkwo" in names


def test_t091_top_introducers(conn):
    r = _handle_graph_query(conn, {"query": "who introduced the most people"})
    assert r["query_type"] == "top_introducers"
    assert "results" in r


def test_t092_who_do_i_know_in_boston(seeded_conn):
    conn, ids = seeded_conn
    r = _handle_graph_query(conn, {"query": "who do I know in Boston"})
    assert r["query_type"] == "filter_by_context"
    assert r["qualifier"] == "boston"
    names = [x["name"] for x in r["results"]]
    assert "Marcus Chen" in names


def test_t093_who_do_i_know_in_healthcare(conn):
    p = _handle_add_person(conn, {"name": "Dr. Health", "role": "healthcare consultant"})
    r = _handle_graph_query(conn, {"query": "who do I know in healthcare"})
    assert r["query_type"] == "filter_by_context"


def test_t094_unsupported_pattern(conn):
    r = _handle_graph_query(conn, {"query": "what is the meaning of life"})
    assert r["query_type"] == "unsupported"
    assert "message" in r


# ---------------------------------------------------------------------------
# SUITE 11 — MCP Integration
# ---------------------------------------------------------------------------

def test_t100_server_imports():
    import importlib
    import server as srv
    assert srv is not None
    assert callable(srv.create_server)


@pytest.mark.asyncio
async def test_t101_server_exposes_11_tools():
    s = create_server()
    tools = await s.list_tools()
    assert len(tools) == 11, f"Expected 11 tools, got {len(tools)}"


@pytest.mark.asyncio
async def test_t102_tool_names_match_dispatch():
    from server import _HANDLERS
    s = create_server()
    tools = await s.list_tools()
    listed_names = {t.name for t in tools}
    handler_names = set(_HANDLERS.keys())
    assert listed_names == handler_names


def test_t103_unknown_tool_returns_error(conn):
    r = _dispatch_tool(conn, "rolodex_does_not_exist", {})
    assert "error" in r
    assert "Unknown tool" in r["error"]


def test_t104_missing_required_field_returns_error(conn):
    r = _dispatch_tool(conn, "rolodex_add_person", {})
    assert "error" in r


# ---------------------------------------------------------------------------
# SUITE 12 — SKILL.md Validation
# ---------------------------------------------------------------------------

SKILL_PATH = (Path(__file__).parent.parent.parent /
              "skills" / "hermes-rolodex" / "SKILL.md")


def _load_skill():
    return SKILL_PATH.read_text(encoding="utf-8")


def test_t110_skill_has_valid_frontmatter():
    content = _load_skill()
    fm_match = re.match(r'^---\n(.*?)\n---', content, re.DOTALL)
    assert fm_match, "No YAML frontmatter found in SKILL.md"
    fm = yaml.safe_load(fm_match.group(1))
    assert fm.get("name"), "Missing 'name' in frontmatter"
    assert fm.get("description"), "Missing 'description' in frontmatter"
    assert fm.get("invocation"), "Missing 'invocation' in frontmatter"


def test_t111_skill_has_trigger_phrases():
    content = _load_skill()
    for phrase in ["remember that", "who was", "I just met",
                   "meeting brief", "birthday"]:
        assert phrase in content, f"Missing trigger phrase: '{phrase}'"


def test_t112_skill_references_all_tools():
    content = _load_skill()
    tools = [
        "rolodex_add_person", "rolodex_fuzzy_recall", "rolodex_get_person",
        "rolodex_add_memory", "rolodex_set_reminder", "rolodex_meeting_brief",
        "rolodex_draft_outreach", "rolodex_fading_check",
        "rolodex_upcoming_events", "rolodex_graph_query", "rolodex_queue_unknown",
    ]
    for tool in tools:
        assert tool in content, f"Tool not referenced in SKILL.md: {tool}"


def test_t113_skill_under_150_lines():
    content = _load_skill()
    lines = content.strip().split("\n")
    assert len(lines) <= 150, f"SKILL.md too long: {len(lines)} lines (max 150)"


# ---------------------------------------------------------------------------
# SUITE 13 — Security
# ---------------------------------------------------------------------------

SERVER_SRC = (Path(__file__).parent.parent.parent /
              "mcp-servers" / "hermes-rolodex" / "server.py").read_text()


def test_t120_no_hardcoded_secrets():
    secret_patterns = [
        r'sk-[a-zA-Z0-9]{20,}',
        r'(?i)(API_KEY|SECRET_KEY|PASSWORD)\s*=\s*["\'][^"\'){8,}["\']',
    ]
    for pat in secret_patterns:
        m = re.search(pat, SERVER_SRC)
        assert m is None, f"Potential hardcoded secret found: {m.group(0) if m else ''}"


def test_t121_db_path_uses_env_var():
    assert "ROLODEX_DB_PATH" in SERVER_SRC
    assert ".hermes/rolodex.db" in SERVER_SRC


def test_t122_no_fstring_sql():
    fstring_sql = re.findall(
        r'f["\'][^"\]*(?:SELECT|INSERT|UPDATE|DELETE|WHERE)[^"\]*["\']',
        SERVER_SRC,
        re.IGNORECASE,
    )
    assert not fstring_sql, f"F-string SQL found: {fstring_sql[:2]}"


def test_t123_fts_query_sanitized():
    from server import _fts_sanitize
    assert ";" not in _fts_sanitize("DROP TABLE; --")
    assert '"' not in _fts_sanitize('MATCH "injection"')
    assert _fts_sanitize("bonsai trees") == "bonsai trees"
