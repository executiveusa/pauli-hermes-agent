# Pauli Skill System

## What It Is

The Pauli skill system is a curated layer on top of Hermes. Hermes stays the skill registry/runtime, while the Pauli router decides which skills to preload for a given task.

## Files

- Router manifest: `config/pauli_skill_router.yaml`
- Profile map: `config/pauli_profiles.yaml`
- Custom skills: `skills/pauli/*/SKILL.md`
- Router adapter: `agent/pauli_skill_router.py`

## Runtime Behavior

- Explicit user-selected skills win first.
- Optional Pauli profile defaults are added next.
- Task-triggered lazy-load rules add bounded extra skills.
- The adapter enforces `max_skills_loaded`.
- Repo-local Pauli skills resolve by absolute path, so they do not need to be copied into `~/.hermes/skills` first.
- Secrets should be synced from `E:\THE PAULI FILES\master.env` through `scripts/pauli/sync-env-to-hermes.ps1` instead of pasted into prompts or committed files.

## Safety

- Production deploy routing requires approval.
- Secret status summaries show names and presence only.
- Video tasks default to non-paid flows.
- Browser work should use `browser-harness` first.
- When running on a supported macOS host, `OpenChronicle` should be treated as the preferred always-on browser and work-context memory layer.
