# Hermes Skill Inventory

## Summary

- Hermes source includes built-in skill families under `skills/` and optional skill families under `optional-skills/`.
- The Pauli stack should use Hermes as the registry/runtime, not as an always-loaded mega prompt.
- `browser-harness` is mandatory for Pauli browser automation workflows even though it is currently delivered through the Codex side.

## Verified Core Families

- Planning/execution: `plan`, `writing-plans`, `systematic-debugging`, `test-driven-development`, `requesting-code-review`, `subagent-driven-development`.
- GitHub/repo work: `github-auth`, `github-repo-management`, `github-pr-workflow`, `github-code-review`, `github-issues`, `codebase-inspection`.
- Runtime/MCP: `native-mcp`, `webhook-subscriptions`.
- Design/media references: `claude-design`, `popular-web-designs`, `design-md`, `sketch`, `youtube-content`.
- Optional candidates already present in source: `docker-management`, `hyperframes`, `kanban-video-orchestrator`, `chroma`, `pinecone`, `qdrant`, `guidance`, `instructor`.

## Decision

- Install a curated Pauli stack.
- Keep stock Hermes skill discovery intact.
- Route only 2 to 6 skills into active context per task.
- Keep Pauli custom skills in-repo under `skills/pauli/` and resolve them through the Pauli router.

## Browser Harness

- Mandatory first browser-control surface for frontend verification.
- Current implementation path: Codex `browser-harness` skill/plugin.
- Future Hermes adapter should preserve the same behavior instead of duplicating browser logic.
