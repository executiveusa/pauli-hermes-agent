"""DCG-style destructive-command gate for Hermes.

Design lifted from agent-flywheel's DCG (dangerous command gate) / SLB work,
adopted 2026-09-12. This module is the policy surface; enforcement lives in
``tools.approval.check_all_command_guards``, which already gates every
``terminal`` / ``execute_code`` call against HARDLINE and DANGEROUS pattern
tiers with an approval callback.

The three tiers, from the DCG design:

- ``HARDLINE``  - catastrophic, no recovery story (wipe a disk, fork bomb,
  nuke the repo's git history). Blocked outright; the approval callback is
  not consulted because no in-session yes makes them safe.
- ``DANGEROUS`` - destructive but recoverable-with-cost (rm -rf a temp dir,
  git reset --hard, force push, pipe a script to a shell). Requires explicit
  human approval; 'always' persistence is per-pattern and recorded.
- ``SAFE``      - everything else. Passes without interaction.

``classify_command`` here is the read-only policy view: it reports the tier
and matched rule without executing or approving anything, so Hermes (and
Heisenberg above it) can plan around gates instead of discovering them
mid-mission.
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

from tools.approval import (
    DANGEROUS_PATTERNS_COMPILED,
    HARDLINE_PATTERNS_COMPILED,
    detect_dangerous_command,
)


class Tier(str, Enum):
    SAFE = "safe"
    DANGEROUS = "dangerous"
    HARDLINE = "hardline"


@dataclass(frozen=True)
class CommandVerdict:
    command: str
    tier: Tier
    rule: str
    needs_approval: bool
    blocked_outright: bool


def classify_command(command: str) -> CommandVerdict:
    """Classify a shell command against the DCG tiers. Never executes it."""
    for pattern_re, description in HARDLINE_PATTERNS_COMPILED:
        if pattern_re.search(command):
            return CommandVerdict(
                command=command,
                tier=Tier.HARDLINE,
                rule=description,
                needs_approval=False,
                blocked_outright=True,
            )
    is_dangerous, _key, description = detect_dangerous_command(command)
    if is_dangerous:
        return CommandVerdict(
            command=command,
            tier=Tier.DANGEROUS,
            rule=description or "dangerous pattern",
            needs_approval=True,
            blocked_outright=False,
        )
    return CommandVerdict(
        command=command, tier=Tier.SAFE, rule="", needs_approval=False, blocked_outright=False
    )


__all__ = ["Tier", "CommandVerdict", "classify_command", "DANGEROUS_PATTERNS_COMPILED"]
