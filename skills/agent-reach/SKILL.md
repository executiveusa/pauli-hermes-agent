---
name: agent-reach
description: >
  Use for YouTube transcripts, video research, web pages, RSS, GitHub research,
  social/community discovery, LinkedIn research, or any cross-platform internet
  collection task. Trigger when the user shares a URL or asks to search, scrape,
  research, compare public discussion, extract a transcript, or collect sources.
  Agent Reach fetches evidence; Hermes analyzes, synthesizes, and stores it.
platforms: [linux, macos, windows]
metadata:
  hermes:
    category: research
  upstream:
    repository: https://github.com/Panniantong/Agent-Reach
    version: 1.5.0
    commit: b4d52c46c9113cb0f653d6df4cf71ebadf4930ac
---

# Agent Reach for Hermes

Agent Reach is Hermes' read-only internet acquisition router. It selects and
health-checks upstream tools such as `yt-dlp`, Jina Reader, Exa through
`mcporter`, `gh`, OpenCLI, platform CLIs, and transcription backends.

## Hard rules

1. **Inspect health first.** Run `agent-reach doctor --json` before any
   multi-backend or login-backed task and use the reported `active_backend`.
2. **Use upstream tools directly.** Agent Reach is the installer, selector,
   health checker, and router. Do not invent a new command when a documented
   upstream command exists.
3. **Read-only by default.** Search, read, download public transcripts, and
   collect evidence. Do not post, comment, like, follow, message, or modify an
   account unless the user explicitly requests and approves that exact write.
4. **Never expose credentials.** Never print cookies, tokens, browser-session
   data, API keys, or authenticated command environments.
5. **Do not automate login.** For cookie/session platforms, use only a session
   the user already controls or a user-exported credential flow documented by
   Agent Reach.
6. **Keep the workspace clean.** Use `/tmp/agent-reach-*` for temporary output
   and `~/.agent-reach/` for Agent Reach state. Save into a repo, ICM workspace,
   or second brain only when the user asks.
7. **Cite what was collected.** Preserve source URL, title, author/channel,
   publication date when available, retrieval time, and timestamps for video.
8. **Fail honestly.** A search result is not proof that a page was read. A
   downloaded audio file is not a transcript. Mark blocked, partial, inferred,
   and verified evidence separately.

## First-run check

```bash
command -v agent-reach >/dev/null 2>&1 || \
  bash "<skill-directory>/scripts/bootstrap.sh" --apply
agent-reach doctor --json
```

The skill directory is injected by Hermes when the skill loads. Resolve the
script path against that directory. The bootstrap script never uses `sudo`.
Windows users should follow `references/setup.md`.

## Route by intent

| Intent | Load |
|---|---|
| YouTube transcript, video search, multi-video synthesis | `references/youtube.md` |
| Deep research across multiple sources/platforms | `references/research-workflow.md` |
| Platform command and fallback routing | `references/platform-routing.md` |
| Install, update, health, credentials, environment | `references/setup.md` |
| Copy-paste task examples | `references/usage-prompts.md` |

## Default execution contract

1. Clarify the outcome only when the target, date window, language, or required
   source types are genuinely ambiguous.
2. Run `agent-reach doctor --json` and record the backend selected per platform.
3. Collect in parallel where independent, with conservative rate limits.
4. Normalize every item into a source record.
5. Verify important claims with at least two independent sources when possible.
6. Synthesize only after collection is complete.
7. Return findings, evidence links, gaps, and the exact next action.
8. After a substantial multi-platform run, execute `agent-reach check-update`.

## Source record

```json
{
  "source_url": "https://...",
  "platform": "youtube",
  "title": "...",
  "author": "...",
  "published_at": "...",
  "retrieved_at": "...",
  "backend": "yt-dlp",
  "evidence_type": "transcript|page|post|comment|metadata",
  "status": "verified|partial|blocked",
  "notes": "..."
}
```

## Output standard

For research tasks, return:

- **Answer:** direct conclusion.
- **Evidence:** sources tied to claims.
- **Disagreement:** conflicting evidence or uncertainty.
- **Coverage:** platforms searched and blocked.
- **Artifacts:** transcript/report paths only when files were requested.
- **Next action:** one concrete follow-up.
