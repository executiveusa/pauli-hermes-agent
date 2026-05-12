"""
Hermes Rolodex API endpoints for web UI integration.

Provides REST API wrappers around the MCP server tools for the Hermes Rolodex UI.
"""

import asyncio
import json
from typing import Any, Dict, Optional
from pathlib import Path
from datetime import datetime, timedelta, timezone
import aiosqlite

# Database connection
DB_PATH = Path.home() / ".hermes" / "rolodex.db"


async def get_db() -> aiosqlite.Connection:
    """Get async SQLite connection to rolodex database."""
    db = await aiosqlite.connect(str(DB_PATH))
    await db.execute("PRAGMA foreign_keys=ON")
    return db


async def fuzzy_recall(query: str, limit: int = 10) -> Dict[str, Any]:
    """
    Search for people using fuzzy matching.

    Args:
        query: Search query (name, email, or notes)
        limit: Maximum number of results

    Returns:
        Dictionary with matches list
    """
    db = await get_db()
    try:
        # Use FTS5 for fuzzy search
        cursor = await db.execute("""
            SELECT p.id, p.name, p.email, p.phone, p.strength, p.strength_label, p.last_contact_at
            FROM people p
            LEFT JOIN people_fts fts ON p.id = fts.rowid
            WHERE fts.people_fts MATCH ?
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
    """
    Get full person details including memories and connections.

    Args:
        person_id_or_name: Person ID or name

    Returns:
        Dictionary with person data and related information
    """
    db = await get_db()
    try:
        # Get person
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

        # Get memories
        cursor = await db.execute("""
            SELECT id, person_id, content, context, created_at
            FROM memory_items
            WHERE person_id = ?
            ORDER BY created_at DESC
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

        # Get connections
        cursor = await db.execute("""
            SELECT id, target_person_id, relationship_type, strength, description
            FROM connections
            WHERE source_person_id = ?
            ORDER BY strength DESC
            LIMIT 10
        """, (person_id,))

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
    """
    Add a new person to the rolodex.

    Args:
        name: Person's name
        email: Optional email address
        phone: Optional phone number

    Returns:
        Dictionary with created person
    """
    db = await get_db()
    try:
        now = datetime.now(timezone.utc).isoformat()

        cursor = await db.execute("""
            INSERT INTO people (name, email, phone, strength, strength_label, last_contact_at)
            VALUES (?, ?, ?, 0.5, 'WARM', ?)
            RETURNING id, name, email, phone, strength, strength_label, last_contact_at
        """, (name, email, phone, now))

        row = await cursor.fetchone()
        await db.commit()

        return {
            "person": {
                "id": row[0],
                "name": row[1],
                "email": row[2],
                "phone": row[3],
                "strength": row[4],
                "strength_label": row[5],
                "last_contact_at": row[6],
            }
        }
    finally:
        await db.close()


async def add_memory(person_id: str, content: str, context: Optional[str] = None) -> Dict[str, Any]:
    """
    Add a memory about a person.

    Args:
        person_id: ID of the person
        content: Memory content
        context: Optional context information

    Returns:
        Dictionary with created memory
    """
    db = await get_db()
    try:
        now = datetime.now(timezone.utc).isoformat()

        cursor = await db.execute("""
            INSERT INTO memory_items (person_id, content, context, created_at)
            VALUES (?, ?, ?, ?)
            RETURNING id, person_id, content, context, created_at
        """, (person_id, content, context, now))

        row = await cursor.fetchone()
        await db.commit()

        return {
            "memory": {
                "id": row[0],
                "person_id": row[1],
                "content": row[2],
                "context": row[3],
                "created_at": row[4],
            }
        }
    finally:
        await db.close()


async def upcoming_events(days_ahead: int = 30) -> Dict[str, Any]:
    """
    Get upcoming events for the next N days.

    Args:
        days_ahead: Number of days to look ahead

    Returns:
        Dictionary with upcoming events
    """
    db = await get_db()
    try:
        future_date = (datetime.now(timezone.utc) + timedelta(days=days_ahead)).isoformat()

        cursor = await db.execute("""
            SELECT pe.id, p.id, p.name, pe.event_type, pe.event_date, pe.description
            FROM person_events pe
            JOIN people p ON pe.person_id = p.id
            WHERE pe.event_date <= ? AND pe.event_date >= datetime('now')
            ORDER BY pe.event_date ASC
        """, (future_date,))

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
    """
    Generate a weekly meeting brief.

    Returns:
        Dictionary with meeting brief data
    """
    db = await get_db()
    try:
        # Get ACTIVE relationships
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

        # Get FADING relationships
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

        # Count by strength
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
    """
    Call an MCP tool with the given input.

    Args:
        tool_name: Name of the tool to call
        input_data: Input parameters for the tool

    Returns:
        Result from the tool
    """
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
