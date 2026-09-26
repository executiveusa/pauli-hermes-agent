"""Regression: the ZIP-update directory replace must never leave a half-deleted tree.

Ported (gap #7 audit) from NousResearch/hermes-agent @ v2026.8.16.2,
``tests/hermes_cli/test_update_zip_atomic_replace.py`` — adapted to import
``_atomic_replace_dir`` from this fork's ``hermes_cli.main`` (where the
update logic lives inline) rather than the separate ``hermes_cli.update_cmd``
module upstream extracted it into. Only the single-entry success-path test is
ported; upstream's later two-phase/whole-loop rollback tests
(``test_update_zip_two_phase.py``) exercise a loop-wide atomicity
generalization (issue #76104) that restructures the surrounding update loop
and was judged out of scope for this bounded fix.

Issue #49145: on Windows the ZIP-update path did ``rmtree(dst); copytree(...)``.
A copy that failed partway (file locks / flaky I/O -- the very conditions the
ZIP path exists to work around) left the directory deleted with nothing
copied back, which broke ``hermes --tui`` because ``ui-tui/`` had vanished.

``_atomic_replace_dir`` stages the new copy first and only swaps it in on full
success, so a mid-copy failure leaves the original directory intact.
"""

from __future__ import annotations

from pathlib import Path

from hermes_cli.main import _atomic_replace_dir


def test_atomic_replace_swaps_content_on_success(tmp_path: Path) -> None:
    src = tmp_path / "src" / "ui-tui"
    src.mkdir(parents=True)
    (src / "new.txt").write_text("NEW")

    dst = tmp_path / "install" / "ui-tui"
    dst.mkdir(parents=True)
    (dst / "old.txt").write_text("OLD")

    _atomic_replace_dir(str(src), str(dst))

    assert (dst / "new.txt").read_text() == "NEW"
    assert not (dst / "old.txt").exists()
    # No staging/backup siblings left behind.
    assert not (dst.parent / "ui-tui.hermes-update-staging").exists()
    assert not (dst.parent / "ui-tui.hermes-update-old").exists()


def test_atomic_replace_creates_dst_when_absent(tmp_path: Path) -> None:
    src = tmp_path / "src" / "ui-tui"
    src.mkdir(parents=True)
    (src / "new.txt").write_text("NEW")

    dst = tmp_path / "install" / "ui-tui"
    dst.parent.mkdir(parents=True)

    _atomic_replace_dir(str(src), str(dst))

    assert (dst / "new.txt").read_text() == "NEW"
    assert not (dst.parent / "ui-tui.hermes-update-staging").exists()
