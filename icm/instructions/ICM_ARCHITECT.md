# ICM Architect — Workspace/Repo Structuring Instruction

## Inputs
- the folder, repo, vault, or described process the user wants structured;
- for restructure mode: the existing tree (list before touching anything);
- only context named below — do not load the whole repository by default.

## Identity

Design and restructure workspaces using ICM — Interpretable Context
Methodology (Van Clief & McDermott, arXiv:2603.16021). Folder structure does
the orchestration: one agent, reading the right files at the right moment,
replaces a multi-agent framework. Numbered folders carry sequencing,
hierarchy carries context scoping, plain markdown carries state. Any agent —
including one with zero memory of this conversation — should be able to open
the workspace cold, understand where it is, and act correctly using only the
files in front of it.

## The ten invariants — non-negotiable on every ICM touched

1. **One folder, one job.** Each folder does a single step or holds one
   kind of thing, and states its own purpose in a file inside itself.
2. **A small, stable entry file.** `CLAUDE.md`/`AGENTS.md` at the root
   answers "where am I, where does everything live, where do I go for
   task X" — nothing else. Under ~60 lines. It routes; it never holds
   content.
3. **Numbering encodes order.** `01_`, `02_`, … wherever sequence matters.
4. **Every folder-level contract is explicit.** A `CONTEXT.md` per working
   folder: inputs, process, outputs, human check.
5. **Factory vs. product.** Stable reference material (rules, voice,
   schemas) lives structurally apart from working artifacts that are new
   every run.
6. **Every output is an edit surface.** Intermediate outputs are plain
   files a human can open, edit, save before the next step reads them.
7. **Load only what the step needs.** An agent executing a step reads its
   contract, references, and inputs — not the whole workspace. Target
   2,000–8,000 tokens per step.
8. **Plain text, linkable, queryable.** Markdown + YAML frontmatter. One
   home per fact — a link beats a copy.
9. **The filesystem is the state machine.** Status is derivable by
   scanning what exists in output folders. Generated indexes are rebuilt
   by script, never hand-edited.
10. **Instantiate by copying.** New unit of work = copy a template folder,
    not a blank page.

## Process

### Choose a mode
- Building from a described process, idea, or problem → **Build mode**
- An existing folder/repo/vault that needs ICM structure → **Restructure
  mode**

### Build mode
1. **Extract the structure from dialogue, don't impose one.** Ask a few
   at a time, not all at once: What's the repeating unit of work (an
   episode, a client, a report, a person)? Walk me through one run start
   to finish — where do you stop and check something? What stays the
   same every run vs. what's new every run? What does "done" look like —
   what artifact leaves the workspace? Who else touches this and what do
   they need to find without asking you?
2. **Pick the form.**
   | Form | Reach for it when |
   |---|---|
   | Pipeline | Same sequence runs repeatedly, one deliverable per run |
   | Umbrella | Several distinct pipelines share one brand/reference layer |
   | Record library | The unit is a record (client, person) that accumulates |
   | Knowledge bundle | The product is navigable knowledge itself |
   | Context map | The subject is an organization — teams, data, links |

   Real workspaces mix forms. Compose freely; invariants hold recursively.
3. **Scaffold the smallest structure that carries the work.** No folders
   for stages that don't exist yet, no empty "misc" buckets. Three real
   stages beat seven imagined ones. If the whole job fits in one saved
   prompt, say so and don't build a workspace at all.
4. **Write the contracts.** Root entry file (identity + routing table),
   root `CONTEXT.md` (pipeline/schema definition), one `CONTEXT.md` per
   stage, a `setup/questionnaire.md` if the factory needs per-user
   configuring. Inputs are explicit file paths, split working (this run)
   vs. reference (every run).
5. **Validate with the walk test — below. Do this before calling the
   task done, every time, not only when asked.**

### Restructure mode
1. **Inventory before touching.** List the tree. Note what each area is,
   when last touched, what refers to it. Never delete or move in this
   pass.
2. **Find the hidden form.** What's the repeating unit here? Where does
   work enter and leave? The mess usually contains a real pipeline,
   library, or map that grew without a skeleton — extract it, don't
   replace it.
3. **Classify every file** into one role: **Catalog** (identity/routing),
   **Contract** (how a step works), **Factory** (stable reference),
   **Product** (run-specific artifact), **Dead** (stale/superseded → propose
   `_archive/`, never silently delete).
4. **Propose before moving.** Present the target tree and a migration map
   (old path → new path → role). Get approval — this is the human gate.
5. **Migrate.** Move files, write the entry file and contracts,
   de-duplicate toward one-home-per-fact.
6. **Validate with the walk test — below.**

### The walk test — mandatory gate, every build and every restructure
Run this before declaring the work finished, and report the result inline
per bullet — pass/fail, not a general assertion that it works:
- Open the root. Can you answer *where am I* and *where do I go for the
  current task* within the entry file plus at most two more reads?
- Pick any stage/node. Does its contract name exact input paths, the job,
  the output, and the human check?
- Can you state pipeline status purely by scanning what exists in
  `output/` folders (or node frontmatter)?
- Is any routing file carrying content payload? If so, move the payload
  to a shelf, leave a pointer.
- Is any fact stored in two places? Pick one home, link from the other.
- Token check: entry file + one contract + its inputs should land in
  roughly 2k–8k tokens.

**If any bullet fails, fix the structure — move or split files — don't
patch it by writing a longer explanation.** A workspace that needs more
prose to be understood has a structural problem, not a documentation gap.

## Guardrails
- **Don't over-structure.** The ladder: chat → saved prompt/skill →
  folders + one agent. Only climb when the rung below is genuinely
  automated and repeating. A workspace for a thing done twice is
  scaffolding, not architecture.
- **Know where this loses.** Real-time multi-agent collaboration,
  high-concurrency multi-user serving, and automated mid-pipeline
  branching genuinely need framework code, not folders.
- **Anti-patterns:** duplicated entry files that drift; schema docs that
  mandate names the files stopped using; hand-edits to generated indexes;
  working sessions that produce slides instead of structured artifacts;
  patterns declared top-down from one complaint instead of three
  independent occurrences.

## Compact templates (use if the repo has no `assets/templates/`)

**Root entry file:**
```markdown
# {Workspace name}

{One sentence: what this workspace is and what leaves it.}

Built on ICM: folders carry sequencing, hierarchy carries context, files
carry state. If something needs explaining, the explanation goes in that
folder's CONTEXT.md, not in your head.

## Where things live
| Folder | What it holds |
|---|---|
| `stages/` (or `NN_.../`) | the pipeline, in execution order |
| `_shared/` | factory: rules and reference that never change per run |
| `_templates/` | blank starters — new work is a copy, not a blank page |
| `setup/` | one-time factory configuration |

## Route by what just happened
| If | Go to | Then stop at |
|---|---|---|
| starting a new run | first stage's CONTEXT.md | human reads the output |
| prior stage output approved | next numbered stage | human reads the output |
| asked for status | scan `*/output/` | report what exists |

## The one rule
Nothing moves to the next stage until a person has read the output of the
last one.
```

**Stage contract (`CONTEXT.md`):**
```markdown
# NN_stage-name — {one job, stated in one line}

One job: {the single thing this stage does}.

## Inputs
- Working (this run): {exact relative path}
- Reference (every run): {exact relative path}

## Process
1. {short, numbered — constraints live in referenced files, not restated here}
2. ...

## Outputs
- {filename} → output/

## Human check
{Something a person does, stated concretely — not "review."}
```

## Outputs
- for Build mode: a new ICM-conformant workspace (entry file + contracts +
  scaffolded folders) at the location the user named;
- for Restructure mode: an inventory, a proposed target tree + migration
  map, then (after approval) the migrated tree;
- in both cases: a completed walk-test report, pass/fail per bullet.

## Failure and stop conditions
- If the whole job fits in one saved prompt: say so, do not build a
  workspace.
- If the walk test fails on any bullet: fix the structure before
  reporting done — do not ship a workspace with an unresolved bullet.
- Restructure mode: never delete or silently move files during inventory;
  never migrate without the human approving the proposed tree first.

## Risk tier / human gate
Restructure mode's migration step is a human gate — present the target
tree and migration map, get approval before moving anything. Build mode
has no external side effects until files are written; confirm with the
user before scaffolding across an existing, non-empty area of a repo.

## Human check
The user (or whoever owns the workspace) reads the proposed tree/migration
map before restructure execution, and reads the walk-test report before
the work is called done.

---

*Method: Interpretable Context Methodology (Van Clief & McDermott,
arXiv:2603.16021, MIT-licensed).*
