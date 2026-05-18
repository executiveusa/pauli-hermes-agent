#!/usr/bin/env python3
"""
Hermes Rolodex™ — MCP Server
SQLite FTS5 relationship graph for the Pauli Second Brain™

Storage : ~/.hermes/rolodex.db  (override: ROLODEX_DB_PATH env var)
Depends : mcp>=1.2.0  (stdlib only otherwise)
Tools   : 11 — add_person, fuzzy_recall, get_person, add_memory,
               set_reminder, meeting_brief, draft_outreach,
               fading_check, upcoming_events, graph_query, queue_unknown

Governed by: Emerald Tablets™ | Kupuri Media™ × Pauli Second Brain™
"""

import json
import os
import re
import sqlite3
import uuid
from datetime import datetime, timedelta
from pathlib import Path

from mcp.server import Server
from mcp.server.stdio import stdio_server
import mcp.types as types


# ---------------------------------------------------------------------------
# DB helpers
# ---------------------------------------------------------------------------

def _get_db_path() -> str:
    return os.environ.get("ROLODEX_DB_PATH",
                          str(Path.home() / ".hermes" / "rolodex.db"))


def _get_connection() -> sqlite3.Connection:
    path = _get_db_path()
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(path)
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA foreign_keys=ON")
    conn.execute("PRAGMA journal_mode=WAL")
    return conn


def _init_db() -> sqlite3.Connection:
    """Create all tables and indexes; return an open connection."""
    conn = _get_connection()
    conn.executescript("""
        CREATE TABLE IF NOT EXISTS people (
            id              TEXT PRIMARY KEY,
            name            TEXT NOT NULL,
            role            TEXT,
            company         TEXT,
            email           TEXT,
            phone           TEXT,
            location        TEXT,
            birthday        TEXT,
            photo_url       TEXT,
            strength        REAL DEFAULT 0.7,
            strength_label  TEXT DEFAULT 'WARM',
            last_contact_at TEXT,
            notes           TEXT,
            context_tags    TEXT DEFAULT '[]',
            created_at      TEXT DEFAULT (datetime('now')),
            updated_at      TEXT DEFAULT (datetime('now'))
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS people_fts
            USING fts5(
                id UNINDEXED,
                name,
                role,
                company,
                notes,
                context_tags_flat
            );

        CREATE TABLE IF NOT EXISTS connections (
            id              TEXT PRIMARY KEY,
            person_a_id     TEXT NOT NULL REFERENCES people(id) ON DELETE CASCADE,
            person_b_id     TEXT NOT NULL REFERENCES people(id) ON DELETE CASCADE,
            connection_type TEXT,
            context         TEXT,
            strength        REAL DEFAULT 0.5,
            created_at      TEXT DEFAULT (datetime('now')),
            UNIQUE(person_a_id, person_b_id)
        );

        CREATE TABLE IF NOT EXISTS memory_items (
            id          TEXT PRIMARY KEY,
            person_id   TEXT NOT NULL REFERENCES people(id) ON DELETE CASCADE,
            text        TEXT NOT NULL,
            source      TEXT DEFAULT 'HERMES',
            context     TEXT,
            timestamp   TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS person_events (
            id          TEXT PRIMARY KEY,
            person_id   TEXT NOT NULL REFERENCES people(id) ON DELETE CASCADE,
            type        TEXT NOT NULL,
            title       TEXT NOT NULL,
            date        TEXT NOT NULL,
            fired       INTEGER DEFAULT 0,
            fired_at    TEXT,
            created_at  TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS unknown_queue (
            id          TEXT PRIMARY KEY,
            description TEXT NOT NULL,
            session_id  TEXT,
            created_at  TEXT DEFAULT (datetime('now')),
            resolved    INTEGER DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_memory_person  ON memory_items(person_id);
        CREATE INDEX IF NOT EXISTS idx_events_date    ON person_events(date, fired);
        CREATE INDEX IF NOT EXISTS idx_events_person  ON person_events(person_id);
        CREATE INDEX IF NOT EXISTS idx_strength_label ON people(strength_label);
        CREATE INDEX IF NOT EXISTS idx_last_contact   ON people(last_contact_at);
    """)
    conn.commit()
    return conn


# ---------------------------------------------------------------------------
# Domain helpers
# ---------------------------------------------------------------------------

def _strength_label(strength: float) -> str:
    if strength >= 0.70:
        return "ACTIVE"
    if strength >= 0.30:
        return "WARM"
    return "FADING"


def _apply_decay(conn: sqlite3.Connection, person_id: str,
                 person_dict: dict) -> dict:
    """Compute strength decay; persist to DB if delta > 0.01. Returns updated dict."""
    last_contact = person_dict.get("last_contact_at")
    if not last_contact:
        return person_dict

    try:
        last_dt = datetime.fromisoformat(last_contact)
    except (ValueError, TypeError):
        return person_dict

    days = (datetime.now() - last_dt).total_seconds() / 86400.0
    current = float(person_dict.get("strength") or 0.7)
    decayed = max(0.01, current * (0.95 ** (days / 7.0)))

    if abs(decayed - current) > 0.01:
        new_label = _strength_label(decayed)
        conn.execute(
            "UPDATE people SET strength=?, strength_label=?, updated_at=? WHERE id=?",
            [decayed, new_label, datetime.now().isoformat(), person_id],
        )
        conn.commit()
        person_dict = dict(person_dict)
        person_dict["strength"] = decayed
        person_dict["strength_label"] = new_label

    return person_dict


def _fts_sanitize(query: str) -> str:
    """Strip FTS5 special characters from a query string."""
    sanitized = re.sub(r'[^\w\s]', ' ', query)
    return ' '.join(sanitized.split())


def _get_person_by_id_or_name(conn: sqlite3.Connection, args: dict):
    """Return a sqlite3.Row or None. Accepts id, person_id, or name."""
    person_id = args.get("id") or args.get("person_id")
    name = args.get("name")
    if person_id:
        return conn.execute(
            "SELECT * FROM people WHERE id=?", [person_id]
        ).fetchone()
    if name:
        return conn.execute(
            "SELECT * FROM people WHERE lower(name) LIKE ?",
            [f"%{name.lower()}%"],
        ).fetchone()
    return None


def _parse_tags(raw) -> list:
    if not raw:
        return []
    if isinstance(raw, list):
        return raw
    try:
        return json.loads(raw)
    except Exception:
        return [str(raw)]


# ---------------------------------------------------------------------------
# Tool handlers
# ---------------------------------------------------------------------------

def _handle_add_person(conn: sqlite3.Connection, args: dict) -> dict:
    name = (args.get("name") or "").strip()
    if not name:
        return {"error": "name is required"}

    pid = str(uuid.uuid4())
    context_tags = _parse_tags(args.get("context_tags", []))
    context_tags_json = json.dumps(context_tags)
    context_tags_flat = " ".join(str(t) for t in context_tags)

    role = args.get("role") or ""
    company = args.get("company") or ""
    notes = args.get("notes") or ""

    conn.execute(
        """INSERT INTO people
               (id, name, role, company, email, phone, location,
                birthday, photo_url, notes, context_tags)
           VALUES (?,?,?,?,?,?,?,?,?,?,?)""",
        [
            pid, name, role, company,
            args.get("email"), args.get("phone"), args.get("location"),
            args.get("birthday"), args.get("photo_url"),
            notes, context_tags_json,
        ],
    )

    conn.execute(
        "INSERT INTO people_fts (id, name, role, company, notes, context_tags_flat)"
        " VALUES (?,?,?,?,?,?)",
        [pid, name, role, company, notes, context_tags_flat],
    )

    birthday = args.get("birthday")
    if birthday:
        conn.execute(
            "INSERT INTO person_events (id, person_id, type, title, date)"
            " VALUES (?,?,'BIRTHDAY',?,?)",
            [str(uuid.uuid4()), pid, f"{name}'s Birthday", birthday],
        )

    conn.commit()
    return {"id": pid, "name": name, "created": True,
            "message": f"Added {name} to Rolodex"}


def _handle_fuzzy_recall(conn: sqlite3.Connection, args: dict) -> dict:
    query = (args.get("query") or "").strip()
    if not query:
        return {"results": [], "query": query,
                "suggestion": "Provide a description to search"}

    scores: dict[str, dict] = {}

    safe_q = _fts_sanitize(query)
    if safe_q:
        try:
            rows = conn.execute(
                "SELECT id FROM people_fts WHERE people_fts MATCH ?",
                [safe_q],
            ).fetchall()
            for row in rows:
                pid = row["id"]
                scores[pid] = {"score": 3, "match_layer": "fts"}
        except Exception:
            pass

    words = list({w for w in query.split() if len(w) > 2})
    word_hits: dict[str, set] = {}
    for word in words:
        pat = "%" + word.lower() + "%"
        rows = conn.execute(
            """SELECT id FROM people WHERE
                   lower(name)         LIKE ? OR
                   lower(role)         LIKE ? OR
                   lower(company)      LIKE ? OR
                   lower(notes)        LIKE ? OR
                   lower(context_tags) LIKE ? OR
                   lower(location)     LIKE ?""",
            [pat, pat, pat, pat, pat, pat],
        ).fetchall()
        for row in rows:
            pid = row["id"]
            word_hits.setdefault(pid, set()).add(word)

    for pid, matched_words in word_hits.items():
        like_score = len(matched_words)
        if pid not in scores:
            scores[pid] = {"score": like_score, "match_layer": "context"}
        elif scores[pid].get("match_layer") != "fts":
            scores[pid]["score"] = max(scores[pid]["score"], like_score)

    q_lower = query.lower()
    m = re.search(r"who does\s+(\w+(?:\s+\w+)?)\s+know", q_lower)
    if m:
        seed_name = m.group(1).strip()
        seed = conn.execute(
            "SELECT id FROM people WHERE lower(name) LIKE ?",
            [f"%{seed_name}%"],
        ).fetchone()
        if seed:
            seed_id = seed["id"]
            graph_rows = conn.execute(
                """SELECT CASE WHEN person_a_id=? THEN person_b_id
                               ELSE person_a_id END AS other_id
                   FROM connections
                   WHERE person_a_id=? OR person_b_id=?""",
                [seed_id, seed_id, seed_id],
            ).fetchall()
            for gr in graph_rows:
                pid = gr["other_id"]
                if pid not in scores:
                    scores[pid] = {"score": 2, "match_layer": "graph_traversal"}

    if not scores:
        return {
            "results": [], "query": query,
            "suggestion": f"No one found matching '{query}' — try different keywords",
        }

    conf_map = {3: "HIGH", 2: "HIGH", 1: "MEDIUM", 0: "LOW"}

    results = []
    for pid, info in scores.items():
        person = conn.execute("SELECT * FROM people WHERE id=?", [pid]).fetchone()
        if not person:
            continue
        score = info["score"]
        confidence = conf_map.get(score, "LOW") if score >= 0 else "LOW"
        if info["match_layer"] == "graph_traversal" and score < 3:
            confidence = "MEDIUM"

        mems = conn.execute(
            "SELECT text FROM memory_items WHERE person_id=?"
            " ORDER BY timestamp DESC LIMIT 3",
            [pid],
        ).fetchall()

        results.append({
            "id": pid,
            "name": person["name"],
            "role": person["role"],
            "company": person["company"],
            "location": person["location"],
            "strength_label": person["strength_label"],
            "confidence": confidence,
            "match_layer": info["match_layer"],
            "context_tags": _parse_tags(person["context_tags"]),
            "recent_memories": [m["text"] for m in mems],
            "notes_preview": (person["notes"] or "")[:100],
        })

    order = {"HIGH": 3, "MEDIUM": 2, "LOW": 1}
    results.sort(
        key=lambda r: (order.get(r["confidence"], 0),
                       scores.get(r["id"], {}).get("score", 0)),
        reverse=True,
    )
    return {"results": results[:4], "query": query}


def _handle_get_person(conn: sqlite3.Connection, args: dict) -> dict:
    person = _get_person_by_id_or_name(conn, args)
    if not person:
        return {"error": "Person not found"}

    pid = person["id"]
    pdict = _apply_decay(conn, pid, dict(person))

    conn_rows = conn.execute(
        """SELECT p.id, p.name, p.role, p.company,
                  c.connection_type, c.context, c.strength
           FROM connections c
           JOIN people p ON p.id =
               CASE WHEN c.person_a_id=? THEN c.person_b_id
                    ELSE c.person_a_id END
           WHERE c.person_a_id=? OR c.person_b_id=?""",
        [pid, pid, pid],
    ).fetchall()
    pdict["connections"] = [dict(r) for r in conn_rows]

    mem_rows = conn.execute(
        "SELECT * FROM memory_items WHERE person_id=? ORDER BY timestamp DESC",
        [pid],
    ).fetchall()
    pdict["memory_items"] = [dict(m) for m in mem_rows]

    ev_rows = conn.execute(
        "SELECT * FROM person_events WHERE person_id=? ORDER BY date",
        [pid],
    ).fetchall()
    pdict["events"] = [dict(e) for e in ev_rows]

    pdict["context_tags"] = _parse_tags(pdict.get("context_tags"))
    return pdict


def _handle_add_memory(conn: sqlite3.Connection, args: dict) -> dict:
    person_id = args.get("person_id")
    text = (args.get("text") or "").strip()
    if not person_id or not text:
        return {"error": "person_id and text are required"}

    person = conn.execute(
        "SELECT id, name, strength FROM people WHERE id=?", [person_id]
    ).fetchone()
    if not person:
        return {"error": "Person not found"}

    mid = str(uuid.uuid4())
    conn.execute(
        "INSERT INTO memory_items (id, person_id, text, source, context)"
        " VALUES (?,?,?,?,?)",
        [mid, person_id, text,
         args.get("source", "HERMES"), args.get("context")],
    )

    new_strength = min(1.0, float(person["strength"] or 0.7) + 0.1)
    new_label = _strength_label(new_strength)
    now = datetime.now().isoformat()
    conn.execute(
        "UPDATE people SET last_contact_at=?, strength=?, strength_label=?,"
        " updated_at=? WHERE id=?",
        [now, new_strength, new_label, now, person_id],
    )
    conn.commit()
    return {
        "id": mid,
        "person_name": person["name"],
        "text": text,
        "message": f"Memory added for {person['name']}",
    }


def _handle_set_reminder(conn: sqlite3.Connection, args: dict) -> dict:
    person_id = args.get("person_id")
    title = args.get("title")
    date = args.get("date")
    if not person_id or not title or not date:
        return {"error": "person_id, title, and date are required"}

    person = conn.execute(
        "SELECT name FROM people WHERE id=?", [person_id]
    ).fetchone()
    if not person:
        return {"error": "Person not found"}

    eid = str(uuid.uuid4())
    event_type = args.get("type", "REMINDER")
    conn.execute(
        "INSERT INTO person_events (id, person_id, type, title, date)"
        " VALUES (?,?,?,?,?)",
        [eid, person_id, event_type, title, date],
    )
    conn.commit()
    return {
        "id": eid,
        "person_name": person["name"],
        "title": title,
        "date": date,
        "type": event_type,
        "message": f"Reminder set for {person['name']}: {title} on {date}",
    }


def _handle_meeting_brief(conn: sqlite3.Connection, args: dict) -> dict:
    person = _get_person_by_id_or_name(conn, args)
    if not person:
        return {"error": "Person not found"}

    pid = person["id"]
    pdict = _apply_decay(conn, pid, dict(person))

    mems = conn.execute(
        "SELECT text FROM memory_items WHERE person_id=?"
        " ORDER BY timestamp DESC LIMIT 3",
        [pid],
    ).fetchall()
    recent_context = [m["text"] for m in mems]

    all_mems = conn.execute(
        "SELECT text FROM memory_items WHERE person_id=?"
        " ORDER BY timestamp DESC LIMIT 5",
        [pid],
    ).fetchall()
    open_threads = [m["text"] for m in all_mems]

    conn_rows = conn.execute(
        """SELECT p.name, p.role, p.company, c.connection_type, c.context
           FROM connections c
           JOIN people p ON p.id =
               CASE WHEN c.person_a_id=? THEN c.person_b_id
                    ELSE c.person_a_id END
           WHERE c.person_a_id=? OR c.person_b_id=?""",
        [pid, pid, pid],
    ).fetchall()
    connection_network = [dict(r) for r in conn_rows]

    today = datetime.now().date().isoformat()
    future = (datetime.now().date() + timedelta(days=30)).isoformat()
    ev_rows = conn.execute(
        "SELECT * FROM person_events WHERE person_id=?"
        " AND date BETWEEN ? AND ? AND fired=0 ORDER BY date",
        [pid, today, future],
    ).fetchall()
    upcoming_events = [dict(e) for e in ev_rows]

    name = pdict.get("name", "")
    role_company = " | ".join(
        filter(None, [pdict.get("role"), pdict.get("company")])
    )
    last_contact = pdict.get("last_contact_at") or "No recent contact recorded"
    how_connected = (
        f"Via {connection_network[0]['connection_type']}"
        if connection_network else "Direct contact"
    )

    narrative = (
        f"# Meeting Brief: {name}\n\n"
        f"**Role:** {role_company or 'Unknown'}\n"
        f"**Location:** {pdict.get('location') or 'Unknown'}\n"
        f"**Relationship:** {pdict.get('strength_label', 'WARM')}\n"
        f"**Last Contact:** {last_contact}\n\n"
        "## Recent Context\n"
        + ("\n".join(f"- {m}" for m in recent_context)
           if recent_context else "- No recent interactions recorded")
        + "\n\n## Network\n"
        + ("\n".join(
            f"- {c['name']} ({c.get('connection_type', 'connected')})"
            for c in connection_network
        ) if connection_network else "- No connections recorded")
        + f"\n\n## Notes\n{pdict.get('notes') or 'No notes.'}\n"
    )

    return {
        "name": name,
        "role_company": role_company,
        "location": pdict.get("location"),
        "strength_label": pdict.get("strength_label"),
        "last_contact": last_contact,
        "how_connected": how_connected,
        "recent_context": recent_context,
        "open_threads": open_threads,
        "upcoming_events": upcoming_events,
        "connection_network": connection_network,
        "narrative": narrative,
    }


def _handle_draft_outreach(conn: sqlite3.Connection, args: dict) -> dict:
    person = _get_person_by_id_or_name(conn, args)
    if not person:
        return {"error": "Person not found"}

    pid = person["id"]
    pdict = _apply_decay(conn, pid, dict(person))

    last_mem = conn.execute(
        "SELECT text FROM memory_items WHERE person_id=?"
        " ORDER BY timestamp DESC LIMIT 1",
        [pid],
    ).fetchone()
    last_topic = last_mem["text"] if last_mem else "Nothing recorded yet"

    last_contact = pdict.get("last_contact_at")
    days_since = 0
    if last_contact:
        try:
            last_dt = datetime.fromisoformat(last_contact)
            days_since = int((datetime.now() - last_dt).total_seconds() / 86400)
        except (ValueError, TypeError):
            pass

    name = pdict.get("name", "")
    first_name = name.split()[0] if name else name
    tone = args.get("tone", "warm")
    label = pdict.get("strength_label", "WARM")

    instructions = (
        f"Draft a {tone} re-engagement message to {first_name}.\n"
        f"Last topic: {last_topic}\n"
        f"Days since last contact: {days_since}\n"
        f"Relationship strength: {label}\n"
        f"Keep it brief, personal, and reference the last topic naturally."
    )
    return {
        "person_name": name,
        "first_name": first_name,
        "last_topic": last_topic,
        "days_since_contact": days_since,
        "strength_label": label,
        "instructions": instructions,
    }


def _handle_fading_check(conn: sqlite3.Connection, args: dict) -> dict:
    days = int(args.get("days") or 30)
    cutoff = (datetime.now() - timedelta(days=days)).isoformat()

    rows = conn.execute(
        """SELECT p.*,
               (SELECT text FROM memory_items
                WHERE person_id=p.id ORDER BY timestamp DESC LIMIT 1
               ) AS last_topic
           FROM people p
           WHERE p.strength_label = 'FADING'
              OR (p.last_contact_at IS NOT NULL AND p.last_contact_at < ?)""",
        [cutoff],
    ).fetchall()

    people = [dict(r) for r in rows]
    return {
        "fading_count": len(people),
        "people": people,
        "message": f"Found {len(people)} fading or overdue contacts",
    }


def _handle_upcoming_events(conn: sqlite3.Connection, args: dict) -> dict:
    days = int(args.get("days") or 30)
    today = datetime.now().date()
    end_date = today + timedelta(days=days)

    rows = conn.execute(
        """SELECT e.*,
                  p.name  AS person_name,
                  p.role  AS role,
                  CAST(julianday(e.date) - julianday('now') AS INTEGER)
                      AS days_until
           FROM person_events e
           JOIN people p ON p.id = e.person_id
           WHERE e.date BETWEEN ? AND ? AND e.fired = 0
           ORDER BY e.date ASC""",
        [today.isoformat(), end_date.isoformat()],
    ).fetchall()

    events = [dict(r) for r in rows]
    return {
        "upcoming_count": len(events),
        "window_days": days,
        "events": events,
    }


def _handle_graph_query(conn: sqlite3.Connection, args: dict) -> dict:
    raw_query = args.get("query") or ""
    q = raw_query.lower()

    m = re.search(r"who does\s+(.+?)\s+know", q)
    if m:
        seed_name = m.group(1).strip()
        seed = conn.execute(
            "SELECT * FROM people WHERE lower(name) LIKE ?",
            [f"%{seed_name}%"],
        ).fetchone()
        if not seed:
            return {"query_type": "connections_of", "seed_person": seed_name,
                    "connections": [], "count": 0}
        seed_id = seed["id"]
        rows = conn.execute(
            """SELECT p.id, p.name, p.role, p.company,
                      c.connection_type, c.context
               FROM connections c
               JOIN people p ON p.id =
                   CASE WHEN c.person_a_id=? THEN c.person_b_id
                        ELSE c.person_a_id END
               WHERE c.person_a_id=? OR c.person_b_id=?""",
            [seed_id, seed_id, seed_id],
        ).fetchall()
        return {
            "query_type": "connections_of",
            "seed_person": seed["name"],
            "connections": [dict(r) for r in rows],
            "count": len(rows),
        }

    if re.search(r"introduc", q):
        rows = conn.execute(
            """SELECT p.id, p.name,
                      COUNT(c.id) AS connection_count
               FROM people p
               LEFT JOIN connections c
                   ON c.person_a_id = p.id OR c.person_b_id = p.id
               GROUP BY p.id
               ORDER BY connection_count DESC
               LIMIT 5""",
        ).fetchall()
        return {
            "query_type": "top_introducers",
            "results": [dict(r) for r in rows],
        }

    m = re.search(r"who do i know in\s+(.+)", q)
    if m:
        qualifier = m.group(1).strip()
        pat = "%" + qualifier + "%"
        rows = conn.execute(
            """SELECT id, name, role, company, location FROM people WHERE
                   lower(location)     LIKE ? OR
                   lower(role)         LIKE ? OR
                   lower(company)      LIKE ? OR
                   lower(context_tags) LIKE ?""",
            [pat, pat, pat, pat],
        ).fetchall()
        return {
            "query_type": "filter_by_context",
            "qualifier": qualifier,
            "results": [dict(r) for r in rows],
            "count": len(rows),
        }

    return {"query_type": "unsupported",
            "message": f"Query pattern not recognized: '{raw_query}'"}


def _handle_queue_unknown(conn: sqlite3.Connection, args: dict) -> dict:
    description = (args.get("description") or "").strip()
    if not description:
        return {"error": "description is required"}

    qid = str(uuid.uuid4())
    conn.execute(
        "INSERT INTO unknown_queue (id, description, session_id)"
        " VALUES (?,?,?)",
        [qid, description, args.get("session_id")],
    )
    conn.commit()
    return {
        "id": qid,
        "description": description,
        "message": "Added to unknown queue for future resolution",
    }


# ---------------------------------------------------------------------------
# Dispatch table
# ---------------------------------------------------------------------------

_HANDLERS = {
    "rolodex_add_person":      _handle_add_person,
    "rolodex_fuzzy_recall":    _handle_fuzzy_recall,
    "rolodex_get_person":      _handle_get_person,
    "rolodex_add_memory":      _handle_add_memory,
    "rolodex_set_reminder":    _handle_set_reminder,
    "rolodex_meeting_brief":   _handle_meeting_brief,
    "rolodex_draft_outreach":  _handle_draft_outreach,
    "rolodex_fading_check":    _handle_fading_check,
    "rolodex_upcoming_events": _handle_upcoming_events,
    "rolodex_graph_query":     _handle_graph_query,
    "rolodex_queue_unknown":   _handle_queue_unknown,
}


def _dispatch_tool(conn: sqlite3.Connection, name: str, args: dict) -> dict:
    if name not in _HANDLERS:
        return {"error": f"Unknown tool: {name}"}
    return _HANDLERS[name](conn, args)


# ---------------------------------------------------------------------------
# Tool definitions
# ---------------------------------------------------------------------------

def _build_tool_definitions() -> list:
    def _tool(name, description, required, props):
        schema = {"type": "object", "properties": props}
        if required:
            schema["required"] = required
        return types.Tool(name=name, description=description,
                          inputSchema=schema)

    str_t = {"type": "string"}
    int_t = {"type": "integer"}
    arr_t = {"type": "array", "items": {"type": "string"}}

    return [
        _tool("rolodex_add_person",
              "Add a person to the Hermes Rolodex relationship graph.",
              ["name"],
              {"name": str_t, "role": str_t, "company": str_t,
               "email": str_t, "phone": str_t, "location": str_t,
               "birthday": str_t, "photo_url": str_t, "notes": str_t,
               "context_tags": arr_t}),

        _tool("rolodex_fuzzy_recall",
              "Natural-language fuzzy search across the Rolodex. "
              "Returns up to 4 results with confidence scores. "
              "Example: 'that woman with the red boots from Austin'",
              ["query"],
              {"query": str_t}),

        _tool("rolodex_get_person",
              "Fetch full person profile including connections, memories, "
              "and events. Applies strength decay before returning.",
              [],
              {"id": str_t, "name": str_t}),

        _tool("rolodex_add_memory",
              "Log an interaction or note for a person. "
              "Boosts relationship strength by 0.1 and updates last_contact_at.",
              ["person_id", "text"],
              {"person_id": str_t, "text": str_t,
               "source": str_t, "context": str_t}),

        _tool("rolodex_set_reminder",
              "Create a reminder, meeting note, or birthday event for a person.",
              ["person_id", "title", "date"],
              {"person_id": str_t, "title": str_t, "date": str_t,
               "type": str_t}),

        _tool("rolodex_meeting_brief",
              "Pre-meeting brief: who they are, last interactions, "
              "connection network, and a full narrative.",
              [],
              {"id": str_t, "name": str_t, "person_id": str_t}),

        _tool("rolodex_draft_outreach",
              "Return context for Hermes to draft a re-engagement message. "
              "Does NOT draft itself — returns instructions field.",
              [],
              {"id": str_t, "name": str_t, "person_id": str_t,
               "tone": str_t}),

        _tool("rolodex_fading_check",
              "List contacts whose relationship strength is FADING or "
              "who haven't been contacted in N days (default 30).",
              [],
              {"days": int_t}),

        _tool("rolodex_upcoming_events",
              "List birthdays, meetings, and reminders in the next N days "
              "(default 30).",
              [],
              {"days": int_t}),

        _tool("rolodex_graph_query",
              "Second-degree network queries. Supports: "
              "'who does X know', 'who introduced the most people', "
              "'who do I know in [city/industry]'.",
              ["query"],
              {"query": str_t}),

        _tool("rolodex_queue_unknown",
              "Queue an unresolved person description for later resolution.",
              ["description"],
              {"description": str_t, "session_id": str_t}),
    ]


# ---------------------------------------------------------------------------
# Server wrapper
# ---------------------------------------------------------------------------

class RolodexMCPServer:
    """
    Wraps mcp.server.Server and exposes async list_tools() for direct testing
    (AGENT-3 Guardian check pattern).
    """

    def __init__(self):
        self._tool_defs = _build_tool_definitions()
        self.app = self._build_app()

    def _build_app(self) -> Server:
        app = Server("hermes-rolodex")
        tool_defs = self._tool_defs

        @app.list_tools()
        async def handle_list_tools():
            return tool_defs

        @app.call_tool()
        async def handle_call_tool(name: str, arguments: dict):
            conn = _get_connection()
            try:
                result = _dispatch_tool(conn, name, arguments or {})
                conn.commit()
                return [types.TextContent(type="text",
                                          text=json.dumps(result))]
            except Exception as exc:
                return [types.TextContent(type="text",
                                          text=json.dumps({"error": str(exc)}))]
            finally:
                conn.close()

        return app

    async def list_tools(self) -> list:
        """Direct async accessor — used by AGENT-3 Guardian check."""
        return self._tool_defs


def create_server() -> RolodexMCPServer:
    """Factory used by AGENT-3 Guardian and external callers."""
    return RolodexMCPServer()


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

async def main() -> None:
    _init_db()
    server = create_server()
    async with stdio_server() as (read_stream, write_stream):
        await server.app.run(
            read_stream,
            write_stream,
            server.app.create_initialization_options(),
        )


if __name__ == "__main__":
    import asyncio
    asyncio.run(main())
