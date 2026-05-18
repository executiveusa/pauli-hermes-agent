# Install Selected Hermes Skills

## Suggested Command Set

```powershell
hermes skills install hermes-agent
hermes skills install hermes-agent-skill-authoring
hermes skills install native-mcp
hermes skills install plan
hermes skills install writing-plans
hermes skills install systematic-debugging
hermes skills install requesting-code-review
hermes skills install subagent-driven-development
hermes skills install kanban-orchestrator
hermes skills install webhook-subscriptions
hermes skills install github-auth
hermes skills install github-repo-management
hermes skills install github-pr-workflow
hermes skills install github-code-review
hermes skills install github-issues
hermes skills install codebase-inspection
hermes skills install test-driven-development
hermes skills install python-debugpy
hermes skills install node-inspect-debugger
hermes skills install docker-management
hermes skills install claude-design
hermes skills install popular-web-designs
hermes skills install design-md
hermes skills install sketch
hermes skills install architecture-diagram
hermes skills install excalidraw
hermes skills install p5js
hermes skills install youtube-content
hermes skills install hyperframes
hermes skills install kanban-video-orchestrator
hermes skills install comfyui
hermes skills install obsidian
hermes skills install notion
hermes skills install google-workspace
hermes skills install ocr-and-documents
hermes skills install llm-wiki
hermes skills install honcho
hermes skills install chroma
```

## Notes

- Optional skills can be installed now and health-gated at runtime.
- Repo-local Pauli skills are not installed with `hermes skills install`; they are loaded from this repo by the Pauli router.
- Browser automation should use `browser-harness` from Codex until a Hermes-facing adapter is wired.
