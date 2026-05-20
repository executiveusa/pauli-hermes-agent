"""
Hermes Rolodex API endpoints for web UI integration.

Provides REST API wrappers around the MCP server tools for the Hermes Rolodex UI.
"""

import uuid
from typing import Any, Dict, Optional
from pathlib import Path
from datetime import datetime, timedelta, timezone
import aiosqlite

DB_PATH = Path.home() / ".hermes" / "rolodex.db"


async def get_db() -> aiosqlite.Connection:
    db = await aiosqlite.connect(str(DB_PATH))
    await db.execute("PRAGMA foreign_keys=ON")
    return db


async def fuzzy_recall(query: str, limit: int = 10) -> Dict[str, Any]:
    db = await get_db()
    try:
        cursor = await db.execute("""
            SELECT p.id, p.name, p.email, p.phone, p.strength, p.strength_label, p.last_contact_at
            FROM people_fts fts
            JOIN people p ON p.id = fts.id
            WHERE people_fts MATCH ?
            ORDER BY rank, p.strength DESC
            LIMIT ?
        """, (query.replace(" ", " AND "), limit))

        rows = await cursor.fetchall()
        matches = [
            {
                "id": row[0],
                "name": row[1],
                "email": row[2],
                "phone": row[3],
                "strength": row[4],
                "strength_label": row[5],
                "last_contact_at": row[6],
            }
            for row in rows
        ]

        return {"matches": matches, "query": query}
    finally:
        await db.close()


async def get_person(person_id_or_name: str) -> Dict[str, Any]:
    db = await get_db()
    try:
        cursor = await db.execute("""
            SELECT id, name, email, phone, strength, strength_label, last_contact_at, notes
            FROM people
            WHERE id = ? OR name = ?
            LIMIT 1
        """, (person_id_or_name, person_id_or_name))

        person_row = await cursor.fetchone()
        if not person_row:
            return {"error": "Person not found"}

        person_id = person_row[0]
        person = {
            "id": person_id,
            "name": person_row[1],
            "email": person_row[2],
            "phone": person_row[3],
            "strength": person_row[4],
            "strength_label": person_row[5],
            "last_contact_at": person_row[6],
            "notes": person_row[7],
        }

        # memory_items uses: id, person_id, text, source, context, timestamp
        cursor = await db.execute("""
            SELECT id, person_id, text, context, timestamp
            FROM memory_items
            WHERE person_id = ?
            ORDER BY timestamp DESC
            LIMIT 20
        """, (person_id,))

        memories = [
            {
                "id": row[0],
                "person_id": row[1],
                "content": row[2],
                "context": row[3],
                "created_at": row[4],
            }
            for row in await cursor.fetchall()
        ]

        # connections uses: id, person_a_id, person_b_id, connection_type, context, strength
        cursor = await db.execute("""
            SELECT id,
                   CASE WHEN person_a_id = ? THEN person_b_id ELSE person_a_id END,
                   connection_type, strength, context
            FROM connections
            WHERE person_a_id = ? OR person_b_id = ?
            ORDER BY strength DESC
            LIMIT 10
        """, (person_id, person_id, person_id))

        connections = [
            {
                "id": row[0],
                "target_person_id": row[1],
                "relationship_type": row[2],
                "strength": row[3],
                "description": row[4],
            }
            for row in await cursor.fetchall()
        ]

        return {
            "person": person,
            "memories": memories,
            "connections": connections,
        }
    finally:
        await db.close()


async def add_person(name: str, email: Optional[str] = None, phone: Optional[str] = None) -> Dict[str, Any]:
    db = await get_db()
    try:
        now = datetime.now(timezone.utc).isoformat()
        person_id = str(uuid.uuid4())

        await db.execute("""
            INSERT INTO people (id, name, email, phone, strength, strength_label, last_contact_at)
            VALUES (?, ?, ?, ?, 0.5, 'WARM', ?)
        """, (person_id, name, email, phone, now))

        # Sync FTS — people_fts is not a content= table so we manage it explicitly
        await db.execute("""
            INSERT INTO people_fts (id, name, role, company, notes, context_tags_flat)
            VALUES (?, ?, '', '', '', '')
        """, (person_id, name))

        await db.commit()

        return {
            "person": {
                "id": person_id,
                "name": name,
                "email": email,
                "phone": phone,
                "strength": 0.5,
                "strength_label": "WARM",
                "last_contact_at": now,
            }
        }
    finally:
        await db.close()


async def add_memory(person_id: str, content: str, context: Optional[str] = None) -> Dict[str, Any]:
    db = await get_db()
    try:
        now = datetime.now(timezone.utc).isoformat()
        memory_id = str(uuid.uuid4())

        # memory_items schema: id, person_id, text, source, context, timestamp
        await db.execute("""
            INSERT INTO memory_items (id, person_id, text, source, context, timestamp)
            VALUES (?, ?, ?, 'HERMES', ?, ?)
        """, (memory_id, person_id, content, context, now))

        await db.commit()

        return {
            "memory": {
                "id": memory_id,
                "person_id": person_id,
                "content": content,
                "context": context,
                "created_at": now,
            }
        }
    finally:
        await db.close()


async def upcoming_events(days_ahead: int = 30) -> Dict[str, Any]:
    db = await get_db()
    try:
        today = datetime.now(timezone.utc).date().isoformat()
        future_date = (datetime.now(timezone.utc).date() + timedelta(days=days_ahead)).isoformat()

        # person_events schema: id, person_id, type, title, date, fired, fired_at, created_at
        cursor = await db.execute("""
            SELECT pe.id, p.id, p.name, pe.type, pe.date, pe.title
            FROM person_events pe
            JOIN people p ON pe.person_id = p.id
            WHERE pe.date BETWEEN ? AND ? AND pe.fired = 0
            ORDER BY pe.date ASC
        """, (today, future_date))

        events = [
            {
                "id": row[0],
                "person_id": row[1],
                "person_name": row[2],
                "event_type": row[3],
                "event_date": row[4],
                "description": row[5],
            }
            for row in await cursor.fetchall()
        ]

        return {"events": events}
    finally:
        await db.close()


async def meeting_brief() -> Dict[str, Any]:
    db = await get_db()
    try:
        cursor = await db.execute("""
            SELECT id, name, email, strength, last_contact_at
            FROM people
            WHERE strength_label = 'ACTIVE'
            ORDER BY strength DESC
            LIMIT 10
        """)

        active = [
            {
                "id": row[0],
                "name": row[1],
                "email": row[2],
                "strength": row[3],
                "last_contact_at": row[4],
            }
            for row in await cursor.fetchall()
        ]

        cursor = await db.execute("""
            SELECT id, name, email, strength, last_contact_at
            FROM people
            WHERE strength_label = 'FADING'
            ORDER BY last_contact_at DESC
            LIMIT 5
        """)

        fading = [
            {
                "id": row[0],
                "name": row[1],
                "email": row[2],
                "strength": row[3],
                "last_contact_at": row[4],
            }
            for row in await cursor.fetchall()
        ]

        cursor = await db.execute("""
            SELECT strength_label, COUNT(*) as count
            FROM people
            GROUP BY strength_label
        """)

        counts = {row[0]: row[1] for row in await cursor.fetchall()}

        return {
            "brief": {
                "generated_at": datetime.now(timezone.utc).isoformat(),
                "active_relationships": active,
                "fading_relationships": fading,
                "summary": {
                    "total_contacts": sum(counts.values()),
                    "by_strength": counts,
                }
            }
        }
    finally:
        await db.close()


async def call_mcp_tool(tool_name: str, input_data: Dict[str, Any]) -> Dict[str, Any]:
    handlers = {
        "fuzzy_recall": lambda: fuzzy_recall(**input_data),
        "get_person": lambda: get_person(**input_data),
        "add_person": lambda: add_person(**input_data),
        "add_memory": lambda: add_memory(**input_data),
        "upcoming_events": lambda: upcoming_events(**input_data),
        "meeting_brief": lambda: meeting_brief(),
    }

    if tool_name not in handlers:
        return {"error": f"Unknown tool: {tool_name}"}

    try:
        result = await handlers[tool_name]()
        return result
    except Exception as e:
        return {"error": str(e)}
