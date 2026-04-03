# GitHub Workspace Context
# Load this when: working with repos, cloning, pushing, reading repo contents

## Access
- Account: executiveusa (371 repos — see repos.json in this folder)
- CLI: `gh` (GitHub CLI, pre-authenticated)
- Archon X repo: `executiveusa/archonx-os`
- Ivette/Kupuri repo: `executiveusa/AKASHPORTFOLIO`
- Hermes repo: `executiveusa/pauli-hermes-agent`

## Key Repo Groups
| Group | Prefix | Example Repos |
|-------|--------|--------------|
| Pauli / Archon X | P | pauli-hermes-agent, archonx-os, pauli-comic-funnel |
| Kupuri / Ivette | K | kupuri-media-cdmx, AKASHPORTFOLIO, Synthia-avatar |
| Skills | s | pauli-taste-skill, pauli-impeccable-design-, paulsuperpowers |
| Tools | T | pauli-beads, pauli-sercets-vault-, pauli-infranodus |

## Rules
- ALWAYS activate `jcodemunch` before exploring any repo with >10 files
- Read `workspace/github/repos.json` before cloning — avoid duplicates
- After cloning, record in repos.json with correct category code
- Never push directly to `main` — branch → PR → ZTE auto-deploy handles merge

## Creating a PRD / Handoff
1. Use BMAD method: `/graph <feature intent>`
2. Generate PRD → save to `workspace/agents/handoffs/<project>/PRD.md`
3. Hand off to coding-agent or Archon X system
