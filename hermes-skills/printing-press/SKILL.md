---
name: printing-press
description: Default Hermes workflow for turning repeatable SaaS/API/web operations into local token-efficient CLIs and reusable agent skills.
version: 0.1.0
author: Hermes Team
license: MIT
metadata:
  hermes:
    category: devops
    tags:
      - cli
      - automation
      - token-efficiency
      - workflow
      - printing-press
---

# Printing Press Skill

## Mission
Adopt Printing Press as Hermes's default **agent tool factory** for repeatable SaaS/API/web workflows.

## Default policy
1. Prefer local CLI + skill over raw browser exploration.
2. Prefer local CLI + skill over ad-hoc direct API loops.
3. Prefer local CLI + skill over bulky MCP context ingestion.

## Core aliases
```bash
alias pp-install='npx -y @mvanhorn/printing-press install'
alias pp-list='npx -y @mvanhorn/printing-press list'
alias pp-search='npx -y @mvanhorn/printing-press search'
alias pp-new='printing-press'
```

## Primary workflows
- `workflows/install-library-cli.md`
- `workflows/create-cli.md`
- `workflows/audit-cli.md`
- `workflows/promote-cli.md`
- `workflows/token-budget.md`

## Standard lifecycle
1. User asks for an outcome.
2. Check Hermes CLI registry for an existing tool.
3. If found, run the registered CLI skill.
4. If missing, inspect API/site/spec/HAR and generate a CLI with Printing Press.
5. Dogfood on 3 real tasks.
6. Add compact commands + skill notes.
7. Register CLI in registry.
8. Promote to production status.

## Candidate triggers
- Any workflow repeated 3 times is a CLI candidate.
- Any workflow with high token/API/browser cost is an immediate CLI candidate.
- Any workflow touching money, legal actions, private data, or public publishing requires approval gates before autonomous execution.

## Codex mode default
Use `codex` mode by default when invoking Printing Press generation:

```bash
/printing-press <target-app-or-url> codex
```

Reason: offloads bulk code generation while keeping the planning/review loop lean and token efficient.
