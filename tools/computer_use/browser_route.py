"""Placeholder stub for the typed cua-driver browser route.

NOT vendored from upstream. This is fork-native scaffolding written to
satisfy an import, deliberately scoped out of the
"Cross-platform upstream computer-use package" gap (hermes-upstream-gap-map
item #1, openspec/changes/hermes-upstream-gap-map/proposal.md).

Why this file exists
---------------------
Upstream NousResearch/hermes-agent at tag v2026.8.16.2 grew a real
``tools/computer_use/browser_route.py`` (644 lines) implementing a
session-scoped "typed" browser automation adapter (``cua_browser_*``
actions: state, prepare, navigate, click, type, pointer, dialog,
set_input_files, download) on top of cua-driver's ``browser_*`` MCP tools.
``tools/computer_use/cua_backend.py`` — which *is* in scope for gap #1 and
is vendored verbatim in this same PR — imports
``from tools.computer_use.browser_route import CuaTypedBrowserRoute`` at
module load time, so leaving the file out entirely would break the import
of the whole computer_use package, not just the browser feature.

Real typed-browser routing is intentionally NOT ported here. It is a
separate, self-contained feature (its own state machine, ref/capability
model, and ~5 dedicated upstream test files:
test_computer_use_cua_0_9.py, test_computer_use_cua_0_10_permissions.py,
test_computer_use_browser_authorization.py,
test_computer_use_browser_contract_020.py) layered on top of — not
required by — cross-platform desktop control, which is what gap #1 asks
for. Porting it now would roughly double this PR's diff for a feature
orthogonal to "make computer_use work on Windows/Linux, not just macOS."

What this stub does instead
----------------------------
It defines the same public shape cua_backend.py imports
(``CuaTypedBrowserRoute`` with ``observe`` / ``prepare`` / ``mutate``, and
a ``.state`` object with a ``.clear()`` cua_backend.py's transport-reset
handler calls) but every entry point returns a clean "unavailable"
refusal — the exact same
``{"ok": False, "status": "refused", "code": "typed_browser_unavailable",
"native_fallback_required": True}`` shape that upstream's own
``tools/computer_use/backend.py::ComputerUseBackend`` base class already
returns by default for any backend that doesn't implement typed browser
routing at all (see its ``_typed_browser_unavailable`` helper). So a model
that reaches for ``cua_browser_*`` (still present in schema.py's action
enum, vendored unmodified) gets a well-formed refusal telling it to fall
back to the native AX/PX/foreground actions, not a crash or an
AttributeError.

Follow-up
---------
Port the real ``tools/computer_use/browser_route.py`` — together with
``doctor.py`` and ``permissions.py`` (hermes-upstream-gap-map item #2:
"computer-use diagnostics + permission model + lifecycle cleanup"), or as
its own dedicated PR if it turns out not to fit cleanly into #2 — and
delete this stub in that same change.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Callable, Dict, Optional, Set


ToolCaller = Callable[[str, Dict[str, Any]], Dict[str, Any]]
ToolProbe = Callable[[str], bool]


def _refusal(code: str, message: str, **extra: Any) -> Dict[str, Any]:
    payload: Dict[str, Any] = {
        "ok": False,
        "status": "refused",
        "code": code,
        "message": message,
    }
    payload.update(extra)
    return payload


@dataclass
class BrowserRouteState:
    """Placeholder state container.

    Mirrors the shape of upstream's real ``BrowserRouteState`` closely
    enough that ``cua_backend.py``'s ``_handle_transport_reset`` (which
    calls ``route.state.clear()`` on transport reset) keeps working
    without special-casing the stub.
    """

    pid: Optional[int] = None
    window_id: Optional[int] = None
    target_id: Optional[str] = None
    tab_ids: Set[str] = field(default_factory=set)
    tab_id: Optional[str] = None
    binding_quality: Optional[str] = None
    mutation_allowed: bool = False
    refs: Dict[str, Set[str]] = field(default_factory=dict)
    continuation: Optional[str] = None
    verification_required: bool = False

    def clear_refs(self) -> None:
        self.refs.clear()
        self.continuation = None

    def clear(self) -> None:
        self.pid = None
        self.window_id = None
        self.target_id = None
        self.tab_ids.clear()
        self.tab_id = None
        self.binding_quality = None
        self.mutation_allowed = False
        self.clear_refs()
        self.verification_required = False


class CuaTypedBrowserRoute:
    """Placeholder adapter — see module docstring.

    Every entry point refuses cleanly instead of performing typed browser
    automation. Not vendored from upstream; matches upstream's
    ``CuaTypedBrowserRoute.__init__`` signature only so ``cua_backend.py``
    (vendored verbatim) can construct it without modification.
    """

    def __init__(
        self,
        *,
        session_id: str,
        call_tool: ToolCaller,
        has_tool: ToolProbe,
    ) -> None:
        self._session_id = session_id
        self._call_tool = call_tool
        self._has_tool = has_tool
        self.state = BrowserRouteState()

    def observe(self, **kwargs: Any) -> Dict[str, Any]:
        return self._unavailable()

    def prepare(self, **kwargs: Any) -> Dict[str, Any]:
        return self._unavailable()

    def mutate(
        self,
        tool: str,
        *,
        tab_id: Optional[str] = None,
        args: Optional[Dict[str, Any]] = None,
    ) -> Dict[str, Any]:
        return self._unavailable()

    def _unavailable(self) -> Dict[str, Any]:
        return _refusal(
            "typed_browser_unavailable",
            "Typed cua_browser_* routing is not ported in this fork yet "
            "(gap #1 landed cross-platform desktop control only; typed "
            "browser routing is a scoped-out follow-up — see this "
            "module's docstring). Use the native AX/PX/foreground ladder "
            "instead.",
            native_fallback=True,
        )
