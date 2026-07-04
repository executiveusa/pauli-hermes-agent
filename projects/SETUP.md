# 🚀 Complete System Setup Guide

## Your Automated Project Delivery System is Ready!

This document walks you through setup and first use of the **10-Stage Automated Project Delivery System** based on Interpretable Context Methodology (ICM).

---

## Prerequisites

✅ **Already have:**
- GitHub account & access to repositories
- Vercel account (for deployments)
- VPS with Docker capability (or local Docker)
- This repository cloned

❌ **Still need:**
- Sandcastle configured
- GitHub credentials
- Vercel token
- Docker running on VPS

---

## Step 1: Sandcastle Setup (VPS/Local)

### Option A: Use Docker on Your VPS (Recommended)

```bash
# SSH into your VPS
ssh user@your-vps-ip

# Install Docker (if not already installed)
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Start Docker daemon
sudo systemctl start docker
sudo systemctl enable docker

# Verify Docker works
docker --version  # Should show version 20+
docker ps        # Should show empty or running containers
```

### Option B: Use Local Docker (Development)

```bash
# On your local machine
docker --version  # Should show version 20+

# Start Docker Desktop (macOS/Windows) or daemon (Linux)
# macOS: open -a Docker
# Windows: Start Docker Desktop
# Linux: sudo systemctl start docker
```

### Configure Sandcastle in .env

```bash
cd /home/user/pauli-hermes-agent

# Copy template
cp .env.sandcastle.example .env

# Edit .env
nano .env
# Or use editor of choice
```

**Set these values:**
```env
# Enable Sandcastle
SANDCASTLE_ENABLED=true
SANDCASTLE_DEFAULT_PROVIDER=docker  # Use Docker

# Docker provider
DOCKER_SANDBOX_ENABLED=true
SANDCASTLE_ALLOW_NO_SANDBOX=false

# Vercel (for deployments)
VERCEL_SANDBOX_ENABLED=false  # Not needed yet
```

**Test Sandcastle setup:**
```bash
# Verify Docker is running
docker ps

# Verify configuration
cat .env | grep SANDCASTLE

# Test a simple sandbox run
hermes "Create a test directory in sandbox"
# Should succeed without errors
```

---

## Step 2: GitHub Configuration

### Create GitHub Token

```bash
# Go to GitHub → Settings → Developer settings → Personal access tokens
# Create token with permissions:
  ✅ repo (full)
  ✅ workflow
  ✅ read:org

# Copy the token
export GITHUB_TOKEN="ghp_your_token_here"
```

### Configure Git

```bash
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"

# Or add to .env
echo 'GITHUB_TOKEN=ghp_your_token' >> .env
```

### Test GitHub access

```bash
gh auth status  # Should show "Logged in to github.com"
gh repo list    # Should list your repos
```

---

## Step 3: Vercel Configuration

### Get Vercel Token

```bash
# Go to https://vercel.com/account/tokens
# Create a token with:
  ✅ Full access
  ✅ Allowed scope: Your team/org

# Copy the token
export VERCEL_TOKEN="vercel_your_token_here"
```

### Add to .env

```bash
echo 'VERCEL_TOKEN=vercel_your_token_here' >> .env
echo 'VERCEL_ORG_ID=your_org_id' >> .env
echo 'VERCEL_PROJECT_ID=your_project_id' >> .env
```

### Test Vercel access

```bash
vercel --auth "$VERCEL_TOKEN"
vercel projects list
```

---

## Step 4: Design System Setup

The workflow requires your design system to be documented. Create it:

```bash
mkdir -p design-system
cat > design-system/README.md << 'EOF'
# Your Design System

## Colors
- Primary: #your-color
- Secondary: #your-color
- Accent: #your-color

## Typography
- Font: Your font choice
- Sizes: [list]

## Components
- Button
- Input
- Card
- etc.

## Spacing
- Base: 4px or 8px
- Scale: 4, 8, 12, 16, 24, 32...

(See 03-design/DESIGN_SPEC.md for full format)
EOF
```

Or point to existing design system:
```bash
# If your repo has a design-system folder
# The workflow will auto-detect and use it
```

---

## Step 5: Environment Variables Complete List

```bash
cat > .env << 'EOF'
# ============= SANDCASTLE CONFIG =============
SANDCASTLE_ENABLED=true
SANDCASTLE_DEFAULT_PROVIDER=docker
DOCKER_SANDBOX_ENABLED=true
SANDCASTLE_ALLOW_NO_SANDBOX=false
SANDCASTLE_MAX_PARALLEL=1
SANDCASTLE_LOG_LEVEL=info

# ============= GITHUB CONFIG =============
GITHUB_TOKEN=ghp_your_token_here

# ============= VERCEL CONFIG =============
VERCEL_TOKEN=vercel_your_token_here
VERCEL_ORG_ID=your_org_id

# ============= OPTIONAL: FREE MODE =============
FREE_MODE=true
ANTHROPIC_BASE_URL=http://127.0.0.1:4000
EOF
```

**Verify:**
```bash
source .env
echo "SANDCASTLE_ENABLED: $SANDCASTLE_ENABLED"
echo "GITHUB_TOKEN: ${GITHUB_TOKEN:0:10}..."
echo "VERCEL_TOKEN: ${VERCEL_TOKEN:0:10}..."
```

---

## Step 6: Test Everything

### Test 1: Sandcastle

```bash
# Create a test sandbox run
hermes "Create a test project in sandcastle with docker"

# Should:
# ✅ Create sandbox
# ✅ Run command
# ✅ Return results
```

### Test 2: GitHub Integration

```bash
# Verify you can list repos
gh repo list

# Should show your repositories
```

### Test 3: Vercel Integration

```bash
# List Vercel projects
vercel projects list

# Should show your projects
```

### Test 4: Workflow Structure

```bash
# Check projects folder structure
ls -la projects/
# Should show: 00-intake through 10-report

# Check a stage
cat projects/01-scan/AGENT_PROMPT.md
# Should show agent instructions
```

---

## Step 7: First Project Run

Now you're ready! Let's run a complete project.

### Provide a Repository

**Pick any GitHub repo you own or can fork:**
```bash
# Example: A React app that needs improvements
https://github.com/yourusername/your-test-repo
```

### Create Input File

```bash
cat > projects/00-intake/input.json << 'EOF'
{
  "repo_url": "https://github.com/yourusername/your-test-repo",
  "repo_name": "your-test-repo",
  "project_name": "Test Project",
  "user_request": "Add a dark mode toggle to the UI",
  "submitted_at": "2026-07-04T19:00:00Z"
}
EOF
```

### Run Stage 01: Scan

```bash
# Trigger Stage 01
hermes "Scan and analyze the repository: https://github.com/yourusername/your-test-repo"

# System will:
# 1. Clone the repo
# 2. Analyze structure
# 3. Generate repo-analysis.json
# 4. Create SCAN_SUMMARY.md
```

### Review Scan Results

```bash
# Check what it found
cat projects/01-scan/SCAN_SUMMARY.md

# Should include:
# - Project type (React, Node, etc.)
# - Tech stack
# - Current issues
# - Recommendation for next step
```

### Run Stage 02: PRD

```bash
# When ready, trigger Stage 02
hermes "Generate a PRD from the scan results"

# System will:
# 1. Read scan output
# 2. Generate requirements document
# 3. Create PROJECT_PRD.md
# 4. Define scope
```

### Continue Through Stages

```bash
# Stage 03: Design check
hermes "Check design system compliance"

# Stage 04: Planning
hermes "Create implementation plan"

# Review the plan before continuing:
cat projects/04-plan/IMPLEMENTATION_PLAN.md

# Stage 05: Development (Sandcastle)
hermes "sandbox the development work"
# Watches Sandcastle, gets approval

# Stages 06-10: Automatic
# Testing → Deployment → Verification → Merge → Report
```

### Get Your Result

```bash
# Check final report
cat projects/10-report/PROJECT_COMPLETION_SUMMARY.md

# Get Vercel link
cat projects/07-deploy/DEPLOYMENT.json | grep "url"

# Should output:
# https://your-project.vercel.app
```

---

## Step 8: Organize Your System

Once working, organize projects by date or name:

```bash
# Create organization structure
mkdir -p projects-archive/2026-07
mv projects projects-archive/2026-07/project-001-dark-mode

# For new projects
mkdir projects

# Start next project
cat > projects/00-intake/input.json << 'EOF'
{
  "repo_url": "https://github.com/yourusername/next-project",
  ...
}
EOF
```

---

## Troubleshooting

### Docker Issues

```bash
# Docker not starting
sudo systemctl start docker

# Permission denied
sudo usermod -aG docker $USER
newgrp docker

# Test Docker
docker run hello-world
```

### GitHub Token Issues

```bash
# Generate new token
# https://github.com/settings/tokens/new

# Test token
gh auth login  # Provide token when prompted
gh repo list
```

### Vercel Issues

```bash
# Re-authenticate with Vercel
vercel login --token $VERCEL_TOKEN

# Verify you can deploy
vercel --prod --confirm
```

### Sandcastle Issues

```bash
# Check configuration
cat .env | grep SANDCASTLE

# Check Docker daemon
docker ps  # Should not error

# Verify Sandcastle can run
hermes "Test sandcastle with simple command"
```

---

## Common Workflows

### Quick Project Update

```bash
# Point at repo
# It scans, analyzes, plans, builds, tests, deploys
# You get Vercel link when done
hermes "Update https://github.com/owner/repo with latest design system"
```

### Feature Implementation

```bash
# Implement a specific feature
hermes "Add authentication system to https://github.com/owner/repo"
# Full workflow: scan → PRD → plan → build → deploy → report
```

### Bug Fixes

```bash
# Fix specific issues
hermes "Fix the failing tests in https://github.com/owner/repo"
# Workflow handles diagnosis, fix, verification
```

### Batch Processing

```bash
# Process multiple repos
# Each gets a separate projects folder
# Stages run sequentially or in parallel

hermes "Process all these repos:
  1. https://github.com/owner/repo1
  2. https://github.com/owner/repo2
  3. https://github.com/owner/repo3
"
```

---

## Performance Tips

### Reduce Sandcastle Time

```env
# Faster provider selection
SANDCASTLE_DEFAULT_PROVIDER=docker

# Limit parallelism if VPS is small
SANDCASTLE_MAX_PARALLEL=1
```

### Cache Dependencies

```bash
# Pre-pull common base images
docker pull node:20-alpine
docker pull python:3.11-slim
docker pull ubuntu:latest
```

### Monitor System

```bash
# Watch Docker resource usage
docker stats

# Check disk space
df -h

# Monitor CPU/Memory
top -u docker
```

---

## What's Next

1. ✅ Environment setup complete
2. ✅ Test with first project (dark mode example)
3. ✅ Review all 10 stages in projects/
4. ✅ Customize design system if needed
5. ✅ Run full automated projects
6. ✅ Archive completed projects
7. ✅ Iterate and improve

---

## Support & Customization

### Modify a Stage

Edit the prompt in any stage folder:
```bash
# Example: Modify Stage 02 PRD generation
nano projects/02-prd/AGENT_PROMPT.md
# Make changes
# Next PRD generation uses new prompt
```

### Add a Custom Stage

Create new stage folder:
```bash
mkdir projects/11-custom
cat > projects/11-custom/AGENT_PROMPT.md << 'EOF'
# Your custom stage instructions
EOF
```

### Change Deployment Target

Instead of Vercel, deploy to:
- Netlify (update Stage 07 prompt)
- AWS (update Stage 07 prompt)
- Self-hosted (update Stage 07 prompt)

---

## You're All Set! 🎉

Your complete automated project delivery system is now operational.

**Next action:**
```bash
# Point at your first test repository
hermes "Begin analysis of https://github.com/yourusername/test-repo"

# System will guide you through all 10 stages
# You get a Vercel link when complete!
```

**Questions?**
- Review `WORKFLOW.md` for architecture
- Check individual stage prompts in `projects/XX-*/`
- See results in `projects/10-report/` for completed projects

**Ready to automate everything?** 🚀
