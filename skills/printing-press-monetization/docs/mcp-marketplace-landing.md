# MCP Server Marketplace — Landing Page Copy

**URL:** mcpmarket.dev (or mcp-servers.lemonsqueezy.com)

---

## Hero Section

**Headline:**
MCP servers for your entire SaaS stack.

**Subheadline:**
Stop building integrations. Start shipping. Production-ready MCP servers for Salesforce, HubSpot, Notion, Linear, Stripe, QuickBooks, Shopify, and Jira — ready in minutes.

**CTA:** Browse servers — from $29/month

---

## The Problem

Every enterprise team using Claude, Cursor, or any MCP-compatible AI tool faces the same problem:

**Your SaaS tools don't talk to your AI tools.**

Want Claude to search Salesforce records? You need an MCP server. Want Cursor to create Linear tickets? You need an MCP server. Building one from scratch takes weeks of engineering time.

**We built them for you.**

---

## Server Catalog

| API | Price | What You Get |
|-----|-------|--------------|
| Salesforce | $49/mo | Leads, opportunities, contacts, reports, SOQL queries |
| HubSpot | $29/mo | CRM contacts, deals, companies, email activity |
| Notion | $29/mo | Pages, databases, blocks, search, comments |
| Linear | $29/mo | Issues, projects, cycles, teams, roadmaps |
| Stripe | $29/mo | Customers, invoices, subscriptions, payments |
| QuickBooks | $49/mo | Invoices, expenses, reports, accounts |
| Shopify | $39/mo | Products, orders, customers, inventory, analytics |
| Jira | $29/mo | Issues, sprints, projects, boards, workflows |

**Bundle: Any 3 servers — 20% off**

---

## How It Works

**1. Subscribe**
Pick your servers. Monthly subscription, cancel anytime.

**2. Get Access**
Receive a private GitHub repo with your MCP server + auto-updates.

**3. Install in 60 seconds**

```bash
# Clone your private repo
git clone https://github.com/mcp-marketplace/[your-server].git

# Add to Claude Code config
cat >> ~/.claude/claude.json << 'EOF'
{
  "mcpServers": {
    "salesforce": {
      "command": "node",
      "args": ["./salesforce-mcp/index.js"],
      "env": {
        "SF_CLIENT_ID": "your-client-id",
        "SF_CLIENT_SECRET": "your-client-secret",
        "SF_INSTANCE_URL": "https://yourcompany.my.salesforce.com"
      }
    }
  }
}
EOF
```

**4. Use it**

```
You: Show me all deals closing this month in Salesforce
Claude: [calls Salesforce MCP] Found 23 deals closing in August...

You: Create a Linear ticket for the Shopify webhook bug
Claude: [calls Linear MCP] Created LIN-2847: "Shopify webhook processing delay"
```

---

## For Developers (API)

Each MCP server exposes standard MCP tools:

**Salesforce MCP tools:**
- `salesforce_query` — SOQL query execution
- `salesforce_create_record` — Create any object
- `salesforce_update_record` — Update records
- `salesforce_search` — Full-text search
- `salesforce_get_record` — Fetch by ID
- `salesforce_list_objects` — Schema discovery

All servers follow the same pattern. Pick it up once, use it everywhere.

---

## Pricing Details

**Monthly:**
Single server: $29–49/month depending on API complexity.

**Annual (2 months free):**
Single server: $249–399/year.

**Team bundle (5 seats):**
Any 3 servers: $99/month (20% off)
Any 5 servers: $149/month (25% off)

**Enterprise:**
All 8 servers + custom servers + SLA: $499/month. Contact us.

---

## What's Included

Every MCP server subscription includes:
- Private GitHub repo access
- Automatic updates (merged PRs when APIs change)
- Authentication handled (OAuth, API keys, service accounts)
- Error handling and rate limit management
- MCP schema definitions (tools, resources, prompts)
- Setup guide and environment variable reference
- Email support

---

## Distribution Strategy

Submit to these registries after launch:

**MCP Registries:**
- mcp.so — largest MCP server registry
- mcpmarket.com — dedicated marketplace
- glama.ai — curated MCP index
- smithery.ai — MCP discovery platform

**Developer Communities:**
- r/ClaudeAI — "MCP servers for Salesforce, HubSpot, Notion, and 5 more"
- Hacker News — "Show HN: MCP marketplace — production servers for your SaaS stack"
- Claude Discord
- Cursor community

**LinkedIn (target IDs and RevOps):**
- "We just launched MCP servers for Salesforce and HubSpot. Your Claude/Cursor setup can now search CRM records, pull deal data, and create contacts automatically. $29/month. Link in comments."

---

## Setup Checklist

- [ ] Register `mcpmarket.dev`
- [ ] Create Stripe products (run `scripts/setup-stripe-products.sh`)
- [ ] Set up GitHub App for private repo delivery
- [ ] Print 8 MCP servers with CLI Printing Press (30 min parallel)
- [ ] Deploy minimal Next.js storefront to Vercel
- [ ] Submit to mcp.so, mcpmarket.com, glama.ai
- [ ] Post to r/ClaudeAI and Hacker News
- [ ] Target first 10 customers via LinkedIn DMs to RevOps/IT leads

**Revenue target:** 10 customers × 3 servers × $29 average = $870/month from first 10 customers.
