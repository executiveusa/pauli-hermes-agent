# Synthia Design Studio — Hermes Skill

## What this does

Dispatches design briefs from Hermes to the Synthia Design Studio — a virtual AI design office
that produces Awwwards-quality frontends, brand assets, motion, audio, and 3D.

## When to use this skill

Use `dispatch_design` whenever you need:
- A new UI component, screen, or landing page
- Brand asset creation (logo, color system, typography)
- Motion/animation work (Remotion video output)
- Audio branding (ElevenLabs/Suno)
- 3D assets (Blender GLB/GLTF)
- A full multi-page site

## How it works

```
You (Hermes)                   Synthia Design Studio
    │                                    │
    │  POST /api/design/dispatch ────────►│
    │  { requestedBy, designType,        │
    │    projectName, brief, udecFloor } │
    │                                    │
    │◄─────────── taskId + instructions ─│
    │                                    │
    │                          HERMES routes →
    │                          RALPHY × 3 builds →
    │                          LENA scores (UDEC) →
    │                          gate (≥ 8.5 → ship) →
    │                          MARCO iterates if needed
    │                                    │
    │◄────── Vibe Graph node (resource) ─│
    │        approved output path        │
```

## Quality Floor

**UDEC 8.5** — nothing ships below this. The studio self-enforces.
Hard blocks: MOT < 7.0 or ACC < 7.0 → full rebuild, not patch.

## Example call

```python
# In your Hermes task handler:
result = dispatch_design(
    agent_id="forjadora",
    design_type="dashboard",
    project_name="cockpit-repos-panel",
    brief="""
    Repos panel for the SphereOS Cockpit.
    Shows: last commit, open issues, build status, active Vibe Graph nodes.
    Aesthetic: Linear/GitHub dark mode. No mock data. No gradients.
    Must score ≥ 8.5 on UDEC TYP + ACC axes.
    """,
    udec_floor=8.5,
    priority="high"
)
# result.taskId → write to synthia-superdesign/tasks/queue/
# result.designStudio.outputPath → where HTML will appear
```

## After dispatch

The design studio writes the approved file to:
`.superdesign/design_iterations/{project_name}/{file}.html`

And posts a Vibe Graph node:
```json
{
  "agentId": "synthia",
  "kind": "resource",
  "label": "{projectName} — approved design output",
  "content": "UDEC {score} | approved | path: ...",
  "tags": ["design-output", "{designType}", "{projectName}"]
}
```

Retrieve it with: `GET /api/vibe?agent=synthia`

## Design Studio Repo

`https://github.com/executiveusa/synthia-superdesign`

8 agents: HERMES (director), RALPHY (builder), LENA (critic), MARCO (synthesist),
SCOUT (research), AURORA (motion), BASS (audio), BLENDER (3D).

No databases. Folder interface. Drop a task file, get a UDEC-approved HTML back.
