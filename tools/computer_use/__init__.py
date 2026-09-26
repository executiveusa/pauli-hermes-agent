"""Computer use toolset — universal (any-model) macOS desktop control.

Vendored verbatim from upstream NousResearch/hermes-agent at tag
v2026.8.16.2, part of hermes-upstream-gap-map item #1 ("Cross-platform
upstream computer-use package" — openspec/changes/hermes-upstream-gap-map).
Only this note was added. Note upstream's own module docstring below still
says "macOS desktop control" even though the package (via cua_backend.py)
now also supports Windows and Linux — that phrasing is upstream's, left
as-is rather than editorializing on their doc copy.

The `capture.py` module this docstring's "Wiring" section references does
not exist upstream at this tag (its responsibilities live inline in
`tool.py`/`cua_backend.py`); this fork's prior version referenced the same
non-existent module, so that is a pre-existing upstream doc inaccuracy,
not something introduced by this vendor.

The `browser_route.py` module referenced by `cua_backend.py` (imported
below transitively) was a minimal fork-native stub at gap #1 time,
deliberately scoped out pending a follow-up. hermes-upstream-gap-map item
#2 ("Computer-use diagnostics + permission model + lifecycle cleanup")
replaced it with the real upstream implementation — see
`tools/computer_use/browser_route.py`'s own docstring. That same PR also
added `doctor.py` and `permissions.py` to this package and wired the
`release_computer_use_session` lifecycle export (below) into its two
missing call sites (`run_agent.py::AIAgent.close()` and
`tools/approval.py`'s YOLO toggle functions).

Architecture
------------
This toolset drives macOS apps through cua-driver's background computer-use
primitive (SkyLight private SPIs for focus-without-raise + pid-scoped event
posting). Unlike #4562's pyautogui backend, it does NOT steal the user's
cursor, keyboard focus, or Space — the agent and the user can co-work on the
same machine.

Unlike #4562's Anthropic-native `computer_20251124` tool, the schema here is
a plain OpenAI function-calling schema that every tool-capable model can
drive. Vision models get SOM (set-of-mark) captures — a screenshot with
numbered overlays on every interactable element plus the AX tree — so they
click by element index instead of pixel coordinates. Non-vision models can
drive via the AX tree alone.

Wiring
------
* `tool.py`       — registers the `computer_use` tool via tools.registry.
* `backend.py`    — abstract `ComputerUseBackend`; swappable implementation.
* `cua_backend.py`— default backend; speaks MCP over stdio to `cua-driver`.
* `schema.py`     — shared schema + docstring for the generic `computer_use`
                    tool. Model-agnostic.
* `capture.py`    — screenshot post-processing (PNG coercion, sizing, SOM
                    overlay if the backend did not).

The outer integration points (multimodal tool-result plumbing, screenshot
eviction in the Anthropic adapter, image-aware token estimation, the
COMPUTER_USE_GUIDANCE prompt block, approval hook, and the skill) live
alongside this package. See agent/anthropic_adapter.py and
agent/prompt_builder.py for the salvaged hunks from PR #4562.
"""

from __future__ import annotations

# Re-export the public surface so `from tools.computer_use import ...` works.
from tools.computer_use.tool import (  # noqa: F401
    handle_computer_use,
    release_computer_use_session,
    set_approval_callback,
    check_computer_use_requirements,
    get_computer_use_schema,
    release_computer_use_session,
)
