"""Open Brain — Supabase-backed persistent memory tool.

Connects to the user's self-hosted Supabase instance (second_brain) to store
and retrieve long-term memories, notes, linked concepts, and collections.

Uses Supabase REST API (PostgREST via Kong) — no direct PG connection needed.

Environment variables:
    SUPABASE_URL        — e.g. http://31.220.58.212:8001
    SUPABASE_KEY        — anon key (JWT) for row-level security
    SUPABASE_SERVICE_KEY — service-role key for admin operations (optional)
"""

import json
import logging
import os
from datetime import datetime, timezone
from typing import Optional

import httpx

from tools.registry import registry

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

_SUPABASE_URL: Optional[str] = None
_SUPABASE_KEY: Optional[str] = None
_TABLE = "memories"


def _get_config():
    global _SUPABASE_URL, _SUPABASE_KEY
    if _SUPABASE_URL is None:
        _SUPABASE_URL = os.getenv("SUPABASE_URL", "").rstrip("/")
        _SUPABASE_KEY = os.getenv("SUPABASE_KEY", "")
    return _SUPABASE_URL, _SUPABASE_KEY


def _headers():
    url, key = _get_config()
    return {
        "apikey": key,
        "Authorization": f"Bearer {key}",
        "Content-Type": "application/json",
        "Prefer": "return=representation",
    }


def _rest_url(path: str) -> str:
    url, _ = _get_config()
    return f"{url}/rest/v1/{path}"


# ---------------------------------------------------------------------------
# Availability check
# ---------------------------------------------------------------------------

def check_requirements() -> bool:
    return bool(os.getenv("SUPABASE_URL")) and bool(os.getenv("SUPABASE_KEY"))


# ---------------------------------------------------------------------------
# Core functions
# ---------------------------------------------------------------------------

def store_memory(
    content: str,
    title: str = "",
    collection: str = "general",
    tags: Optional[list] = None,
    links: Optional[list] = None,
    task_id: str = None,
) -> str:
    """Store a memory in the Open Brain."""
    try:
        payload = {
            "content": content,
            "title": title or content[:80],
            "collection": collection,
            "tags": tags or [],
            "links": links or [],
            "created_at": datetime.now(timezone.utc).isoformat(),
        }
        with httpx.Client(timeout=15) as client:
            resp = client.post(
                _rest_url(_TABLE),
                headers=_headers(),
                json=payload,
            )
            resp.raise_for_status()
            data = resp.json()
            record = data[0] if isinstance(data, list) and data else data
            return json.dumps({
                "success": True,
                "id": record.get("id"),
                "message": f"Memory stored in '{collection}'",
            })
    except Exception as e:
        logger.error("store_memory failed: %s", e)
        return json.dumps({"success": False, "error": str(e)})


def search_memories(
    query: str,
    collection: str = "",
    limit: int = 10,
    task_id: str = None,
) -> str:
    """Search memories by text content (full-text or ILIKE)."""
    try:
        params = {
            "select": "id,title,content,collection,tags,created_at",
            "order": "created_at.desc",
            "limit": str(min(limit, 50)),
        }
        # Use PostgREST text search if available, fall back to ILIKE
        if query:
            params["or"] = f"(title.ilike.*{query}*,content.ilike.*{query}*)"
        if collection:
            params["collection"] = f"eq.{collection}"

        with httpx.Client(timeout=15) as client:
            resp = client.get(
                _rest_url(_TABLE),
                headers=_headers(),
                params=params,
            )
            resp.raise_for_status()
            results = resp.json()
            return json.dumps({
                "success": True,
                "count": len(results),
                "memories": results,
            })
    except Exception as e:
        logger.error("search_memories failed: %s", e)
        return json.dumps({"success": False, "error": str(e)})


def get_linked_memories(
    memory_id: str,
    task_id: str = None,
) -> str:
    """Get a memory and its linked memories by ID."""
    try:
        with httpx.Client(timeout=15) as client:
            # Fetch the source memory
            resp = client.get(
                _rest_url(_TABLE),
                headers=_headers(),
                params={"id": f"eq.{memory_id}", "select": "*"},
            )
            resp.raise_for_status()
            records = resp.json()
            if not records:
                return json.dumps({"success": False, "error": "Memory not found"})

            source = records[0]
            linked_ids = source.get("links", [])

            linked = []
            if linked_ids:
                # Fetch linked memories
                ids_filter = ",".join(str(lid) for lid in linked_ids)
                resp2 = client.get(
                    _rest_url(_TABLE),
                    headers=_headers(),
                    params={
                        "id": f"in.({ids_filter})",
                        "select": "id,title,content,collection,tags",
                    },
                )
                resp2.raise_for_status()
                linked = resp2.json()

            return json.dumps({
                "success": True,
                "memory": source,
                "linked": linked,
            })
    except Exception as e:
        logger.error("get_linked_memories failed: %s", e)
        return json.dumps({"success": False, "error": str(e)})


def list_collections(task_id: str = None) -> str:
    """List all memory collections with counts."""
    try:
        with httpx.Client(timeout=15) as client:
            resp = client.get(
                _rest_url(_TABLE),
                headers=_headers(),
                params={"select": "collection", "order": "collection"},
            )
            resp.raise_for_status()
            rows = resp.json()
            # Count per collection
            counts: dict = {}
            for row in rows:
                c = row.get("collection", "general")
                counts[c] = counts.get(c, 0) + 1
            return json.dumps({
                "success": True,
                "collections": [
                    {"name": k, "count": v} for k, v in sorted(counts.items())
                ],
            })
    except Exception as e:
        logger.error("list_collections failed: %s", e)
        return json.dumps({"success": False, "error": str(e)})


# ---------------------------------------------------------------------------
# Registry
# ---------------------------------------------------------------------------

registry.register(
    name="brain_store",
    toolset="open_brain",
    schema={
        "name": "brain_store",
        "description": (
            "Store a memory/note in the Open Brain (Supabase). "
            "Use for saving insights, decisions, research, meeting notes, "
            "or any durable knowledge the user wants to persist long-term."
        ),
        "parameters": {
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "The memory content to store",
                },
                "title": {
                    "type": "string",
                    "description": "Short title for the memory (auto-generated if omitted)",
                },
                "collection": {
                    "type": "string",
                    "description": "Collection/category (e.g. 'projects', 'research', 'decisions'). Default: 'general'",
                },
                "tags": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Tags for filtering",
                },
                "links": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "IDs of related memories to link",
                },
            },
            "required": ["content"],
        },
    },
    handler=lambda args, **kw: store_memory(
        content=args["content"],
        title=args.get("title", ""),
        collection=args.get("collection", "general"),
        tags=args.get("tags"),
        links=args.get("links"),
        task_id=kw.get("task_id"),
    ),
    check_fn=check_requirements,
    requires_env=["SUPABASE_URL", "SUPABASE_KEY"],
    emoji="🧠",
)

registry.register(
    name="brain_search",
    toolset="open_brain",
    schema={
        "name": "brain_search",
        "description": (
            "Search the Open Brain for memories, notes, and knowledge. "
            "Searches titles and content. Filter by collection."
        ),
        "parameters": {
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query",
                },
                "collection": {
                    "type": "string",
                    "description": "Filter by collection name (optional)",
                },
                "limit": {
                    "type": "integer",
                    "description": "Max results (default: 10, max: 50)",
                },
            },
            "required": ["query"],
        },
    },
    handler=lambda args, **kw: search_memories(
        query=args["query"],
        collection=args.get("collection", ""),
        limit=args.get("limit", 10),
        task_id=kw.get("task_id"),
    ),
    check_fn=check_requirements,
    requires_env=["SUPABASE_URL", "SUPABASE_KEY"],
    emoji="🔍",
)

registry.register(
    name="brain_links",
    toolset="open_brain",
    schema={
        "name": "brain_links",
        "description": (
            "Get a memory and all its linked/related memories from the Open Brain. "
            "Use to explore concept relationships and knowledge graphs."
        ),
        "parameters": {
            "type": "object",
            "properties": {
                "memory_id": {
                    "type": "string",
                    "description": "ID of the memory to fetch links for",
                },
            },
            "required": ["memory_id"],
        },
    },
    handler=lambda args, **kw: get_linked_memories(
        memory_id=args["memory_id"],
        task_id=kw.get("task_id"),
    ),
    check_fn=check_requirements,
    requires_env=["SUPABASE_URL", "SUPABASE_KEY"],
    emoji="🔗",
)

registry.register(
    name="brain_collections",
    toolset="open_brain",
    schema={
        "name": "brain_collections",
        "description": "List all memory collections in the Open Brain with item counts.",
        "parameters": {
            "type": "object",
            "properties": {},
        },
    },
    handler=lambda args, **kw: list_collections(task_id=kw.get("task_id")),
    check_fn=check_requirements,
    requires_env=["SUPABASE_URL", "SUPABASE_KEY"],
    emoji="📚",
)
