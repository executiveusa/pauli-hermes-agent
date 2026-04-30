---
name: open-design
description: >
  Use the Open Design repository (https://github.com/nexu-io/open-design.git)
  as a design-system and UI reference for generating screens, components, and
  front-end implementation plans.
version: 1.0.0
author: Hermes Agent
license: MIT
tags: [design, ui, frontend, design-system, open-source]
triggers:
  - open design
  - use open-design
  - clone open-design
  - reference nexu open design
  - generate UI from open-design
---

# Open Design Skill

This skill gives the agent a direct workflow for using the Open Design repository:

- Repository URL: `https://github.com/nexu-io/open-design.git`
- Primary use: UI inspiration, component patterns, tokens, and implementation references.

## Access Workflow

1. Clone the repository when needed:
   ```bash
   git clone https://github.com/nexu-io/open-design.git
   ```
2. Inspect key files for reusable styles, component composition, and layout patterns.
3. Reuse patterns by adapting them to the user's stack and constraints (React/Vue/HTML/CSS/Tailwind).
4. Cite which files or directories were used when producing design guidance or code.

## Usage Notes

- Treat this repo as a reference source, not a hard dependency.
- Prefer adapting ideas over copying full files.
- If a user asks for exact parity, explain any framework/library differences before implementation.
- If network cloning is unavailable, ask for pasted files or a local snapshot path.

## Suggested Agent Behavior

When a prompt is about UI/UX, component libraries, design language, landing pages, or style consistency:

1. Load this skill.
2. Pull or inspect the repository.
3. Produce:
   - a concise design direction,
   - a component breakdown,
   - and implementation-ready code matching the user's tech stack.
