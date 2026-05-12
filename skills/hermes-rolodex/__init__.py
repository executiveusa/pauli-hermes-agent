"""Hermes Rolodex™ — intelligent relationship management skill."""

try:
    from .api import (
        call_mcp_tool,
        fuzzy_recall,
        get_person,
        add_person,
        add_memory,
        upcoming_events,
        meeting_brief,
    )
    __all__ = [
        "call_mcp_tool", "fuzzy_recall", "get_person", "add_person",
        "add_memory", "upcoming_events", "meeting_brief",
    ]
except ImportError:
    # aiosqlite not installed; skill available but DB functions disabled
    __all__ = []
