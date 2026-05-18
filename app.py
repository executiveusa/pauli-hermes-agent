"""
Vercel entrypoint for Hermes Agent Web Server.
Forwards to the FastAPI web server via hermes_cli.web_server.
"""

import sys
from pathlib import Path

# Add project root to path
PROJECT_ROOT = Path(__file__).parent.resolve()
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

# Import and expose the FastAPI app
from hermes_cli.web_server import app
