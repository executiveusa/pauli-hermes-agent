# Phase 01 Report: Baseline Bootability + Packaging

## Objective
Make the repo installable and importable with a minimal Pauli package surface, while preserving the existing untracked control-room work.

## What Changed
- Updated `pyproject.toml` package discovery to include `pauli` and `pauli.*`.
- Added a minimal `pauli` package with an OpenClaude dispatcher shim.
- Added `agent/pauli_skill_router.py` for safe task classification and skill selection.
- Added safe config files:
  - `config/pauli_skill_router.yaml`
  - `config/pauli_worker_registry.yaml`
- Added the requested minimal Pauli policy skills under `skills/pauli/`.
- Added wrapper scripts under `scripts/pauli/openclaude/` that call the canonical Pauli dispatcher code.
- Added a minimal `api_server.py` FastAPI entrypoint plus explicit Vercel routing metadata so preview builds stop auto-detecting the unrelated `tinker-atropos` app.
- Added regression tests for:
  - Pauli package importability
  - destructive-task blocking
  - config file presence
  - OpenClaude wrapper script presence
  - OpenClaude safe/dangerous task dispatch behavior
  - FastAPI entrypoint importability

## Verification
- `py -3 -m pytest tests\agent\test_pauli_skill_router.py -q -o addopts=''`
  - PASS
- `py -3 -m pytest tests\openclaude\test_openclaude_dispatcher.py -q -o addopts=''`
  - PASS
- `py -3 -m pytest tests\agent\test_pauli_skill_router.py tests\openclaude\test_openclaude_dispatcher.py -q -o addopts=''`
  - PASS
- `py -3 -m pytest tests\test_api_server.py tests\agent\test_pauli_skill_router.py tests\openclaude\test_openclaude_dispatcher.py -q -o addopts=''`
  - PASS
- `py -3 -m compileall -q agent cli.py hermes_cli gateway tools pauli scripts\pauli\openclaude config`
  - PASS
- `py -3 -m pip install -e .`
  - PASS
  - Resolver warnings appeared for unrelated environment package conflicts (`browser-use`, `fastmcp`, `openhands-ai`) after `openai`/`mcp` version resolution, but repo tests still passed.

## Safety
- Secret scan of the changed files found only expected policy keywords such as `secret` and `password`.
- No actual secrets were added.
- The existing untracked `apps/` tree remains preserved and untouched.

## Phase 1 Status
- Editable install works with the new Pauli package surface.
- Compileall passes.
- The Pauli router and OpenClaude shim import cleanly.
- The new safety registry and policy skills are present.
- Vercel now has a concrete `api_server:app` entrypoint instead of auto-scanning the repo.

## Next Step
Proceed to Phase 2: wire the Pauli router into the gateway path and make the routing visible in gateway logs.
