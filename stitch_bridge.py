#!/usr/bin/env python3
"""
Stitch MCP Bridge - Connects Hermes services to MCP
Handles all 8 service connections with unified interface
"""

import os
import json
import asyncio
import aiohttp
from typing import Any, Dict, List, Optional
from dataclasses import dataclass
from datetime import datetime
import sys

@dataclass
class StitchConnection:
    name: str
    api_url: str
    api_key: str
    agent_type: Optional[str] = None
    tools: List[str] = None
    status: str = "disconnected"
    last_check: Optional[datetime] = None
    error: Optional[str] = None

class StitchBridge:
    """Main Stitch bridge that manages all MCP connections"""
    
    def __init__(self, config_path: str = "stitch-mcp.config.json"):
        self.config_path = config_path
        self.connections: Dict[str, StitchConnection] = {}
        self.session: Optional[aiohttp.ClientSession] = None
        self.results = {"passed": [], "failed": []}
        self._load_config()
    
    def _load_config(self):
        """Load MCP configuration from file"""
        try:
            with open(self.config_path, 'r') as f:
                config = json.load(f)
                
            # Create connections from config
            for server_name, server_config in config.get("mcpServers", {}).items():
                api_url = server_config["env"].get("API_URL") or f"http://localhost:{server_config['args'][-1] if '--port' in server_config['args'] else '8000'}"
                api_key = server_config["env"].get("API_KEY", "")
                
                self.connections[server_name] = StitchConnection(
                    name=server_name,
                    api_url=api_url.replace("localhost", "127.0.0.1"),
                    api_key=api_key,
                    agent_type=server_config["env"].get("AGENT_TYPE"),
                    tools=server_config.get("tools", []),
                )
                print(f"✓ Loaded connection: {server_name}")
        except FileNotFoundError:
            print(f"Error: {self.config_path} not found")
            sys.exit(1)
        except json.JSONDecodeError:
            print(f"Error: {self.config_path} is not valid JSON")
            sys.exit(1)
    
    async def init(self):
        """Initialize async session"""
        self.session = aiohttp.ClientSession()
    
    async def cleanup(self):
        """Clean up async resources"""
        if self.session:
            await self.session.close()
    
    async def test_connection(self, connection_name: str) -> bool:
        """Test a single connection"""
        conn = self.connections[connection_name]
        
        try:
            # Try health check endpoint
            health_url = f"{conn.api_url}/health" if "localhost" not in conn.api_url else f"{conn.api_url}/health"
            
            headers = {}
            if conn.api_key:
                headers["Authorization"] = f"Bearer {conn.api_key}"
            
            async with self.session.get(health_url, headers=headers, timeout=aiohttp.ClientTimeout(total=5)) as resp:
                if resp.status == 200:
                    conn.status = "connected"
                    conn.last_check = datetime.now()
                    conn.error = None
                    print(f"✅ {connection_name}: PASSED (HTTP {resp.status})")
                    self.results["passed"].append(connection_name)
                    return True
                else:
                    conn.status = "error"
                    conn.error = f"HTTP {resp.status}"
                    conn.last_check = datetime.now()
                    print(f"❌ {connection_name}: FAILED (HTTP {resp.status})")
                    self.results["failed"].append((connection_name, f"HTTP {resp.status}"))
                    return False
        except asyncio.TimeoutError:
            conn.status = "timeout"
            conn.error = "Connection timeout"
            conn.last_check = datetime.now()
            print(f"⏱️  {connection_name}: TIMEOUT")
            self.results["failed"].append((connection_name, "Timeout"))
            return False
        except aiohttp.ClientConnectorError as e:
            conn.status = "unreachable"
            conn.error = str(e)
            conn.last_check = datetime.now()
            print(f"🔌 {connection_name}: UNREACHABLE - {e}")
            self.results["failed"].append((connection_name, "Connection refused"))
            return False
        except Exception as e:
            conn.status = "error"
            conn.error = str(e)
            conn.last_check = datetime.now()
            print(f"⚠️  {connection_name}: ERROR - {e}")
            self.results["failed"].append((connection_name, str(e)))
            return False
    
    async def test_all_connections(self) -> Dict[str, Any]:
        """Test all connections concurrently"""
        print("\n" + "="*60)
        print("STITCH MCP BRIDGE - CONNECTION TEST SUITE")
        print("="*60 + "\n")
        
        # Test each connection
        tasks = [self.test_connection(name) for name in self.connections.keys()]
        results = await asyncio.gather(*tasks)
        
        # Summary
        print("\n" + "="*60)
        print("TEST SUMMARY")
        print("="*60)
        print(f"✅ Passed: {len(self.results['passed'])}/{len(self.connections)}")
        print(f"❌ Failed: {len(self.results['failed'])}/{len(self.connections)}")
        
        if self.results["passed"]:
            print(f"\n✅ Connected Services:")
            for service in self.results["passed"]:
                print(f"   • {service}")
        
        if self.results["failed"]:
            print(f"\n❌ Failed Services:")
            for service, error in self.results["failed"]:
                print(f"   • {service}: {error}")
        
        print("\n" + "="*60)
        print("CONNECTION DETAILS")
        print("="*60)
        
        for name, conn in self.connections.items():
            status_icon = "✅" if conn.status == "connected" else "❌"
            print(f"\n{status_icon} {name}")
            print(f"   Status: {conn.status}")
            print(f"   URL: {conn.api_url}")
            if conn.tools:
                print(f"   Tools: {', '.join(conn.tools[:3])}{'...' if len(conn.tools) > 3 else ''}")
            if conn.last_check:
                print(f"   Last Check: {conn.last_check.strftime('%Y-%m-%d %H:%M:%S')}")
            if conn.error:
                print(f"   Error: {conn.error}")
        
        return {
            "timestamp": datetime.now().isoformat(),
            "total_connections": len(self.connections),
            "passed": len(self.results["passed"]),
            "failed": len(self.results["failed"]),
            "connections": {
                name: {
                    "status": conn.status,
                    "url": conn.api_url,
                    "error": conn.error,
                    "tools": conn.tools or []
                }
                for name, conn in self.connections.items()
            }
        }
    
    async def invoke_tool(self, service: str, tool: str, params: Dict[str, Any]) -> Dict[str, Any]:
        """Invoke a tool on a specific service"""
        if service not in self.connections:
            return {"error": f"Service {service} not found"}
        
        conn = self.connections[service]
        
        try:
            url = f"{conn.api_url}/invoke/{tool}"
            headers = {"Content-Type": "application/json"}
            if conn.api_key:
                headers["Authorization"] = f"Bearer {conn.api_key}"
            
            async with self.session.post(url, json=params, headers=headers, timeout=aiohttp.ClientTimeout(total=30)) as resp:
                result = await resp.json()
                return {
                    "success": resp.status == 200,
                    "status": resp.status,
                    "result": result
                }
        except Exception as e:
            return {"error": str(e), "success": False}
    
    def get_status_report(self) -> str:
        """Generate human-readable status report"""
        report = []
        report.append("\n📊 HERMES STITCH MCP STATUS REPORT")
        report.append("=" * 60)
        
        for name, conn in self.connections.items():
            status_symbol = "🟢" if conn.status == "connected" else "🔴" if conn.status in ["error", "unreachable"] else "🟡"
            report.append(f"{status_symbol} {name.upper()}")
            report.append(f"   Status: {conn.status}")
            report.append(f"   URL: {conn.api_url}")
            if conn.tools:
                report.append(f"   Available Tools: {len(conn.tools)}")
                for tool in conn.tools[:2]:
                    report.append(f"      • {tool}")
                if len(conn.tools) > 2:
                    report.append(f"      • ... and {len(conn.tools) - 2} more")
        
        return "\n".join(report)


async def main():
    """Main entry point"""
    bridge = StitchBridge()
    await bridge.init()
    
    try:
        results = await bridge.test_all_connections()
        print(bridge.get_status_report())
        
        # Save results to file
        with open("stitch-mcp-results.json", "w") as f:
            json.dump(results, f, indent=2)
        print(f"\n💾 Results saved to stitch-mcp-results.json")
        
        # Exit code based on failures
        sys.exit(0 if len(results["failed"]) == 0 else 1)
    finally:
        await bridge.cleanup()


if __name__ == "__main__":
    asyncio.run(main())
