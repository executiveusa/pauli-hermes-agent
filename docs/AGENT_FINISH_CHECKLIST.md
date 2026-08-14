# Hermes Finish Checklist

## Mission
Finish Hermes without breaking the Pauli architecture. Upstream Hermes is a capability source, not a branch to merge wholesale.

## Locked role
Hermes is the business orchestrator/governor. It plans, delegates, verifies, reports, and routes work across the fleet. It does not replace Pi, BARS, Jarvis, Lightning, or Pauli's Place.

## Non-negotiable boundaries
- Pauli's Place remains canonical mission/evidence state.
- Pi owns Personal/Human OS work; Hermes receives bounded business handoffs.
- BARS owns long-running computer/media/operator work; Hermes retains direct computer use as an executive capability.
- Jarvis owns presence/voice/device communications surfaces.
- Lightning observes, evaluates, and curates memory without becoming duplicate execution authority.
- No secrets in source, logs, issues, or PRs.
- No production/domain changes without verified preview evidence and approval.

## 1. Baseline and fork safety
- [ ] Record current `main` SHA and clean baseline.
- [ ] Identify canonical upstream Hermes repository and common ancestor.
- [ ] Generate upstream-vs-Pauli capability matrix: PORT / ADAPT / KEEP OURS / PARK.
- [ ] Identify overlap files where Pauli architecture modified upstream core.
- [ ] Create rollback point before any upstream capability port.
- [ ] Never perform a blind upstream merge.

## 2. Core powers
- [ ] Computer use is mandatory.
- [ ] Windows computer-use adapter: UI Automation + screenshot/vision + mouse/keyboard fallback + PowerShell/CLI.
- [ ] Preserve macOS computer-use compatibility where upstream supports it.
- [ ] Browser automation reconciled and tested.
- [ ] Terminal and background-process control reconciled.
- [ ] File operations reconciled.
- [ ] Code execution reconciled.
- [ ] Subagent delegation reconciled.
- [ ] Multi-agent/reviewer path evaluated.
- [ ] Clarification/human-blocker behavior preserved.

## 3. Upstream platform features
- [ ] Updater/snapshot/rollback safety ported or adapted.
- [ ] Plugin architecture evaluated and integrated without duplicating Pauli authority.
- [ ] Gateway/plugin/shell hooks evaluated and integrated.
- [ ] Skill registry/progressive disclosure reconciled.
- [ ] Self-authored/self-repaired skills require review/evidence gates.
- [ ] MCP/FastMCP/remote gateway support reconciled.
- [ ] Cron/scheduled work reconciled.
- [ ] Watchers evaluated for feed/API/GitHub monitoring.
- [ ] Messaging/email surfaces reconciled with approval policy.
- [ ] Session search and memory boundaries reconciled.
- [ ] Observability linked to mission/evidence IDs.
- [ ] Authenticated webhook-triggered runs evaluated.

## 4. High-value skills
### GitHub / engineering
- [ ] codebase inspection
- [ ] GitHub auth
- [ ] code review
- [ ] issue-to-PR
- [ ] issues
- [ ] PR workflow
- [ ] repo management
- [ ] planning
- [ ] systematic debugging
- [ ] TDD
- [ ] simplify-code
- [ ] requesting code review
- [ ] subagent-driven development

### Research
- [ ] grounded citations
- [ ] competitor/news monitoring
- [ ] blocked-page recovery
- [ ] domain intelligence
- [ ] OSINT
- [ ] web search/scraping
- [ ] paper/research workflows

### Productivity
- [ ] Google Workspace
- [ ] Notion
- [ ] Airtable
- [ ] documents/PDF/XLSX/PPTX
- [ ] meeting-to-action items
- [ ] weekly review/planning

### DevOps / evaluation
- [ ] Docker
- [ ] watchers
- [ ] tunnels where justified
- [ ] LLM evaluation harness
- [ ] adversarial UX testing
- [ ] Gauntlet integration
- [ ] optional specialist ML/model skills remain lazy/on-demand

## 5. Authority and interoperability
- [ ] Stable request/result envelope across Hermes, Pi, BARS, Jarvis, Lightning.
- [ ] `personal` routes to Pi.
- [ ] `business` routes to Hermes.
- [ ] long computer/media jobs can route to BARS.
- [ ] voice/presence routes through Jarvis.
- [ ] observations/evaluations route to Lightning without execution authority.
- [ ] mission/evidence IDs survive every hop.

## 6. Safety
- [ ] Classify side effects: read-only / reversible / external write / money / destructive.
- [ ] Enforce human approval for high-risk side effects.
- [ ] Secrets redaction tests pass.
- [ ] Tool allow/deny policy tested.
- [ ] Concurrent-session/worktree isolation tested.
- [ ] Retry/idempotency/reconciliation behavior tested.

## 7. Golden-path proof
- [ ] One owner command enters Hermes.
- [ ] Hermes plans and delegates.
- [ ] A worker/tool executes.
- [ ] Evidence is captured.
- [ ] Independent review occurs.
- [ ] Lightning observes the outcome.
- [ ] Useful memory is curated.
- [ ] Owner report follows DECISION / CHANGES / PROOF / STATUS / COMMERCIAL IMPACT / RISKS / ROLLBACK / NEXT / HUMAN APPROVAL.

## Definition of done
Hermes is finished when selected upstream capabilities are reconciled behind tests, the Pauli-specific authority model remains intact, Windows/browser/computer-use paths are real, one end-to-end mission is proven, and consequential actions are observable, permissioned, and reversible.
