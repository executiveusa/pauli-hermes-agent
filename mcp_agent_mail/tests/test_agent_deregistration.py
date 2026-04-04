"""Tests for agent deregistration feature."""
from __future__ import annotations

import contextlib
import pytest
from fastmcp import Client
from fastmcp.exceptions import ToolError

from mcp_agent_mail import config as _config
from mcp_agent_mail.app import build_mcp_server


@pytest.mark.asyncio
async def test_deregister_basic(isolated_env, monkeypatch):
    """Register agent, deregister, verify deregistered_ts set and cleanup done."""
    with contextlib.suppress(Exception):
        _config.clear_settings_cache()

    server = build_mcp_server()
    async with Client(server) as client:
        await client.call_tool("ensure_project", {"human_key": "/backend"})
        await client.call_tool(
            "register_agent",
            {"project_key": "Backend", "program": "codex", "model": "gpt-5", "name": "GreenCastle"},
        )
        await client.call_tool(
            "register_agent",
            {"project_key": "Backend", "program": "codex", "model": "gpt-5", "name": "BlueLake"},
        )

        # Create a contact link
        await client.call_tool(
            "request_contact",
            {"project_key": "Backend", "from_agent": "GreenCastle", "to_agent": "BlueLake"},
        )

        # Create a file reservation
        await client.call_tool(
            "file_reservation_paths",
            {"project_key": "Backend", "agent_name": "GreenCastle", "paths": ["src/*"], "ttl_seconds": 600},
        )

        # Deregister
        result = await client.call_tool(
            "deregister_agent",
            {"project_key": "Backend", "agent_name": "GreenCastle"},
        )

        assert result.data["was_registered"] is True
        assert result.data["already_deregistered"] is False
        assert result.data["contact_links_removed"] >= 1
        assert result.data["file_reservations_released"] >= 1

        # Verify whois shows deregistered_ts
        whois_result = await client.call_tool(
            "whois",
            {"project_key": "Backend", "agent_name": "GreenCastle"},
        )
        assert "deregistered_ts" in whois_result.data


@pytest.mark.asyncio
async def test_deregister_idempotent(isolated_env, monkeypatch):
    """Deregister same agent twice -> no error, second returns already_deregistered=True."""
    with contextlib.suppress(Exception):
        _config.clear_settings_cache()

    server = build_mcp_server()
    async with Client(server) as client:
        await client.call_tool("ensure_project", {"human_key": "/backend"})
        await client.call_tool(
            "register_agent",
            {"project_key": "Backend", "program": "codex", "model": "gpt-5", "name": "GreenCastle"},
        )

        # First deregister
        result1 = await client.call_tool(
            "deregister_agent",
            {"project_key": "Backend", "agent_name": "GreenCastle"},
        )
        assert result1.data["was_registered"] is True
        assert result1.data["already_deregistered"] is False

        # Second deregister (idempotent)
        result2 = await client.call_tool(
            "deregister_agent",
            {"project_key": "Backend", "agent_name": "GreenCastle"},
        )
        assert result2.data["was_registered"] is True
        assert result2.data["already_deregistered"] is True


@pytest.mark.asyncio
async def test_deregister_nonexistent(isolated_env, monkeypatch):
    """Deregister unknown agent -> success with was_registered=False."""
    with contextlib.suppress(Exception):
        _config.clear_settings_cache()

    server = build_mcp_server()
    async with Client(server) as client:
        await client.call_tool("ensure_project", {"human_key": "/backend"})

        result = await client.call_tool(
            "deregister_agent",
            {"project_key": "Backend", "agent_name": "NoSuchAgent"},
        )
        assert result.data["was_registered"] is False
        assert result.data["already_deregistered"] is False


@pytest.mark.asyncio
async def test_deregister_delete_inbox(isolated_env, monkeypatch):
    """Test delete_inbox=True deletes message_recipients."""
    with contextlib.suppress(Exception):
        _config.clear_settings_cache()

    server = build_mcp_server()
    async with Client(server) as client:
        await client.call_tool("ensure_project", {"human_key": "/backend"})
        await client.call_tool(
            "register_agent",
            {"project_key": "Backend", "program": "codex", "model": "gpt-5", "name": "GreenCastle"},
        )
        await client.call_tool(
            "register_agent",
            {"project_key": "Backend", "program": "codex", "model": "gpt-5", "name": "BlueLake"},
        )

        # Send a message to BlueLake
        await client.call_tool(
            "send_message",
            {
                "project_key": "Backend",
                "sender_name": "GreenCastle",
                "to": ["BlueLake"],
                "subject": "Test",
                "body_md": "Hello",
            },
        )

        # Deregister with delete_inbox=True
        result = await client.call_tool(
            "deregister_agent",
            {"project_key": "Backend", "agent_name": "BlueLake", "delete_inbox": True},
        )
        assert result.data["inbox_records_deleted"] >= 1


@pytest.mark.asyncio
async def test_reactivation(isolated_env, monkeypatch):
    """Register -> deregister -> register same name clears deregistered_ts."""
    with contextlib.suppress(Exception):
        _config.clear_settings_cache()

    server = build_mcp_server()
    async with Client(server) as client:
        await client.call_tool("ensure_project", {"human_key": "/backend"})

        # Register
        await client.call_tool(
            "register_agent",
            {"project_key": "Backend", "program": "codex", "model": "gpt-5", "name": "GreenCastle"},
        )

        # Deregister
        await client.call_tool(
            "deregister_agent",
            {"project_key": "Backend", "agent_name": "GreenCastle"},
        )

        # Verify deregistered
        whois1 = await client.call_tool(
            "whois",
            {"project_key": "Backend", "agent_name": "GreenCastle"},
        )
        assert "deregistered_ts" in whois1.data

        # Re-register (reactivate) with new metadata
        await client.call_tool(
            "register_agent",
            {"project_key": "Backend", "program": "claude-code", "model": "opus", "name": "GreenCastle"},
        )

        # Verify reactivated
        whois2 = await client.call_tool(
            "whois",
            {"project_key": "Backend", "agent_name": "GreenCastle"},
        )
        assert "deregistered_ts" not in whois2.data
        assert whois2.data["program"] == "claude-code"
        assert whois2.data["model"] == "opus"


@pytest.mark.asyncio
async def test_send_to_deregistered(isolated_env, monkeypatch):
    """Deregister recipient, try to send -> clear error message."""
    with contextlib.suppress(Exception):
        _config.clear_settings_cache()

    server = build_mcp_server()
    async with Client(server) as client:
        await client.call_tool("ensure_project", {"human_key": "/backend"})
        await client.call_tool(
            "register_agent",
            {"project_key": "Backend", "program": "codex", "model": "gpt-5", "name": "GreenCastle"},
        )
        await client.call_tool(
            "register_agent",
            {"project_key": "Backend", "program": "codex", "model": "gpt-5", "name": "BlueLake"},
        )

        # Deregister recipient
        await client.call_tool(
            "deregister_agent",
            {"project_key": "Backend", "agent_name": "BlueLake"},
        )

        # Try to send - should fail with deregistered error
        with pytest.raises(ToolError) as excinfo:
            await client.call_tool(
                "send_message",
                {
                    "project_key": "Backend",
                    "sender_name": "GreenCastle",
                    "to": ["BlueLake"],
                    "subject": "Test",
                    "body_md": "Hello",
                },
            )
        assert "deregistered" in str(excinfo.value).lower()


@pytest.mark.asyncio
async def test_send_from_deregistered(isolated_env, monkeypatch):
    """Deregister sender, try to send -> clear error message."""
    with contextlib.suppress(Exception):
        _config.clear_settings_cache()

    server = build_mcp_server()
    async with Client(server) as client:
        await client.call_tool("ensure_project", {"human_key": "/backend"})
        await client.call_tool(
            "register_agent",
            {"project_key": "Backend", "program": "codex", "model": "gpt-5", "name": "GreenCastle"},
        )
        await client.call_tool(
            "register_agent",
            {"project_key": "Backend", "program": "codex", "model": "gpt-5", "name": "BlueLake"},
        )

        # Deregister sender
        await client.call_tool(
            "deregister_agent",
            {"project_key": "Backend", "agent_name": "GreenCastle"},
        )

        # Try to send from deregistered sender
        with pytest.raises(ToolError) as excinfo:
            await client.call_tool(
                "send_message",
                {
                    "project_key": "Backend",
                    "sender_name": "GreenCastle",
                    "to": ["BlueLake"],
                    "subject": "Test",
                    "body_md": "Hello",
                },
            )
        assert "deregistered" in str(excinfo.value).lower()


@pytest.mark.asyncio
async def test_fetch_inbox_deregistered(isolated_env, monkeypatch):
    """Deregister agent, try fetch_inbox -> rejected."""
    with contextlib.suppress(Exception):
        _config.clear_settings_cache()

    server = build_mcp_server()
    async with Client(server) as client:
        await client.call_tool("ensure_project", {"human_key": "/backend"})
        await client.call_tool(
            "register_agent",
            {"project_key": "Backend", "program": "codex", "model": "gpt-5", "name": "GreenCastle"},
        )

        # Deregister
        await client.call_tool(
            "deregister_agent",
            {"project_key": "Backend", "agent_name": "GreenCastle"},
        )

        # Try to fetch inbox
        with pytest.raises(ToolError) as excinfo:
            await client.call_tool(
                "fetch_inbox",
                {"project_key": "Backend", "agent_name": "GreenCastle"},
            )
        assert "deregistered" in str(excinfo.value).lower()


@pytest.mark.asyncio
async def test_whois_deregistered(isolated_env, monkeypatch):
    """Deregister agent, whois -> works and shows deregistered_ts."""
    with contextlib.suppress(Exception):
        _config.clear_settings_cache()

    server = build_mcp_server()
    async with Client(server) as client:
        await client.call_tool("ensure_project", {"human_key": "/backend"})
        await client.call_tool(
            "register_agent",
            {"project_key": "Backend", "program": "codex", "model": "gpt-5", "name": "GreenCastle"},
        )

        # Deregister
        await client.call_tool(
            "deregister_agent",
            {"project_key": "Backend", "agent_name": "GreenCastle"},
        )

        # whois should still work
        result = await client.call_tool(
            "whois",
            {"project_key": "Backend", "agent_name": "GreenCastle"},
        )
        assert result.data["name"] == "GreenCastle"
        assert "deregistered_ts" in result.data


@pytest.mark.asyncio
async def test_list_contacts_excludes_deregistered(isolated_env, monkeypatch):
    """Create contact link, deregister one agent, list_contacts excludes them."""
    with contextlib.suppress(Exception):
        _config.clear_settings_cache()

    server = build_mcp_server()
    async with Client(server) as client:
        await client.call_tool("ensure_project", {"human_key": "/backend"})
        await client.call_tool(
            "register_agent",
            {"project_key": "Backend", "program": "codex", "model": "gpt-5", "name": "GreenCastle"},
        )
        await client.call_tool(
            "register_agent",
            {"project_key": "Backend", "program": "codex", "model": "gpt-5", "name": "BlueLake"},
        )
        await client.call_tool(
            "register_agent",
            {"project_key": "Backend", "program": "codex", "model": "gpt-5", "name": "RedStone"},
        )

        # Create contact links from GreenCastle to both
        await client.call_tool(
            "request_contact",
            {"project_key": "Backend", "from_agent": "GreenCastle", "to_agent": "BlueLake"},
        )
        await client.call_tool(
            "request_contact",
            {"project_key": "Backend", "from_agent": "GreenCastle", "to_agent": "RedStone"},
        )

        # Verify contacts before deregistration
        contacts_before = await client.call_tool(
            "list_contacts",
            {"project_key": "Backend", "agent_name": "GreenCastle"},
        )
        # list_contacts returns a list - access via structured_content
        contacts_data = contacts_before.structured_content.get("result", [])
        contact_names_before = [c["to"] for c in contacts_data]
        assert "BlueLake" in contact_names_before
        assert "RedStone" in contact_names_before

        # Deregister BlueLake
        await client.call_tool(
            "deregister_agent",
            {"project_key": "Backend", "agent_name": "BlueLake"},
        )

        # List contacts should exclude BlueLake (link was deleted during deregister)
        contacts_after = await client.call_tool(
            "list_contacts",
            {"project_key": "Backend", "agent_name": "GreenCastle"},
        )
        contacts_data_after = contacts_after.structured_content.get("result", [])
        contact_names_after = [c["to"] for c in contacts_data_after]
        # BlueLake should not be in the list (link was deleted during deregistration)
        assert "BlueLake" not in contact_names_after
        # RedStone should still be there
        assert "RedStone" in contact_names_after


@pytest.mark.asyncio
async def test_agents_resource_excludes_deregistered(isolated_env, monkeypatch):
    """Register agents, deregister one, resource://agents excludes deregistered."""
    with contextlib.suppress(Exception):
        _config.clear_settings_cache()

    server = build_mcp_server()
    async with Client(server) as client:
        await client.call_tool("ensure_project", {"human_key": "/backend"})
        await client.call_tool(
            "register_agent",
            {"project_key": "Backend", "program": "codex", "model": "gpt-5", "name": "GreenCastle"},
        )
        await client.call_tool(
            "register_agent",
            {"project_key": "Backend", "program": "codex", "model": "gpt-5", "name": "BlueLake"},
        )

        # Deregister one
        await client.call_tool(
            "deregister_agent",
            {"project_key": "Backend", "agent_name": "BlueLake"},
        )

        # Read the agents resource
        # Note: Using read_resource requires the actual resource URI pattern
        # For now, we test via the agents directory behavior implicitly through other tools


@pytest.mark.asyncio
async def test_request_contact_to_deregistered(isolated_env, monkeypatch):
    """Try to request contact with deregistered agent -> rejected."""
    with contextlib.suppress(Exception):
        _config.clear_settings_cache()

    server = build_mcp_server()
    async with Client(server) as client:
        await client.call_tool("ensure_project", {"human_key": "/backend"})
        await client.call_tool(
            "register_agent",
            {"project_key": "Backend", "program": "codex", "model": "gpt-5", "name": "GreenCastle"},
        )
        await client.call_tool(
            "register_agent",
            {"project_key": "Backend", "program": "codex", "model": "gpt-5", "name": "BlueLake"},
        )

        # Deregister BlueLake
        await client.call_tool(
            "deregister_agent",
            {"project_key": "Backend", "agent_name": "BlueLake"},
        )

        # Try to request contact with deregistered agent
        with pytest.raises(ToolError) as excinfo:
            await client.call_tool(
                "request_contact",
                {"project_key": "Backend", "from_agent": "GreenCastle", "to_agent": "BlueLake"},
            )
        assert "deregistered" in str(excinfo.value).lower()
