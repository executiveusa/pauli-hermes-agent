# ICM Quick Reference

Fast lookup for ICM principles, naming conventions, and patterns.

## The Ten Invariants (Enforce These)

1. **One folder, one job** — single purpose per folder
2. **Small, stable entry file** — ~60 lines max, routes only
3. **Numbering encodes order** — `01_`, `02_`, … for sequence
4. **Explicit folder contracts** — `CONTEXT.md`: inputs, process, outputs, check
5. **Factory vs. product** — reference (stable) separate from outputs (per-run)
6. **Edit surfaces** — intermediate outputs are plain files humans can edit
7. **Load only what you need** — 2k–8k tokens per stage, not 50k monolith
8. **Plain text, linkable** — Markdown, YAML, wikilinks, relative paths
9. **Filesystem = state machine** — status from what exists, not hand-edited indexes
10. **Copy to instantiate** — new work = template copy, not blank page

## Naming Conventions

| Pattern | Use | Example |
|---------|-----|---------|
| `NN_kebab-name` | Stage folders | `01_research`, `02_script`, `03_production` |
| `_meta`, `_shared`, `_system`, `_templates`, `_archive`, `_index` | System folders (sort to top) | `_shared/voice.md` |
| `CLAUDE.md` | Entry file (Claude Code) | Root identity + routing table |
| `AGENTS.md` | Entry file (other agents) | Root identity + routing table |
| `CONTEXT.md` | Contract file | At root, at each stage |
| `output/` | Working outputs (per-run) | `stages/01_/output/` |
| `references/` | Stage-specific reference | `stages/01_/references/structure.md` |
| Kebab-case or Title Case | Records/nodes | Pick one, document in schema |
| Type prefix | Typed content | `data-`, `list-`, `schema-` |

## File Hierarchy (Five Layers)

| Layer | File | Question | Role | Size |
|-------|------|----------|------|------|
| L0 | `CLAUDE.md` | Where am I? | Routing | 300–800 tokens |
| L1 | Root `CONTEXT.md` | Where do I go? | Routing | 200–500 tokens |
| L2 | Stage `CONTEXT.md` | What do I do? | **Control point** | 200–500 tokens |
| L3 | `_shared/`, `references/` | What rules apply? | Factory (stable) | 500–2k tokens |
| L4 | `output/`, run artifacts | What am I working with? | Product (per-run) | Varies |

- L0–L2: Catalog (small, stable, no payload)
- L2: The control point (Inputs list makes context explicit and auditable)
- L3 vs L4: Factory/product split

## Stage Contract Template

Every working folder's `CONTEXT.md`:

```markdown
# [NN_stage] — [One sentence: what this does]

One job: [2-3 sentences explaining why this is a separate stage]

## Inputs
- Working: ../[prev-stage]/output/[file]
- Reference: ../../_shared/[file]
- Reference: references/[file]

## Process
1. [Numbered step]
2. [Numbered step]
3. [Numbered step]

## Outputs
- [file] → output/

## Human Check
[One action: "Read the X and verify Y. Edit in place."]
```

**Rules:**
- Inputs: exact paths, working vs. reference split
- Process: short, constraints in L3 files
- One human check, stated as an action

## The Five Forms at a Glance

| Form | Unit | Key Pattern | Use Case |
|------|------|-------------|----------|
| **Pipeline** | A run | `01 → 02 → 03` | Repeating sequence → deliverable |
| **Umbrella** | Different kinds of runs | Root map + sub-pipelines | Several lines, shared brand |
| **Record library** | A record | Template + index | People, clients, sessions (lookup) |
| **Knowledge bundle** | Knowledge | Factory + product | Brain, wiki, domain model |
| **Context map** | Entity + edges | Graph structure | Organization, teams, processes |

## Token Discipline

**Per-stage context = 2k–8k tokens**

Stage context = entry + contract + references + inputs

If it balloons:
- Split the stage into smaller steps
- Tighten the inputs list (don't load everything)
- Move details into L3 files the contract points at (not inlines)

## The Walk Test (Validation)

Can an agent with no memory:

1. **Orient:** Read root entry, know what this workspace is and where to go?
2. **Act:** Read a stage contract, know inputs, job, outputs?
3. **Report:** Derive status by scanning `output/` folders?

If any fails → fix the structure (move/split files, not add explanation).

## Anti-Patterns (What to Avoid)

| Pattern | Problem | Fix |
|---------|---------|-----|
| Routing file with content | Too much context | Move content to shelf, leave link |
| Same fact in two places | Drift | One home, one link |
| Hand-edited index | Falls out of sync | Script it, rebuild on schedule |
| Entry file + "explain more" | Exploding size | Add contract, don't expand entry |
| Stages that do two jobs | Unclear gates | Split into two stages |
| Contract that restates reference | Bloat | Point at L3, don't repeat |

## When NOT to Use ICM

- **Real-time multi-agent collaboration** — needs message-passing, not file handoffs
- **High concurrency / multi-user** — needs queueing and state isolation
- **Automated mid-pipeline branching** — system decides branches (not human-reviewed)

**When to use:** Sequential, human-reviewed, repeatable work (most knowledge work).

## Resources

- **Paper:** Interpretable Context Methodology (arXiv:2603.16021)
- **Repo:** github.com/RinDig/icm-architect
- **Community:** Clief Notes (skool.com/cliefnotes)

## Checklist: New Workspace

- [ ] Identity clear (one sentence: what is this?)
- [ ] Form chosen (Pipeline / Umbrella / Record library / Knowledge bundle / Context map)
- [ ] Repeating unit identified
- [ ] Stages/layers named (`01_`, `02_`, …)
- [ ] `CLAUDE.md` written (~60 lines)
- [ ] Root `CONTEXT.md` written (explains the shape)
- [ ] Stage `CONTEXT.md` written (each stage: inputs, process, outputs, check)
- [ ] `_shared/` populated (factory: voice, schema, rules)
- [ ] `_templates/` populated (template for new work)
- [ ] Walk test passed (agent can orient, act, report)
- [ ] Token count checked (per-stage context 2k–8k)

## Checklist: Restructure

- [ ] Inventory complete (what exists, who touches it)
- [ ] Form identified (what's the real repeating unit?)
- [ ] Files classified (catalog, contract, factory, product, dead)
- [ ] Migration map created (old → new → role)
- [ ] Approval from owner
- [ ] Files moved, contracts written
- [ ] One-home-per-fact (de-duplicated, linked)
- [ ] Walk test passed
