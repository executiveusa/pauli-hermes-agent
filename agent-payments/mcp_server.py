#!/usr/bin/env python3
"""Minimal MCP wrapper for agent-payments commands."""

import json
import logging
from http.server import BaseHTTPRequestHandler, HTTPServer
from urllib.parse import urlparse, parse_qs

from apify import discover_actors, inspect_actor, run_actor
from awal import check_status, get_address, get_balance, fetch_wallet_qr, x402_details, x402_pay
from paypal import paypal_info

logger = logging.getLogger(__name__)

class RequestHandler(BaseHTTPRequestHandler):
    def _send_json(self, data, status=200):
        body = json.dumps(data, ensure_ascii=False).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        parsed = urlparse(self.path)
        path = parsed.path
        params = {k: v[0] for k, v in parse_qs(parsed.query).items()}

        try:
            if path == "/ping":
                self._send_json({"ok": True})
            elif path == "/status":
                self._send_json(check_status())
            elif path == "/address":
                self._send_json({"address": get_address()})
            elif path == "/balance":
                self._send_json(get_balance())
            elif path == "/x402/details":
                url = params.get("url")
                self._send_json(x402_details(url))
            elif path == "/paypal":
                self._send_json({"info": paypal_info()})
            else:
                self._send_json({"error": "not_found"}, status=404)
        except Exception as exc:
            logger.exception("MCP request failed")
            self._send_json({"error": str(exc)}, status=500)

    def do_POST(self):
        parsed = urlparse(self.path)
        path = parsed.path
        length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(length).decode("utf-8") if length else ""
        data = {}
        if body:
            data = json.loads(body)

        try:
            if path == "/x402/pay":
                url = data.get("url")
                max_amount = data.get("max_amount", 1000000)
                self._send_json(x402_pay(url, int(max_amount)))
            elif path == "/apify/discover":
                query = data.get("query", "")
                limit = int(data.get("limit", 5))
                self._send_json(discover_actors(query, limit))
            elif path == "/apify/inspect":
                self._send_json({"markdown": inspect_actor(data.get("actor_id", ""))})
            elif path == "/apify/run":
                result = run_actor(data.get("actor_id", ""), data.get("input", {}))
                self._send_json(result)
            else:
                self._send_json({"error": "not_found"}, status=404)
        except Exception as exc:
            logger.exception("MCP request failed")
            self._send_json({"error": str(exc)}, status=500)


def run_server(host: str = "127.0.0.1", port: int = 8766):
    server = HTTPServer((host, port), RequestHandler)
    print(f"agent-payments MCP server listening on http://{host}:{port}")
    server.serve_forever()


if __name__ == "__main__":
    run_server()
