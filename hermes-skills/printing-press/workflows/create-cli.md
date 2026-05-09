# Workflow: Create a new CLI with Printing Press

## Goal
Generate an agent-native CLI when no suitable installed CLI exists.

## Inputs
- API spec, website URL, HAR capture, or app URL.
- Target command scope and critical workflows.

## Steps
1. Install Printing Press generator:
   ```bash
   go install github.com/mvanhorn/cli-printing-press/v4/cmd/printing-press@latest
   ```
2. Run generator in codex mode:
   ```bash
   pp-new <target-app-or-url> codex
   ```
3. Add local conventions:
   - `--json --compact` first-class output mode.
   - Read-only and destructive command annotations.
   - Local SQLite cache/sync/search commands.
4. Implement at least one compound workflow command that replaces a multi-step API loop.
5. Smoke test against 3 real tasks.
6. Draft skill notes and registry entry.

## Exit criteria
- Generated CLI completes three representative tasks.
- SQLite-backed sync/search path exists where practical.
- Registry status set to `candidate` or `dogfooded`.
