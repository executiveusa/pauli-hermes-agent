# ⚡ Quick Start (5 Minutes)

## One-Time Setup

```bash
cd /home/user/pauli-hermes-agent

# 1. Copy config template
cp .env.sandcastle.example .env

# 2. Add your tokens to .env
nano .env
# Set: GITHUB_TOKEN, VERCEL_TOKEN, DOCKER_SANDBOX_ENABLED=true

# 3. Test Docker is running
docker ps  # Should work

# 4. Done! ✅
```

---

## Run Your First Project

```bash
# 1. Give it a repo
cat > projects/00-intake/input.json << 'EOF'
{
  "repo_url": "https://github.com/owner/repo",
  "repo_name": "repo-name",
  "project_name": "My Project",
  "user_request": "Add dark mode toggle"
}
EOF

# 2. Start the workflow
hermes "Analyze and implement this project: https://github.com/owner/repo"

# 3. System does everything automatically:
#    ✅ Analyzes repo
#    ✅ Creates PRD
#    ✅ Plans implementation
#    ✅ Builds in sandbox
#    ✅ Tests everything
#    ✅ Deploys to Vercel
#    ✅ Verifies deployment
#    ✅ Merges to production
#    ✅ Sends you Vercel link

# 4. Check result
cat projects/10-report/PROJECT_COMPLETION_SUMMARY.md
```

---

## The 10 Stages

| Stage | What Happens | Input | Output |
|-------|--------------|-------|--------|
| **00** | You provide repo | URL | input.json |
| **01** | Scan repo | input.json | repo-analysis.json |
| **02** | Generate PRD | analysis | PROJECT_PRD.md |
| **03** | Check design system | PRD | DESIGN_SPEC.md |
| **04** | Create plan | PRD + design | IMPLEMENTATION_PLAN.md |
| **05** | Build in Sandcastle | plan | Feature branch + code |
| **06** | Run tests | code | TEST_REPORT.md |
| **07** | Deploy to Vercel | tests ✅ | DEPLOYMENT.json |
| **08** | Verify deployment | URL | VERIFICATION_REPORT.md |
| **09** | Merge to main | verified ✅ | Merged to main |
| **10** | Final report | everything | Vercel link + summary |

---

## File Structure

```
projects/
├── 00-intake/        ← You put input here
├── 01-scan/          ← Agent analyzes
├── 02-prd/           ← Agent generates requirements
├── 03-design/        ← Agent checks design system
├── 04-plan/          ← Agent creates plan
├── 05-develop/       ← Sandcastle builds
├── 06-test/          ← Tests verified
├── 07-deploy/        ← Vercel deployed
├── 08-verify/        ← Deployment checked
├── 09-production/    ← Merged to main
└── 10-report/        ← Final results + Vercel link
```

---

## Commands

```bash
# Start workflow for a repo
hermes "Analyze and implement https://github.com/owner/repo"

# Or break into stages:
hermes "Scan this repo"
hermes "Generate requirements"
hermes "Create implementation plan"
hermes "Build in sandbox"
hermes "Deploy to Vercel"
hermes "Merge to production"

# Check status
ls projects/*/  # See completed stages
cat projects/01-scan/SCAN_SUMMARY.md  # See analysis
cat projects/10-report/PROJECT_COMPLETION_SUMMARY.md  # See final result

# View Vercel link
cat projects/07-deploy/DEPLOYMENT.json | grep "url"
```

---

## What You Need

```
✅ Have:
  - GitHub account
  - Vercel account
  - Docker (local or VPS)
  - This repo cloned

❌ Get:
  - GitHub token (https://github.com/settings/tokens)
  - Vercel token (https://vercel.com/account/tokens)
  - Docker running: docker ps
```

---

## Troubleshooting

| Problem | Fix |
|---------|-----|
| "Docker not found" | `sudo systemctl start docker` |
| "Permission denied" | `sudo usermod -aG docker $USER` |
| "Token invalid" | Create new token, update .env |
| "Sandbox failed" | Check Docker: `docker ps` |
| "Deploy failed" | Check Vercel token & permissions |

---

## After First Project

```
✅ You completed a full workflow!

Next:
1. Review all generated files
2. Customize design system if needed
3. Run another project
4. Archive completed projects
5. Scale to multiple repos
```

---

## One Command to Rule Them All

```bash
hermes "Full automated workflow for https://github.com/owner/repo: scan → PRD → plan → build → test → deploy → verify → merge → report"
```

That's it! The system handles all 10 stages automatically. ✨

---

See `WORKFLOW.md` for full details and `SETUP.md` for complete setup.
