"""Regression tests for Computer Use readiness under a thin GUI PATH.

Vendored verbatim from upstream NousResearch/hermes-agent at tag
v2026.8.16.2, part of hermes-upstream-gap-map item #2 ("Computer-use
diagnostics + permission model + lifecycle cleanup" —
openspec/changes/hermes-upstream-gap-map). New file — the fork had no
prior permissions test coverage. Originally at ``tests/computer_use/
test_permissions_resolution.py`` upstream; moved to ``tests/tools/`` and
renamed ``test_computer_use_permissions_resolution.py`` to match this
fork's existing `tests/tools/test_computer_use*.py` convention. Only this
note and the file's location/name were changed; test bodies are
unmodified upstream.
"""

from __future__ import annotations

import sys
from unittest.mock import MagicMock, patch

import pytest


@pytest.mark.skipif(sys.platform == "win32", reason="POSIX user-local path regression")
def test_status_finds_user_local_driver_when_path_omits_it(tmp_path, monkeypatch):
    """Desktop status must agree with the runtime resolver, not bare PATH."""
    from tools.computer_use import permissions

    driver = tmp_path / ".local" / "bin" / "cua-driver"
    driver.parent.mkdir(parents=True)
    driver.write_text("#!/bin/sh\nexit 0\n")
    driver.chmod(0o755)

    monkeypatch.delenv("HERMES_CUA_DRIVER_CMD", raising=False)
    monkeypatch.setenv("HOME", str(tmp_path))
    monkeypatch.setenv("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")

    # No platform faking: ``~/.local/bin/cua-driver`` is a POSIX resolution
    # candidate on Linux exactly as on macOS, so the regression reproduces on
    # the host we actually run on.
    with patch.object(permissions, "_run", return_value=MagicMock(stdout="0.0.0")), \
         patch.object(permissions, "_doctor", return_value={"ok": True, "checks": []}):
        status = permissions.computer_use_status()

    assert status["installed"] is True
