$ErrorActionPreference = "Stop"

@'
from pauli.flywheel.adapter import inspect_ralphy, build_flywheel_bootstrap_command
import json
payload = inspect_ralphy()
payload["bootstrap_command"] = build_flywheel_bootstrap_command()
print(json.dumps(payload, indent=2))
'@ | & "E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\.venv\Scripts\python.exe" -
