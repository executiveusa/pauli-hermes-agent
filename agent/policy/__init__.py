"""Hermes policy layer - ACIP injection defense + DCG destructive-command gates.

Design lifted from agent-flywheel's meta_skill (ACIP) and DCG/SLB projects
(2026-09-12 adoption review). This package is the single import surface for
the fleet's prompt-injection and destructive-command defenses:

- ``agent.policy.acip``   - untrusted-content isolation + injection signal scan
- ``agent.policy.dcg``    - destructive command classification and gating

Enforcement seams that already exist and are exercised by the regression
harness in ``tests/agent/policy/``:

- ``agent.tool_dispatch_helpers.make_tool_result_message`` wraps web/browser/
  MCP tool results in ``<untrusted_tool_result>`` delimiters (ACIP isolation).
- ``tools.approval.check_all_command_guards`` gates terminal commands against
  HARDLINE + DANGEROUS pattern tiers with an approval callback (DCG).
"""

from agent.policy.acip import (
    INSTRUCTION_HIERARCHY_PREAMBLE,
    InjectionSignal,
    scan_for_injection,
    wrap_untrusted,
)
from agent.policy.dcg import (
    CommandVerdict,
    Tier,
    classify_command,
)

__all__ = [
    "INSTRUCTION_HIERARCHY_PREAMBLE",
    "InjectionSignal",
    "scan_for_injection",
    "wrap_untrusted",
    "CommandVerdict",
    "Tier",
    "classify_command",
]
