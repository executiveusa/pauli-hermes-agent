$ErrorActionPreference = "Stop"

@'
from pauli.coursegen.adapter import inspect_coursegen
import json
print(json.dumps(inspect_coursegen(), indent=2))
'@ | & "E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\.venv\Scripts\python.exe" -
