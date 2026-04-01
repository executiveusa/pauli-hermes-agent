---
name: opencli-rs
description: Use opencli-rs to read and act on supported websites through the user's Chrome login session. Prefer this over browser automation for sites that opencli-rs already supports.
version: 0.1.0
author: executiveusa
license: MIT
platforms: [macos, linux]
metadata:
  hermes:
    tags: [browser, chrome, social, research, automation, cli]
    category: browser
    related_skills: [web-research]
---

# opencli-rs

Use this skill when the user wants Hermes to handle supported websites through their existing **Chrome browser session** instead of Playwright-style browser automation.

This skill wraps the Rust CLI from `nashsu/opencli-rs` and is adapted from `nashsu/opencli-rs-skill` for Hermes Agent.

## When to Use

Load this skill when all of the following are true:

- The user wants to browse, search, read, monitor, or post on a site that opencli-rs supports.
- The user wants Hermes to use their **Chrome browser login state**.
- A lighter CLI-first workflow is preferable to full browser automation.

Typical triggers:

- "Use my Chrome session to search YouTube / Reddit / X / Bilibili / Zhihu / Google / Bloomberg"
- "Check what is trending on X / Weibo / Hacker News / Google Trends"
- "Read my bookmarks / saved posts / feed / notifications on a supported site"
- "Post / reply / like / follow using my existing logged-in browser session"

Prefer this skill over browser automation for supported sites.

## Supported Modes

- **Public**: no browser required
- **Browser**: requires Chrome open and logged in, plus the opencli-rs Chrome extension
- **Desktop**: requires the target desktop app to be running

Representative supported services include:

- Public / browser-capable content: Hacker News, Dev.to, Lobsters, StackOverflow, Wikipedia, arXiv, BBC, Google, Bloomberg, Reuters, Yahoo Finance
- Browser-backed social/content sites: X/Twitter, Reddit, YouTube, Bilibili, Zhihu, Xiaohongshu, Weibo, Douban, WeRead, Facebook, Instagram, TikTok, LinkedIn, Medium, Substack, Jike, Xueqiu
- Desktop app bridges: Cursor, Codex, Notion, ChatGPT, Discord, ChatWise

When unsure, run `opencli-rs --help` or `opencli-rs list` first.

## Prerequisites

Before using browser-mode commands, verify:

1. Chrome is installed and currently running.
2. The user is logged into the target site in Chrome.
3. The opencli-rs Chrome extension is installed.
4. `opencli-rs` is available in PATH.

## Quick Reference

```bash
# Install CLI
curl -fsSL https://raw.githubusercontent.com/nashsu/opencli-rs/main/scripts/install.sh | sh

# Diagnostics
opencli-rs doctor
opencli-rs list
opencli-rs --help

# Public / browser read flows
opencli-rs hackernews top --limit 20 --format json
opencli-rs google search --query "Hermes Agent" --format json
opencli-rs youtube search --query "LLM tutorial" --limit 10 --format json
opencli-rs reddit search --query "transformer papers" --limit 10 --format json
opencli-rs twitter trending --limit 20 --format json
opencli-rs bilibili hot --limit 10 --format json

# Personal / logged-in browser data
opencli-rs twitter bookmarks --format json
opencli-rs reddit saved --limit 20 --format json
opencli-rs weread shelf --format json
opencli-rs xueqiu watchlist --format json

# Write actions (confirm first)
opencli-rs twitter post --text "Hello from Hermes"
opencli-rs twitter reply --url "https://x.com/.../status/123" --text "Great post"
opencli-rs instagram comment --url "https://..." --text "Nice post"
```

Use `--format json` whenever possible so Hermes can parse the result reliably.

## Procedure

### 1) Confirm the fast path

- Check whether the target site is likely supported.
- If yes, prefer `opencli-rs` over browser automation.
- If the request is read-only, proceed directly with CLI inspection.

### 2) Verify installation and Chrome readiness

Run:

```bash
command -v opencli-rs >/dev/null 2>&1 && echo installed || echo missing
opencli-rs doctor
```

If missing, install:

```bash
curl -fsSL https://raw.githubusercontent.com/nashsu/opencli-rs/main/scripts/install.sh | sh
```

If browser-mode commands are needed, also remind the user that Chrome must be open and logged in to the target site, with the extension installed.

### 3) Discover the right command

Start with:

```bash
opencli-rs list
opencli-rs <site> --help
```

Then run the narrowest command that matches the user intent.

Examples:

```bash
opencli-rs google news --query "AI regulation" --limit 10 --format json
opencli-rs youtube transcript --id <video_id> --lang en --format json
opencli-rs reddit subreddit --name MachineLearning --sort top --limit 10 --format json
opencli-rs twitter profile --username openai --limit 10 --format json
```

### 4) Parse and report results cleanly

- Prefer JSON output.
- Summarize the findings compactly.
- Preserve URLs, IDs, usernames, titles, and timestamps when relevant.
- If the command output is noisy, filter to the fields the user actually needs.

### 5) For write operations, require explicit confirmation

Before any irreversible action such as posting, replying, liking, following, publishing, messaging, or deleting:

- Show the exact command or content that will be sent.
- Ask for explicit user confirmation.
- Only execute after confirmation.

### 6) Fall back intelligently when unsupported

If `opencli-rs` does not support the site or command:

1. Check `opencli-rs <site> --help`.
2. Try discovery helpers like:

```bash
opencli-rs explore <url>
opencli-rs cascade <url>
opencli-rs generate <url>
```

3. If still unsupported, switch to Hermes browser tooling or another suitable tool.
4. Note the gap so the workflow can later be turned into a new adapter or follow-up skill.

## Pitfalls

### Chrome session not available

Symptoms:

- Authentication errors
- Empty personal feeds
- Browser-mode commands failing

Fix:

- Ensure Chrome is open.
- Ensure the user is logged into the target site in Chrome.
- Ensure the opencli-rs extension is installed and enabled.

### CLI installed but command missing

Symptoms:

- `unknown command`
- Site exists but requested subcommand does not

Fix:

- Run `opencli-rs <site> --help`.
- Use a supported command.
- Fall back to another Hermes capability if the exact flow is unavailable.

### Unsafe write actions

Risk:

- Public posts, replies, likes, follows, deletions, or messages may be immediate and user-visible.

Rule:

- Never perform write actions without explicit approval.

### Overusing browser automation

Rule:

- If opencli-rs supports the target site and task, use it first. It is usually faster, cheaper in context, and more reliable than full browser control.

## Verification

A task is successfully completed when:

- `opencli-rs doctor` reports a healthy setup for the required mode.
- The command returns structured output or an expected success message.
- For read operations, Hermes can extract the requested facts from the result.
- For write operations, the command succeeds after explicit user confirmation.

## Notes for Hermes

- Prefer `--format json`.
- Use targeted commands instead of broad scraping.
- Keep summaries short and operational.
- Preserve user privacy: do not dump personal feed/history data unless requested.
- If this skill proves broadly useful, keep it in `optional-skills/browser/opencli-rs/` and surface it through `hermes skills browse` / `hermes skills install official/browser/opencli-rs`.
