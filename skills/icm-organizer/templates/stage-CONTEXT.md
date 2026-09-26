# [NN_stage-name] — [One-sentence description of the job]

**One job:** [2-3 sentences: what this stage does, why it's separate, what gates it passes through.]

## Inputs

**Working (this run):**
- `../[stage-N]/output/[file]` — [what it is, what stage emitted it]

**Reference (every run):**
- `../../_shared/[file]` — [what it is]
- `references/[file]` — [what it is, why this stage needs it specifically]

## Process

1. [Numbered step]: [What the agent does]
2. [Numbered step]: [What the agent does]
3. [Numbered step]: [What the agent does]

**Constraints:** reference material (voice, schema, rules) is in the files above, not repeated here.

## Outputs

- `[output-file]` → `output/` — [what it is, ready for next stage or human review]

## Human Check

[Exactly one action a human does before the next stage reads:]

Example: "Read the output aloud. Verify the tone matches `voice.md`. Edit in place; the next stage reads whatever is here."
