"""
Stitch MCP Integration Tool for Hermes Agent
Enables agents to interact with Stitch bridge and manage MCP connections
"""

import json
import os
import subprocess
import sys
from typing import Dict, Any, List, Optional
from tools.registry import registry

def check_requirements() -> bool:
    """Check if stitch bridge is available"""
    return os.path.exists("stitch_bridge.py")


def invoke_stitch_tool(service: str, tool: str, params: Dict[str, Any] = None, task_id: str = None) -> str:
    """
    Invoke a tool on a Stitch MCP service
    
    Args:
        service: Service name (hermes-orchestrator, agent-zero, open-brain, vercel-deployment, etc.)
        tool: Tool name to invoke
        params: Parameters for the tool
        task_id: Track the task executing this
    
    Returns:
        JSON string with results
    """
    try:
        if not params:
            params = {}
        
        # Build request
        request = {
            "service": service,
            "tool": tool,
            "params": params,
            "task_id": task_id
        }
        
        # Call stitch bridge via subprocess
        result = subprocess.run(
            [sys.executable, "-c", f"""
import asyncio
import aiohttp
import json
import sys

async def invoke():
    config = json.load(open('stitch-mcp.config.json'))
    
    # Find service config
    service_config = config['mcpServers'].get('{service}')
    if not service_config:
        return {{"error": "Service not found: {service}"}}
    
    api_url = service_config['env'].get('API_URL')
    if not api_url:
        # Calculate from port
        port = [a for a in service_config['args'] if a.lstrip('-').isdigit()]
        api_url = f'http://127.0.0.1:{{port[0] if port else 8000}}'
    
    api_key = service_config['env'].get('API_KEY', '')
    
    try:
        async with aiohttp.ClientSession() as session:
            headers = {{"Content-Type": "application/json"}}
            if api_key:
                headers["Authorization"] = f"Bearer {{api_key}}"
            
            url = f"{{api_url}}/invoke/{tool}"
            
            async with session.post(url, json={json.dumps(params)}, headers=headers, timeout=aiohttp.ClientTimeout(total=30)) as resp:
                if resp.status == 200:
                    data = await resp.json()
                    return {{"success": True, "status": resp.status, "result": data}}
                else:
                    return {{"success": False, "status": resp.status, "error": f"HTTP {{resp.status}}"}}
    except Exception as e:
        return {{"error": str(e), "success": False}}

print(json.dumps(asyncio.run(invoke())))
"""],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        if result.returncode == 0:
            return result.stdout
        else:
            return json.dumps({
                "error": f"Stitch bridge error: {result.stderr}",
                "success": False
            })
    
    except Exception as e:
        return json.dumps({
            "error": f"Failed to invoke stitch tool: {str(e)}",
            "success": False
        })


def test_stitch_connections(task_id: str = None) -> str:
    """
    Test all Stitch MCP connections
    
    Returns:
        JSON string with test results
    """
    try:
        result = subprocess.run(
            [sys.executable, "stitch_bridge.py"],
            capture_output=True,
            text=True,
            timeout=60,
            cwd=os.getcwd()
        )
        
        # Try to load results file
        if os.path.exists("stitch-mcp-results.json"):
            with open("stitch-mcp-results.json", "r") as f:
                return json.dumps(json.load(f))
        else:
            return json.dumps({
                "error": "Test completed but results file not found",
                "stdout": result.stdout,
                "stderr": result.stderr,
                "success": result.returncode == 0
            })
    
    except subprocess.TimeoutExpired:
        return json.dumps({
            "error": "Stitch connection test timed out",
            "success": False
        })
    except Exception as e:
        return json.dumps({
            "error": f"Failed to run stitch tests: {str(e)}",
            "success": False
        })


def get_stitch_status(task_id: str = None) -> str:
    """Get current status of all Stitch MCP connections"""
    try:
        if os.path.exists("stitch-mcp-results.json"):
            with open("stitch-mcp-results.json", "r") as f:
                return json.dumps(json.load(f))
        else:
            return json.dumps({
                "error": "No test results available yet. Run test_stitch_connections first.",
                "success": False
            })
    except Exception as e:
        return json.dumps({
            "error": f"Failed to read stitch status: {str(e)}",
            "success": False
        })


def list_stitch_services(task_id: str = None) -> str:
    """List all available Stitch MCP services and their tools"""
    try:
        with open("stitch-mcp.config.json", "r") as f:
            config = json.load(f)
        
        services = {}
        for service_name, service_config in config.get("mcpServers", {}).items():
            services[service_name] = {
                "tools": service_config.get("tools", []),
                "type": service_config["env"].get("AGENT_TYPE", "system"),
                "api_url": service_config["env"].get("API_URL", "calculated")
            }
        
        return json.dumps({
            "success": True,
            "services": services,
            "total": len(services)
        })
    except Exception as e:
        return json.dumps({
            "error": f"Failed to list services: {str(e)}",
            "success": False
        })


def configure_stitch_connection(service: str, api_url: str, api_key: str = None, task_id: str = None) -> str:
    """
    Configure or update a Stitch MCP connection
    
    Args:
        service: Service name
        api_url: API URL for the service
        api_key: Optional API key
        task_id: Task ID for tracking
    
    Returns:
        JSON string with config update result
    """
    try:
        # Load current config
        with open("stitch-mcp.config.json", "r") as f:
            config = json.load(f)
        
        if service not in config["mcpServers"]:
            return json.dumps({
                "error": f"Service '{service}' not found in configuration",
                "success": False
            })
        
        # Update the service
        config["mcpServers"][service]["env"]["API_URL"] = api_url
        if api_key:
            config["mcpServers"][service]["env"]["API_KEY"] = api_key
        
        # Save updated config
        with open("stitch-mcp.config.json", "w") as f:
            json.dump(config, f, indent=2)
        
        return json.dumps({
            "success": True,
            "message": f"Updated configuration for {service}",
            "service": service,
            "api_url": api_url
        })
    except Exception as e:
        return json.dumps({
            "error": f"Failed to configure connection: {str(e)}",
            "success": False
        })


# Register all Stitch tools
registry.register(
    name="invoke_stitch_tool",
    toolset="stitch-mcp",
    schema={
        "name": "invoke_stitch_tool",
        "description": "Invoke a tool on any Stitch MCP service (Hermes, Agent Zero, Open Brain, Vercel, VPS, Skill Wizard, Cron, Beads)",
        "parameters": {
            "type": "object",
            "properties": {
                "service": {
                    "type": "string",
                    "description": "Service name: hermes-orchestrator, agent-zero, open-brain, vercel-deployment, vps-orchestration, skill-wizard, cron-scheduler, beads-task-tracking"
                },
                "tool": {
                    "type": "string",
                    "description": "Tool name to invoke on the service"
                },
                "params": {
                    "type": "object",
                    "description": "Parameters for the tool"
                }
            },
            "required": ["service", "tool"]
        }
    },
    handler=lambda args, **kw: invoke_stitch_tool(
        service=args.get("service", ""),
        tool=args.get("tool", ""),
        params=args.get("params", {}),
        task_id=kw.get("task_id")
    ),
    check_fn=check_requirements,
)

registry.register(
    name="test_stitch_connections",
    toolset="stitch-mcp",
    schema={
        "name": "test_stitch_connections",
        "description": "Test all Stitch MCP connections to verify they are reachable",
        "parameters": {
            "type": "object",
            "properties": {},
            "required": []
        }
    },
    handler=lambda args, **kw: test_stitch_connections(task_id=kw.get("task_id")),
    check_fn=check_requirements,
)

registry.register(
    name="get_stitch_status",
    toolset="stitch-mcp",
    schema={
        "name": "get_stitch_status",
        "description": "Get current status of all Stitch MCP connections",
        "parameters": {
            "type": "object",
            "properties": {},
            "required": []
        }
    },
    handler=lambda args, **kw: get_stitch_status(task_id=kw.get("task_id")),
    check_fn=check_requirements,
)

registry.register(
    name="list_stitch_services",
    toolset="stitch-mcp",
    schema={
        "name": "list_stitch_services",
        "description": "List all available Stitch MCP services and their tools",
        "parameters": {
            "type": "object",
            "properties": {},
            "required": []
        }
    },
    handler=lambda args, **kw: list_stitch_services(task_id=kw.get("task_id")),
    check_fn=check_requirements,
)

registry.register(
    name="configure_stitch_connection",
    toolset="stitch-mcp",
    schema={
        "name": "configure_stitch_connection",
        "description": "Configure or update a Stitch MCP connection",
        "parameters": {
            "type": "object",
            "properties": {
                "service": {
                    "type": "string",
                    "description": "Service name to configure"
                },
                "api_url": {
                    "type": "string",
                    "description": "New API URL for the service"
                },
                "api_key": {
                    "type": "string",
                    "description": "Optional API key for the service"
                }
            },
            "required": ["service", "api_url"]
        }
    },
    handler=lambda args, **kw: configure_stitch_connection(
        service=args.get("service", ""),
        api_url=args.get("api_url", ""),
        api_key=args.get("api_key"),
        task_id=kw.get("task_id")
    ),
    check_fn=check_requirements,
)
