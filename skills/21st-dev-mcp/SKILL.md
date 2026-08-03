---
name: 21st-dev-mcp
description: Connect Hermes to 21st.dev for governed UI component search, inspection, installation, generation, review, and publishing without exposing credentials or bypassing project design rules.
version: 0.1.0
platforms: [windows, linux, macos]
---

# 21st.dev MCP Skill

## Purpose

Use 21st.dev as a specialist UI catalog and MCP service inside the existing Hermes website-design workflow. This skill does not replace project discovery, content strategy, wireframes, design-system inspection, accessibility review, or production verification.

Official integration surfaces:

- CLI package: `@21st-dev/cli`
- MCP endpoint: `https://21st.dev/api/mcp`
- interactive authentication: `21st login`
- non-interactive secret: `API_KEY_21ST`

Never print, commit, log, paste into a PR, or store the full API key in project files.

## Trigger conditions

Use this skill when the user asks Hermes to:

- search for existing React UI components, themes, templates, or SVG logos;
- compare multiple design directions before implementation;
- install a 21st.dev component into an existing frontend;
- review a frontend with 21st.dev tools;
- publish or manage an owner-approved component library;
- connect 21st.dev to Hermes or another approved MCP client.

Do not trigger only because a page needs styling. First inspect the existing codebase and determine whether a new dependency or external component is justified.

## Required operating mode

Treat all existing projects as brownfield unless the user explicitly declares greenfield.

Before any install or generation:

1. Identify the target repository and branch.
2. Record the current commit SHA and working-tree state.
3. Identify framework, package manager, component conventions, CSS system, aliases, and existing design tokens.
4. State the exact UI problem and acceptance criteria.
5. Search existing local components before adding an external one.
6. Define rollback: files and dependencies to remove or revert.

## Connection options

### Option A — CLI-first Hermes workflow

Use when Hermes has shell access and can run the 21st CLI:

```bash
npm i -g @21st-dev/cli
21st login
21st whoami
21st search "pricing table"
```

`21st login` opens a browser and stores authentication locally. The human owner must complete login and approve any account or billing action.

Install 21st's agent skills only after reviewing what the command adds:

```bash
npx @21st-dev/cli install-skill
```

### Option B — MCP HTTP connection

Use for an MCP-capable client:

```text
endpoint: https://21st.dev/api/mcp
auth header: x-api-key: ${API_KEY_21ST}
```

For clients supported by the 21st CLI, prefer its merge-safe initializer rather than hand-editing configuration:

```bash
21st init --client claude
21st init --client cursor
21st init --client codex
```

Inspect the resulting diff. The initializer must not overwrite other MCP servers. Never commit a resolved key value; committed configuration must reference `${API_KEY_21ST}` or another approved secret store.

## Secret policy

Allowed:

- browser login performed by the owner;
- environment variable `API_KEY_21ST` stored in the user's secure environment;
- OS credential manager or an approved secrets manager;
- masked presence checks such as `configured: true`.

Forbidden:

- placing the key in `SKILL.md`, `.mcp.json`, README files, screenshots, chat transcripts, logs, or commits;
- passing the key in a command that will remain in shell history when an environment-variable alternative exists;
- asking a client to send the key through email or chat;
- copying credentials from another person's machine.

## Governed component workflow

Use this order:

```text
INSPECT -> SPECIFY -> SEARCH -> READ CODE -> APPROVE -> INSTALL -> ADAPT -> TEST -> VISUAL REVIEW -> REPORT
```

### 1. Inspect

Check:

- existing components and dependencies;
- package manager lockfile;
- TypeScript and path aliases;
- Tailwind or CSS setup;
- React/Next.js version;
- accessibility and performance constraints.

### 2. Specify

Write a bounded component brief:

- user task;
- exact location;
- required states;
- content and data contract;
- responsive behavior;
- keyboard and screen-reader behavior;
- performance budget;
- visual constraints;
- acceptance evidence.

### 3. Search

```bash
21st search "<specific component need>"
```

Search is discovery, not approval. Compare the result against local components and project conventions.

### 4. Read code before installation

Use the CLI retrieval tools to inspect the component and dependencies. Reject components that:

- add an unnecessary UI framework;
- conflict with the design system;
- contain opaque remote scripts;
- require unapproved analytics or network calls;
- weaken accessibility;
- materially increase bundle size without value.

### 5. Approval gate

Require visible human approval before:

- installing dependencies;
- consuming a limited install allowance or paid AI credits;
- generating UI with credits;
- publishing or changing visibility;
- modifying a private team library.

### 6. Install

```bash
21st add <component-reference>
```

Record the command, package changes, files added, and source reference. Never claim the install succeeded until the files and dependencies exist and checks pass.

### 7. Adapt

Integrate the component into the project's own tokens, content, conventions, and architecture. Do not ship an untouched marketplace demo as a finished product.

### 8. Test and review

Run the repository's existing commands, at minimum where available:

```bash
npm run lint
npm run typecheck
npm test
npm run build
```

Then use browser/computer vision to inspect:

- desktop and mobile layouts;
- keyboard navigation;
- focus states;
- overflow and clipping;
- loading, empty, error, and success states;
- reduced motion;
- console and network errors.

### 9. Report

Record:

```text
component source
files changed
dependencies changed
commands run
checks passed/failed
visual evidence
accessibility evidence
rollback command
credit or install allowance consumed
```

## Generate and explore rules

Generation commands may consume credits. Before running:

1. Confirm generation is necessary instead of adapting an existing component.
2. State the number of variants and credit impact.
3. Obtain owner approval.
4. Generate meaningfully different directions, not cosmetic variations.
5. Keep only the selected direction and remove discarded generated files.

Example:

```bash
21st generate "<bounded UI brief>" --variants 3
```

## Publishing rules

Publishing is a separate state-changing workflow. Require approval for the exact file, name, description, tags, and visibility.

```bash
21st publish ./Component.tsx --description "<measurable description>"
21st publish-theme ./theme.css --name "<approved name>"
21st components --status all
```

Never publish client code, client branding, licensed assets, private data, or proprietary components without written authorization.

## Browser and remote-client setup

For owner or client computers:

```text
OBSERVE -> EXPLAIN -> APPROVE -> ACT -> VERIFY -> RECORD
```

The human must:

- approve browser login;
- enter credentials directly;
- approve package installation and firewall/security prompts;
- confirm account, plan, and potential credit use.

Hermes must not install persistent remote access, capture passwords, or leave an authenticated support session open.

## Verification statuses

Use only these states:

```text
21ST_NOT_INSTALLED
21ST_CLI_INSTALLED
21ST_AUTH_REQUIRED
21ST_AUTH_VERIFIED
21ST_MCP_CONFIGURED
21ST_MCP_VERIFIED
21ST_COMPONENT_PROPOSED
21ST_COMPONENT_INSTALLED_UNVERIFIED
21ST_COMPONENT_VERIFIED
```

`21ST_MCP_VERIFIED` requires a real authenticated tool-list or search request from the intended MCP client. `21ST_COMPONENT_VERIFIED` requires repository checks and browser review.

## Failure handling

- CLI missing: stop and present install command.
- Node/npm missing: stop and repair the approved Node installation first.
- authentication missing: open owner-controlled login; do not request the key in chat.
- rate or install limit reached: stop; do not create another account or bypass limits.
- credits unavailable: use search/manual implementation or request explicit plan approval.
- config conflict: restore backup and merge only the 21st server block.
- build failure: revert component files and package changes or fix within the bounded slice.
- output conflicts with local design system: reject or adapt before merge.

## Rollback

1. Revert the integration commit or restore the pre-install branch.
2. Remove only dependencies added by the selected component.
3. Remove the 21st MCP server block without touching other servers.
4. Unset `API_KEY_21ST` only when the owner requests disconnection.
5. Use `21st logout` if available in the installed CLI or remove the local session through the official account flow.
6. Verify existing project checks still pass.

## Completion report

```text
DECISION
CHANGES
PROOF
STATUS
COMMERCIAL IMPACT
RISKS
ROLLBACK
NEXT
HUMAN APPROVAL
```
