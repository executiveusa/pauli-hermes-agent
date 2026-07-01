---
name: repo-inventory
description: Inventory GitHub repositories and Vercel projects, then map repos to deployments for a visibility sweep.
---

# Repo Inventory

## Use when

- The mission is “go through all GitHub repos connected to Vercel.”
- A repo/project map is missing.
- The agent must discover what is blocking visibility across projects.

## Script

```bash
node scripts/inventory-github-vercel.mjs --github-owner <owner> --out runs/inventory.json
```

## Output

A JSON object containing `githubRepos`, `vercelProjects`, and `matches`.
