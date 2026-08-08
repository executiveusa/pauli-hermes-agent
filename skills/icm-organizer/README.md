# ICM Organizer Skill

Transform any GitHub repository, project, or workflow idea into an **ICM (Interpretable Context Methodology) workspace** — where folder structure becomes your agent orchestration layer.

## What This Skill Does

Converts a messy repo, scattered process, or described workflow into one of **five proven ICM forms**:

1. **Pipeline** — Sequential stages producing a deliverable each run
2. **Umbrella** — Multiple pipelines sharing a common brand/reference layer
3. **Record Library** — Accumulating records (people, clients, projects) that get looked up
4. **Knowledge Bundle** — Navigable knowledge itself (brain, wiki, domain model)
5. **Context Map** — Organization as a graph (teams, processes, relationships)

## Why ICM?

Instead of:
- ❌ Orchestration code + framework complexity
- ❌ Agents passing context through message payloads (token wasteful)
- ❌ Multi-agent choreography that only the developer understands

You get:
- ✅ Folder structure that IS the orchestration
- ✅ Numbered folders = sequence, hierarchy = scoping
- ✅ Plain markdown files = state (humans can read it, edit it, understand it)
- ✅ One agent walking the right files at the right moment
- ✅ A human can open any folder and immediately know what state the system is in

**Result:** Transparent, auditable, modifiable workflows that don't require a framework.

## How to Use It

### Option 1: Analyze an Existing Repo

```bash
/icm-organizer
# Paste repo URL or local path
```

### Option 2: Structure a Workflow You Describe

```bash
/icm-organizer build
# Describe your repeating process
```

### Option 3: Restructure an Existing Folder

```bash
/icm-organizer restructure /path/to/folder
```

## The Skill Walks You Through

1. **Intake** — What are we organizing? What's the repeating unit?
2. **Form selection** — Pipeline? Record library? Knowledge bundle? Context map? Umbrella?
3. **Scaffolding** — Build the minimal folder tree that holds the work
4. **Contracts** — Every stage/folder gets a `CONTEXT.md` (inputs, process, outputs, human check)
5. **Validation** — Walk test: can an agent with no memory operate it?

## What You Get

A workspace folder structured like this (Pipeline example):

```
my-project/
├─ CLAUDE.md                    # Entry: "I am X, go here for task Y"
├─ CONTEXT.md                   # Root contract: the shape of this workspace
├─ stages/
│  ├─ 01_intake/
│  │  ├─ CONTEXT.md             # Inputs, process, outputs, human check
│  │  ├─ references/            # Rules, templates, schema
│  │  └─ output/                # This stage's output (next stage's input)
│  ├─ 02_analysis/
│  │  ├─ CONTEXT.md
│  │  ├─ references/
│  │  └─ output/
│  └─ 03_delivery/
│     ├─ CONTEXT.md
│     ├─ references/
│     └─ output/
├─ _shared/                     # Factory: stable stuff (voice, schema, rules)
├─ _templates/                  # Template for new runs
└─ setup/questionnaire.md       # Configure once, use every run
```

**Key principles:**
- Numbered folders encode sequence (`01_`, `02_`, …)
- Each folder has one job
- `CONTEXT.md` in every working folder says: inputs, process, outputs, what a human checks
- Reference material (stable) is separate from working outputs (per-run)
- Status is readable by scanning what files exist (`output/` folders)

## The Ten Invariants (Enforced)

Every ICM workspace respects these:

1. One folder, one job
2. Small, stable entry file (~60 lines)
3. Numbering encodes order
4. Explicit contracts (`CONTEXT.md`) per working folder
5. Factory (stable) vs. product (per-run) separation
6. Every output is editable by humans
7. Load only what the stage needs (2k–8k tokens)
8. Plain text, linkable, queryable
9. Filesystem is the state machine
10. Instantiate by copying templates

## Files in This Skill

| File | Purpose |
|------|---------|
| `icm-organizer.md` | Skill definition (what it does, how to invoke) |
| `quick-reference.md` | Fast lookup: invariants, naming, layers, forms, walk test |
| `templates/CLAUDE.md` | Template for workspace entry file |
| `templates/stage-CONTEXT.md` | Template for stage contract |
| `templates/workspace-CONTEXT.md` | Template for root contract |
| `templates/form-selector.md` | Decision tree to pick the right form |
| `README.md` | This file |

## Quick Start

### Build a Pipeline Workspace

```bash
/icm-organizer build

# I'll ask:
# - What is the repeating sequence?
# - Where do you naturally pause to check?
# - What stays the same every run vs. what's new?

# Then I'll create:
# - CLAUDE.md (entry)
# - CONTEXT.md (root contract)
# - stages/01_*, 02_*, 03_*/ (with CONTEXT.md per stage)
# - _shared/ (voice, schema)
# - _templates/ (template for new runs)
```

### Restructure a Messy Repo

```bash
/icm-organizer restructure ./my-repo

# I'll:
# 1. Audit what exists
# 2. Classify files (catalog, contract, factory, product, dead)
# 3. Propose a structure
# 4. Migrate and validate
```

## When to Use This Skill

✅ **Use ICM when:**
- You have a repeating workflow (same sequence, new deliverables)
- Multiple agents need to work on it sequentially
- Humans review/edit at stage boundaries
- You want humans to understand the system by reading files
- The work is knowledge work (analysis, writing, design, planning)

❌ **Don't use ICM when:**
- You need real-time multi-agent collaboration (use a message framework)
- High concurrency / multi-user serving (use queueing + databases)
- Automated mid-pipeline branching without human review (use orchestration frameworks)

## Example Outputs

### Pipeline: Content Studio

```
content-studio/
├─ CLAUDE.md: "Content production from brief to delivery"
├─ stages/
│  ├─ 01_research/          → output/research.md
│  ├─ 02_outline/           → output/outline.md
│  ├─ 03_draft/             → output/draft.md
│  ├─ 04_review/            → output/reviewed.md
│  └─ 05_publish/           → output/published.md
├─ _shared/                 voice.md, brand.md, style-guide.md
└─ _templates/              questionnaire.md
```

### Record Library: Customer Tracker

```
customers/
├─ 00_START-HERE.md
├─ _index/log.md            (id, status, owner)
├─ _templates/
│  └─ customer-template/    (the schema)
├─ records/
│  ├─ acme-corp/
│  ├─ zenith-systems/
│  └─ starlight-ai/
└─ _shared/                 pricing, contract template, brand
```

## References

- **Paper:** Interpretable Context Methodology (Van Clief & McDermott, arXiv:2603.16021)
- **Source repo:** github.com/RinDig/icm-architect
- **Community:** Clief Notes (skool.com/cliefnotes)

## The Principle

> "Think of the workspace as a library. The routing files are the catalog: small, stable, they point at everything and store almost nothing. The content lives on the shelves (stage folders, node files, reference material). One librarian — one model — walks the building, and the question decides which shelf gets walked to."

— From the ICM paper

## Next Steps

1. **Invoke the skill:** `/icm-organizer` or `/icm-organizer build` or `/icm-organizer restructure`
2. **Walk through the dialogue:** Answer questions about your workflow
3. **Get a structured workspace** — scaffolded, validated, ready to use
4. **Use with agents:** The workspace routes agents correctly; folder structure = orchestration

---

**Questions?** Check `quick-reference.md` for fast lookups, or read the ICM paper at arXiv:2603.16021.
