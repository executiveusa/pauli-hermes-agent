#!/usr/bin/env python3
"""Idempotently mount the STARNET router into api_server.py.

This keeps the existing Hermes API composition root intact while making the
Pauli/STARNET router an explicit, repeatable deployment step.
"""
from pathlib import Path
import sys

path = Path(sys.argv[1] if len(sys.argv) > 1 else "api_server.py")
text = path.read_text(encoding="utf-8")
start = "# BEGIN PAULI STARNET ROUTER\n"
end = "# END PAULI STARNET ROUTER\n"
block = (
    start
    + "from starnet_gateway import router as starnet_router\n"
    + "app.include_router(starnet_router, prefix=\"/starnet\")\n"
    + end
)

if start in text:
    before, rest = text.split(start, 1)
    if end not in rest:
        raise SystemExit("existing STARNET router marker is incomplete")
    _, after = rest.split(end, 1)
    text = before + block + after
else:
    needle = 'if __name__ == "__main__":'
    if needle not in text:
        raise SystemExit("api_server.py composition root not found")
    text = text.replace(needle, block + "\n\n" + needle, 1)

path.write_text(text, encoding="utf-8")
print("STARNET router mounted in api_server.py")
