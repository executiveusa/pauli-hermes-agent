#!/usr/bin/env python3
import json
import os
import sys
from http.server import BaseHTTPRequestHandler, HTTPServer
from urllib.parse import urlparse, parse_qs

import requests

TOKEN = os.getenv("HOSTINGER_API_TOKEN") or os.getenv("HOSTINGER_TOKEN")
BASE_URL = os.getenv("HOSTINGER_API_BASE", "https://developers.hostinger.com")
HOST = os.getenv("HOSTINGER_BRIDGE_HOST", "127.0.0.1")
PORT = int(os.getenv("HOSTINGER_BRIDGE_PORT", "8765"))


def _headers():
    return {
        "Authorization": f"Bearer {TOKEN}",
        "Accept": "application/json",
        "Content-Type": "application/json",
    }


def _fetch(path, params=None):
    if not TOKEN:
        raise RuntimeError("HOSTINGER_API_TOKEN is not set")
    url = f"{BASE_URL}{path}"
    response = requests.get(url, headers=_headers(), params=params, timeout=30)
    response.raise_for_status()
    try:
        return response.json()
    except ValueError:
        return response.text


class Handler(BaseHTTPRequestHandler):
    def _send_json(self, payload, status=200):
        body = json.dumps(payload).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        parsed = urlparse(self.path)
        path = parsed.path
        params = parse_qs(parsed.query)

        try:
            if path == "/health":
                self._send_json({"ok": True, "service": "hostinger-bridge"})
            elif path == "/inventory":
                self._send_json({
                    "domains": _fetch("/api/domains/v1/portfolio"),
                    "websites": _fetch("/api/hosting/v1/websites"),
                    "vps": _fetch("/api/vps/v1/virtual-machines"),
                    "subscriptions": _fetch("/api/billing/v1/subscriptions"),
                })
            elif path == "/domains":
                self._send_json(_fetch("/api/domains/v1/portfolio"))
            elif path == "/websites":
                self._send_json(_fetch("/api/hosting/v1/websites"))
            elif path == "/vps":
                self._send_json(_fetch("/api/vps/v1/virtual-machines"))
            elif path == "/subscriptions":
                self._send_json(_fetch("/api/billing/v1/subscriptions"))
            else:
                self._send_json({"error": "not_found"}, status=404)
        except Exception as exc:
            self._send_json({"error": str(exc)}, status=502)

    def log_message(self, format, *args):
        return


if __name__ == "__main__":
    if not TOKEN:
        print("HOSTINGER_API_TOKEN is not set", file=sys.stderr)
        sys.exit(2)
    server = HTTPServer((HOST, PORT), Handler)
    print(f"Hostinger bridge listening on http://{HOST}:{PORT}")
    server.serve_forever()
