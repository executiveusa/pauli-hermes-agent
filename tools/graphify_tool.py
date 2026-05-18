#!/usr/bin/env python3
"""
Graphify Tool Module - Semantic Knowledge Graph Traversals

Enables the agent to autonomously query and build/rebuild the semantic index
of the Obsidian Second Brain vault using Graphify (graphifyy).
"""

import os
import json
import subprocess
import sys
from pathlib import Path
from typing import Optional, Dict, Any
from tools.registry import registry, tool_error

DEFAULT_VAULT_PATH = "E:\\OBSIDIAN SECOND BRAIN"

def get_vault_path() -> str:
    """Resolve the active Obsidian Vault path, prioritizing environment variables."""
    return os.environ.get("OBSIDIAN_VAULT_PATH", DEFAULT_VAULT_PATH)

def graphify_rebuild_handler() -> str:
    """
    Manually triggers a rebuild of the Graphify semantic index of the vault.
    
    Returns:
        JSON string describing the outcome of the build.
    """
    vault_path = get_vault_path()
    print(f"Graphify Rebuild Triggered for: {vault_path}")
    
    vault_abs_path = os.path.abspath(vault_path)
    out_dir = Path(vault_abs_path) / "graphify-out"
    out_dir.mkdir(parents=True, exist_ok=True)
    
    try:
        py_exe = sys.executable or "python"
        res = subprocess.run(
            [py_exe, "-m", "graphifyy", "build", vault_abs_path, "--output", str(out_dir)],
            capture_output=True,
            text=True
        )
        if res.returncode == 0:
            return json.dumps({
                "success": True,
                "message": f"Successfully built Graphify semantic graph index of the Obsidian Vault at: {vault_abs_path}",
                "output_dir": str(out_dir)
            }, ensure_ascii=False)
        else:
            return json.dumps({
                "success": False,
                "error": f"Graphify build failed with exit code {res.returncode}",
                "details": res.stderr
            }, ensure_ascii=False)
    except Exception as e:
        return json.dumps({
            "success": False,
            "error": f"Failed to execute Graphify index builder: {str(e)}"
        }, ensure_ascii=False)

def graphify_query_handler(query: str) -> str:
    """
    Queries the Graphify semantic knowledge index.
    
    Args:
        query: Semantic/NL question to query the index with.
        
    Returns:
        JSON string with retrieved nodes and relations.
    """
    vault_path = get_vault_path()
    vault_abs_path = os.path.abspath(vault_path)
    out_dir = Path(vault_abs_path) / "graphify-out"
    
    if not out_dir.exists():
        # Auto-trigger a build if the index has not been built yet
        rebuild_res = json.loads(graphify_rebuild_handler())
        if not rebuild_res.get("success"):
            return json.dumps({
                "success": False,
                "error": "Graphify index does not exist and auto-rebuild failed.",
                "details": rebuild_res.get("error")
            }, ensure_ascii=False)

    try:
        py_exe = sys.executable or "python"
        res = subprocess.run(
            [py_exe, "-m", "graphifyy", "query", query, "--dir", vault_abs_path, "--output", str(out_dir)],
            capture_output=True,
            text=True
        )
        if res.returncode == 0:
            return json.dumps({
                "success": True,
                "query": query,
                "results": res.stdout
            }, ensure_ascii=False)
        else:
            return json.dumps({
                "success": False,
                "error": f"Graphify query failed with exit code {res.returncode}",
                "details": res.stderr
            }, ensure_ascii=False)
    except Exception as e:
        return json.dumps({
            "success": False,
            "error": f"Failed to execute Graphify query: {str(e)}"
        }, ensure_ascii=False)

def check_graphify_requirements() -> bool:
    """Verifies that the graphifyy package is available in the python environment."""
    try:
        import graphifyy
        return True
    except ImportError:
        return False

# =============================================================================
# OpenAI Tool Schemas
# =============================================================================

GRAPHIFY_REBUILD_SCHEMA = {
    "name": "graphify_rebuild",
    "description": (
        "Rebuilds the semantic Graphify index of the locked Obsidian Vault at 'E:\\OBSIDIAN SECOND BRAIN'. "
        "Call this whenever new notes have been added or updated to ensure the index is synchronized."
    ),
    "parameters": {
        "type": "object",
        "properties": {},
        "required": []
    }
}

GRAPHIFY_QUERY_SCHEMA = {
    "name": "graphify_query",
    "description": (
        "Queries the Graphify semantic index of the locked Obsidian Vault at 'E:\\OBSIDIAN SECOND BRAIN'. "
        "Allows traversing multi-hop knowledge relationships, resolving conceptual links, and fetching "
        "highly contextual markdown note summaries from the vault."
    ),
    "parameters": {
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "The natural language query or concept search term."
            }
        },
        "required": ["query"]
    }
}

# --- Register tools ---
registry.register(
    name="graphify_rebuild",
    toolset="research",
    schema=GRAPHIFY_REBUILD_SCHEMA,
    handler=lambda args, **kw: graphify_rebuild_handler(),
    check_fn=check_graphify_requirements,
    emoji="🕸️",
)

registry.register(
    name="graphify_query",
    toolset="research",
    schema=GRAPHIFY_QUERY_SCHEMA,
    handler=lambda args, **kw: graphify_query_handler(query=args.get("query", "")),
    check_fn=check_graphify_requirements,
    emoji="🔍",
)
