# Hermes Workflows

## Skills vs. Workflows — The Distinction

| Concept | What It Is | Example |
|---------|-----------|---------|
| **Skill** | One capability. One input → one output. | `youtube-channel-scraper` |
| **Workflow** | Chain of multiple skills toward a specific outcome. | `scroll-world-design` |

A skill answers: *"How do I do X?"*
A workflow answers: *"How do I achieve outcome Y, from start to finish, with no slop?"*

## Workflow Anatomy

Every workflow in this folder:

1. **Has a named outcome** (e.g. "High-quality scroll websites, every time")
2. **Chains ≥2 skills** in ICM stages
3. **Graphs all information** it gathers before reasoning over it
4. **Compares itself** to existing workflows to avoid collision
5. **Runs judge subagents** at the end before delivery
6. **Requires human only at intake and delivery** — no manual intervention between

## Registry

All workflows are indexed in `WORKFLOW_REGISTRY.json` by outcome.

## ICM Contract (all workflows)

Every workflow follows Interpretable Context Methodology:

- Each stage is a folder with a `CONTEXT.md` contract
- Each stage takes the prior stage's output as input
- Gates block advancement if output fails validation
- All side effects produce receipts
- A separate verifier/judge checks every final output

## Folder Layout

```
hermes-workflows/
├── README.md                     ← you are here
├── WORKFLOW_REGISTRY.json        ← index of all workflows by outcome
├── _icm/
│   ├── methodology.md
│   └── _config/agent-voice.md
└── [workflow-name]/
    ├── CLAUDE.md                 ← quick start / entry point
    ├── AGENTS.md                 ← who does what
    ├── CONTEXT.md                ← router: which stage to open
    ├── resources/                ← lazy-loaded references
    ├── subagents/                ← specialist agents
    ├── stages/                   ← ICM stages 00-NN
    │   └── NN_name/CONTEXT.md
    └── runs/                     ← output artifacts per run
```

## Adding a New Workflow

1. Choose a name derived from the **outcome**, not the process
2. Create `hermes-workflows/[name]/`
3. Follow the anatomy above
4. Register in `WORKFLOW_REGISTRY.json`
5. In each stage CONTEXT.md, explicitly list what prior workflows it must not collide with
