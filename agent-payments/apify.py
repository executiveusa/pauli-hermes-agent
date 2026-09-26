#!/usr/bin/env python3
"""Apify CLI helpers for the agent-payments integration."""

import json
import os
import re
import urllib.parse
import urllib.request
from typing import Any, Dict, List, Optional


def _http_get_text(url: str, headers: Dict[str, str] = None, timeout: int = 30) -> str:
    req = urllib.request.Request(url, headers=headers or {})
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return resp.read().decode("utf-8", errors="replace")


def _http_get_json(url: str, headers: Dict[str, str] = None, timeout: int = 30) -> Any:
    text = _http_get_text(url, headers=headers, timeout=timeout)
    return json.loads(text)


def _http_post_json(url: str, body: Dict[str, Any], headers: Dict[str, str] = None, timeout: int = 60) -> Any:
    payload = json.dumps(body or {}).encode("utf-8")
    req_headers = dict(headers or {})
    req_headers["Content-Type"] = "application/json"
    req = urllib.request.Request(url, data=payload, headers=req_headers, method="POST")
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return json.loads(resp.read().decode("utf-8", errors="replace"))


def get_apify_token() -> Optional[str]:
    return os.environ.get("APIFY_TOKEN") or os.environ.get("APIFY_KEY")


def discover_actors(query: str, limit: int = 5) -> Dict[str, Any]:
    encoded = urllib.parse.quote_plus(query)
    url = f"https://api.apify.com/v2/store?search={encoded}&limit={limit}"
    data = _http_get_json(url)
    items = data.get("items") if isinstance(data, dict) else None
    if not isinstance(items, list):
        return {"error": "Could not parse Apify search response", "raw": data}
    actors = []
    for item in items:
        actors.append({
            "id": item.get("id"),
            "title": item.get("title"),
            "description": item.get("description"),
            "username": item.get("username"),
            "name": item.get("name"),
            "star_count": item.get("starCount"),
            "rating": item.get("rating"),
            "price": item.get("price"),
            "url": item.get("url"),
        })
    return {"query": query, "actors": actors}


def make_actor_url(actor_id: str) -> str:
    if actor_id.count("~") == 1:
        return f"https://apify.com/{actor_id.replace('~', '/')}"
    return f"https://apify.com/{actor_id}"


def inspect_actor(actor_id: str) -> str:
    actor_url = make_actor_url(actor_id)
    if actor_url.endswith("/"):
        actor_url = actor_url[:-1]
    markdown_url = f"{actor_url}.md"
    text = _http_get_text(markdown_url)
    return text


def run_actor(actor_id: str, input_payload: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
    token = get_apify_token()
    if not token:
        return {"error": "APIFY_TOKEN environment variable is required for actor execution."}
    url = f"https://api.apify.com/v2/acts/{urllib.parse.quote(actor_id)}/run-sync-get-dataset-items"
    headers = {"Authorization": f"Bearer {token}"}
    body = input_payload or {}
    try:
        data = _http_post_json(url, body, headers=headers)
        return {"actor_id": actor_id, "result": data}
    except Exception as exc:
        return {"error": str(exc)}


def extract_price_and_inputs(markdown: str) -> Dict[str, Any]:
    sections = re.split(r"\n#{1,6} ", markdown)
    cleaned = []
    for section in sections:
        title = section.split("\n", 1)[0].strip()
        body = section[len(title) :].strip() if title else section
        cleaned.append({"title": title, "body": body})
    return {"sections": cleaned}
