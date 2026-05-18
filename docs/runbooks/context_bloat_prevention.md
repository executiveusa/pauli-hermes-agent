# Context Bloat Prevention

## Core Rule

Load only the skills the current task needs.

## Practical Limits

- Default task: 3 skills max.
- Complex repo task: 6 skills max.
- Video task: 5 skills max.

## Memory Rule

- Search memory first.
- Do not dump the full vault or project corpus into the prompt.

## Repo Rule

- Use `jcodemunch` and `codebase-inspection` before broad file ingestion on large repos.

## Browser Rule

- Use `browser-harness` for UI verification after substantial frontend changes.
