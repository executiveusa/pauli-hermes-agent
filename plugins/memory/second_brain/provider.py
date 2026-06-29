import json
import logging
import os
import threading
from typing import Any, Dict, List, Optional
from uuid import uuid4

from agent.memory_provider import MemoryProvider

logger = logging.getLogger(__name__)

class SecondBrainProvider(MemoryProvider):
    def __init__(self):
        self._supabase = None
        self._session_id = None
        self._next_query = None

    @property
    def name(self) -> str:
        return "second_brain"

    def is_available(self) -> bool:
        url = os.environ.get("SUPABASE_URL")
        key = os.environ.get("SUPABASE_KEY") or os.environ.get("SUPABASE_SERVICE_KEY")
        if not url or not key:
            return False
        
        try:
            import supabase
            return True
        except ImportError:
            return False

    def initialize(self, session_id: str, **kwargs) -> None:
        self._session_id = session_id
        url = os.environ.get("SUPABASE_URL")
        key = os.environ.get("SUPABASE_SERVICE_KEY") or os.environ.get("SUPABASE_KEY")
        
        from supabase import create_client
        self._supabase = create_client(url, key)
        
        # Optionally, create a conversation record if not exists
        try:
            self._supabase.table("conversations").insert({
                "id": session_id,
                "title": "Hermes Chat Session"
            }).execute()
        except Exception as e:
            # Might already exist or fail, ignore for now
            logger.debug(f"Could not insert conversation {session_id}: {e}")

    def system_prompt_block(self) -> str:
        return "You have access to a 'second_brain' memory database via tools."

    def prefetch(self, query: str, *, session_id: str = "") -> str:
        if not self._supabase:
            return ""
        
        # Use the explicit query if provided, else use a queued query
        q = query or self._next_query
        self._next_query = None
        
        if not q:
            return ""

        try:
            res = self._supabase.rpc(
                "search_memories_fulltext", 
                {"query": q, "count": 5}
            ).execute()
            
            memories = res.data
            if not memories:
                return ""
                
            formatted = "Relevant memories from Second Brain:\n"
            for m in memories:
                formatted += f"- {m.get('title', 'Untitled')}: {m.get('summary', '')}\n"
            return formatted
        except Exception as e:
            logger.error(f"Second Brain prefetch failed: {e}")
            return ""

    def queue_prefetch(self, query: str, *, session_id: str = "") -> None:
        self._next_query = query

    def sync_turn(self, user_content: str, assistant_content: str, *, session_id: str = "") -> None:
        if not self._supabase:
            return
            
        sid = session_id or self._session_id
        
        def _write():
            try:
                # Assuming conversation_messages schema has conversation_id, role, content
                if user_content:
                    self._supabase.table("conversation_messages").insert({
                        "conversation_id": sid,
                        "role": "user",
                        "content": user_content
                    }).execute()
                if assistant_content:
                    self._supabase.table("conversation_messages").insert({
                        "conversation_id": sid,
                        "role": "assistant",
                        "content": assistant_content
                    }).execute()
            except Exception as e:
                logger.error(f"Second Brain sync_turn failed: {e}")

        threading.Thread(target=_write, daemon=True).start()

    def get_tool_schemas(self) -> List[Dict[str, Any]]:
        return [
            {
                "type": "function",
                "function": {
                    "name": "search_memories_fulltext",
                    "description": "Search the Second Brain using full-text search.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "query": {"type": "string", "description": "Search query"},
                            "count": {"type": "integer", "description": "Number of results to return", "default": 5}
                        },
                        "required": ["query"]
                    }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "add_memory_to_brain",
                    "description": "Add a new long-term memory to the Second Brain.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "title": {"type": "string", "description": "Short title for the memory"},
                            "summary": {"type": "string", "description": "Brief summary"},
                            "content": {"type": "string", "description": "Full detailed content"},
                            "category": {"type": "string", "description": "Optional category name"}
                        },
                        "required": ["title", "content"]
                    }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "sync_to_obsidian",
                    "description": "Export the Second Brain to an Obsidian Markdown Vault.",
                    "parameters": {
                        "type": "object",
                        "properties": {},
                        "required": []
                    }
                }
            }
        ]

    def handle_tool_call(self, tool_name: str, args: Dict[str, Any], **kwargs) -> str:
        if not self._supabase:
            return json.dumps({"error": "Supabase client not initialized."})

        try:
            if tool_name == "search_memories_fulltext":
                res = self._supabase.rpc(
                    "search_memories_fulltext", 
                    {"query": args.get("query", ""), "count": args.get("count", 5)}
                ).execute()
                return json.dumps({"success": True, "data": res.data})
                
            elif tool_name == "add_memory_to_brain":
                data = {
                    "title": args.get("title"),
                    "summary": args.get("summary", ""),
                    "content": args.get("content"),
                }
                if args.get("category"):
                    data["category"] = args.get("category")
                    
                res = self._supabase.table("memories").insert(data).execute()
                return json.dumps({"success": True, "data": res.data})
                
            elif tool_name == "sync_to_obsidian":
                from .obsidian_sync import ObsidianSyncer
                syncer = ObsidianSyncer(self._supabase)
                result = syncer.run_sync()
                return json.dumps(result)
                
            else:
                return json.dumps({"error": f"Unknown tool: {tool_name}"})
        except Exception as e:
            return json.dumps({"error": str(e)})

    def get_config_schema(self) -> List[Dict[str, Any]]:
        return [
            {
                "key": "supabase_url",
                "description": "Supabase API URL",
                "env_var": "SUPABASE_URL",
                "required": True
            },
            {
                "key": "supabase_key",
                "description": "Supabase Service Key",
                "env_var": "SUPABASE_SERVICE_KEY",
                "secret": True,
                "required": True
            }
        ]
