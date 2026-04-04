# Pi Coding Agent — Extensions, Tools, and Skills

## The Three Ways to Extend Pi

Pi ships with **only 4 built-in tools**: read, write, edit, bash. Everything else is added through extensions, skills, or prompt templates. Nothing is enabled by default.

---

## 1. Built-in Tools

**What they are**: The 4 tools the LLM can call out of the box. These are hardcoded into Pi.

**Source code**: `https://github.com/badlogic/pi-mono/tree/main/packages/coding-agent/src/core/tools/`

Files: `bash.ts`, `read.ts`, `write.ts`, `edit.ts` (plus `ls.ts`, `grep.ts`, `find.ts` which are also built-in).

**How they work**: The LLM calls them by name with parameters. Pi's host Node.js process executes them. For bash, it spawns a child process. For read/write/edit, it does filesystem operations directly.

---

## 2. Skills

**What they are**: A folder containing a `SKILL.md` file and optionally script files. The LLM teaches itself how to use them by reading the SKILL.md, then runs the scripts via the **existing bash tool**. No new tools are registered. No TypeScript. No build step.

**Where they live**:
- `~/.pi/agent/skills/` (global, available in all projects)
- `.pi/skills/` (project-level)
- Installed via Pi packages

**How they load**: On-demand (progressive disclosure). At startup, Pi scans skill directories and puts **only the name + description** from each SKILL.md frontmatter into the system prompt. The full instructions are NOT loaded until the LLM decides the skill is relevant and reads the file.

**The complete runtime flow** (using brave-search as example):

1. Pi starts, scans skills, sees `brave-search/SKILL.md`, puts description in system prompt
2. User says "search for python async tutorials"
3. LLM sees the description, decides brave-search is relevant
4. LLM uses the **read** tool (built-in) to read the full SKILL.md
5. LLM reads the instructions, learns the commands
6. LLM uses the **bash** tool (built-in) to run: `/path/to/brave-search/search.js "python async tutorials"`
7. `search.js` runs as a child process, reads `$BRAVE_API_KEY` from the environment, calls the Brave Search API, prints results to stdout
8. Bash tool captures stdout, returns it to the LLM
9. LLM reads results, responds to user

**Key point**: The LLM is the one running the command through bash. Skills assume a trusted environment — the LLM has access to environment variables, which means it could `echo $BRAVE_API_KEY` if it wanted to.

### What's inside a skill folder (real example: brave-search)

```
brave-search/
├── SKILL.md          ← instructions for both LLM and human
├── package.json      ← declares npm dependencies
├── search.js         ← Node.js script that calls Brave Search API, prints results to stdout
└── content.js        ← Node.js script that fetches a URL, extracts readable markdown
```

**SKILL.md contents**:
```markdown
---
name: brave-search
description: Web search and content extraction via Brave Search API. Use for searching documentation, facts, or any web content.
---
# Brave Search

## Setup
cd {baseDir} && npm install

## Search
{baseDir}/search.js "query"              # Basic search (5 results)
{baseDir}/search.js "query" -n 10        # More results (max 20)
{baseDir}/search.js "query" --content    # Include page content as markdown
{baseDir}/search.js "query" --freshness pw  # Results from last week

## Extract Page Content
{baseDir}/content.js https://example.com
```

`{baseDir}` is replaced at runtime with the actual path to the skill folder.

**Setup**: You run `npm install` once in the skill directory. The `package.json` declares what dependencies the scripts need. You also need to set `BRAVE_API_KEY` as an environment variable (get one from https://api-dashboard.search.brave.com/register).

**The skill IS the bundle** — the SKILL.md, the code files, and the package.json all live in one directory. You don't need to go find separate pieces.

### Where to find skills

**Pi skills repo**: `https://github.com/badlogic/pi-skills`

Available skills: brave-search, browser-tools, gccli (Google Calendar), gdcli (Google Drive), gmcli (Gmail), subagent, transcribe, vscode, youtube-transcript.

Install: `git clone https://github.com/badlogic/pi-skills ~/.pi/agent/skills/pi-skills`

These skills follow the **Agent Skills standard** (SKILL.md format developed by Anthropic), also compatible with Claude Code and OpenAI Codex.

**There is no centralized Pi skill registry.** Skills are shared via git repos, npm, or Discord.

### browser-tools skill

Uses **Chrome DevTools Protocol (CDP) directly** — not Playwright, not Puppeteer. Connects to Chrome running with `--remote-debugging-port=9222`. Standalone JS scripts:

- `browser-nav.js` — navigate to URLs
- `browser-eval.js` — execute JS in the active tab
- `browser-search.js` — Google search
- `browser-screenshot.js` — take screenshots
- `browser-click.js` — click elements
- `browser-picker.js` — interactive element selector

Requires a visible Chrome window with remote debugging enabled.

---

## 3. Extensions

**What they are**: TypeScript modules that run in Pi's host Node.js process. They can register new tools, intercept/modify bash commands, hook into events, replace built-in tool implementations, render UI, persist state into sessions.

**Key difference from skills**: With a skill, the LLM runs the command through bash. With an extension, the host Node.js process runs the code. The LLM calls a registered tool by name, the extension code executes, and returns results. The LLM never sees how it was done.

**This is why extensions can protect secrets**: The extension code reads credentials from memory/files in the Node.js process. The LLM only sees the tool name and the returned results, never the credentials. Skills can't do this because the LLM is the one running bash and has access to everything bash has access to.

**Where they live**:
- `~/.pi/agent/extensions/` (global, auto-discovered)
- `.pi/extensions/` (project-level)
- Loaded explicitly with `pi --extension /path/to/extension.ts`
- Installed via Pi packages

**How they work**: An extension exports a default function that receives `ExtensionAPI`:

```typescript
import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";

export default function (pi: ExtensionAPI) {
  // Register new tools
  pi.registerTool({ name: "deploy", ... });

  // Register slash commands
  pi.registerCommand("stats", { ... });

  // Hook events
  pi.on("tool_call", async (event, ctx) => { ... });
  pi.on("tool_result", async (event, ctx) => { ... });

  // Intercept bash commands before execution
  pi.setBashSpawnHook((cmd, cwd, env) => { ... });
}
```

**Key capabilities**:
- `pi.registerTool()` — add new tools the LLM can call (appear alongside the built-in 4)
- `pi.setBashSpawnHook()` — intercept/modify bash commands before execution
- `pi.on("tool_call")` — intercept or block any tool call
- `pi.on("tool_result")` — modify tool results before LLM sees them
- `pi.on("input")` — intercept user input before agent processes it
- `pi.on("context")` — modify messages sent to the LLM each turn
- Pluggable operations — replace built-in tool implementations (`BashOperations`, `ReadOperations`, etc.)
- `pi.registerCommand()` — add slash commands
- `pi.sendMessage()` — send custom messages
- UI rendering — status bars, overlays, custom message renderers
- State persistence into sessions

**Always loaded** — unlike skills which load on-demand, extensions are active from startup.

### Where to find extensions

**Example extensions in Pi repo**: `https://github.com/badlogic/pi-mono/tree/main/packages/coding-agent/examples/extensions`

50+ examples including: sandbox (ASRT), permission-gate, SSH execution, plan mode, sub-agents, MCP integration, custom editors, status bars, overlays, Doom.

**These are NOT loaded by default.** They are reference code. You copy them or point Pi at them to use them.

**npm packages**: Search npmjs.com for the `pi-package` keyword.

**There is no centralized extension registry for Pi.**

### Pi Packages (bundling extensions + skills together)

A Pi package is an npm or git repo that bundles extensions, skills, prompts, and themes together. Declared via `package.json`:

```json
{
  "name": "my-pi-package",
  "keywords": ["pi-package"],
  "pi": {
    "extensions": ["./extensions"],
    "skills": ["./skills"],
    "prompts": ["./prompts"],
    "themes": ["./themes"]
  }
}
```

Install: `pi install npm:@foo/pi-tools` or `pi install git:github.com/user/repo`

Manage: `pi list`, `pi update`, `pi remove`, `pi config`

---

## Extensions vs Skills — When to Use Which

| | Skills | Extensions |
|---|---|---|
| **Who runs the code** | LLM runs it via bash | Host Node.js process runs it |
| **New tools registered** | No — uses existing bash tool | Yes — `pi.registerTool()` |
| **Complexity** | Low — markdown + scripts | Higher — TypeScript against ExtensionAPI |
| **Loading** | On-demand (token-efficient) | Always loaded at startup |
| **Can modify Pi behavior** | No | Yes — hook events, intercept bash, replace tools |
| **Can protect secrets** | No — LLM has bash access to env vars | Yes — code runs in host process, LLM never sees credentials |
| **Good for** | CLI tools the LLM invokes | Security layers, new tool types, UI changes, system-level modifications |

**Rule of thumb**: If the LLM just needs to run a CLI command, make it a skill. If you need to change how Pi itself works, or protect something from the LLM, make it an extension.

---

## Credentials & Security Interaction

If a security extension strips environment variables from `process.env` (to hide them from the LLM's bash access), **skills that depend on those env vars will break**. Each credential needs a deliberate choice:

- **Leave in environment** — skills work, but LLM could read the secret via bash
- **Strip and expose only through extension tools** — more secure, but requires rewriting the skill as an extension tool

---

## Deprecated — Ignore

`https://github.com/badlogic/agent-tools` — same tools as pi-skills but without SKILL.md wrappers. Superseded by pi-skills. Ignore it.

---

## Key URLs

| Resource | URL |
|----------|-----|
| Pi repo | https://github.com/badlogic/pi-mono |
| Pi coding agent | https://github.com/badlogic/pi-mono/tree/main/packages/coding-agent |
| Built-in tools source | https://github.com/badlogic/pi-mono/tree/main/packages/coding-agent/src/core/tools/ |
| Extensions docs | https://github.com/badlogic/pi-mono/blob/main/packages/coding-agent/docs/extensions.md |
| Skills docs | https://github.com/badlogic/pi-mono/blob/main/packages/coding-agent/docs/skills.md |
| Example extensions (50+) | https://github.com/badlogic/pi-mono/tree/main/packages/coding-agent/examples/extensions |
| Pi skills repo | https://github.com/badlogic/pi-skills |
| Pi website | https://shittycodingagent.ai/ |
| ASRT (sandbox runtime) | https://github.com/anthropic-experimental/sandbox-runtime |
