"""
OpenAI-compatible API server platform adapter.

Exposes an HTTP server with endpoints:
- POST /v1/chat/completions        — OpenAI Chat Completions format (stateless)
- POST /v1/responses               — OpenAI Responses API format (stateful via previous_response_id)
- GET  /v1/responses/{response_id} — Retrieve a stored response
- DELETE /v1/responses/{response_id} — Delete a stored response
- GET  /v1/models                  — lists hermes-agent as an available model
- GET  /health                     — health check

Any OpenAI-compatible frontend (Open WebUI, LobeChat, LibreChat,
AnythingLLM, NextChat, ChatBox, etc.) can connect to hermes-agent
through this adapter by pointing at http://localhost:8642/v1.

Requires:
- aiohttp (already available in the gateway)
"""

import asyncio
import json
import logging
import os
import sqlite3
import time
import uuid
from typing import Any, Dict, List, Optional

try:
    from aiohttp import web
    AIOHTTP_AVAILABLE = True
except ImportError:
    AIOHTTP_AVAILABLE = False
    web = None  # type: ignore[assignment]

from gateway.config import Platform, PlatformConfig
from gateway.platforms.base import (
    BasePlatformAdapter,
    SendResult,
)

logger = logging.getLogger(__name__)

# Default settings
DEFAULT_HOST = "127.0.0.1"
DEFAULT_PORT = 8642
MAX_STORED_RESPONSES = 100


def check_api_server_requirements() -> bool:
    """Check if API server dependencies are available."""
    return AIOHTTP_AVAILABLE


class ResponseStore:
    """
    SQLite-backed LRU store for Responses API state.

    Each stored response includes the full internal conversation history
    (with tool calls and results) so it can be reconstructed on subsequent
    requests via previous_response_id.

    Persists across gateway restarts.  Falls back to in-memory SQLite
    if the on-disk path is unavailable.
    """

    def __init__(self, max_size: int = MAX_STORED_RESPONSES, db_path: str = None):
        self._max_size = max_size
        if db_path is None:
            try:
                from hermes_cli.config import get_hermes_home
                db_path = str(get_hermes_home() / "response_store.db")
            except Exception:
                db_path = ":memory:"
        try:
            self._conn = sqlite3.connect(db_path, check_same_thread=False)
        except Exception:
            self._conn = sqlite3.connect(":memory:", check_same_thread=False)
        self._conn.execute("PRAGMA journal_mode=WAL")
        self._conn.execute(
            """CREATE TABLE IF NOT EXISTS responses (
                response_id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                accessed_at REAL NOT NULL
            )"""
        )
        self._conn.execute(
            """CREATE TABLE IF NOT EXISTS conversations (
                name TEXT PRIMARY KEY,
                response_id TEXT NOT NULL
            )"""
        )
        self._conn.commit()

    def get(self, response_id: str) -> Optional[Dict[str, Any]]:
        """Retrieve a stored response by ID (updates access time for LRU)."""
        row = self._conn.execute(
            "SELECT data FROM responses WHERE response_id = ?", (response_id,)
        ).fetchone()
        if row is None:
            return None
        import time
        self._conn.execute(
            "UPDATE responses SET accessed_at = ? WHERE response_id = ?",
            (time.time(), response_id),
        )
        self._conn.commit()
        return json.loads(row[0])

    def put(self, response_id: str, data: Dict[str, Any]) -> None:
        """Store a response, evicting the oldest if at capacity."""
        import time
        self._conn.execute(
            "INSERT OR REPLACE INTO responses (response_id, data, accessed_at) VALUES (?, ?, ?)",
            (response_id, json.dumps(data, default=str), time.time()),
        )
        # Evict oldest entries beyond max_size
        count = self._conn.execute("SELECT COUNT(*) FROM responses").fetchone()[0]
        if count > self._max_size:
            self._conn.execute(
                "DELETE FROM responses WHERE response_id IN "
                "(SELECT response_id FROM responses ORDER BY accessed_at ASC LIMIT ?)",
                (count - self._max_size,),
            )
        self._conn.commit()

    def delete(self, response_id: str) -> bool:
        """Remove a response from the store. Returns True if found and deleted."""
        cursor = self._conn.execute(
            "DELETE FROM responses WHERE response_id = ?", (response_id,)
        )
        self._conn.commit()
        return cursor.rowcount > 0

    def get_conversation(self, name: str) -> Optional[str]:
        """Get the latest response_id for a conversation name."""
        row = self._conn.execute(
            "SELECT response_id FROM conversations WHERE name = ?", (name,)
        ).fetchone()
        return row[0] if row else None

    def set_conversation(self, name: str, response_id: str) -> None:
        """Map a conversation name to its latest response_id."""
        self._conn.execute(
            "INSERT OR REPLACE INTO conversations (name, response_id) VALUES (?, ?)",
            (name, response_id),
        )
        self._conn.commit()

    def close(self) -> None:
        """Close the database connection."""
        try:
            self._conn.close()
        except Exception:
            pass

    def __len__(self) -> int:
        row = self._conn.execute("SELECT COUNT(*) FROM responses").fetchone()
        return row[0] if row else 0


# ---------------------------------------------------------------------------
# CORS middleware
# ---------------------------------------------------------------------------

_CORS_HEADERS = {
    "Access-Control-Allow-Methods": "GET, POST, DELETE, OPTIONS",
    "Access-Control-Allow-Headers": "Authorization, Content-Type",
}


if AIOHTTP_AVAILABLE:
    @web.middleware
    async def cors_middleware(request, handler):
        """Add CORS headers for explicitly allowed origins; handle OPTIONS preflight."""
        adapter = request.app.get("api_server_adapter")
        origin = request.headers.get("Origin", "")
        cors_headers = None
        if adapter is not None:
            if not adapter._origin_allowed(origin):
                return web.Response(status=403)
            cors_headers = adapter._cors_headers_for_origin(origin)

        if request.method == "OPTIONS":
            if cors_headers is None:
                return web.Response(status=403)
            return web.Response(status=200, headers=cors_headers)

        response = await handler(request)
        if cors_headers is not None:
            response.headers.update(cors_headers)
        return response
else:
    cors_middleware = None  # type: ignore[assignment]


class APIServerAdapter(BasePlatformAdapter):
    """
    OpenAI-compatible HTTP API server adapter.

    Runs an aiohttp web server that accepts OpenAI-format requests
    and routes them through hermes-agent's AIAgent.
    """

    def __init__(self, config: PlatformConfig):
        super().__init__(config, Platform.API_SERVER)
        extra = config.extra or {}
        self._host: str = extra.get("host", os.getenv("API_SERVER_HOST", DEFAULT_HOST))
        self._port: int = int(extra.get("port", os.getenv("API_SERVER_PORT", str(DEFAULT_PORT))))
        self._api_key: str = extra.get("key", os.getenv("API_SERVER_KEY", ""))
        self._cors_origins: tuple[str, ...] = self._parse_cors_origins(
            extra.get("cors_origins", os.getenv("API_SERVER_CORS_ORIGINS", "")),
        )
        self._app: Optional["web.Application"] = None
        self._runner: Optional["web.AppRunner"] = None
        self._site: Optional["web.TCPSite"] = None
        self._response_store = ResponseStore()
        self._start_time: float = time.time()
        # Supabase REST client (lazy — only if env vars are set)
        self._supabase_url: str = os.getenv("SUPABASE_URL", "").rstrip("/")
        self._supabase_key: str = os.getenv("SUPABASE_SERVICE_KEY", "") or os.getenv("SUPABASE_KEY", "")
        # In-memory fallback stores (used when Supabase is unavailable)
        self._beads_log: list = []
        self._agent_mail: list = []

    @staticmethod
    def _parse_cors_origins(value: Any) -> tuple[str, ...]:
        """Normalize configured CORS origins into a stable tuple."""
        if not value:
            return ()

        if isinstance(value, str):
            items = value.split(",")
        elif isinstance(value, (list, tuple, set)):
            items = value
        else:
            items = [str(value)]

        return tuple(str(item).strip() for item in items if str(item).strip())

    def _cors_headers_for_origin(self, origin: str) -> Optional[Dict[str, str]]:
        """Return CORS headers for an allowed browser origin."""
        if not origin or not self._cors_origins:
            return None

        if "*" in self._cors_origins:
            headers = dict(_CORS_HEADERS)
            headers["Access-Control-Allow-Origin"] = "*"
            return headers

        if origin not in self._cors_origins:
            return None

        headers = dict(_CORS_HEADERS)
        headers["Access-Control-Allow-Origin"] = origin
        headers["Vary"] = "Origin"
        return headers

    def _origin_allowed(self, origin: str) -> bool:
        """Allow non-browser clients and explicitly configured browser origins."""
        if not origin:
            return True

        if not self._cors_origins:
            return False

        return "*" in self._cors_origins or origin in self._cors_origins

    # ------------------------------------------------------------------
    # Auth helper
    # ------------------------------------------------------------------

    def _check_auth(self, request: "web.Request") -> Optional["web.Response"]:
        """
        Validate Bearer token from Authorization header.

        Returns None if auth is OK, or a 401 web.Response on failure.
        If no API key is configured, all requests are allowed.
        """
        if not self._api_key:
            return None  # No key configured — allow all (local-only use)

        auth_header = request.headers.get("Authorization", "")
        if auth_header.startswith("Bearer "):
            token = auth_header[7:].strip()
            if token == self._api_key:
                return None  # Auth OK

        return web.json_response(
            {"error": {"message": "Invalid API key", "type": "invalid_request_error", "code": "invalid_api_key"}},
            status=401,
        )

    # ------------------------------------------------------------------
    # Agent creation helper
    # ------------------------------------------------------------------

    def _create_agent(
        self,
        ephemeral_system_prompt: Optional[str] = None,
        session_id: Optional[str] = None,
        stream_delta_callback=None,
    ) -> Any:
        """
        Create an AIAgent instance using the gateway's runtime config.

        Uses _resolve_runtime_agent_kwargs() to pick up model, api_key,
        base_url, etc. from config.yaml / env vars.
        """
        from run_agent import AIAgent
        from gateway.run import _resolve_runtime_agent_kwargs, _resolve_gateway_model

        runtime_kwargs = _resolve_runtime_agent_kwargs()
        model = _resolve_gateway_model()

        max_iterations = int(os.getenv("HERMES_MAX_ITERATIONS", "90"))

        agent = AIAgent(
            model=model,
            **runtime_kwargs,
            max_iterations=max_iterations,
            quiet_mode=True,
            verbose_logging=False,
            ephemeral_system_prompt=ephemeral_system_prompt or None,
            session_id=session_id,
            platform="api_server",
            stream_delta_callback=stream_delta_callback,
        )
        return agent

    # ------------------------------------------------------------------
    # HTTP Handlers
    # ------------------------------------------------------------------

    async def _handle_health(self, request: "web.Request") -> "web.Response":
        """GET /health — simple health check."""
        return web.json_response({"status": "ok", "platform": "hermes-agent"})

    async def _handle_models(self, request: "web.Request") -> "web.Response":
        """GET /v1/models — return hermes-agent as an available model."""
        auth_err = self._check_auth(request)
        if auth_err:
            return auth_err

        return web.json_response({
            "object": "list",
            "data": [
                {
                    "id": "hermes-agent",
                    "object": "model",
                    "created": int(time.time()),
                    "owned_by": "hermes",
                    "permission": [],
                    "root": "hermes-agent",
                    "parent": None,
                }
            ],
        })

    async def _handle_chat_completions(self, request: "web.Request") -> "web.Response":
        """POST /v1/chat/completions — OpenAI Chat Completions format."""
        auth_err = self._check_auth(request)
        if auth_err:
            return auth_err

        # Parse request body
        try:
            body = await request.json()
        except (json.JSONDecodeError, Exception):
            return web.json_response(
                {"error": {"message": "Invalid JSON in request body", "type": "invalid_request_error"}},
                status=400,
            )

        messages = body.get("messages")
        if not messages or not isinstance(messages, list):
            return web.json_response(
                {"error": {"message": "Missing or invalid 'messages' field", "type": "invalid_request_error"}},
                status=400,
            )

        stream = body.get("stream", False)

        # Extract system message (becomes ephemeral system prompt layered ON TOP of core)
        system_prompt = None
        conversation_messages: List[Dict[str, str]] = []

        for msg in messages:
            role = msg.get("role", "")
            content = msg.get("content", "")
            if role == "system":
                # Accumulate system messages
                if system_prompt is None:
                    system_prompt = content
                else:
                    system_prompt = system_prompt + "\n" + content
            elif role in ("user", "assistant"):
                conversation_messages.append({"role": role, "content": content})

        # Extract the last user message as the primary input
        user_message = ""
        history = []
        if conversation_messages:
            user_message = conversation_messages[-1].get("content", "")
            history = conversation_messages[:-1]

        if not user_message:
            return web.json_response(
                {"error": {"message": "No user message found in messages", "type": "invalid_request_error"}},
                status=400,
            )

        session_id = str(uuid.uuid4())
        completion_id = f"chatcmpl-{uuid.uuid4().hex[:29]}"
        model_name = body.get("model", "hermes-agent")
        created = int(time.time())

        if stream:
            import queue as _q
            _stream_q: _q.Queue = _q.Queue()

            def _on_delta(delta):
                _stream_q.put(delta)

            # Start agent in background
            agent_task = asyncio.ensure_future(self._run_agent(
                user_message=user_message,
                conversation_history=history,
                ephemeral_system_prompt=system_prompt,
                session_id=session_id,
                stream_delta_callback=_on_delta,
            ))

            return await self._write_sse_chat_completion(
                request, completion_id, model_name, created, _stream_q, agent_task
            )

        # Non-streaming: run the agent and return full response
        try:
            result, usage = await self._run_agent(
                user_message=user_message,
                conversation_history=history,
                ephemeral_system_prompt=system_prompt,
                session_id=session_id,
            )
        except Exception as e:
            logger.error("Error running agent for chat completions: %s", e, exc_info=True)
            return web.json_response(
                {"error": {"message": f"Internal server error: {e}", "type": "server_error"}},
                status=500,
            )

        final_response = result.get("final_response", "")
        if not final_response:
            final_response = result.get("error", "(No response generated)")

        response_data = {
            "id": completion_id,
            "object": "chat.completion",
            "created": created,
            "model": model_name,
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": final_response,
                    },
                    "finish_reason": "stop",
                }
            ],
            "usage": {
                "prompt_tokens": usage.get("input_tokens", 0),
                "completion_tokens": usage.get("output_tokens", 0),
                "total_tokens": usage.get("total_tokens", 0),
            },
        }

        return web.json_response(response_data)

    async def _write_sse_chat_completion(
        self, request: "web.Request", completion_id: str, model: str,
        created: int, stream_q, agent_task,
    ) -> "web.StreamResponse":
        """Write real streaming SSE from agent's stream_delta_callback queue."""
        import queue as _q

        response = web.StreamResponse(
            status=200,
            headers={"Content-Type": "text/event-stream", "Cache-Control": "no-cache"},
        )
        await response.prepare(request)

        # Role chunk
        role_chunk = {
            "id": completion_id, "object": "chat.completion.chunk",
            "created": created, "model": model,
            "choices": [{"index": 0, "delta": {"role": "assistant"}, "finish_reason": None}],
        }
        await response.write(f"data: {json.dumps(role_chunk)}\n\n".encode())

        # Stream content chunks as they arrive from the agent
        loop = asyncio.get_event_loop()
        while True:
            try:
                delta = await loop.run_in_executor(None, lambda: stream_q.get(timeout=0.5))
            except _q.Empty:
                if agent_task.done():
                    # Drain any remaining items
                    while True:
                        try:
                            delta = stream_q.get_nowait()
                            if delta is None:
                                break
                            content_chunk = {
                                "id": completion_id, "object": "chat.completion.chunk",
                                "created": created, "model": model,
                                "choices": [{"index": 0, "delta": {"content": delta}, "finish_reason": None}],
                            }
                            await response.write(f"data: {json.dumps(content_chunk)}\n\n".encode())
                        except _q.Empty:
                            break
                    break
                continue

            if delta is None:  # End of stream sentinel
                break

            content_chunk = {
                "id": completion_id, "object": "chat.completion.chunk",
                "created": created, "model": model,
                "choices": [{"index": 0, "delta": {"content": delta}, "finish_reason": None}],
            }
            await response.write(f"data: {json.dumps(content_chunk)}\n\n".encode())

        # Get usage from completed agent
        usage = {"input_tokens": 0, "output_tokens": 0, "total_tokens": 0}
        try:
            result, agent_usage = await agent_task
            usage = agent_usage or usage
        except Exception:
            pass

        # Finish chunk
        finish_chunk = {
            "id": completion_id, "object": "chat.completion.chunk",
            "created": created, "model": model,
            "choices": [{"index": 0, "delta": {}, "finish_reason": "stop"}],
            "usage": {
                "prompt_tokens": usage.get("input_tokens", 0),
                "completion_tokens": usage.get("output_tokens", 0),
                "total_tokens": usage.get("total_tokens", 0),
            },
        }
        await response.write(f"data: {json.dumps(finish_chunk)}\n\n".encode())
        await response.write(b"data: [DONE]\n\n")

        return response

    async def _handle_responses(self, request: "web.Request") -> "web.Response":
        """POST /v1/responses — OpenAI Responses API format."""
        auth_err = self._check_auth(request)
        if auth_err:
            return auth_err

        # Parse request body
        try:
            body = await request.json()
        except (json.JSONDecodeError, Exception):
            return web.json_response(
                {"error": {"message": "Invalid JSON in request body", "type": "invalid_request_error"}},
                status=400,
            )

        raw_input = body.get("input")
        if raw_input is None:
            return web.json_response(
                {"error": {"message": "Missing 'input' field", "type": "invalid_request_error"}},
                status=400,
            )

        instructions = body.get("instructions")
        previous_response_id = body.get("previous_response_id")
        conversation = body.get("conversation")
        store = body.get("store", True)

        # conversation and previous_response_id are mutually exclusive
        if conversation and previous_response_id:
            return web.json_response(
                {"error": {"message": "Cannot use both 'conversation' and 'previous_response_id'", "type": "invalid_request_error"}},
                status=400,
            )

        # Resolve conversation name to latest response_id
        if conversation:
            previous_response_id = self._response_store.get_conversation(conversation)
            # No error if conversation doesn't exist yet — it's a new conversation

        # Normalize input to message list
        input_messages: List[Dict[str, str]] = []
        if isinstance(raw_input, str):
            input_messages = [{"role": "user", "content": raw_input}]
        elif isinstance(raw_input, list):
            for item in raw_input:
                if isinstance(item, str):
                    input_messages.append({"role": "user", "content": item})
                elif isinstance(item, dict):
                    role = item.get("role", "user")
                    content = item.get("content", "")
                    # Handle content that may be a list of content parts
                    if isinstance(content, list):
                        text_parts = []
                        for part in content:
                            if isinstance(part, dict) and part.get("type") == "input_text":
                                text_parts.append(part.get("text", ""))
                            elif isinstance(part, dict) and part.get("type") == "output_text":
                                text_parts.append(part.get("text", ""))
                            elif isinstance(part, str):
                                text_parts.append(part)
                        content = "\n".join(text_parts)
                    input_messages.append({"role": role, "content": content})
        else:
            return web.json_response(
                {"error": {"message": "'input' must be a string or array", "type": "invalid_request_error"}},
                status=400,
            )

        # Reconstruct conversation history from previous_response_id
        conversation_history: List[Dict[str, str]] = []
        if previous_response_id:
            stored = self._response_store.get(previous_response_id)
            if stored is None:
                return web.json_response(
                    {"error": {"message": f"Previous response not found: {previous_response_id}", "type": "invalid_request_error"}},
                    status=404,
                )
            conversation_history = list(stored.get("conversation_history", []))
            # If no instructions provided, carry forward from previous
            if instructions is None:
                instructions = stored.get("instructions")

        # Append new input messages to history (all but the last become history)
        for msg in input_messages[:-1]:
            conversation_history.append(msg)

        # Last input message is the user_message
        user_message = input_messages[-1].get("content", "") if input_messages else ""
        if not user_message:
            return web.json_response(
                {"error": {"message": "No user message found in input", "type": "invalid_request_error"}},
                status=400,
            )

        # Truncation support
        if body.get("truncation") == "auto" and len(conversation_history) > 100:
            conversation_history = conversation_history[-100:]

        # Run the agent
        session_id = str(uuid.uuid4())
        try:
            result, usage = await self._run_agent(
                user_message=user_message,
                conversation_history=conversation_history,
                ephemeral_system_prompt=instructions,
                session_id=session_id,
            )
        except Exception as e:
            logger.error("Error running agent for responses: %s", e, exc_info=True)
            return web.json_response(
                {"error": {"message": f"Internal server error: {e}", "type": "server_error"}},
                status=500,
            )

        final_response = result.get("final_response", "")
        if not final_response:
            final_response = result.get("error", "(No response generated)")

        response_id = f"resp_{uuid.uuid4().hex[:28]}"
        created_at = int(time.time())

        # Build the full conversation history for storage
        # (includes tool calls from the agent run)
        full_history = list(conversation_history)
        full_history.append({"role": "user", "content": user_message})
        # Add agent's internal messages if available
        agent_messages = result.get("messages", [])
        if agent_messages:
            full_history.extend(agent_messages)
        else:
            full_history.append({"role": "assistant", "content": final_response})

        # Build output items (includes tool calls + final message)
        output_items = self._extract_output_items(result)

        response_data = {
            "id": response_id,
            "object": "response",
            "status": "completed",
            "created_at": created_at,
            "model": body.get("model", "hermes-agent"),
            "output": output_items,
            "usage": {
                "input_tokens": usage.get("input_tokens", 0),
                "output_tokens": usage.get("output_tokens", 0),
                "total_tokens": usage.get("total_tokens", 0),
            },
        }

        # Store the complete response object for future chaining / GET retrieval
        if store:
            self._response_store.put(response_id, {
                "response": response_data,
                "conversation_history": full_history,
                "instructions": instructions,
            })
            # Update conversation mapping so the next request with the same
            # conversation name automatically chains to this response
            if conversation:
                self._response_store.set_conversation(conversation, response_id)

        return web.json_response(response_data)

    # ------------------------------------------------------------------
    # GET / DELETE response endpoints
    # ------------------------------------------------------------------

    async def _handle_get_response(self, request: "web.Request") -> "web.Response":
        """GET /v1/responses/{response_id} — retrieve a stored response."""
        auth_err = self._check_auth(request)
        if auth_err:
            return auth_err

        response_id = request.match_info["response_id"]
        stored = self._response_store.get(response_id)
        if stored is None:
            return web.json_response(
                {"error": {"message": f"Response not found: {response_id}", "type": "invalid_request_error"}},
                status=404,
            )

        return web.json_response(stored["response"])

    async def _handle_delete_response(self, request: "web.Request") -> "web.Response":
        """DELETE /v1/responses/{response_id} — delete a stored response."""
        auth_err = self._check_auth(request)
        if auth_err:
            return auth_err

        response_id = request.match_info["response_id"]
        deleted = self._response_store.delete(response_id)
        if not deleted:
            return web.json_response(
                {"error": {"message": f"Response not found: {response_id}", "type": "invalid_request_error"}},
                status=404,
            )

        return web.json_response({
            "id": response_id,
            "object": "response",
            "deleted": True,
        })

    # ------------------------------------------------------------------
    # Cron jobs API
    # ------------------------------------------------------------------

    # Check cron module availability once (not per-request)
    _CRON_AVAILABLE = False
    try:
        from cron.jobs import (
            list_jobs as _cron_list,
            get_job as _cron_get,
            create_job as _cron_create,
            update_job as _cron_update,
            remove_job as _cron_remove,
            pause_job as _cron_pause,
            resume_job as _cron_resume,
            trigger_job as _cron_trigger,
        )
        _CRON_AVAILABLE = True
    except ImportError:
        pass

    _JOB_ID_RE = __import__("re").compile(r"[a-f0-9]{12}")
    # Allowed fields for update — prevents clients injecting arbitrary keys
    _UPDATE_ALLOWED_FIELDS = {"name", "schedule", "prompt", "deliver", "skills", "skill", "repeat", "enabled"}
    _MAX_NAME_LENGTH = 200
    _MAX_PROMPT_LENGTH = 5000

    def _check_jobs_available(self) -> Optional["web.Response"]:
        """Return error response if cron module isn't available."""
        if not self._CRON_AVAILABLE:
            return web.json_response(
                {"error": "Cron module not available"}, status=501,
            )
        return None

    def _check_job_id(self, request: "web.Request") -> tuple:
        """Validate and extract job_id. Returns (job_id, error_response)."""
        job_id = request.match_info["job_id"]
        if not self._JOB_ID_RE.fullmatch(job_id):
            return job_id, web.json_response(
                {"error": "Invalid job ID format"}, status=400,
            )
        return job_id, None

    async def _handle_list_jobs(self, request: "web.Request") -> "web.Response":
        """GET /api/jobs — list all cron jobs."""
        auth_err = self._check_auth(request)
        if auth_err:
            return auth_err
        cron_err = self._check_jobs_available()
        if cron_err:
            return cron_err
        try:
            include_disabled = request.query.get("include_disabled", "").lower() in ("true", "1")
            jobs = self._cron_list(include_disabled=include_disabled)
            return web.json_response({"jobs": jobs})
        except Exception as e:
            return web.json_response({"error": str(e)}, status=500)

    async def _handle_create_job(self, request: "web.Request") -> "web.Response":
        """POST /api/jobs — create a new cron job."""
        auth_err = self._check_auth(request)
        if auth_err:
            return auth_err
        cron_err = self._check_jobs_available()
        if cron_err:
            return cron_err
        try:
            body = await request.json()
            name = (body.get("name") or "").strip()
            schedule = (body.get("schedule") or "").strip()
            prompt = body.get("prompt", "")
            deliver = body.get("deliver", "local")
            skills = body.get("skills")
            repeat = body.get("repeat")

            if not name:
                return web.json_response({"error": "Name is required"}, status=400)
            if len(name) > self._MAX_NAME_LENGTH:
                return web.json_response(
                    {"error": f"Name must be ≤ {self._MAX_NAME_LENGTH} characters"}, status=400,
                )
            if not schedule:
                return web.json_response({"error": "Schedule is required"}, status=400)
            if len(prompt) > self._MAX_PROMPT_LENGTH:
                return web.json_response(
                    {"error": f"Prompt must be ≤ {self._MAX_PROMPT_LENGTH} characters"}, status=400,
                )
            if repeat is not None and (not isinstance(repeat, int) or repeat < 1):
                return web.json_response({"error": "Repeat must be a positive integer"}, status=400)

            kwargs = {
                "prompt": prompt,
                "schedule": schedule,
                "name": name,
                "deliver": deliver,
            }
            if skills:
                kwargs["skills"] = skills
            if repeat is not None:
                kwargs["repeat"] = repeat

            job = self._cron_create(**kwargs)
            return web.json_response({"job": job})
        except Exception as e:
            return web.json_response({"error": str(e)}, status=500)

    async def _handle_get_job(self, request: "web.Request") -> "web.Response":
        """GET /api/jobs/{job_id} — get a single cron job."""
        auth_err = self._check_auth(request)
        if auth_err:
            return auth_err
        cron_err = self._check_jobs_available()
        if cron_err:
            return cron_err
        job_id, id_err = self._check_job_id(request)
        if id_err:
            return id_err
        try:
            job = self._cron_get(job_id)
            if not job:
                return web.json_response({"error": "Job not found"}, status=404)
            return web.json_response({"job": job})
        except Exception as e:
            return web.json_response({"error": str(e)}, status=500)

    async def _handle_update_job(self, request: "web.Request") -> "web.Response":
        """PATCH /api/jobs/{job_id} — update a cron job."""
        auth_err = self._check_auth(request)
        if auth_err:
            return auth_err
        cron_err = self._check_jobs_available()
        if cron_err:
            return cron_err
        job_id, id_err = self._check_job_id(request)
        if id_err:
            return id_err
        try:
            body = await request.json()
            # Whitelist allowed fields to prevent arbitrary key injection
            sanitized = {k: v for k, v in body.items() if k in self._UPDATE_ALLOWED_FIELDS}
            if not sanitized:
                return web.json_response({"error": "No valid fields to update"}, status=400)
            # Validate lengths if present
            if "name" in sanitized and len(sanitized["name"]) > self._MAX_NAME_LENGTH:
                return web.json_response(
                    {"error": f"Name must be ≤ {self._MAX_NAME_LENGTH} characters"}, status=400,
                )
            if "prompt" in sanitized and len(sanitized["prompt"]) > self._MAX_PROMPT_LENGTH:
                return web.json_response(
                    {"error": f"Prompt must be ≤ {self._MAX_PROMPT_LENGTH} characters"}, status=400,
                )
            job = self._cron_update(job_id, sanitized)
            if not job:
                return web.json_response({"error": "Job not found"}, status=404)
            return web.json_response({"job": job})
        except Exception as e:
            return web.json_response({"error": str(e)}, status=500)

    async def _handle_delete_job(self, request: "web.Request") -> "web.Response":
        """DELETE /api/jobs/{job_id} — delete a cron job."""
        auth_err = self._check_auth(request)
        if auth_err:
            return auth_err
        cron_err = self._check_jobs_available()
        if cron_err:
            return cron_err
        job_id, id_err = self._check_job_id(request)
        if id_err:
            return id_err
        try:
            success = self._cron_remove(job_id)
            if not success:
                return web.json_response({"error": "Job not found"}, status=404)
            return web.json_response({"ok": True})
        except Exception as e:
            return web.json_response({"error": str(e)}, status=500)

    async def _handle_pause_job(self, request: "web.Request") -> "web.Response":
        """POST /api/jobs/{job_id}/pause — pause a cron job."""
        auth_err = self._check_auth(request)
        if auth_err:
            return auth_err
        cron_err = self._check_jobs_available()
        if cron_err:
            return cron_err
        job_id, id_err = self._check_job_id(request)
        if id_err:
            return id_err
        try:
            job = self._cron_pause(job_id)
            if not job:
                return web.json_response({"error": "Job not found"}, status=404)
            return web.json_response({"job": job})
        except Exception as e:
            return web.json_response({"error": str(e)}, status=500)

    async def _handle_resume_job(self, request: "web.Request") -> "web.Response":
        """POST /api/jobs/{job_id}/resume — resume a paused cron job."""
        auth_err = self._check_auth(request)
        if auth_err:
            return auth_err
        cron_err = self._check_jobs_available()
        if cron_err:
            return cron_err
        job_id, id_err = self._check_job_id(request)
        if id_err:
            return id_err
        try:
            job = self._cron_resume(job_id)
            if not job:
                return web.json_response({"error": "Job not found"}, status=404)
            return web.json_response({"job": job})
        except Exception as e:
            return web.json_response({"error": str(e)}, status=500)

    async def _handle_run_job(self, request: "web.Request") -> "web.Response":
        """POST /api/jobs/{job_id}/run — trigger immediate execution."""
        auth_err = self._check_auth(request)
        if auth_err:
            return auth_err
        cron_err = self._check_jobs_available()
        if cron_err:
            return cron_err
        job_id, id_err = self._check_job_id(request)
        if id_err:
            return id_err
        try:
            job = self._cron_trigger(job_id)
            if not job:
                return web.json_response({"error": "Job not found"}, status=404)
            return web.json_response({"job": job})
        except Exception as e:
            return web.json_response({"error": str(e)}, status=500)

    # ------------------------------------------------------------------
    # Output extraction helper
    # ------------------------------------------------------------------

    @staticmethod
    def _extract_output_items(result: Dict[str, Any]) -> List[Dict[str, Any]]:
        """
        Build the full output item array from the agent's messages.

        Walks *result["messages"]* and emits:
        - ``function_call`` items for each tool_call on assistant messages
        - ``function_call_output`` items for each tool-role message
        - a final ``message`` item with the assistant's text reply
        """
        items: List[Dict[str, Any]] = []
        messages = result.get("messages", [])

        for msg in messages:
            role = msg.get("role")
            if role == "assistant" and msg.get("tool_calls"):
                for tc in msg["tool_calls"]:
                    func = tc.get("function", {})
                    items.append({
                        "type": "function_call",
                        "name": func.get("name", ""),
                        "arguments": func.get("arguments", ""),
                        "call_id": tc.get("id", ""),
                    })
            elif role == "tool":
                items.append({
                    "type": "function_call_output",
                    "call_id": msg.get("tool_call_id", ""),
                    "output": msg.get("content", ""),
                })

        # Final assistant message
        final = result.get("final_response", "")
        if not final:
            final = result.get("error", "(No response generated)")

        items.append({
            "type": "message",
            "role": "assistant",
            "content": [
                {
                    "type": "output_text",
                    "text": final,
                }
            ],
        })
        return items

    # ------------------------------------------------------------------
    # Agent execution
    # ------------------------------------------------------------------

    async def _run_agent(
        self,
        user_message: str,
        conversation_history: List[Dict[str, str]],
        ephemeral_system_prompt: Optional[str] = None,
        session_id: Optional[str] = None,
        stream_delta_callback=None,
    ) -> tuple:
        """
        Create an agent and run a conversation in a thread executor.

        Returns ``(result_dict, usage_dict)`` where *usage_dict* contains
        ``input_tokens``, ``output_tokens`` and ``total_tokens``.
        """
        loop = asyncio.get_event_loop()

        def _run():
            agent = self._create_agent(
                ephemeral_system_prompt=ephemeral_system_prompt,
                session_id=session_id,
                stream_delta_callback=stream_delta_callback,
            )
            result = agent.run_conversation(
                user_message=user_message,
                conversation_history=conversation_history,
            )
            usage = {
                "input_tokens": getattr(agent, "session_prompt_tokens", 0) or 0,
                "output_tokens": getattr(agent, "session_completion_tokens", 0) or 0,
                "total_tokens": getattr(agent, "session_total_tokens", 0) or 0,
            }
            return result, usage

        return await loop.run_in_executor(None, _run)

    # ------------------------------------------------------------------
    # Dashboard API endpoints
    # ------------------------------------------------------------------

    async def _handle_list_tools(self, request: "web.Request") -> "web.Response":
        """GET /v1/tools — list registered tools and their availability."""
        auth_err = self._check_auth(request)
        if auth_err:
            return auth_err
        try:
            from tools.registry import registry
            toolsets = registry.get_available_toolsets()
            return web.json_response({"object": "list", "data": toolsets})
        except Exception as e:
            logger.error("Error listing tools: %s", e)
            return web.json_response({"object": "list", "data": {}})

    async def _handle_list_sessions(self, request: "web.Request") -> "web.Response":
        """GET /v1/sessions — list recent sessions from SessionDB."""
        auth_err = self._check_auth(request)
        if auth_err:
            return auth_err
        try:
            from hermes_state import SessionDB
            db = SessionDB()
            q = request.query.get("q", "")
            limit = min(int(request.query.get("limit", "50")), 200)
            if q:
                results = db.search(q, limit=limit)
            else:
                results = db.recent(limit=limit)
            return web.json_response({"object": "list", "data": results})
        except Exception as e:
            logger.error("Error listing sessions: %s", e)
            return web.json_response({"object": "list", "data": []})

    async def _handle_list_processes(self, request: "web.Request") -> "web.Response":
        """GET /v1/processes — list tracked background processes."""
        auth_err = self._check_auth(request)
        if auth_err:
            return auth_err
        try:
            from tools.process_registry import ProcessRegistry
            reg = ProcessRegistry()
            running = []
            with reg._lock:
                for sid, ps in reg._running.items():
                    running.append({
                        "id": ps.id, "command": ps.command,
                        "pid": ps.pid, "started_at": ps.started_at,
                        "exited": ps.exited, "exit_code": ps.exit_code,
                    })
                for sid, ps in reg._finished.items():
                    running.append({
                        "id": ps.id, "command": ps.command,
                        "pid": ps.pid, "started_at": ps.started_at,
                        "exited": ps.exited, "exit_code": ps.exit_code,
                    })
            return web.json_response({"object": "list", "data": running})
        except Exception as e:
            logger.error("Error listing processes: %s", e)
            return web.json_response({"object": "list", "data": []})

    async def _handle_list_skills(self, request: "web.Request") -> "web.Response":
        """GET /v1/skills — list installed skills."""
        auth_err = self._check_auth(request)
        if auth_err:
            return auth_err
        try:
            import pathlib
            hermes_home = os.environ.get("HERMES_HOME", os.path.expanduser("~/.hermes"))
            skills_dir = pathlib.Path(hermes_home) / "skills"
            skills = []
            if skills_dir.is_dir():
                for p in sorted(skills_dir.iterdir()):
                    if p.is_dir() and not p.name.startswith("."):
                        desc = ""
                        readme = p / "README.md"
                        if readme.exists():
                            desc = readme.read_text(encoding="utf-8", errors="replace").split("\n")[0][:200]
                        skills.append({"name": p.name, "description": desc, "path": str(p)})
            return web.json_response({"object": "list", "data": skills})
        except Exception as e:
            logger.error("Error listing skills: %s", e)
            return web.json_response({"object": "list", "data": []})

    async def _handle_health_metrics(self, request: "web.Request") -> "web.Response":
        """GET /v1/health/metrics — system health metrics for dashboard."""
        auth_err = self._check_auth(request)
        if auth_err:
            return auth_err
        try:
            import platform as plat
            data = {
                "uptime": {"value": round(time.time() - self._start_time, 1), "unit": "seconds"},
                "platform": {"value": plat.system() + " " + plat.release()},
                "python": {"value": plat.python_version()},
                "host": {"value": self._host + ":" + str(self._port)},
            }
            try:
                import psutil
                data["cpu"] = {"value": psutil.cpu_percent(interval=0), "unit": "%"}
                mem = psutil.virtual_memory()
                data["memory"] = {"value": round(mem.used / 1e9, 1), "unit": "GB / " + str(round(mem.total / 1e9, 1)) + " GB"}
            except ImportError:
                pass
            return web.json_response({"object": "metrics", "data": data})
        except Exception as e:
            logger.error("Error getting health metrics: %s", e)
            return web.json_response({"object": "metrics", "data": {}})

    # ------------------------------------------------------------------
    # Agent-mail & Beads API
    # ------------------------------------------------------------------

    async def _handle_agent_mail_send(self, request: "web.Request") -> "web.Response":
        """POST /v1/agent-mail — Send a message between agents."""
        body = await request.json()
        from_agent = body.get("from", "")
        to_agent = body.get("to", "")
        subject = body.get("subject", "")
        message_body = body.get("body", "")
        priority = body.get("priority", "normal")
        thread_id = body.get("thread_id")
        if not from_agent or not to_agent or not message_body:
            return web.json_response(
                {"error": "from, to, and body are required"}, status=400
            )
        import uuid
        mail_id = str(uuid.uuid4())
        mail_entry = {
            "id": mail_id,
            "from": from_agent,
            "to": to_agent,
            "subject": subject,
            "body": message_body,
            "priority": priority,
            "thread_id": thread_id or mail_id,
            "status": "pending",
            "created_at": time.time(),
        }
        if not hasattr(self, "_agent_mail"):
            self._agent_mail = []
        self._agent_mail.append(mail_entry)
        return web.json_response({"object": "agent_mail", "id": mail_id, "status": "sent"})

    async def _handle_agent_mail_inbox(self, request: "web.Request") -> "web.Response":
        """GET /v1/agent-mail/{agent} — Fetch inbox for an agent."""
        agent_name = request.match_info["agent"]
        status_filter = request.query.get("status", "pending")
        inbox = getattr(self, "_agent_mail", [])
        messages = [
            m for m in inbox
            if m["to"] == agent_name and m["status"] == status_filter
        ]
        return web.json_response({"object": "list", "data": messages})

    async def _handle_agent_mail_ack(self, request: "web.Request") -> "web.Response":
        """POST /v1/agent-mail/{mail_id}/ack — Acknowledge a message."""
        mail_id = request.match_info["mail_id"]
        inbox = getattr(self, "_agent_mail", [])
        for m in inbox:
            if m["id"] == mail_id:
                m["status"] = "acknowledged"
                return web.json_response({"object": "agent_mail", "id": mail_id, "status": "acknowledged"})
        return web.json_response({"error": "message not found"}, status=404)

    async def _handle_beads_list(self, request: "web.Request") -> "web.Response":
        """GET /v1/beads — List beads (action/issue log)."""
        status_filter = request.query.get("status")
        limit = int(request.query.get("limit", "50"))

        # Try Supabase first
        if self._supabase_url and self._supabase_key:
            try:
                params = f"order=created_at.desc&limit={limit}"
                if status_filter:
                    params += f"&status=eq.{status_filter}"
                async with web.session() if False else \
                        __import__("aiohttp").ClientSession() as sess:
                    async with sess.get(
                        f"{self._supabase_url}/rest/v1/beads?{params}",
                        headers={"apikey": self._supabase_key, "Authorization": f"Bearer {self._supabase_key}"},
                    ) as resp:
                        if resp.status == 200:
                            data = await resp.json()
                            return web.json_response({"object": "list", "data": data})
            except Exception as exc:
                logger.debug("Supabase beads fetch failed, using in-memory: %s", exc)

        beads = self._beads_log
        if status_filter:
            beads = [b for b in beads if b.get("status") == status_filter]
        return web.json_response({"object": "list", "data": beads[-limit:]})

    async def _handle_beads_create(self, request: "web.Request") -> "web.Response":
        """POST /v1/beads — Create a new bead entry."""
        body = await request.json()
        bead = {
            "id": str(uuid.uuid4()),
            "agent": body.get("agent", "hermes"),
            "bead_type": body.get("type", "action"),
            "title": body.get("title", ""),
            "description": body.get("description", ""),
            "status": body.get("status", "open"),
            "metadata": body.get("metadata", {}),
            "created_at": time.time(),
        }
        if not bead["title"]:
            return web.json_response({"error": "title is required"}, status=400)

        # Try Supabase first
        if self._supabase_url and self._supabase_key:
            try:
                import aiohttp as _aiohttp
                async with _aiohttp.ClientSession() as sess:
                    async with sess.post(
                        f"{self._supabase_url}/rest/v1/beads",
                        headers={
                            "apikey": self._supabase_key,
                            "Authorization": f"Bearer {self._supabase_key}",
                            "Content-Type": "application/json",
                            "Prefer": "return=representation",
                        },
                        json={
                            "agent_id": None,
                            "bead_type": bead["bead_type"],
                            "title": bead["title"],
                            "description": bead["description"] or None,
                            "status": bead["status"],
                            "metadata": bead["metadata"],
                        },
                    ) as resp:
                        if resp.status in (200, 201):
                            row = (await resp.json())[0]
                            return web.json_response({"object": "bead", **row})
            except Exception as exc:
                logger.debug("Supabase bead insert failed, using in-memory: %s", exc)

        self._beads_log.append(bead)
        return web.json_response({"object": "bead", **bead})

    async def _handle_beads_update(self, request: "web.Request") -> "web.Response":
        """PATCH /v1/beads/{bead_id} — Update a bead's status."""
        bead_id = request.match_info["bead_id"]
        body = await request.json()

        # Try Supabase first
        if self._supabase_url and self._supabase_key:
            try:
                import aiohttp as _aiohttp
                patch = {k: body[k] for k in ("status", "title", "description", "metadata") if k in body}
                async with _aiohttp.ClientSession() as sess:
                    async with sess.patch(
                        f"{self._supabase_url}/rest/v1/beads?id=eq.{bead_id}",
                        headers={
                            "apikey": self._supabase_key,
                            "Authorization": f"Bearer {self._supabase_key}",
                            "Content-Type": "application/json",
                            "Prefer": "return=representation",
                        },
                        json=patch,
                    ) as resp:
                        if resp.status == 200:
                            rows = await resp.json()
                            if rows:
                                return web.json_response({"object": "bead", **rows[0]})
            except Exception as exc:
                logger.debug("Supabase bead update failed, using in-memory: %s", exc)

        for b in self._beads_log:
            if b["id"] == bead_id:
                for key in ("status", "title", "description", "metadata"):
                    if key in body:
                        b[key] = body[key]
                if body.get("status") == "resolved":
                    b["resolved_at"] = time.time()
                return web.json_response({"object": "bead", **b})
        return web.json_response({"error": "bead not found"}, status=404)

    async def _handle_list_agents(self, request: "web.Request") -> "web.Response":
        """GET /v1/agents — List registered agents from agents/registry.yaml."""
        import yaml as _yaml
        from pathlib import Path as _Path
        registry_path = _Path(__file__).parent.parent.parent / "agents" / "registry.yaml"
        agents = []
        if registry_path.is_file():
            try:
                data = _yaml.safe_load(registry_path.read_text(encoding="utf-8"))
                agents = data.get("agents", [])
                # Mark hermes as online since we're serving
                for a in agents:
                    if a.get("name") == "hermes":
                        a["status"] = "online"
                        a["endpoint"] = f"http://{self._host}:{self._port}"
            except Exception:
                pass
        if not agents:
            agents = [
                {"name": "hermes", "display_name": "Hermes", "endpoint": f"http://{self._host}:{self._port}", "type": "hermes", "status": "online", "role": "Business orchestrator", "capabilities": ["coding", "research", "deployment"]},
                {"name": "agent-zero", "display_name": "Agent Zero", "endpoint": "http://localhost:8643", "type": "zero", "status": "offline", "role": "Personal assistant", "capabilities": ["personal_memory"]},
            ]
        return web.json_response({"object": "list", "agents": agents})

    async def _handle_list_tasks(self, request: "web.Request") -> "web.Response":
        """GET /v1/tasks — List tasks (from beads log, filtered to actionable items)."""
        tasks = []
        for b in getattr(self, "_beads_log", []):
            if b.get("type") in ("action", "milestone") or b.get("status") in ("open", "in_progress"):
                tasks.append({
                    "id": b.get("id", ""),
                    "title": b.get("title", ""),
                    "status": b.get("status", "open"),
                    "agent": b.get("agent", ""),
                    "created_at": b.get("created_at", ""),
                })
        return web.json_response({"object": "list", "tasks": tasks})

    async def _handle_search_memories(self, request: "web.Request") -> "web.Response":
        """GET /v1/memories — Search Open Brain memories via Supabase REST."""
        query = request.query.get("query", "")
        limit = min(int(request.query.get("limit", "20")), 50)
        supa_url = os.environ.get("SUPABASE_URL", "").rstrip("/")
        supa_key = os.environ.get("SUPABASE_KEY", "")
        if not supa_url or not supa_key:
            return web.json_response({"memories": [], "error": "SUPABASE_URL/KEY not configured"})
        params = {
            "select": "id,title,content,collection,tags,created_at",
            "order": "created_at.desc",
            "limit": str(limit),
        }
        if query:
            params["or"] = f"(title.ilike.*{query}*,content.ilike.*{query}*)"
        headers = {"apikey": supa_key, "Authorization": f"Bearer {supa_key}"}
        try:
            async with aiohttp.ClientSession() as session:
                async with session.get(f"{supa_url}/rest/v1/memories", headers=headers, params=params, timeout=aiohttp.ClientTimeout(total=10)) as resp:
                    data = await resp.json()
                    return web.json_response({"memories": data if isinstance(data, list) else []})
        except Exception as e:
            return web.json_response({"memories": [], "error": str(e)})

    async def _handle_create_memory(self, request: "web.Request") -> "web.Response":
        """POST /v1/memories — Store a new memory in Open Brain."""
        body = await request.json()
        supa_url = os.environ.get("SUPABASE_URL", "").rstrip("/")
        supa_key = os.environ.get("SUPABASE_KEY", "")
        if not supa_url or not supa_key:
            return web.json_response({"error": "SUPABASE_URL/KEY not configured"}, status=500)
        payload = {
            "content": body.get("content", ""),
            "title": body.get("title", body.get("content", "")[:80]),
            "collection": body.get("collection", "general"),
            "tags": body.get("tags", []),
            "links": body.get("links", []),
        }
        headers = {"apikey": supa_key, "Authorization": f"Bearer {supa_key}", "Content-Type": "application/json", "Prefer": "return=representation"}
        try:
            async with aiohttp.ClientSession() as session:
                async with session.post(f"{supa_url}/rest/v1/memories", headers=headers, json=payload, timeout=aiohttp.ClientTimeout(total=10)) as resp:
                    data = await resp.json()
                    return web.json_response({"success": True, "memory": data[0] if isinstance(data, list) and data else data})
        except Exception as e:
            return web.json_response({"success": False, "error": str(e)}, status=500)

    async def _handle_list_deploys(self, request: "web.Request") -> "web.Response":
        """GET /v1/deploys — List known deployments (static + dynamic)."""
        deploys = getattr(self, "_deploys_log", [
            {"name": "pauli-hermes-agent.vercel.app", "environment": "production", "status": "ready", "platform": "vercel", "created_at": ""},
            {"name": f"VPS {self._host}:{self._port}", "environment": "vps", "status": "ready", "platform": "docker", "created_at": ""},
        ])
        return web.json_response({"object": "list", "deploys": deploys})

    async def _handle_fetch_secret(self, request: "web.Request") -> "web.Response":
        """GET /v1/secrets/{secret_path} — Fetch a secret from Infisical."""
        try:
            from tools.infisical_tool import fetch_secret
            from model_tools import handle_function_call
        except ImportError:
            return web.json_response(
                {"success": False, "error": "Infisical tool not available"}
                , status=503
            )

        secret_path = request.match_info.get("secret_path", "")
        environment = request.rel_url.query.get("environment", "production")

        if not secret_path:
            return web.json_response(
                {"success": False, "error": "Missing secret_path parameter"},
                status=400
            )

        try:
            # Call the tool through the registry
            result_json = handle_function_call(
                "fetch_secret",
                {"secret_path": secret_path, "environment": environment},
                task_id=None
            )
            result = json.loads(result_json)
            
            if result.get("success"):
                return web.json_response(result)
            else:
                status_code = result.get("code", 500)
                return web.json_response(result, status=status_code)

        except Exception as e:
            return web.json_response(
                {"success": False, "error": f"Error fetching secret: {str(e)[:100]}"},
                status=500
            )

    async def _handle_cache_status(self, request: "web.Request") -> "web.Response":
        """GET /v1/cache-status — Get Infisical cache status and TTL info."""
        try:
            from tools.infisical_tool import get_cache_status
            from model_tools import handle_function_call
        except ImportError:
            return web.json_response(
                {"success": False, "error": "Infisical tool not available"},
                status=503
            )

        try:
            result_json = handle_function_call(
                "get_cache_status",
                {},
                task_id=None
            )
            result = json.loads(result_json)
            return web.json_response(result)

        except Exception as e:
            return web.json_response(
                {"success": False, "error": f"Error getting cache status: {str(e)[:100]}"},
                status=500
            )

    async def _handle_rotate_secrets(self, request: "web.Request") -> "web.Response":
        """POST /v1/rotate-secrets — Clear local cache and force fresh fetches."""
        try:
            from tools.infisical_tool import rotate_secrets
            from model_tools import handle_function_call
        except ImportError:
            return web.json_response(
                {"success": False, "error": "Infisical tool not available"},
                status=503
            )

        try:
            result_json = handle_function_call(
                "rotate_secrets",
                {},
                task_id=None
            )
            result = json.loads(result_json)
            return web.json_response(result)

        except Exception as e:
            return web.json_response(
                {"success": False, "error": f"Error rotating secrets: {str(e)[:100]}"},
                status=500
            )

    # ------------------------------------------------------------------
    # BasePlatformAdapter interface
    # ------------------------------------------------------------------

    async def connect(self) -> bool:
        """Start the aiohttp web server."""
        if not AIOHTTP_AVAILABLE:
            logger.warning("[%s] aiohttp not installed", self.name)
            return False

        try:
            self._app = web.Application(middlewares=[cors_middleware])
            self._app["api_server_adapter"] = self
            self._app.router.add_get("/health", self._handle_health)
            self._app.router.add_get("/v1/models", self._handle_models)
            self._app.router.add_post("/v1/chat/completions", self._handle_chat_completions)
            self._app.router.add_post("/v1/responses", self._handle_responses)
            self._app.router.add_get("/v1/responses/{response_id}", self._handle_get_response)
            self._app.router.add_delete("/v1/responses/{response_id}", self._handle_delete_response)
            # Cron jobs management API
            self._app.router.add_get("/api/jobs", self._handle_list_jobs)
            self._app.router.add_post("/api/jobs", self._handle_create_job)
            self._app.router.add_get("/api/jobs/{job_id}", self._handle_get_job)
            self._app.router.add_patch("/api/jobs/{job_id}", self._handle_update_job)
            self._app.router.add_delete("/api/jobs/{job_id}", self._handle_delete_job)
            self._app.router.add_post("/api/jobs/{job_id}/pause", self._handle_pause_job)
            self._app.router.add_post("/api/jobs/{job_id}/resume", self._handle_resume_job)
            self._app.router.add_post("/api/jobs/{job_id}/run", self._handle_run_job)
            # Dashboard API
            self._app.router.add_get("/v1/tools", self._handle_list_tools)
            self._app.router.add_get("/v1/sessions", self._handle_list_sessions)
            self._app.router.add_get("/v1/processes", self._handle_list_processes)
            self._app.router.add_get("/v1/skills", self._handle_list_skills)
            self._app.router.add_get("/v1/health/metrics", self._handle_health_metrics)
            # Agent-mail & Beads API
            self._app.router.add_post("/v1/agent-mail", self._handle_agent_mail_send)
            self._app.router.add_get("/v1/agent-mail/{agent}", self._handle_agent_mail_inbox)
            self._app.router.add_post("/v1/agent-mail/{mail_id}/ack", self._handle_agent_mail_ack)
            self._app.router.add_get("/v1/beads", self._handle_beads_list)
            self._app.router.add_post("/v1/beads", self._handle_beads_create)
            self._app.router.add_patch("/v1/beads/{bead_id}", self._handle_beads_update)
            self._app.router.add_get("/v1/agents", self._handle_list_agents)
            self._app.router.add_get("/v1/tasks", self._handle_list_tasks)
            self._app.router.add_get("/v1/memories", self._handle_search_memories)
            self._app.router.add_post("/v1/memories", self._handle_create_memory)
            self._app.router.add_get("/v1/deploys", self._handle_list_deploys)
            # Infisical secrets management API
            self._app.router.add_get("/v1/secrets/{secret_path}", self._handle_fetch_secret)
            self._app.router.add_get("/v1/cache-status", self._handle_cache_status)
            self._app.router.add_post("/v1/rotate-secrets", self._handle_rotate_secrets)

            self._runner = web.AppRunner(self._app)
            await self._runner.setup()
            self._site = web.TCPSite(self._runner, self._host, self._port)
            await self._site.start()

            self._mark_connected()
            logger.info(
                "[%s] API server listening on http://%s:%d",
                self.name, self._host, self._port,
            )
            return True

        except Exception as e:
            logger.error("[%s] Failed to start API server: %s", self.name, e)
            return False

    async def disconnect(self) -> None:
        """Stop the aiohttp web server."""
        self._mark_disconnected()
        if self._site:
            await self._site.stop()
            self._site = None
        if self._runner:
            await self._runner.cleanup()
            self._runner = None
        self._app = None
        logger.info("[%s] API server stopped", self.name)

    async def send(
        self,
        chat_id: str,
        content: str,
        reply_to: Optional[str] = None,
        metadata: Optional[Dict[str, Any]] = None,
    ) -> SendResult:
        """
        Not used — HTTP request/response cycle handles delivery directly.
        """
        return SendResult(success=False, error="API server uses HTTP request/response, not send()")

    async def get_chat_info(self, chat_id: str) -> Dict[str, Any]:
        """Return basic info about the API server."""
        return {
            "name": "API Server",
            "type": "api",
            "host": self._host,
            "port": self._port,
        }
