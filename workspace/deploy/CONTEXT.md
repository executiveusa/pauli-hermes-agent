# Deploy Workspace Context
# Load this when: deploying to VPS, Vercel, Coolify, managing environments

## Infrastructure
| Service | URL | Notes |
|---------|-----|-------|
| VPS | 31.220.58.212 | Hetzner — main server |
| Coolify | 31.220.58.212:8000 | Container management |
| Hermes API | 31.220.58.212:8642 | OpenAI-compat API |
| Voice Dashboard | 31.220.58.212:9000 | web-ui/ static |
| Supabase | 31.220.58.212:8001 | Second brain DB |
| n8n | 31.220.58.212 | Automation workflows |
| Vercel Project | prj_hLAOeM1ml6C0Pp5uIApODsIhC83A | Dashboard |

## Secrets
- Source: Infisical (slug: agent-hermes, project: e2f5c669-4fdd-4c9f-be8c-fc56bf62549c)
- Sync: `bash scripts/infisical-sync.sh` → writes .env (never commit)
- NEVER read from E:\THE PAULI FILES\master.env directly in code

## Deploy Pipeline
1. Push to feature branch
2. ZTE auto-deploy fires (`.github/workflows/zte-autodeploy.yml`)
3. Smoke test → Emerald Audit → Vercel deploy → Auto-merge

## Two Dashboard Targets
- `web-ui/` → Archon X dashboard (English, Archon branding) → Vercel
- `web-ui-ivette/` → Ivette/Kupuri dashboard (CDMX Spanish, Synthia branding) → Vercel
