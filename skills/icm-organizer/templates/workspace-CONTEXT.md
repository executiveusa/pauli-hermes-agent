# [Project Name] — Workspace Contract

**What this is:** [One sentence: the repeating unit and what it produces.]

**Form:** [Pipeline / Umbrella / Record library / Knowledge bundle / Context map]

## The Pipeline

```
[Stage 1] → [Stage 2] → [Stage 3] → ... → [Output]
    ↓           ↓           ↓
  human      human       human
  gate       gate        gate
```

**Sequence:** Stages run in order. Humans review and edit at each boundary before the next stage reads.

**Status:** Check `stages/*/output/` to see what's complete. The files tell the story.

## Inputs & Outputs

**Entry point:** `stages/01_[first-stage]/` reads [description of what enters the workspace].

**Exit point:** `stages/[final]/output/[file]` is the deliverable.

**Internal handoffs:** Each stage's `output/` feeds into the next stage's inputs (the human can edit in between).

## Reference (Stable)

`_shared/`
- `voice.md` — tone, style, vocabulary
- `schema.md` — structure, format, must-haves
- `rules.md` — constraints, legal, brand

These apply to every run. Stage contracts point at them, don't repeat them.

## Instantiation

**New run:** Copy template from `_templates/`, fill blanks, place in `stages/01_[]/input/`. Run.

**Template location:** `_templates/[what-to-copy]/`

## Token Discipline

A stage's full context — entry file + contract + references + its inputs — should be 2,000–8,000 tokens.

**If a stage balloons:** split it, tighten the inputs list, or push detail into an L3 reference file the contract points at.

## The Walk Test

Can an agent with no memory:
1. Read the root entry file and know where to go for the current task?
2. Read a stage contract and understand inputs, job, outputs, and the human check?
3. Determine pipeline status by scanning what exists in `output/` folders?

If not, the structure needs fixing — move or split files until the walk works.
