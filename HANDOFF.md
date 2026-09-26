# Hermes Agent Handoff Document

## Overview
This document outlines the current state of the Pauli-Hermes-Agent fork, the upstream changes from the official Hermes repository (NousResearch/hermes-agent) that are missing, and a plan for safely integrating those upstream changes without losing local custom work.

## Current State: Local Custom Commits
The following commits are present in the `main` branch of this fork but **not** in the upstream `main` branch (NousResearch/hermes-agent). These represent customizations made for the Pauli-Hermes-Agent project.

### Recent Local Custom Commits (Non-Merge)
```
c2e286a7e Add skill enforcement and agent-payments CLI integration
ac3d8c9d1 Add interactive-artifact-skill and vendor skills
fedcd9e6a Add external skills: token-saving, black-swan, vercel-a2a
c02eb93e7 Claude/free mode litellm gateway o vn39 (#53)
c320a2bf0 feat(pauli): phase 1 packaging and OpenClaude shim (#51)
04b4e08ff Remove new upstream workflows to allow pushing without workflow scope
6e7efa0dc Restore origin/main workflow files to allow push without workflow scope
3a008e65b Merge latest upstream/main changes (up to v2026.6.5)
c0febdc33 Merge origin/main into main: synchronize custom features, dashboards, and Vercel configuration
450bea85e Merge upstream changes: pull in latest releases and fixes while preserving custom UI, tools, and PWA modifications
5e5eb99f7 FREE MODE: Universal LiteLLM proxy for free/local AI inference (#52)
92baf580f Merge remote-tracking branch 'upstream/main' into upstream-sync
a398ca7d0 Preserve custom UI and tool modifications
7899311c3 Disable docs auto-deployment to prioritize UI app deployment
da8d468e2 Configure root .vercel project to deploy UI app
280d81042 Fix import path in Sandcastle status route
7c3aedd48 Sandcastle integration: API routes, components, and UI (#48)
ca3b46689 Claude/hermes rolodex a2a skrut (#49)
f80e51034 Add managed agent upgrade blocker report (#47)
f96cf98bd Claude/cool euler m ec2 k (#46)
91d097990 🔐 Add authentication and security hardening to Hermes API (#45)
cf457fd74 feat: add Next.js dashboard UI + FREE MODE toggle
3d790bb3b Claude/synthia gateway integration (#44)
ded81d6cf feat: wire Synthia Gateway for real AI inference + provider badge in UI (#43)
2292e23de feat: Vercel proxy, voice agent fixes, security hardening, and CR review fixes (#42)
02cde98a9 feat: deploy scripts, voice agent, Mercury, Hostinger MCP, NIM proxy
65d54abf5 feat: Hermes Rolodex™ + Vercel deploy + agent-skills (#40)
9036a5c13 feat: Lock Obsidian Vault path, integrate Graphify Semantic indexing, and implement premium React split-screen MCP Apps Canvas UI
cc6aeace0 Codex/create python module for repo ingestion (#39)
651d442b4 Claude/hermes rolodex a2a skrut (#38)
0100ad99b Add printing-press skill pack and workflow templates (#37)
651d442b4 Claude/hermes rolodex a2a skrut (#38)
08bd74c11 chore: add Vercel FastAPI entrypoint (#30)
143d83a67 chore: commit uncommitted files (#31)
97d766d23 Codex/add openchronicle repository (#32)
a84732b7d Codex/run full repo discovery and analysis (#33)
cc1386087 Codex/create python module for repo ingestion (#34)
8147c17ea Web UI customization (#35)
1bd7419fd feat(rolodex): add Hermes Rolodex MCP server, schema, tests, and skills (#36)
33b785c39 Add printing-press skill pack and workflow templates (#28)
36d14bfc6 [ZTE][ZTE-25578128504] ✅ claude/hermes-rolodex-a2a-SKRUT → main (#26)
278e79ff5 chore: commit uncommitted files
642171e9a chore: commit uncommitted files (#29)
c72268cbf Setup Hermes WebUI for Pauli Agent (#27)
7c74ddc0b docs: add OpenChronicle to Community links (#25)
```

## Upstream Changes: Missing from Local Fork
The following commits are present in the upstream `main` branch (NousResearch/hermes-agent) but **not** in the local `main` branch. These represent the latest features, fixes, and improvements from the official Hermes agent that we should consider integrating.

### Recent Upstream Commits (Non-Merge) - Missing Locally
```
3e21cfdeb fix(desktop): clear stale active todos on turn end AND on rehydration
e40175f06 fix(desktop): stop macOS Tahoe misplacing the traffic lights
66c3d595d feat(desktop): cap overlay inner-page width at 75rem
89acc1960 fix(dump): flag API keys visible only to the shell, not the managed backend
64ed99a6e fix(webhook): close per-delivery session at the true end of the run (#57423)
4aad27b75 fix(desktop): extend startup long-timeout to the whole boot data burst
584d3ae53 fix(desktop): extend profile startup REST timeouts (#48504)
4d88facfc fix(desktop): let settings content use full pane width
ed4123792 refactor(providers): dedupe extra_headers normalizer + key picker groups by headers
ab40e952f fix(providers): pass extra headers to model discovery
80a774f97 Merge pull request #57379 from kshitijk4poor/salvage/vllm-local-context
0950dae2f Merge remote-tracking branch 'upstream/main' into HEAD
201b646d6 fix(gateway): complete on_session_end coverage across all eviction paths
90b618f48 fix(gateway): keep idle cached agents alive until session actually expires
46ada7ed4 Merge pull request #57377 from kshitijk4poor/chore/author-map-31856
1c93799b4 fix(agent): self-review follow-ups on vLLM local-context salvage
e73adb504 fix(dashboard): disable ws keepalive ping on loopback to survive event-loop stalls
26edfab00 chore: add trismegistus-wanderer to AUTHOR_MAP for PR #31856 salvage
eb806c7f5 chore(release): add infinitycrew39 to AUTHOR_MAP (#56431 salvage)
b9a197ec5 fix(agent): resolve review findings on vLLM local-context salvage
65cb70b8d refactor(gateway): add SessionStore.peek_session_id public accessor for webhook close
de67f430b chore: map gumclaw@gumroad.com in AUTHOR_MAP for PR #57322 salvage
14882bab7 fix(gateway): close webhook sessions on delivery completion so prune can reap them
53063d92b test(agent): cover local vLLM context-length resolution
cecedcddf fix(agent): honor live vLLM context limits on local endpoints
048270fa0 fix: refresh NVIDIA featured models
9f6046742 refactor(slack): extract _is_list_line helper for list-marker checks
033d7bf25 fix(slack): guard blank-line list continuation on next-item lookahead
d3c8a155c fix(slack): keep blank-line-separated ordered items in one rich_text_list
3a122ba4a fix(usage): capture reasoning_tokens from completion_tokens_details on chat_completions (#57340)
ab942330f chore(release): map yingliang-zhang in AUTHOR_MAP for #57335
67472fbaa fix(tui_gateway): route setup.runtime_check and setup.status to RPC pool
1501a338c fix(cli): stop profile-bound backends before deleting so rmtree converges
5a6720b88 fix(desktop,tui-gateway,zai): stop thinking-off from reverting to medium
c3f06a8fd fix(desktop): refresh profile rail after deletion (#49289)
c5e8a60b0 fix(desktop): skip ensureBackend after profile-delete teardown to prevent respawn loop
254328bf5 fix(auth): remove stale loopback_pkce reference in xAI quarantine removal list
5ef0b8acb feat(auth): make xAI Grok OAuth device-code-only, drop loopback login
472d75193 Prevent deleted profile skeleton revival
6cffc37b5 feat(desktop): collapse profile rail to a select past 13 profiles (#57306)
a2d49de80 fix(terminal): also set MSYS2_ARG_CONV_EXCL for MSYS2/Cygwin bash fallback
51c01062d test(terminal): cover MSYS_NO_PATHCONV defaults on Windows env builders
cc2abd570 fix(terminal): set MSYS_NO_PATHCONV for Windows Git Bash subprocesses
a9b559890 fix(desktop): load remote model options before session
fe82b3a77 fix(desktop): read attachment previews local-first in remote mode
c19bfb50a fix(desktop): restore remote file picker attachments
eb506e656 Merge pull request #57267 from NousResearch/bb/desktop-journey-memory-graph
8da0a56ba style(desktop): fix pre-existing import-order lint in use-prompt-actions
931e2356a feat(desktop): /journey opens the memory graph overlay instead of printing text
42ca43813 style(desktop): fix import ordering + padding lint in remote-artifact files
03406ae25 fix(desktop): restore remote artifact rendering
973887048 fix(desktop): call checkUpdates() in startUpdatePoller so version pill auto-populates
63354edfd Merge pull request #57226 from NousResearch/bb/desktop-multiline-slash
fb44b519d fix(desktop): parse multiline slash commands + hand degenerate payloads back
30e947e0a feat(gateway): persist per-session /model overrides across gateway restarts
b98baa303 feat(config): extra HTTP headers for LLM API calls (#3526 salvage)
4a09b692e feat(api-server): per-client model routing via model_routes (#3176 salvage)
ce9aa869f feat(commands): /compact alias + --preview/--dry-run flags for /compress (#3243 salvage)
fb74ddf7f fix(i18n): add gateway.verbose.mode_log to all locale catalogs
39bff6795 feat(gateway): add 'log' option to display.tool_progress
070ac2a71 fix(status): label provider as custom when config.yaml model.base_url is set
44650a5ce chore: add AUTHOR_MAP entry for @ajmeese7 (#3219 salvage)
c0d694a49 fix(whatsapp): resolve LID sender IDs to phone numbers in bridge message payload
019950560 refactor(image-gen): reuse shared image sniffer + raster allowlist in codex backend
460235d58 test(image-gen): cap Codex reference inputs
ecffd290a feat(image-gen): support Codex image inputs
0a2d4a6ee docs(codex): clarify stale-floor docstring reflects the 10k gate
ede4d1256 test(codex): cover gateway-scale stale timeout floor and TTFB gate
cb1ccc57e fix(codex): extend stale timeout for gateway-scale tool payloads
d733eaa65 Merge pull request #57007 from kshitijk4poor/chore/author-map-crazyboym-55828
be21e06ab chore(release): map ai-lab@foxmail.com to CrazyBoyM
3f2a56d1a fix(cli): reliable interrupts, bounded exit, and exit feedback (#57000)
2068754d6 feat(api-server): inline MEDIA: image tags as base64 data URLs for remote frontends
88bd1c01e fix(email): harden adapter against malformed IMAP responses
c43aa6301 feat(gateway): per-channel model and system prompt overrides (Fixes #1955) - ChannelOverride + channel_overrides; session /model > channel > global - Thread/parent lookup; YAML bridge for discord.channel_overrides - Guard channel_overrides when config lacks platforms (test mocks) - Add sampiyonyus@gmail.com to AUTHOR_MAP
0010c14e6 feat(gateway): per-channel model and system prompt overrides (Fixes #1955)
ebef73f6b feat(gateway): per-channel model and system prompt overrides (Fixes #1955)
902b0b70e test: env-flag 'on' truthy behavior contract (#2863 follow-up)
- 60039d5a3 fix(config): accept 'on' as truthy for env flags via shared env_var_enabled helper
- ac3d8c9d1 Add interactive-artifact-skill and vendor skills
```

Note: The above list is a subset of the full upstream diff. The full diff shows 4316 files changed with 498,512 insertions and 426,994 deletions, indicating significant upstream development.

## Merge Strategy: Safely Integrating Upstream Changes
To integrate the upstream changes without losing local custom work, follow this recommended process:

### 1. Preparation
- Ensure the local repository is clean: `git status` should show no uncommitted changes (or stash them).
- Create a new branch for the merge: `git checkout -b integrate-upstream-main`

### 2. Fetch Latest Upstream
```bash
git fetch upstream --prune --tags
```

### 3. Perform the Merge
```bash
git merge upstream/main --no-ff -m "Merge upstream/main: integrate latest Hermes features while preserving Pauli customizations"
```
- **Note**: Use `--no-ff` to create a merge commit that clearly marks the integration point.

### 4. Resolve Conflicts
- Expect conflicts in files that have been modified both locally and upstream.
- Prioritize preserving local customizations in:
  - `agent/`, `api_server.py`, `hermes_bootstrap.py` (Pauli-specific agent logic)
  - `web/` directory (custom Next.js dashboard UI)
  - `pauli/` directory (Pauli-specific extensions)
  - `skills/` directory (custom skills like rolodex, printing-press, etc.)
  - `config/` and `.vercel/` (Pauli-specific deployment configurations)
- Use `git mergetool` or manually edit files to resolve conflicts.
- After resolving, mark as resolved with `git add <file>` and continue.

### 5. Post-Merge Verification
- Run the test suite: `python -m pytest` (if available)
- Start the agent locally: `hermes dev` or `python -m agent`
- Verify that custom features (Rolodex, Pauli UI, Free Mode, etc.) still function.
- Check that the Hermes Desktop can connect to the local agent.

### 6. Finalize
- Once verified, switch back to `main` and merge the integration branch:
  ```bash
  git checkout main
  git merge integrate-upstream-main
  ```
- Push the updated `main` branch: `git push origin main`

## Hermes Desktop Installation and Connection Guide
To connect the Hermes Desktop application (from https://hermes-ai.net/desktop/) to this local agent instance:

### 1. Install Hermes Desktop
- Download the latest Hermes Desktop installer from: https://hermes-ai.net/desktop/
- Run the installer and follow the prompts.
- Note: Hermes Desktop requires a running Hermes agent instance to connect to.

### 2. Start the Local Hermes Agent
In the `pauli-hermes-agent` directory:
```bash
# Ensure you are in the project root
cd C:\Users\execu\pauli-hermes-agent

# Start the agent in development mode (or production)
hermes dev
# Alternatively, for a more persistent setup:
# hermes start
```
- The agent will start and expose a local API (typically on `http://127.0.0.1:8765` or similar).

### 3. Configure Hermes Desktop
- Launch Hermes Desktop.
- In the connection settings, select "Custom Endpoint" or "Local Agent".
- Enter the local agent's URL (e.g., `http://127.0.0.1:8765`).
- If authentication is enabled (as per our custom auth hardening), provide the required credentials.
- Save the connection and attempt to connect.

### 4. Verify Connection
- Once connected, you should see the Hermes Desktop interface communicating with your local Pauli-Hermes-Agent instance.
- Test basic functionality: send a message, check the agent's response, verify custom skills are accessible.

### 5. Troubleshooting
- If connection fails, ensure the agent is running and accessible at the specified URL.
- Check the agent's logs for connection errors.
- Verify that the agent's API is not blocked by a firewall.

## Action Items
1. [ ] Review the full upstream diff (`git diff HEAD..upstream/main`) to identify critical changes.
2. [ ] Create an integration branch and attempt the merge as outlined above.
3. [ ] Resolve any merge conflicts, prioritizing preservation of Pauli-specific customizations.
4. [ ] Test the merged agent locally to ensure all custom features work.
5. [ ] Install Hermes Desktop and connect it to the local agent.
6. [ ] Document any additional configuration steps required for the desktop connection.
7. [ ] Merge the integration branch into `main` and push to origin.
8. [ ] Notify the team of the successful integration and provide updated connection instructions.

## Notes
- This handoff is intended to be a planning document. The actual merge should be performed by a developer familiar with both the upstream and local codebases.
- Always back up the current state before attempting a major merge (e.g., create a backup branch: `git backup-pre-merge`).
- The Hermes Desktop application is regularly updated; ensure you are using the latest version for compatibility.

---
*Generated on: 2026-07-02*
*Based on repository: C:\Users\execu\pauli-hermes-agent*
*Upstream: https://github.com/NousResearch/hermes-agent*
*Local branch: main (c2e286a7e)*
*Upstream branch: upstream/main (3e21cfdeb)*

---

## ⚠️ NOTES FOR NEXT AGENT

### Composio Integration (2026-07-04)
- Installed `@composio/core`, `@composio/vercel`, `ai`, `@ai-sdk/anthropic`
- Created `agent.ts` template for Vercel AI SDK + Composio integration
- Added `COMPOSIO_API_KEY` to `.env` - GET THE ACTUAL KEY FROM: https://dashboard.composio.dev/executiveusa/HERMES/settings/api-keys
- TODO: Wire up Composio tools, test the agent, configure all integrations

### Agent Mail AI
- TODO: Test and wire Agent Mail AI configurations
- Check for any existing Agent Mail configurations in the codebase

### Testing Required
- [ ] Test Composio agent.ts with real API key
- [ ] Test Agent Mail AI integration
- [ ] Verify all configs are wired correctly
- [ ] Run hermes dev and verify everything works