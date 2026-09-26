# Hermes MCP-to-CLI Bridge

Hermes talks to MCP servers, OpenAPI specs, and GraphQL endpoints through
[`mcp2cli`](https://github.com/knowsuchagency/mcp2cli) instead of loading full
MCP tool schemas into every agent context. `mcp2cli` turns any of those
sources into a plain CLI at runtime — Hermes discovers commands with
`--list`/`--search`, inspects one with `--help`, and calls it with flags.

See `MCP2CLI.md` for the full command/config reference and `AGENTS.md` for
how this fits into the rest of the coding-agent workflow.

## Layout

```
.hermes/
├── bin/hermes-mcp2cli   # dispatcher for the hermes mcp:*, repo:inspect,
│                         # handoff:opencode, and verify commands below
├── mcp/bootstrap.sh      # idempotent script that bakes the standard tools
├── logs/                 # bridge run logs (gitignored contents)
└── skills/               # local skill state (gitignored contents)
.agents/skills/mcp2cli/    # the mcp2cli agent skill (installed via `npx skills add`)
```

## Setup

```bash
# 1. Install uv if missing
curl -LsSf https://astral.sh/uv/install.sh | sh

# 2. Confirm mcp2cli runs
uvx mcp2cli --help

# 3. Install it globally (optional but recommended)
uv tool install mcp2cli

# 4. Install the mcp2cli agent skill
npx skills add knowsuchagency/mcp2cli --skill mcp2cli

# 5. Bake the standard tools for this repo
.hermes/mcp/bootstrap.sh

# 6. Verify everything
.hermes/bin/hermes-mcp2cli verify
```

## Hermes commands

| Command | What it does |
|---|---|
| `hermes mcp:list [TOOL] [--json]` | List baked tools, or the commands exposed by `TOOL` |
| `hermes mcp:search PATTERN [TOOL]` | Search baked tool names, or commands on `TOOL` |
| `hermes mcp:call TOOL COMMAND [ARGS...]` | Call `COMMAND` on a baked `TOOL`, JSON output |
| `hermes mcp:bake NAME [bake create ARGS...]` | Save a new baked tool config |
| `hermes repo:inspect [--json]` | Summarize repo + bridge state (branch, dirty files, baked tools, tool availability) |
| `hermes handoff:opencode [ARGS...]` | Hand a repo code-edit task off to OpenCode |
| `hermes verify` | Run the bridge verification checklist |

These are implemented by `.hermes/bin/hermes-mcp2cli` (run it directly, or
wire it up as `hermes`'s external-command path — see `AGENTS.md`). Example:

```bash
.hermes/bin/hermes-mcp2cli mcp:list --json
.hermes/bin/hermes-mcp2cli mcp:call filesystem list-allowed-directories
```

## Baked tools

| Name | Category | Precedence |
|---|---|---|
| `filesystem` | filesystem MCP | primary way to read/write the repo tree from a CLI context |
| `github` | GitHub | **fallback only** — prefer the `gh` CLI, then GitHub's MCP server |
| `browser` / `docs` | browser-search / docs-search | not baked by default in this environment (none configured); native WebSearch/WebFetch and Context7 cover these today — see `MCP2CLI.md` for how to bake real ones when a server/URL is provided |
| *(ad hoc)* | OpenAPI / GraphQL | baked per-endpoint with `hermes mcp:bake <name> --spec ...` / `--graphql ...` once a spec/URL is provided |

## Rules

- Prefer normal CLI tools first: `gh`, `git`, `npm`, `pnpm`, `docker`, `vercel`.
- Use `mcp2cli` when an MCP/API tool is a better fit than raw shell.
- Use OpenCode only for repo code edits (`hermes handoff:opencode`) — `mcp2cli`
  is for calling tools, not editing files.
- Never expose secrets in command history. Use `env:VAR` or `file:/path`
  reference syntax for every credential (`--auth-header`, `--env`,
  `--oauth-client-secret`, ...) — never a literal token.
- Save reusable tool configs with `hermes mcp:bake` (wraps `mcp2cli bake create`).
- Use `--json` for machine-readable output.
- Run `--list --compact` before calling an unfamiliar tool.
- Destructive-looking commands (delete/remove/drop/purge/rm/wipe/...) are
  blocked by `hermes mcp:call` unless you pass `--approve`.
