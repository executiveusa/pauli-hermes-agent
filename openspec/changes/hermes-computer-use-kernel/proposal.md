# Hermes Phase 03 — Cross-Platform Computer-Use Kernel

## Objective

Land the smallest dependency-safe part of current upstream Hermes computer use that materially improves Windows/Linux readiness without replacing the existing Pauli computer-use backend wholesale.

## Scope

This slice ports two bounded upstream hardening improvements plus targeted tests:

1. `tools/computer_use/permissions.py`
   - OS-aware readiness for macOS, Windows, and Linux
   - `cua-driver doctor --json` as the common readiness signal
   - macOS Accessibility/Screen Recording detail and grant helper
   - sanitized child environment and Windows hidden-process flags

2. `tools/computer_use/vision_routing.py`
   - honor explicit auxiliary vision configuration
   - honor user-declared `supports_vision`
   - use provider tool-result media capability before returning screenshots
   - fail closed toward auxiliary vision when capability is ambiguous

3. `tests/tools/test_computer_use_cross_platform.py`
   - aux vision routing
   - declared vision/text-only behavior
   - provider multimodal behavior
   - no-driver status
   - Windows/Linux doctor readiness
   - macOS permission readiness
   - non-mac grant rejection

## Why this is intentionally smaller than the full upstream computer-use package

Current upstream has materially changed `backend.py`, `cua_backend.py`, `schema.py`, and `tool.py`, and added browser routing/doctor/permission/lifecycle behavior. Because the Pauli fork is more than 11k upstream commits behind, replacing the package wholesale would create an unnecessarily large regression surface.

This phase therefore introduces dormant/readiness policy and a focused vision-routing fix first. It does not change the registered `computer_use` schema or invoke a new backend path.

## Authority and safety

- No production credentials are used.
- No external write action is introduced.
- No computer action is auto-executed by the new readiness helper.
- macOS permission granting remains an explicit operator invocation.
- Windows/Linux readiness is diagnostic only.
- Pauli business/personal/operator boundaries are unchanged.

## Acceptance criteria

1. Cross-platform readiness helper imports only dependencies already present in the Pauli fork.
2. Vision routing respects explicit auxiliary and declared-model capability.
3. Tests cover Windows, Linux, macOS, and vision-routing policy without needing a real desktop session.
4. Existing computer-use tool registration is unchanged in this phase.
5. CI or equivalent executable test evidence is required before this phase can be marked GREEN.
6. Rollback is a normal source revert with no data migration.

## Risk

MEDIUM — runtime library code changes, but isolated from active computer-use invocation until integrated by later slices.
