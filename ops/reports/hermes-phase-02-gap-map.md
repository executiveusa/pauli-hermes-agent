# Hermes Phase 02 Gap Map Report

## DECISION

The first runtime port should be the current upstream cross-platform computer-use implementation, not a bespoke Windows implementation. Current upstream source now explicitly supports macOS, Windows, and Linux through `cua-driver`.

## CHANGES

Documentation/specification only:

- implementation-level capability matrix
- priority order for safe upstream reconciliation
- explicit security review item for outbound `send_message`
- corrected Windows computer-use strategy

## PROOF

### Computer-use package delta

Pauli fork package currently contains six files:

- `__init__.py`
- `backend.py`
- `cua_backend.py`
- `schema.py`
- `tool.py`
- `vision_routing.py`

Current upstream package contains nine files and materially larger implementations:

- all six equivalents above
- `browser_route.py`
- `doctor.py`
- `permissions.py`

Upstream shim additionally exports `release_computer_use_session` and describes the tool as universal desktop control for macOS/Windows/Linux. Pauli shim currently describes macOS only and does not export the session-release helper.

### Toolset delta

Important upstream additions visible in `toolsets.py` include:

- `browser_exec`
- Kanban `request_review` / `request_changes`
- Kanban attachment tools
- GUI-only `project` tools
- GUI-only `desktop_ui` tools
- coding posture
- universal computer-use description

Pauli additions that must be preserved include:

- Graphify tools in the core list
- VPS SSH + Ralphy controls
- model-callable `send_message` (subject to security review)
- Pauli-specific business/ICM skills and orchestration contracts

### Security delta

Current upstream deliberately documents that agents do not receive an agent-callable `send_message` tool in the default core set; outbound delivery is handled outside the agent loop. The Pauli fork still includes `send_message` in `_HERMES_CORE_TOOLS`. This must be treated as an authority/security decision, not blindly synced either direction.

## STATUS

PHASE 02 IMPLEMENTATION: COMPLETE

Merge status: stacked behind Phase 01 until review/merge gates clear.

## RISKS

1. Cross-platform computer-use code is significantly larger and may depend on upstream changes elsewhere in the 11,864-commit delta.
2. Replacing the package wholesale without dependency inspection could break the Pauli fork.
3. Outbound messaging authority differs materially between fork and upstream.
4. Local executable verification remains unavailable because the sandbox cannot resolve GitHub for checkout.

## ROLLBACK

Revert/delete the Phase 02 documentation files. No runtime state is changed.

## NEXT

Phase 03 should perform a dependency-aware computer-use port preflight and then implement only the smallest coherent cross-platform slice with upstream tests or equivalent CI proof.
