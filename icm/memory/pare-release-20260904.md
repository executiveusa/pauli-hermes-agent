# PARÉ release coordination — 2026-09-04

Status: active / NOT READY until runtime and preview gates pass.

## Canonical project
- repo: `executiveusa/PARE`
- branch: `zte/ZTE-20260903-0007/pare-full-rebrand-release`
- PR: #5
- current release work includes sovereign daemon, remote private MCP gateway, Claude Code plugin metadata, luxury landing, agentic Journal, Product Hunt kit, OpenAI/Claude readiness docs, Loop Engineering evidence.

## Identity boundary
- personal Hermes = owner-wide governor/orchestrator.
- MACS Digital Media = separate business/client context.
- Agent Max = MACS business agent.
- Never blend credentials, memory or authority across those identities by convenience.

## PARÉ runtime surfaces
- Studio: human surface on Netlify.
- daemon: owner-controlled runtime on VPS.
- API/SSE: software/agent execution surface.
- `od` CLI: operator compatibility surface.
- stdio MCP: Claude/local-agent compatibility surface.
- `pare-mcp`: private/developer remote MCP gateway; public marketplace use still requires user/workspace identity and hardened authorization.

## Release dependency chain
1. exact revision parity;
2. VPS host stability;
3. daemon localhost health on 7456;
4. private MCP localhost health on 7457;
5. Caddy public health/API/MCP routing;
6. Netlify same-origin proxy health;
7. real Gemini run;
8. second real provider run;
9. real Studio SSE journey;
10. diffusion ON/OFF on real streamed prose;
11. project/artifact persistence;
12. API/CLI/MCP agent access;
13. fresh verifier + Gauntlet;
14. PREVIEW VERIFIED;
15. owner `approve` before production merge/promotion.

## Distribution sequence after proof
- PARÉ homepage + Journal;
- MACS Digital Media proof case;
- Vibe Audit CTA/revenue loop;
- Product Hunt only after production proof;
- Claude Code marketplace after clean-install proof;
- OpenAI Plugins Directory after public remote MCP auth/identity slice and submission packet.

## Hermes operating rule
Before consequential PARÉ work, run the repo walk test, read the active Loop state, check exact PR head, then inspect only directly coupled systems. Do not create a competing platform or expand scope because a new framework is interesting.
