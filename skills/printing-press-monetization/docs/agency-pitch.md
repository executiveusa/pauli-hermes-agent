# Developer Agency Pitch Materials

## Core Value Proposition

**"We build production CLIs + MCP servers for any API in 24 hours."**

Most dev teams spend 2-4 weeks building CLIs from scratch. We deliver the same thing in 24 hours using the CLI Printing Press. Your developers get command-line access to any internal or external API, plus an MCP server that lets Claude, Cursor, and Codex call your API directly.

**Economics:** ~20 minutes to generate. Charged at $500–$2,000. 95%+ margin.

---

## Upwork Profile

**Title:** Custom Go CLI + MCP Server Developer — 24h Delivery

**Hourly Rate:** $150/hr (or fixed-price packages below)

**Bio:**

I specialize in building production-ready CLIs and MCP servers for APIs — REST, GraphQL, gRPC, or internal.

Most teams either skip CLI tooling entirely (forcing everything through a UI) or spend weeks building CLIs that no one maintains. I solve both problems: delivery in 24 hours, production quality, zero maintenance overhead.

What you get:
- Full Go CLI with all API endpoints as named commands (with `--help`, shell completion, config files)
- MCP server for Claude, Cursor, Codex compatibility
- Cross-platform binaries: Linux, macOS, Windows (amd64 + arm64)
- GitHub Actions release pipeline
- Claude Code skill file (AI-native integration)

I've generated 50+ CLIs across fintech, SaaS, crypto, and infrastructure APIs.

**Packages:**
- Basic: $500 — CLI for 1 API, up to 20 commands, 24h delivery
- Standard: $1,000 — CLI + MCP server + Claude Code skill, 24h delivery
- Premium: $2,000 — Full package + 30-day support + auto-update pipeline

---

## LinkedIn Outreach Sequences

### Sequence A: CTOs at Series A/B Startups

**Connection Request Note:**
Hi [Name] — love what you're building at [Company]. I noticed you're working with [relevant tech]. Had an idea relevant to your dev stack — mind connecting?

**Follow-up (Day 3 if accepted):**
Hey [Name], thanks for connecting.

Quick question: do your devs have CLI access to your internal APIs yet, or does everything go through the UI?

Most engineering teams I talk to either don't have CLIs (forces UI dependency) or have one someone built 2 years ago that's broken.

We build production Go CLIs + MCP servers for internal APIs in 24 hours. Your team gets direct terminal access to any service, and your AI tools (Claude, Cursor, etc.) can call your APIs automatically.

Recent example: built a full CLI for a fintech API (40+ endpoints, auth, rate limiting) in 18 hours.

Cost: $1K for CLI + MCP server. Want to see the example repo?

**Follow-up (Day 7 if no reply):**
[Name] — one last ping. If CLI tooling for your APIs isn't a priority right now, totally understood. If timing changes, we're at cliin24h.com.

---

### Sequence B: DevOps Leads

**Initial message:**
Hi [Name] — quick question: how do your devs interact with your internal services today? Mostly through UIs, or do you have CLI tooling?

Asking because we specialize in building Go CLIs + MCP servers for internal APIs (24-hour delivery). It's the fastest way to give your team terminal access to any service plus native AI tool integration.

If it's relevant, happy to share an example. If not, no problem — keep building great things at [Company].

---

## Email Templates

### Cold Outreach — Engineering Managers

**Subject:** CLI for your internal APIs — 24-hour delivery

Hey [Name],

Found [Company] while looking at teams doing interesting work with [relevant tech].

Quick question: do your developers have CLI access to your internal APIs?

Most teams either skip it (everything goes through the UI) or have a stale CLI nobody maintains. The better option: a production-grade CLI + MCP server that lets your devs and AI tools interact with your APIs directly.

We deliver them in 24 hours for $1,000. Built on Go, works everywhere, includes an MCP server for Claude/Cursor compatibility.

Example: a fintech team at a Series B got a 40-endpoint CLI in 18 hours, shipped to their repo as a PR.

Worth a 15-minute call? I can show you the output.

[Your name]
[cliin24h.com]

---

## Fiverr Gig Structure

**Gig Title:** I'll build a production CLI + MCP server for your API in 24 hours

**Tags:** go, cli, mcp, api, developer-tools, command-line

**Basic ($500):**
- CLI for 1 API
- Up to 20 commands
- Linux + macOS binaries
- Basic help text + examples
- Delivery: 24 hours
- Revisions: 2

**Standard ($1,000):**
- CLI + MCP server
- Up to 50 commands
- All platforms (Linux, macOS, Windows, arm64)
- Claude Code skill file
- Release pipeline (GitHub Actions)
- Delivery: 24 hours
- Revisions: 3

**Premium ($2,000):**
- Everything in Standard
- 30-day support
- Auto-update pipeline
- Priority delivery (12 hours)
- Onboarding call
- Revisions: unlimited (30-day period)

---

## Client Questionnaire (Pre-Project)

Send this before starting to avoid back-and-forth:

```
1. What API are we building for?
   [ ] Public API (URL: ___)
   [ ] Internal API (will share docs)
   [ ] No docs yet (share example curl commands)

2. What format are your API docs in?
   [ ] OpenAPI/Swagger spec
   [ ] Postman collection
   [ ] Markdown/PDF documentation
   [ ] None (reverse engineering needed)

3. Priority endpoints (optional — list them or say "all"):

4. Authentication type:
   [ ] API key (header)
   [ ] Bearer token
   [ ] Basic auth
   [ ] OAuth
   [ ] None

5. Delivery preferences:
   [ ] PR to my GitHub repo
   [ ] ZIP download
   [ ] Both

6. Which package? Basic $500 / Standard $1,000 / Premium $2,000
```

---

## Portfolio Projects (to reference in pitches)

### 1. CoinGecko Crypto CLI
Commands: `prices`, `arb`, `signal`, `portfolio`, `market-phase`, `swap`
Platforms: Linux/macOS/Windows, arm64
Special: Cross-API compound commands (CoinGecko + altFINS + ChangeNOW)
Time to build: 20 minutes

### 2. IdeaBrowser Research CLI
Commands: `search`, `trending`, `validate`, `map`, `pitch`, `compare`
Platforms: Linux/macOS/Windows
Special: Browser-sniffed API (no official docs)
Time to build: 25 minutes

### 3. Stripe CLI Extension
Commands: `balance`, `customers list`, `invoices export`, `webhook replay`
Platforms: All
Special: Extends official Stripe CLI with compound reporting commands
Time to build: 15 minutes

---

## Pricing Philosophy

**Never apologize for the price.**

$1,000 for a 24-hour CLI is 90%+ cheaper than hiring a Go developer for a week. The value is in the delivery speed and the MCP integration that no consultant offers.

If a client says it's too expensive:
- Offer Basic at $500 (CLI only, no MCP)
- Ask what their developer rate is (most teams pay $150–250/hr — that's $1,200–2,000/week)
- Point out the MCP server is the future-proofing piece (AI tools call their API for free)

**Never discount below $400 for any package.** Under that, the margin isn't worth it.
