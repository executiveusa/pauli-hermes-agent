"""
Infisical Secrets Management Integration

Connects to Infisical API for dynamic secret retrieval, caching, and hourly rotation.
Replaces plaintext .env file approach with secure centralized secrets management.

Organization ID: bfdc227f-410f-4b15-a0d6-63d1c99472d2
"""

import json
import os
import time
from typing import Dict, Optional, Any
from datetime import datetime, timedelta
import hashlib
import hmac
import base64
import requests
from tools.registry import registry

# Infisical API configuration
INFISICAL_API_BASE = os.getenv("INFISICAL_API_URL", "https://api.infisical.com/api")
INFISICAL_TOKEN = os.getenv("INFISICAL_TOKEN", "")
INFISICAL_ORGANIZATION_ID = "bfdc227f-410f-4b15-a0d6-63d1c99472d2"

# In-memory cache with expiration (3600 seconds = 1 hour)
_secret_cache: Dict[str, Dict[str, Any]] = {}
_cache_timestamps: Dict[str, float] = {}
CACHE_TTL = 3600  # 1 hour rotation


def check_requirements() -> bool:
    """Check if Infisical token is available."""
    return bool(INFISICAL_TOKEN and INFISICAL_TOKEN.strip())


def _is_cache_valid(secret_path: str) -> bool:
    """Check if cached secret is still valid (within TTL)."""
    if secret_path not in _cache_timestamps:
        return False
    elapsed = time.time() - _cache_timestamps[secret_path]
    return elapsed < CACHE_TTL


def _extract_organization_id_from_jwt() -> Optional[str]:
    """Extract organization ID from Infisical JWT token."""
    try:
        # JWT format: header.payload.signature
        parts = INFISICAL_TOKEN.split('.')
        if len(parts) != 3:
            return None
        
        # Decode payload (add padding if needed)
        payload = parts[1]
        # Add padding
        padding = 4 - len(payload) % 4
        if padding != 4:
            payload += '=' * padding
        
        decoded = base64.urlsafe_b64decode(payload)
        token_data = json.loads(decoded)
        return token_data.get("organizationId", INFISICAL_ORGANIZATION_ID)
    except Exception as e:
        print(f"Error extracting org ID from JWT: {e}")
        return INFISICAL_ORGANIZATION_ID


def fetch_secret(
    secret_path: str,
    environment: str = "production",
    task_id: str = None
) -> str:
    """
    Fetch a secret from Infisical by path.
    
    Args:
        secret_path: Path to secret (e.g., "anthropic-api-key", "supabase/url")
        environment: Environment name (production, staging, etc.)
        task_id: Optional task ID for logging
    
    Returns:
        JSON string with secret value or error
    """
    try:
        # Check cache first
        cache_key = f"{environment}:{secret_path}"
        if cache_key in _secret_cache and _is_cache_valid(cache_key):
            return json.dumps({
                "success": True,
                "value": _secret_cache[cache_key]["value"],
                "cached": True,
                "cached_at": _secret_cache[cache_key].get("cached_at")
            })
        
        # Fetch from Infisical API
        org_id = _extract_organization_id_from_jwt()
        headers = {
            "Authorization": f"Bearer {INFISICAL_TOKEN}",
            "Content-Type": "application/json"
        }
        
        # Infisical API endpoint for secrets
        url = f"{INFISICAL_API_BASE}/v3/secrets/raw"
        params = {
            "secretKey": secret_path,
            "environment": environment,
            "workspaceId": org_id
        }
        
        response = requests.get(url, headers=headers, params=params, timeout=10)
        
        if response.status_code == 200:
            data = response.json()
            secret_value = data.get("secret", {}).get("secretValue", "")
            
            # Cache the secret
            _secret_cache[cache_key] = {
                "value": secret_value,
                "cached_at": datetime.now().isoformat()
            }
            _cache_timestamps[cache_key] = time.time()
            
            return json.dumps({
                "success": True,
                "value": secret_value,
                "cached": False,
                "path": secret_path,
                "environment": environment
            })
        
        elif response.status_code == 401:
            return json.dumps({
                "success": False,
                "error": "Infisical authentication failed. Token may be expired.",
                "code": 401
            })
        
        elif response.status_code == 404:
            return json.dumps({
                "success": False,
                "error": f"Secret not found: {secret_path}",
                "code": 404
            })
        
        else:
            return json.dumps({
                "success": False,
                "error": f"Infisical API error: {response.status_code}",
                "details": response.text[:200]
            })

    except requests.exceptions.Timeout:
        return json.dumps({
            "success": False,
            "error": "Infisical API request timeout"
        })
    except requests.exceptions.RequestException as e:
        return json.dumps({
            "success": False,
            "error": f"Infisical API request failed: {str(e)[:100]}"
        })
    except Exception as e:
        return json.dumps({
            "success": False,
            "error": f"Error fetching secret: {str(e)[:100]}"
        })


def list_secrets(
    environment: str = "production",
    path_prefix: str = None,
    task_id: str = None
) -> str:
    """
    List all secrets in an environment (optionally filtered by path prefix).
    
    Args:
        environment: Environment name
        path_prefix: Optional prefix to filter secrets (e.g., "anthropic", "supabase")
        task_id: Optional task ID for logging
    
    Returns:
        JSON string with list of secret paths
    """
    try:
        org_id = _extract_organization_id_from_jwt()
        headers = {
            "Authorization": f"Bearer {INFISICAL_TOKEN}",
            "Content-Type": "application/json"
        }
        
        url = f"{INFISICAL_API_BASE}/v3/secrets"
        params = {
            "environment": environment,
            "workspaceId": org_id
        }
        
        response = requests.get(url, headers=headers, params=params, timeout=10)
        
        if response.status_code == 200:
            data = response.json()
            secrets = data.get("secrets", [])
            
            # Extract secret names/paths
            secret_paths = []
            for secret in secrets:
                secret_name = secret.get("secretKey", "")
                if path_prefix is None or secret_name.startswith(path_prefix):
                    secret_paths.append(secret_name)
            
            return json.dumps({
                "success": True,
                "count": len(secret_paths),
                "environment": environment,
                "secrets": sorted(secret_paths),
                "prefix_filter": path_prefix
            })
        
        else:
            return json.dumps({
                "success": False,
                "error": f"Failed to list secrets: {response.status_code}"
            })

    except Exception as e:
        return json.dumps({
            "success": False,
            "error": f"Error listing secrets: {str(e)[:100]}"
        })


def rotate_secrets(task_id: str = None) -> str:
    """
    Clear local secret cache to force fresh fetches on next access.
    Called hourly or on-demand to ensure secrets don't go stale.
    
    Args:
        task_id: Optional task ID for logging
    
    Returns:
        JSON string with rotation status
    """
    global _secret_cache, _cache_timestamps
    
    try:
        count = len(_secret_cache)
        _secret_cache.clear()
        _cache_timestamps.clear()
        
        return json.dumps({
            "success": True,
            "message": f"Rotated {count} cached secrets",
            "rotated_at": datetime.now().isoformat()
        })
    except Exception as e:
        return json.dumps({
            "success": False,
            "error": f"Error rotating secrets: {str(e)}"
        })


def get_cache_status(task_id: str = None) -> str:
    """
    Get current cache status and expiration times.
    
    Returns:
        JSON string with cache info
    """
    try:
        cache_info = []
        current_time = time.time()
        
        for cache_key, data in _secret_cache.items():
            timestamp = _cache_timestamps.get(cache_key, 0)
            elapsed = current_time - timestamp
            remaining = max(0, CACHE_TTL - elapsed)
            
            cache_info.append({
                "key": cache_key,
                "cached_at": data.get("cached_at"),
                "ttl_remaining": int(remaining),
                "expired": remaining <= 0
            })
        
        return json.dumps({
            "success": True,
            "cache_size": len(_secret_cache),
            "ttl_seconds": CACHE_TTL,
            "cache_entries": cache_info
        })
    except Exception as e:
        return json.dumps({
            "success": False,
            "error": f"Error getting cache status: {str(e)}"
        })


# Register tools
registry.register(
    name="fetch_secret",
    toolset="infisical",
    schema={
        "name": "fetch_secret",
        "description": "Fetch a secret from Infisical by path. Caches locally with 1-hour rotation.",
        "parameters": {
            "type": "object",
            "properties": {
                "secret_path": {
                    "type": "string",
                    "description": "Path to the secret (e.g., 'anthropic-api-key', 'supabase/url')"
                },
                "environment": {
                    "type": "string",
                    "description": "Environment name (production, staging, development)",
                    "default": "production"
                }
            },
            "required": ["secret_path"]
        }
    },
    handler=lambda args, **kw: fetch_secret(
        secret_path=args.get("secret_path", ""),
        environment=args.get("environment", "production"),
        task_id=kw.get("task_id")
    ),
    check_fn=check_requirements,
    requires_env=["INFISICAL_TOKEN"]
)

registry.register(
    name="list_secrets",
    toolset="infisical",
    schema={
        "name": "list_secrets",
        "description": "List all secrets in Infisical, optionally filtered by path prefix.",
        "parameters": {
            "type": "object",
            "properties": {
                "environment": {
                    "type": "string",
                    "description": "Environment to list secrets from",
                    "default": "production"
                },
                "path_prefix": {
                    "type": "string",
                    "description": "Optional prefix filter (e.g., 'anthropic', 'supabase')"
                }
            }
        }
    },
    handler=lambda args, **kw: list_secrets(
        environment=args.get("environment", "production"),
        path_prefix=args.get("path_prefix"),
        task_id=kw.get("task_id")
    ),
    check_fn=check_requirements,
    requires_env=["INFISICAL_TOKEN"]
)

registry.register(
    name="rotate_secrets",
    toolset="infisical",
    schema={
        "name": "rotate_secrets",
        "description": "Clear local secret cache and force fresh fetches from Infisical on next access.",
        "parameters": {
            "type": "object",
            "properties": {}
        }
    },
    handler=lambda args, **kw: rotate_secrets(task_id=kw.get("task_id")),
    check_fn=check_requirements,
    requires_env=["INFISICAL_TOKEN"]
)

registry.register(
    name="get_cache_status",
    toolset="infisical",
    schema={
        "name": "get_cache_status",
        "description": "Get status of local secret cache and TTL expiration times.",
        "parameters": {
            "type": "object",
            "properties": {}
        }
    },
    handler=lambda args, **kw: get_cache_status(task_id=kw.get("task_id")),
    check_fn=check_requirements,
    requires_env=["INFISICAL_TOKEN"]
)
