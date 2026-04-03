# Automation / n8n Workspace Context
# Load this when: working with n8n workflows, lead gen, automation

## n8n Instance
- **Host**: 31.220.58.212 (self-hosted via Coolify)
- **Workflows location**: `n8n-workflows/` in repo
- **Key workflows**: See `n8n-workflows/StoryToolkitAI-main/` for patterns

## Active Workflow Patterns (from n8n-workflows/)
- `github-notion-sync.json` — sync GitHub to Notion
- `mcp-health-check.json` — monitor MCP server health
- `skills-filesystem-sync.json` — sync skills to filesystem

## Lead Generation Workflow (Future-Proof Platform)
Apollo → Apify scraper → filter leads → website scraper → personalized icebreaker → CRM/outreach
Reference: `Building a Future-Proof Autonomous AI Agent Platform.md`

## Available Integrations
- Telegram (messaging gateway)
- WhatsApp (messaging gateway)
- Discord
- Slack
- Supabase
- GitHub
- Vercel

## Usage
- Trigger workflows via n8n REST API from Hermes tools
- `terminal_tool` can call n8n API endpoints directly
- New workflows: design in `workspace/agents/graph-blueprints/` first, then implement in n8n
