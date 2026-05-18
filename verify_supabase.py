import os
from supabase import create_client, Client


url: str = os.environ.get("SUPABASE_URL")
key: str = os.environ.get("SUPABASE_SERVICE_KEY")

if not url or not key:
    print("Missing SUPABASE_URL or SUPABASE_SERVICE_KEY")
    exit(1)

print(f"Connecting to {url}...")
supabase: Client = create_client(url, key)

try:
    # Test a simple RPC call to see if it works
    res = supabase.rpc("search_memories_fulltext", {"query": "test", "count": 1}).execute()
    print("Connection successful! RPC response:")
    print(res.data)
except Exception as e:
    print(f"Connection failed: {e}")
