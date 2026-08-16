# Miscellaneous capability library

Load only for the specific maintenance trigger. These are not default workflow stages.

## git-guardrails-claude-code
Install pre-execution guardrails around destructive Git operations in agent-driven environments. Block or require explicit human approval for high-risk commands such as force push, reset --hard, clean, destructive branch deletion, and equivalent history/data-loss operations. Preserve a documented escape hatch for deliberate recovery work.

## migrate-to-shoehorn
For Total TypeScript-style test code, replace brittle `as` assertions with `@total-typescript/shoehorn` patterns where doing so improves test fixtures without changing production behavior. Scope narrowly and run the affected test/type-check suite.

## scaffold-exercises
Create a learning exercise structure with explicit sections, problems, starting files, solutions, and explainers. Keep the exercise contract separate from the solution and ensure each step has a verifiable learning objective.

## setup-pre-commit
Configure a pre-commit quality gate appropriate to the repository, typically formatting/linting, type checks, and targeted tests. Inspect existing tooling first; do not introduce Husky/lint-staged or duplicate hooks if the repo already has an equivalent mechanism. Keep hooks fast enough that developers will not bypass them.
