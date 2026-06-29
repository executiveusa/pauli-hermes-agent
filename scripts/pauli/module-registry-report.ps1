$ErrorActionPreference = "Stop"

@'
from pauli.mcp.registry import inspect_all_modules
import json
print(json.dumps(inspect_all_modules(), indent=2))
'@ | & "E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\.venv\Scripts\python.exe" -
