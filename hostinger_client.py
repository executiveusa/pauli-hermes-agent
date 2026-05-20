"""
Hostinger API Client for Hermes Agent
Provides utilities for VPS management and monitoring via Hostinger API
"""

import os
import httpx
from typing import Optional, Dict, Any
from datetime import datetime


class HostingerClient:
    """Hostinger API client for VPS and domain management"""

    BASE_URL = "https://api.hostinger.com/v1"

    def __init__(self, api_key: Optional[str] = None):
        """Initialize with API key from environment or parameter"""
        self.api_key = api_key or os.getenv("HOSTINGER_API_KEY")
        if not self.api_key:
            raise ValueError("HOSTINGER_API_KEY not found in environment")

        self.headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }

    async def get_vps_instances(self) -> Dict[str, Any]:
        """Fetch all VPS instances"""
        async with httpx.AsyncClient() as client:
            response = await client.get(
                f"{self.BASE_URL}/vps",
                headers=self.headers,
                timeout=30.0,
            )
            response.raise_for_status()
            return response.json()

    async def get_vps_info(self, vps_id: str) -> Dict[str, Any]:
        """Fetch specific VPS instance information"""
        async with httpx.AsyncClient() as client:
            response = await client.get(
                f"{self.BASE_URL}/vps/{vps_id}",
                headers=self.headers,
                timeout=30.0,
            )
            response.raise_for_status()
            return response.json()

    async def get_domains(self) -> Dict[str, Any]:
        """Fetch all domains"""
        async with httpx.AsyncClient() as client:
            response = await client.get(
                f"{self.BASE_URL}/domains",
                headers=self.headers,
                timeout=30.0,
            )
            response.raise_for_status()
            return response.json()

    async def get_account_info(self) -> Dict[str, Any]:
        """Fetch account information"""
        async with httpx.AsyncClient() as client:
            response = await client.get(
                f"{self.BASE_URL}/account",
                headers=self.headers,
                timeout=30.0,
            )
            response.raise_for_status()
            return response.json()

    async def get_vps_status(self, vps_id: str) -> Dict[str, Any]:
        """Get VPS status (running, stopped, etc)"""
        async with httpx.AsyncClient() as client:
            response = await client.get(
                f"{self.BASE_URL}/vps/{vps_id}/status",
                headers=self.headers,
                timeout=30.0,
            )
            response.raise_for_status()
            return response.json()

    async def reboot_vps(self, vps_id: str) -> Dict[str, Any]:
        """Reboot a VPS instance"""
        async with httpx.AsyncClient() as client:
            response = await client.post(
                f"{self.BASE_URL}/vps/{vps_id}/reboot",
                headers=self.headers,
                timeout=30.0,
            )
            response.raise_for_status()
            return response.json()

    async def get_dns_records(self, domain: str) -> Dict[str, Any]:
        """Get DNS records for a domain"""
        async with httpx.AsyncClient() as client:
            response = await client.get(
                f"{self.BASE_URL}/domains/{domain}/dns",
                headers=self.headers,
                timeout=30.0,
            )
            response.raise_for_status()
            return response.json()

    async def create_dns_record(
        self,
        domain: str,
        record_type: str,
        name: str,
        value: str,
        ttl: int = 3600,
    ) -> Dict[str, Any]:
        """Create a DNS record"""
        async with httpx.AsyncClient() as client:
            response = await client.post(
                f"{self.BASE_URL}/domains/{domain}/dns",
                headers=self.headers,
                json={
                    "type": record_type,
                    "name": name,
                    "value": value,
                    "ttl": ttl,
                },
                timeout=30.0,
            )
            response.raise_for_status()
            return response.json()

    async def test_connection(self) -> bool:
        """Test API connection and key validity"""
        try:
            result = await self.get_account_info()
            return bool(result)
        except Exception as e:
            print(f"❌ Connection failed: {e}")
            return False


async def main():
    """Test the Hostinger client"""
    import asyncio

    print("🔗 Testing Hostinger API Connection...")
    print("")

    try:
        client = HostingerClient()

        # Test connection
        is_connected = await client.test_connection()
        if is_connected:
            print("✅ API connection successful!")
            print("")

            # Get account info
            print("📊 Account Information:")
            account = await client.get_account_info()
            print(f"   Email: {account.get('email', 'N/A')}")
            print(f"   Status: {account.get('status', 'N/A')}")
            print("")

            # Get VPS instances
            print("🖥️  VPS Instances:")
            vps_list = await client.get_vps_instances()
            for vps in vps_list.get("vps", []):
                print(f"   • {vps.get('hostname')} ({vps.get('ip')})")
            print("")

            # Get domains
            print("🌐 Domains:")
            domains = await client.get_domains()
            for domain in domains.get("domains", []):
                print(f"   • {domain.get('domain')}")
            print("")

            print("✅ All tests passed!")
        else:
            print("❌ API key invalid or connection failed")

    except Exception as e:
        print(f"❌ Error: {e}")


if __name__ == "__main__":
    import asyncio
    asyncio.run(main())
