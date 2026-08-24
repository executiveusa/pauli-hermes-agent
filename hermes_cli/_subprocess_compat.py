"""Windows subprocess compatibility helpers.

Hermes is developed on Linux / macOS and tested natively on Windows too.
Several common subprocess patterns break silently-or-loudly on Windows:

* ``["npm", "install", ...]`` — on Windows ``npm`` is ``npm.cmd``, a batch
  shim.  ``subprocess.Popen(["npm", ...])`` fails with WinError 193
  ("not a valid Win32 application") because CreateProcessW can't run a
  ``.cmd`` file without ``shell=True`` or PATHEXT resolution.

* ``start_new_session=True`` — on POSIX, this maps to ``os.setsid()`` and
  actually detaches the child.  On Windows it's silently ignored; the
  Windows equivalent is ``CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS``
  creationflags, which Python only applies when you pass them explicitly.

* Console-window flashes — every ``subprocess.Popen`` of a ``.exe`` on
  Windows spawns a cmd window briefly unless ``CREATE_NO_WINDOW`` is
  passed.  Cosmetic but jarring for background daemons.

This module centralizes the platform-branching logic so the rest of the
codebase doesn't sprinkle ``if sys.platform == "win32":`` everywhere.

**All helpers are no-ops on non-Windows** — calling them in Linux/macOS
code paths is safe by design.  That's the "do no damage on POSIX"
guarantee.
"""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
from typing import Sequence

__all__ = [
    "IS_WINDOWS",
    "resolve_node_command",
    "windows_detach_flags",
    "windows_hide_flags",
    "windows_detach_popen_kwargs",
    "kill_process_tree",
    "bounded_probe_run",
    "bounded_git_probe",
]


IS_WINDOWS = sys.platform == "win32"


# -----------------------------------------------------------------------------
# Node ecosystem launcher resolution
# -----------------------------------------------------------------------------


def resolve_node_command(name: str, argv: Sequence[str]) -> list[str]:
    """Resolve a Node-ecosystem command name to an absolute-path argv.

    On Windows, commands like ``npm``, ``npx``, ``yarn``, ``pnpm``,
    ``playwright``, ``prettier`` ship as ``.cmd`` files (batch shims).
    ``subprocess.Popen(["npm", "install"])`` fails with WinError 193
    because CreateProcessW doesn't execute batch files directly.

    ``shutil.which(name)`` *does* resolve ``.cmd`` via PATHEXT and returns
    the fully-qualified path — which CreateProcessW accepts because the
    extension tells Windows to route through ``cmd.exe /c``.

    On POSIX ``shutil.which`` also returns a fully-qualified path when
    found.  That's a small change from bare-name resolution (the OS does
    its own PATH search) but functionally identical and has the side
    benefit of making the argv reproducible in logs.

    Behavior when the command is not on PATH:
    - On Windows: return the bare name — caller can still try with
      ``shell=True`` as a last resort, OR the subsequent Popen will
      raise FileNotFoundError with a readable error we want to surface.
    - On POSIX: same.  Bare ``npm`` on a Linux box without npm installed
      fails the same way it did before this function existed.

    Args:
        name: The command name to resolve (``npm``, ``npx``, ``node`` …).
        argv: The remaining arguments.  Must NOT include ``name`` itself —
            this function builds the full argv list.

    Returns:
        A list suitable for passing to subprocess.Popen/run/call.
    """
    resolved = shutil.which(name)
    if resolved:
        return [resolved, *argv]
    return [name, *argv]


# -----------------------------------------------------------------------------
# Detached / hidden process creation
# -----------------------------------------------------------------------------


# Win32 CreationFlags — defined here rather than imported from subprocess
# because CREATE_NO_WINDOW and DETACHED_PROCESS aren't guaranteed to be
# present on stdlib subprocess on older Pythons or non-Windows builds.
_CREATE_NEW_PROCESS_GROUP = 0x00000200
_DETACHED_PROCESS = 0x00000008
_CREATE_NO_WINDOW = 0x08000000


def windows_detach_flags() -> int:
    """Return Win32 creationflags that detach a child from the parent
    console and process group.  0 on non-Windows.

    Pair with ``start_new_session=False`` (default) when calling
    subprocess.Popen — on POSIX use ``start_new_session=True`` instead,
    which maps to ``os.setsid()`` in the child.

    Rationale:
    - ``CREATE_NEW_PROCESS_GROUP`` — child has its own process group so
      Ctrl+C in the parent console doesn't propagate.
    - ``DETACHED_PROCESS`` — child has no console at all.  Necessary for
      background daemons (gateway watchers, update respawners) because
      without it, closing the console kills the child.
    - ``CREATE_NO_WINDOW`` — suppress the brief cmd flash that would
      otherwise appear when launching a console app.  Redundant with
      DETACHED_PROCESS but explicit for clarity.
    """
    if not IS_WINDOWS:
        return 0
    return _CREATE_NEW_PROCESS_GROUP | _DETACHED_PROCESS | _CREATE_NO_WINDOW


def windows_hide_flags() -> int:
    """Return Win32 creationflags that merely hide the child's console
    window without detaching the child.  0 on non-Windows.

    Use for short-lived console apps spawned as part of a larger
    operation (``taskkill``, ``where``, version probes) where we want no
    flash but also want to collect stdout/exit code synchronously.

    The key difference from :func:`windows_detach_flags`: NO
    ``DETACHED_PROCESS`` — the child still inherits stdio handles so
    ``capture_output=True`` works.  ``DETACHED_PROCESS`` would sever
    stdio and break stdout capture.
    """
    if not IS_WINDOWS:
        return 0
    return _CREATE_NO_WINDOW


def windows_detach_popen_kwargs() -> dict:
    """Return a dict of Popen kwargs that detach a child on Windows and
    fall back to the POSIX equivalent (``start_new_session=True``) on
    Linux/macOS.

    Usage pattern:

    .. code-block:: python

        subprocess.Popen(
            argv,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            stdin=subprocess.DEVNULL,
            close_fds=True,
            **windows_detach_popen_kwargs(),
        )

    This replaces the unsafe-on-Windows pattern:

    .. code-block:: python

        subprocess.Popen(..., start_new_session=True)

    which silently fails to detach on Windows (the flag is accepted but
    has no effect — the child stays attached to the parent's console
    and dies when the console closes).
    """
    if IS_WINDOWS:
        return {"creationflags": windows_detach_flags()}
    return {"start_new_session": True}


# -----------------------------------------------------------------------------
# Deadlock-safe bounded probes
#
# Vendored from upstream NousResearch/hermes-agent at tag v2026.8.16.2, as a
# dependency of agent/coding_context.py (openspec/changes/hermes-upstream-
# gap-map, "coding posture" item). Functionally unchanged from upstream.
# -----------------------------------------------------------------------------


def kill_process_tree(proc: "subprocess.Popen") -> None:
    """Best-effort terminate *proc* and its descendants on both platforms.

    ``proc.kill()`` alone only terminates the direct child. On Windows a
    suspended descendant (e.g. ``git.exe``) can survive holding duplicates of the
    captured pipe handles, which keeps the pipes from reaching EOF and leaks two
    reader threads + the process per fired timeout — ``taskkill /T /F`` takes the
    whole tree down so the bounded drain that follows can actually reach EOF.
    On POSIX the same class exists: killing the launcher leaves descendants
    (credential helpers, ``git-remote-https``, hook children) running and
    holding the pipe write ends. Callers spawn the child in its own process
    group (``process_group=0``, Python ≥3.11), so when — and only
    when — the child leads its own group (``pgid == pid``), the entire group is
    signalled with ``os.killpg``. The ownership check means a fallback spawn
    that shares our group can never cause us to kill unrelated processes.
    Ported from openai/codex#36793 ("Terminate timed-out Git process trees");
    generalized for the shell-hook runner via openai/codex#37527
    ("Terminate timed-out hook process trees").

    All failures are swallowed — this is cleanup on an already-failing path, and
    the caller's contract is to fail open. ``kill()`` can raise (access denied,
    already reaped); an unhandled raise here would escape the caller's ``except``
    handler and break that contract. The ``taskkill`` spawn itself cannot
    re-enter the deadlock class it fixes: it captures no pipes (DEVNULL), so its
    own timeout cleanup has no reader threads to join.
    """
    if not IS_WINDOWS:
        # Group-kill first: verify the child actually leads its own process
        # group before signalling it, so we never blast a shared group.
        try:
            import signal as _signal

            pgid = os.getpgid(proc.pid)
            if pgid == proc.pid:
                os.killpg(pgid, _signal.SIGKILL)  # windows-footgun: ok — inside `if not IS_WINDOWS` gate
        except Exception:
            pass
    try:
        proc.kill()
    except OSError:
        pass
    if IS_WINDOWS:
        try:
            subprocess.run(
                ["taskkill", "/T", "/F", "/PID", str(proc.pid)],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                stdin=subprocess.DEVNULL,
                timeout=2,
                check=False,
                creationflags=windows_hide_flags(),
            )
        except Exception:
            pass


def bounded_probe_run(
    argv: Sequence[str],
    *,
    timeout: float,
    errors: str = "replace",
) -> "subprocess.CompletedProcess[str] | None":
    """Deadlock-safe ``subprocess.run(argv, capture_output=True, timeout=...)``
    for fail-open probe call sites. Returns a ``CompletedProcess`` when the
    child finished within *timeout* (any exit code), or ``None`` on spawn
    failure or timeout.

    Why not ``subprocess.run``: on Windows, ``run()``'s post-timeout cleanup
    calls an *unbounded* ``communicate()`` after killing the direct child.
    Killing it can leave a descendant (``git.exe`` under a launcher shim,
    ``conhost.exe`` under wmic/powershell) holding duplicates of the captured
    stdout/stderr handles, so the pipes never reach EOF and the reader-thread
    join blocks forever.

    The bounded flow: an explicit ``communicate(timeout)``, then on any
    failure a tree-kill (see :func:`kill_process_tree`) plus a bounded 1s
    post-kill drain; if the pipes are still held after that, they're abandoned
    (the orphaned reader threads are daemonic and cost nothing).

    The spawn contract mirrors the ``run`` calls it replaces: PIPE/PIPE/DEVNULL,
    ``text`` with UTF-8 decoding (*errors* configurable — the process scans use
    ``"ignore"``), and the hidden-window ``creationflags`` on Windows only. On
    POSIX the child is placed in its own process group (``process_group=0``,
    Python ≥3.11) so timeout cleanup can take down descendants with the
    launcher instead of orphaning them.
    """
    _popen_kwargs: dict = {"creationflags": windows_hide_flags()} if IS_WINDOWS else {"process_group": 0}
    try:
        proc = subprocess.Popen(
            list(argv),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            stdin=subprocess.DEVNULL,
            text=True,
            encoding="utf-8",
            errors=errors,
            **_popen_kwargs,
        )
    except Exception:
        return None
    try:
        stdout, stderr = proc.communicate(timeout=timeout)
    except Exception:
        # Timeout OR any other communicate() failure (torn-down pipe, decode
        # error): terminate the child + descendants and drain bounded. Leaving
        # it running would leak the same suspended-descendant class this guards.
        kill_process_tree(proc)
        try:
            proc.communicate(timeout=1)
        except Exception:
            pass
        return None
    return subprocess.CompletedProcess(list(argv), proc.returncode, stdout, stderr)


def bounded_git_probe(argv: Sequence[str], *, timeout: float) -> str:
    """Run a short, throwaway ``git`` probe and return stripped stdout, or ``""``
    on ANY failure (nonzero exit, timeout, spawn error, decode error).

    This is the shared, deadlock-safe replacement for
    ``subprocess.run(["git", ...], timeout=...)`` at fail-open probe call sites
    (``agent.coding_context._git``).

    Why not ``subprocess.run``: on Windows, ``run()``'s post-timeout cleanup
    calls an *unbounded* ``communicate()`` after killing git. Killing the
    PATH-resolved launcher can leave a suspended descendant ``git.exe`` holding
    duplicates of the captured stdout/stderr handles, so the pipes never reach
    EOF and the reader-thread join blocks forever.
    """
    result = bounded_probe_run(argv, timeout=timeout)
    if result is None or result.returncode != 0:
        return ""
    return (result.stdout or "").strip()
