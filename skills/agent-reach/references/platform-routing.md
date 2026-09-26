# Platform routing and boundaries

Always run `agent-reach doctor --json` before login-backed or multi-backend
platform work. Use the reported `active_backend` and the corresponding command
group.

## Zero-config routes

```bash
# General web search
mcporter call 'exa.web_search_exa(query: "QUERY", numResults: 5)'

# Code/documentation context
mcporter call 'exa.get_code_context_exa(query: "QUESTION", tokensNum: 3000)'

# Read a public web page as Markdown
curl -s "https://r.jina.ai/URL"

# Public GitHub
 gh repo view owner/repo
 gh search repos "QUERY" --sort stars --limit 10
 gh search code "QUERY" --language python

# RSS/Atom
python3 - <<'PY'
import feedparser
feed = feedparser.parse("FEED_URL")
for item in feed.entries[:10]:
    print(item.title, item.link)
PY

# YouTube search/transcripts
 yt-dlp --dump-json "ytsearch5:QUERY"
```

## Platform table

| Platform | Preferred route | Important boundary |
|---|---|---|
| Web pages | Jina Reader; web-reader MCP for format/image control | Public pages only; report paywall/login blocks. |
| YouTube | `yt-dlp`; `agent-reach transcribe` fallback | Use `--no-playlist` unless playlist requested. |
| GitHub | `gh` | Writes require explicit user approval. |
| RSS | `feedparser` | Preserve feed and item URLs. |
| Twitter/X | `twitter-cli`, then OpenCLI fallback | User-exported credentials only; never print tokens. |
| Reddit | OpenCLI, then `rdt-cli` | Login is required; no anonymous default path. |
| XiaoHongShu | OpenCLI desktop; MCP/server fallback | Never automate login or read arbitrary browser cookies. |
| Bilibili | `bili-cli`; OpenCLI for subtitles | Do not use yt-dlp for Bilibili. |
| Facebook | OpenCLI | Existing user-controlled browser session only. |
| Instagram | OpenCLI | Existing user-controlled browser session only. |
| LinkedIn | LinkedIn MCP; Jina public-page fallback | Login-backed detail may be unavailable. |
| V2EX | public JSON API | Add a clear User-Agent. |
| Xiaoyuzhou | Agent Reach transcription workflow | Requires configured transcription provider. |

## Twitter/X

`agent-reach configure twitter-cookies` stores explicit credential presence for
health checks. Before `twitter` commands, pass the values in the child process
environment without echoing them.

Stable reads:

```bash
twitter tweet URL_OR_ID
twitter article URL_OR_ID
twitter user-posts @username -n 20
twitter user @username
```

Search retry order:

1. retry once;
2. upgrade `twitter-cli` in a separate bounded action;
3. use `opencli twitter search "QUERY" -f yaml` on desktop;
4. use stable user/feed routes and report search unavailable.

## Reddit

```bash
# Desktop preferred
opencli reddit search "QUERY" -f yaml
opencli reddit read POST_ID -f yaml
opencli reddit subreddit LocalLLaMA -f yaml

# Server/legacy
rdt search "QUERY" --limit 10
rdt read POST_ID
```

Do not recommend a new official Reddit app as the default route.

## XiaoHongShu

```bash
# Desktop preferred
opencli xiaohongshu search "QUERY" -f yaml
opencli xiaohongshu note "FULL_RESULT_URL" -f yaml
opencli xiaohongshu comments NOTE_ID -f yaml
```

Use the full result URL/token from search. A bare note ID is insufficient.
Space requests by 2–3 seconds. Keep the workflow read-only.

## Bilibili

```bash
bili search "QUERY" --type video -n 5
bili video BV_ID
bili hot -n 10
opencli bilibili subtitle BV_ID
```

For videos without subtitles, use `bili audio BV_ID`, then transcribe the
resulting audio with Agent Reach.

## LinkedIn

```bash
mcporter call 'linkedin-scraper.get_person_profile(linkedin_url: "URL")'
mcporter call 'linkedin-scraper.search_people(keyword: "QUERY", limit: 10)'
mcporter call 'linkedin-scraper.get_company_profile(linkedin_url: "URL")'
mcporter call 'linkedin-scraper.search_jobs(keyword: "QUERY", limit: 10)'
```

When the authenticated backend is unavailable, use Jina only for accessible
public pages and mark the result partial.

## Writes

Agent Reach integration is read-only by default. Posting, commenting, liking,
following, messaging, opening an issue, creating a PR, or changing an account
must be handled as a separate explicit action with confirmation, least
privilege, and a rollback or correction path.
