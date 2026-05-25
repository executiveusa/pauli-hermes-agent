# Phase 02 Report - Pauli Skill Router + Gateway Wiring

## Outcome

Phase 2 is implemented on the `safety/local-reconciliation-20260525-131402` branch.

The Pauli router now applies to gateway turns before `AIAgent` construction. When Pauli routing is enabled in config, the gateway:

- classifies the incoming task with `agent.pauli_skill_router`
- preloads the selected skill context into the ephemeral system prompt
- logs `selected_skills`, `required_skills`, `missing_skills`, and `skipped_skills`
- blocks turns cleanly when strict required skills are missing
- preserves the existing gateway/Telegram flow for safe turns

## Files Changed

- `agent/pauli_skill_router.py`
- `gateway/run.py`
- `hermes_cli/config.py`
- `tests/agent/test_pauli_skill_router.py`
- `tests/gateway/test_pauli_gateway_routing.py`

## Notable Behavior

- Added `load_pauli_agent_policy()` to read the new `agent.pauli_profile`, `agent.pauli_gateway_routing`, and `agent.pauli_required_skills_strict` gates from config.
- Added `build_pauli_turn_context()` so the gateway can reuse one shared path for skill preloading, logging, and block handling.
- Added strict-missing-required behavior to `route_task()` using the router config when strictness is not overridden explicitly.
- Added the Pauli gates to the shared Hermes config defaults so they appear in the normal config surface.

## Verification

Ran:

- `py -3 -m pytest tests\\agent\\test_pauli_skill_router.py tests\\gateway\\test_pauli_gateway_routing.py tests\\gateway\\test_agent_cache.py -q -o addopts=''`
- `py -3 -m compileall -q agent cli.py hermes_cli gateway tools pauli scripts\\pauli\\openclaude config`

Result:

- `21 passed`
- compileall passed

## Safety Notes

- The preserved untracked `apps/` tree was left untouched.
- No secrets were added.
- The Pauli router keeps destructive-task blocking intact.

## Next Step

Proceed to Phase 3: Cynthia / Synthia Gateway reality check and integration.
