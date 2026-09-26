# Hermes policy layer: ACIP + DCG (adopted 2026-09-12)

Design lifted from the agent-flywheel mining review: meta_skill's ACIP
injection defense and the DCG destructive-command gate, mapped onto Hermes'
existing enforcement seams and extended where the regression harness found
gaps.

## ACIP - prompt-injection defense

**Isolation (architectural).** Every attacker-controllable tool result
(web, browser, MCP) is wrapped in `<untrusted_tool_result>` delimiters by
`agent.tool_dispatch_helpers.make_tool_result_message`; the model is told
the payload is data, never instructions. `agent.policy.acip` adds:

- `INSTRUCTION_HIERARCHY_PREAMBLE` - explicit hierarchy for system prompts
- `wrap_untrusted()` - the canonical wrapper; **neutralizes embedded closing
  delimiters** so a payload cannot escape from within
- `scan_for_injection()` - signal detection across 10 attack families
  (instruction override, fake system messages, role hijack, authority
  laundering, delimiter escape, credential probes, exfiltration URLs,
  suppression requests, deferred execution, zero-width/base64 smuggling)

**Gap found and fixed by the harness (2026-09-12):** the dispatch wrapper
interpolated payload text verbatim - a poisoned page containing
`</untrusted_tool_result>` broke out of isolation. The wrap now
case-insensitively escapes embedded closing tags. 4 harness fixtures prove
the escape is closed across all untrusted tool classes.

## DCG - destructive-command gate

`agent.policy.dcg.classify_command` is the read-only policy view over the
existing `tools.approval` enforcement (every `terminal`/`execute_code` call):

- HARDLINE (fork bomb, `rm -rf /`, `dd`/`mkfs` on disks): blocked outright
- DANGEROUS (`git push --force`, `git reset --hard`, `curl|sh`, `rm -rf`
  scoped dirs): explicit human approval, per-pattern 'always' persistence
- SAFE: passes

## Regression harness

`tests/agent/policy/test_injection_harness.py` - 68 tests. Fixtures are
authored from the attack *categories* in the L1B3RT4S / P4RS3LT0NGV3
corpora. **Defensive use only: no corpus payloads are imported, and none of
these strings ever enter a production prompt.** A failure here means an
attacker-shaped input regressed a fleet gate - treat as a security defect.
