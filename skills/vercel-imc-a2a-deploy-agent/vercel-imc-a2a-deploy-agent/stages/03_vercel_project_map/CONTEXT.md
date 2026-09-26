# Stage 03_vercel_project_map: Vercel Project Map


## Inputs

- `runs/<run-id>/github-repos.json`
- Vercel token availability
- `skills/vercel-cli-deployment/SKILL.md`

## Process

1. Read Vercel projects through API/CLI.
2. Map projects to GitHub repos via Vercel project link metadata when available.
3. Flag ambiguous or missing mappings.
4. Ask for human input only if mapping is unsafe.

## Outputs

- `runs/<run-id>/vercel-projects.json`
- `runs/<run-id>/project-map.json`
- `stages/03_vercel_project_map/output/project-map.json`

## Gate

Proceed only for exact repo/project matches or single explicit target repo.

