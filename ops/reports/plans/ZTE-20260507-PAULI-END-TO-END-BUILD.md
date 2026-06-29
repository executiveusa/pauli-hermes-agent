# Pauli Hermes End-to-End Build Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a deploy-ready Pauli Hermes meta-agent that can ingest client repos and files, map them into a knowledge graph, generate onboarding courses, watch videos, automate browsers, persist long-term memory, and expose a clear control-room UI with safe deployment workflows.

**Architecture:** Keep upstream Hermes core as the runtime shell and isolate Pauli features into `skills/pauli`, `scripts/pauli`, `pauli/`, MCP adapters, and optional services. Use a routed skill system plus external service adapters so Graphify, Ralphy/flywheel, Browser Harness, video ingestion, course generation, and memory persistence remain composable and testable without polluting Hermes core.

**Tech Stack:** Hermes Agent, Python 3.13, uv, Node.js/npm, Hermes Desktop, OpenRouter, Browser Harness, Graphify, Ralphy, jcodemunch-mcp, Supabase/pgvector, MCP adapters, Docker/Coolify/Hostinger, optional Rust microservices.

---

## File Structure

- Modify: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\cli.py`
  Purpose: hook routed Pauli skills and profile-aware startup behavior into Hermes CLI entry.
- Modify: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\tools\skills_tool.py`
  Purpose: allow repo-local Pauli skill loading by absolute path and support richer local skill packaging.
- Modify: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\config\pauli_skill_router.yaml`
  Purpose: task-routing policy, budgets, and trigger rules for Pauli mode selection.
- Modify: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\config\pauli_profiles.yaml`
  Purpose: declarative profile roles for orchestrator, engineer, studio, memory, and client-agent modes.
- Modify: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\docs\analysis\*.md`
  Purpose: keep current-state, module inventories, blockers, and integration decisions current.
- Modify: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\docs\runbooks\*.md`
  Purpose: operator runbooks for secrets, deploys, browser workflows, Graphify, client-agent creation, memory, and rollback.
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\agent\pauli_skill_router.py`
  Purpose: route tasks to bounded skill sets and resolve repo-local custom skills.
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\agent\pauli_module_registry.py`
  Purpose: canonical registry of optional Pauli modules, required env vars, health checks, and install status.
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\agent\pauli_openrouter_models.py`
  Purpose: enumerate validated OpenRouter free/cheap models and fallback chains.
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\graphify\`
  Purpose: Graphify wrappers, secure path validation, job metadata, and graph export helpers.
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\coursegen\`
  Purpose: codebase-to-course adapter, course templates, quiz generation, and diagram rendering helpers.
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\flywheel\`
  Purpose: Ralphy/flywheel orchestration, bead decomposition, sub-agent mail integration, and ACIP guardrails.
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\video\`
  Purpose: claude-video wrappers, YouTube metadata ingestion, transcript extraction, and graph persistence.
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\browser\`
  Purpose: Browser Harness wrappers, CDP profile config, and domain-skill integration.
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\memory\`
  Purpose: Supabase/pgvector adapters, Obsidian/Notion/Google connectors, and Jcode Munch context maps.
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\agent_creator\`
  Purpose: end-to-end client-agent creation flow, profile bootstrap, and client workspace generation.
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\rust-services\`
  Purpose: Rust-backed scan/watch services with gRPC interfaces for heavy ingestion jobs.
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\skills\pauli\graphify\SKILL.md`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\skills\pauli\course-generator\SKILL.md`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\skills\pauli\flywheel-orchestrator\SKILL.md`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\skills\pauli\agent-creator\SKILL.md`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\skills\pauli\browser-ops\SKILL.md`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\skills\pauli\video-watch\SKILL.md`
  Purpose: real Pauli skill contracts with triggers, gates, workflows, and tests.
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\scripts\pauli\`
  Purpose: install, sync, healthcheck, deploy, graphify, course, browser, and agent-creation commands.
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\tests\agent\test_pauli_*.py`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\tests\tools\test_pauli_*.py`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\tests\integration\test_pauli_*.py`
  Purpose: unit, contract, and end-to-end tests for routing, adapters, services, and workflows.
- Modify: `E:\ACTIVE PROJECTS-PIPELINE\hermes-desktop\src\**\*`
  Purpose: Studio Control Room, Flywheel UI, Graph/Course/Video views, memory health, and browser/deploy controls.

## Workstreams

### Workstream A: Core Hermes Stabilization

**Files:**
- Modify: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\cli.py`
- Modify: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\tools\skills_tool.py`
- Modify: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\agent\prompt_builder.py`
- Modify: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\docs\analysis\current_state.md`
- Test: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\tests\agent\test_pauli_skill_router.py`

- [ ] **Step 1: Refresh current baseline against upstream Hermes and local blockers**

Run:

```powershell
git -C "E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main" fetch origin --tags --prune
git -C "E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main" fetch upstream --tags --prune
git -C "E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main" status --short
git -C "E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main" rev-list --left-right --count origin/main...upstream/main
```

Expected: exact ahead/behind numbers and unchanged dirty state inventory.

- [ ] **Step 2: Re-run router and absolute-skill tests until green**

Run:

```powershell
& "E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\.venv\Scripts\pytest.exe" -o addopts= "E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\tests\agent\test_pauli_skill_router.py" "E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\tests\tools\test_skill_view_absolute_path.py" -q
```

Expected: PASS for current Pauli routing baseline.

- [ ] **Step 3: Validate Hermes API and provider chain**

Run:

```powershell
Invoke-WebRequest -UseBasicParsing http://127.0.0.1:8642/health
```

Then run a redacted chat smoke test via a local script that does not print secrets and records only pass/fail.

Expected: API health passes, provider auth is either fixed or captured as a precise credential blocker.

- [ ] **Step 4: Commit core stabilization fixes**

```bash
git add cli.py tools/skills_tool.py agent/*.py config/*.yaml docs/analysis/current_state.md tests/agent/test_pauli_skill_router.py tests/tools/test_skill_view_absolute_path.py
git commit -m "feat: stabilize pauli skill routing baseline"
```

### Workstream B: Secret, Model, and Budget Layer

**Files:**
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\agent\pauli_openrouter_models.py`
- Modify: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\scripts\pauli\sync-env-to-hermes.ps1`
- Modify: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\skills\pauli\infisical-secrets\SKILL.md`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\docs\runbooks\openrouter_free_model_matrix.md`
- Test: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\tests\agent\test_pauli_openrouter_models.py`

- [ ] **Step 1: Build validated OpenRouter model discovery**

Implement a module that fetches the OpenRouter model list with redacted auth handling and filters:
- free models
- cheap models
- tool-capable models
- coding-capable models

Expected output artifact: cached redacted model matrix in docs or runtime cache, not raw secret logs.

- [ ] **Step 2: Define fallback model chains**

Required chains:
- default bootstrap chain
- coding chain
- browser/vision chain
- memory summarization chain

Expected: free-first route, paid fallback only when configured and approved.

- [ ] **Step 3: Extend secrets sync and secret health**

Add explicit redacted checks for:
- `OPENROUTER_API_KEY`
- `GITHUB_TOKEN`
- `COOLIFY_BASE_URL`
- `COOLIFY_API_KEY`
- `SUPABASE_URL`
- `SUPABASE_ACCESS_TOKEN`
- `TWILIO_*`
- `YOUTUBE_*`

Expected: secret health page and runbook remain value-redacted.

- [ ] **Step 4: Commit secret/model layer**

```bash
git add agent/pauli_openrouter_models.py scripts/pauli skills/pauli/infisical-secrets/SKILL.md docs/runbooks/openrouter_free_model_matrix.md tests/agent/test_pauli_openrouter_models.py
git commit -m "feat: add pauli openrouter model routing and secret health"
```

### Workstream C: Graphify Integration

**Files:**
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\graphify\adapter.py`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\graphify\paths.py`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\graphify\reports.py`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\scripts\pauli\graphify-build.ps1`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\scripts\pauli\graphify-query.ps1`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\scripts\pauli\graphify-export.ps1`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\skills\pauli\graphify\SKILL.md`
- Test: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\tests\integration\test_pauli_graphify.py`

- [ ] **Step 1: Clone or vendor Graphify into `vendor-repos/graphify`**

Store it outside Hermes core. Record commit SHA and license in analysis docs.

- [ ] **Step 2: Implement safe path handling**

All Graphify entrypoints must reject:
- missing paths
- paths outside allowed workspaces unless explicitly named
- destructive overwrite targets

- [ ] **Step 3: Wrap core Graphify commands**

Required wrappers:
- `graphify build`
- `graphify query`
- `graphify path`
- `graphify export`

Required outputs per job:
- `graph.html`
- `graph.json`
- `GRAPH_REPORT.md`

- [ ] **Step 4: Add multimodal ingestion routing**

Handle code, markdown/docs, images, and video metadata. For videos, store extracted transcript/frame references rather than raw huge blobs in prompt context.

- [ ] **Step 5: Commit Graphify integration**

```bash
git add pauli/graphify scripts/pauli/graphify-*.ps1 skills/pauli/graphify/SKILL.md tests/integration/test_pauli_graphify.py docs/analysis/repo_inventory.json
git commit -m "feat: integrate graphify knowledge graph wrappers"
```

### Workstream D: Codebase-to-Course

**Files:**
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\coursegen\adapter.py`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\coursegen\templates\`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\scripts\pauli\generate-course.ps1`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\skills\pauli\course-generator\SKILL.md`
- Test: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\tests\integration\test_pauli_coursegen.py`

- [ ] **Step 1: Clone `zarazhangrui/codebase-to-course` into `vendor-repos/codebase-to-course`**

Record the upstream contract and convert it into a Pauli adapter instead of direct vendoring into core.

- [ ] **Step 2: Define a two-phase course pipeline**

Phase 1:
- analyze actors
- user journeys
- APIs
- data flows

Phase 2:
- generate 4 to 6 modules
- plain-English copy
- animated/system diagrams
- quiz prompts

- [ ] **Step 3: Make the output single-page and self-contained**

Required artifacts:
- course HTML
- course metadata JSON
- source graph references

- [ ] **Step 4: Commit course generation layer**

```bash
git add pauli/coursegen scripts/pauli/generate-course.ps1 skills/pauli/course-generator/SKILL.md tests/integration/test_pauli_coursegen.py
git commit -m "feat: add graphify-to-course generation pipeline"
```

### Workstream E: Flywheel, Ralphy, and Clawdbot Ops

**Files:**
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\flywheel\adapter.py`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\flywheel\beads.py`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\skills\pauli\flywheel-orchestrator\SKILL.md`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\docs\runbooks\flywheel_ops.md`
- Test: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\tests\integration\test_pauli_flywheel.py`

- [ ] **Step 1: Clone `michaelshimeles/ralphy` and `Dicklesworthstone/agentic_coding_flywheel_setup` into `vendor-repos/`**

Record:
- install method
- OS assumptions
- reusable workflows
- parts that are Linux/VPS-only

- [ ] **Step 2: Define what gets imported into Pauli**

Use:
- Ralphy as task/PRD execution loop
- Flywheel concepts for planning, beads, swarm coordination, mail, and review

Do not directly import VPS bootstrap assumptions into Windows local runtime.

- [ ] **Step 3: Add ACIP-style safety mapping**

Translate destructive command guard, prompt-injection hardening, and repo boundaries into Pauli skill/router policy and shell wrappers.

- [ ] **Step 4: Build flywheel orchestration wrappers**

Required commands:
- initialize workstream
- split into beads
- launch sub-agents
- collect reports
- update task board

- [ ] **Step 5: Commit flywheel layer**

```bash
git add pauli/flywheel skills/pauli/flywheel-orchestrator/SKILL.md docs/runbooks/flywheel_ops.md tests/integration/test_pauli_flywheel.py
git commit -m "feat: integrate ralphy and flywheel orchestration"
```

### Workstream F: Browser Harness and Dynamic Computer Use

**Files:**
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\browser\adapter.py`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\browser\profiles.py`
- Modify: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\skills\pauli\browser-ops\SKILL.md`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\tests\integration\test_pauli_browser_ops.py`

- [ ] **Step 1: Install and verify Browser Harness locally**

Required health checks:
- binary present on PATH
- local Chrome/CDP connectable
- remote mode gated behind `BROWSER_USE_API_KEY`

- [ ] **Step 2: Add Pauli browser wrapper commands**

Required operations:
- navigate
- click
- extract text
- capture screenshot
- save domain-skill notes

- [ ] **Step 3: Define autonomous-use policy**

Allowed:
- non-destructive dashboard reads
- content retrieval
- screenshot-based verification

Approval-gated:
- billing changes
- DNS/service deletes
- irreversible client admin actions

- [ ] **Step 4: Commit browser layer**

```bash
git add pauli/browser skills/pauli/browser-ops/SKILL.md tests/integration/test_pauli_browser_ops.py
git commit -m "feat: add browser harness adapter and autonomy policy"
```

### Workstream G: Video Ingestion and YouTube Metadata

**Files:**
- Modify: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\skills\pauli\video-watch\SKILL.md`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\video\adapter.py`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\video\youtube.py`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\scripts\pauli\video-watch.ps1`
- Test: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\tests\integration\test_pauli_video_watch.py`

- [ ] **Step 1: Clone `bradautomates/claude-video` into `vendor-repos/claude-video`**

Record setup and OS dependencies:
- `yt-dlp`
- `ffmpeg`
- optional Whisper path

- [ ] **Step 2: Build `/watch`-style wrappers**

Required stages:
- download or attach media
- extract native captions if present
- fallback transcript path
- frame extraction
- metadata persistence

- [ ] **Step 3: Add YouTube Data API integration**

Required outputs:
- list channel videos
- retrieve title/description/timestamps/metadata
- persist video index into graph + memory

- [ ] **Step 4: Commit video layer**

```bash
git add pauli/video scripts/pauli/video-watch.ps1 skills/pauli/video-watch/SKILL.md tests/integration/test_pauli_video_watch.py
git commit -m "feat: integrate video ingestion and youtube metadata"
```

### Workstream H: Memory, MCP, and Context Compression

**Files:**
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\memory\adapter.py`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\memory\supabase.py`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\mcp\registry.py`
- Modify: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\skills\pauli\jcodemunch\SKILL.md`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\docs\skills\pauli-skill-index.md`
- Test: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\tests\integration\test_pauli_memory.py`

- [ ] **Step 1: Integrate `jcodemunch-mcp`, `mcp2cli`, `supabase-mcp`, and `ext-apps` as external adapters**

Record which are:
- immediate install
- env-gated
- future adapter only

- [ ] **Step 2: Build long-term memory store**

Required behavior:
- Graphify graph metadata persisted
- course outputs indexed
- video transcripts indexed
- retrieval returns snippets, not corpus dumps

- [ ] **Step 3: Add workspace/notes connectors**

Priority order:
- Supabase/pgvector primary
- Obsidian local optional
- Notion/Google Workspace connector optional and env-gated

- [ ] **Step 4: Commit memory/MCP layer**

```bash
git add pauli/memory pauli/mcp docs/skills/pauli-skill-index.md tests/integration/test_pauli_memory.py
git commit -m "feat: add pauli memory store and mcp adapters"
```

### Workstream I: Client Agent Creator

**Files:**
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\agent_creator\flow.py`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\scripts\pauli\create-client-agent.ps1`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\skills\pauli\agent-creator\SKILL.md`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\docs\runbooks\client_agent_creation.md`
- Test: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\tests\integration\test_pauli_agent_creator.py`

- [ ] **Step 1: Define accepted client inputs**

Support:
- repos
- exported chats
- documents/files
- spreadsheets
- videos

- [ ] **Step 2: Chain core modules**

Workflow:
- Graphify ingest
- memory index
- course generation
- flywheel planning
- profile/container bootstrap

- [ ] **Step 3: Generate client output bundle**

Required artifacts:
- Hermes profile or container config
- runbooks
- graph outputs
- course outputs
- task/workflow definitions

- [ ] **Step 4: Commit client-agent layer**

```bash
git add pauli/agent_creator scripts/pauli/create-client-agent.ps1 skills/pauli/agent-creator/SKILL.md docs/runbooks/client_agent_creation.md tests/integration/test_pauli_agent_creator.py
git commit -m "feat: add client agent creation workflow"
```

### Workstream J: Rust Services

**Files:**
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\rust-services\Cargo.toml`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\rust-services\src\main.rs`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\pauli\rust-services\proto\*.proto`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\tests\integration\test_pauli_rust_services.py`

- [ ] **Step 1: Identify heavy Python workloads worth rewriting**

Initial candidates:
- large repo scanning
- graph ingestion
- file watching

- [ ] **Step 2: Build a minimal gRPC scan service**

Required endpoints:
- scan repository
- watch repository
- job status

- [ ] **Step 3: Keep Python orchestration thin**

Hermes should call Rust services via typed clients and persist job metadata locally.

- [ ] **Step 4: Commit Rust service scaffold**

```bash
git add pauli/rust-services tests/integration/test_pauli_rust_services.py
git commit -m "feat: add rust-backed ingestion service scaffold"
```

### Workstream K: Hermes Desktop and Dashboard UX

**Files:**
- Modify: `E:\ACTIVE PROJECTS-PIPELINE\hermes-desktop\src\**\*`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\hermes-desktop\src\features\studio-control-room\**\*`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\hermes-desktop\src\features\flywheel\**\*`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\hermes-desktop\src\features\graph\**\*`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\hermes-desktop\src\features\course\**\*`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\hermes-desktop\src\features\video\**\*`
- Test: `E:\ACTIVE PROJECTS-PIPELINE\hermes-desktop\tests\**\*`

- [ ] **Step 1: Fix desktop dependency/install state first**

Reproduce and resolve the incomplete Windows install, or move dev verification to WSL/Linux if the project requires it.

- [ ] **Step 2: Add Studio Control Room**

Required cards:
- agent status
- Graphify status
- course generation status
- video ingestion status
- flywheel progress
- memory usage
- deploy status

- [ ] **Step 3: Add flywheel and agent-mail controls**

Required views:
- bead list
- task progress
- mail/notifications
- launch controls

- [ ] **Step 4: Add self-explanatory action buttons**

Required actions:
- start Graphify scan
- generate course
- ingest video
- create client agent
- sync secrets
- run healthcheck

- [ ] **Step 5: Commit UI layer**

```bash
git add src tests
git commit -m "feat: add pauli studio control room and flywheel dashboards"
```

### Workstream L: Deployment, Voice, and Recovery

**Files:**
- Modify: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\scripts\pauli\deploy-coolify.ps1`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\docker-compose.pauli.yml`
- Create: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\Dockerfile.pauli`
- Modify: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\docs\runbooks\hostinger_coolify_deploy.md`
- Modify: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\docs\runbooks\voice_twilio.md`
- Test: `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\tests\integration\test_pauli_deploy.py`

- [ ] **Step 1: Define deployable services**

Required deploy units:
- Hermes API/runtime
- Graphify service
- Rust services
- optional desktop/web companion

- [ ] **Step 2: Add health checks and secret injection points**

Support:
- Coolify envs
- Infisical source-of-truth when credentials exist

- [ ] **Step 3: Add Vercel and Twilio adapters**

Required behavior:
- Vercel inspection/repair lane
- Twilio voice health and allowlisted call path

- [ ] **Step 4: Commit deploy layer**

```bash
git add Dockerfile.pauli docker-compose.pauli.yml scripts/pauli docs/runbooks tests/integration/test_pauli_deploy.py
git commit -m "feat: add pauli deployment and recovery stack"
```

## Acceptance Gates

- Hermes API health passes and authenticated chat succeeds with a validated provider.
- Pauli router tests pass.
- Graphify wrappers generate `graph.html`, `graph.json`, and `GRAPH_REPORT.md`.
- Codebase-to-course outputs a usable single-page course from a sample repo.
- Flywheel orchestration can create beads, dispatch worker tasks, and collect results.
- Browser Harness is callable and can complete a safe dashboard read workflow.
- Video watch path extracts transcript/frame metadata without paid calls by default.
- Memory store can persist and retrieve graph/course/video artifacts.
- Client agent creator can produce a profile/config bundle from a sample client repo.
- Hermes Desktop builds and renders the Studio Control Room.
- Deployment artifacts validate for staging.

## Blockers To Resolve Early

- Provider auth is still failing for real Hermes chat completions.
- Hermes Desktop install/build state is still unstable on this Windows host.
- Coolify/Hostinger credentials are incomplete.
- WSL distro and Docker are not yet available locally.
- Some upstream tools are Linux-first and may need WSL or remote staging for full verification.

## Self-Review

- Spec coverage: this plan includes Hermes stabilization, OpenRouter free-model routing, Graphify, codebase-to-course, Ralphy/flywheel, Browser Harness, video ingestion, memory, Rust services, client-agent creation, UI, and deployment.
- Placeholder scan: remaining unknowns are listed as blockers, not fake implementation steps.
- Type consistency: all major workstreams point to explicit folders/files and test targets.

## Execution Handoff

Plan complete and saved to `E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\ops\reports\plans\ZTE-20260507-PAULI-END-TO-END-BUILD.md`.

Two execution options:

1. Subagent-Driven (recommended) - dispatch a fresh subagent per workstream/task and review between tasks.
2. Inline Execution - execute tasks in this session with checkpoints and staged verification.
