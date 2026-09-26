---
name: icm-organizer
description: Organize any GitHub repository, project, or idea into an ICM (Interpretable Context Methodology) workspace. Use when the user wants to structure a repo into a folder-based agent architecture where the filesystem becomes the orchestration layer. Supports both building new workspaces from process descriptions and restructuring existing repos into ICM conventions.
---

# ICM Organizer Skill

Organize any GitHub repo, project, or idea into an ICM (Interpretable Context Methodology) workspace — where folder structure becomes agent architecture.

## What This Does

Takes a messy repo, scattered notes, or a described workflow and scaffolds it into one of five proven ICM forms:

- **Pipeline** — repeating sequence producing a deliverable each run (extract → structure → ship)
- **Umbrella** — portfolio of pipelines sharing a common brand/reference layer
- **Record library** — accumulating records (people, clients, sessions) that get looked up, not runs
- **Knowledge bundle** — navigable knowledge itself (brain, wiki, domain model)
- **Context map** — organization as a graph (teams, processes, data, handoffs)

## How to Use

### Analyze and Structure a GitHub Repo

```
/icm-organizer repo:owner/repo
```

or

```
/icm-organizer
[paste repo details/structure]
```

### Build from a Process Description

```
/icm-organizer build
[describe your workflow]
```

### Restructure an Existing Folder

```
/icm-organizer restructure /path/to/folder
```

## The Workflow

### 1. Intake (What are we organizing?)

- GitHub repo URL or local path
- What's the repeating unit? (run, record, knowledge, org structure)
- Who touches it, and what do they need to find?
- What stays the same every time vs. what's new each time?

### 2. Form Selection

Based on the repeating unit, pick one of five forms. (Workspaces often mix forms recursively.)

### 3. Scaffold the Structure

Build the minimal folder tree:
- Entry file (`CLAUDE.md` or `AGENTS.md`)
- Root contract (`CONTEXT.md`) explaining the shape
- Stage/hub folders with their own `CONTEXT.md` files
- Reference material (`_shared/`, `_system/`) vs. working outputs
- Templates for recurring work (`_templates/`)

### 4. Write Contracts

Every working folder gets a `CONTEXT.md` that names:
- **Inputs** — exact paths, split working (this run) vs. reference (every run)
- **Process** — numbered steps, constraints from L3 files
- **Outputs** — what lands where
- **Human check** — one thing a person does before next stage reads

### 5. Walk Test

Validate the structure: can an agent with no memory orient, act, and report status purely from the files?

## The Ten ICM Invariants (Enforced)

1. **One folder, one job** — each folder does a single step or holds one kind of thing
2. **Small, stable entry file** — `CLAUDE.md` routes; it doesn't hold content (target: ~60 lines)
3. **Numbering encodes order** — `01_`, `02_`, … where it matters
4. **Explicit folder contracts** — `CONTEXT.md` per working folder: inputs, process, outputs, human check
5. **Factory vs. product** — reference material (stable) structurally apart from working artifacts (per-run)
6. **Every output is an edit surface** — intermediate files are plain text a human can open and edit
7. **Load only what you need** — agents load stage contract + its inputs, not the whole workspace (2k–8k tokens per step)
8. **Plain text, linkable, queryable** — Markdown + YAML frontmatter, relative links make it a graph
9. **Filesystem is the state machine** — status derived by scanning what exists, never hand-edited indexes
10. **Instantiate by copying** — new unit of work = copy template, not blank page

## Output: Your ICM Workspace

A structured folder that:
- Explains itself in files (humans can read and understand without running anything)
- Routes agents correctly (entry file → contract → inputs → step)
- Validates with the walk test (agent with no memory can operate)
- Separates method from instance (template vs. deployment)
- Scales gracefully (principles apply recursively at every depth)

## Example Result

```
my-workspace/
├─ CLAUDE.md                    # Entry: identity + routing table
├─ CONTEXT.md                   # Root contract: what this workspace is
├─ stages/
│  ├─ 01_intake/
│  │  ├─ CONTEXT.md             # What, inputs, process, outputs, human check
│  │  ├─ references/
│  │  └─ output/
│  ├─ 02_analysis/
│  │  ├─ CONTEXT.md
│  │  ├─ references/
│  │  └─ output/
│  └─ 03_delivery/
│     ├─ CONTEXT.md
│     ├─ references/
│     └─ output/
├─ _shared/                     # Stable factory: voice.md, schema.md
├─ _templates/                  # Templates for new work
└─ setup/questionnaire.md       # Configure once, reuse every run
```

## When NOT to Use ICM

- **Real-time multi-agent collaboration** — tight message loops need frameworks, not file handoffs
- **High concurrency / multi-user serving** — needs queueing and state isolation
- **Automated mid-pipeline branching** — system-decided branches (not human-reviewed) push toward frameworks

**The case for ICM:** Sequential, human-reviewed, repeatable workflows — which is most knowledge work. Folder structure replaces orchestration code; one agent replaces multi-agent frameworks.

## References

- **Paper:** Interpretable Context Methodology (Van Clief & McDermott, arXiv:2603.16021)
- **Community:** Clief Notes (skool.com/cliefnotes)
- **Source:** github.com/RinDig/icm-architect

## Key Questions I'll Ask

When you invoke this skill, I'll extract the structure by dialogue:

- **What is the repeating unit?** episode, run, client, person, team, idea?
- **Walk me through one cycle** — where do you stop and check before continuing?
- **What's stable vs. new?** voice, rules, schema (same every time) vs. content, data, artifacts (new each time)?
- **What does "done" look like?** what leaves the workspace?
- **Who else needs to find what?** without asking you?

Your pauses become stage boundaries. Your "I always check X before Y" become human gates. Your "it always sounds like Z" becomes factory reference material.

Then I'll pick a form, build the tree, write the contracts, and validate with the walk test.
