---
name: project-review-build-trajectory
description: Analyze a project repo, summarize what happened, classify the project intent, review product and front-end state, recommend the best build trajectory, ask what to build next, and prepare repo/image/front-end execution plans.
version: 0.1.0
author: executiveusa
license: MIT
metadata:
  hermes:
    tags: [operator, repo-review, project-audit, trajectory, ui-uplift, assets, github]
    category: operator
    related_skills: [opencli-rs, web-research]
---

# project-review-build-trajectory

This skill is the **project operator workflow** for Hermes.

Use it when the user wants Hermes to enter an existing project, understand what happened, classify what kind of thing it is, review the project from a systems perspective, recommend the best next trajectory, and then ask what to build next with concrete options.

It is designed for projects that already have GitHub repos and is intended to support both:

- **single-project review and next-step planning**
- **portfolio-wide front-end and asset upgrade workflows**

---

## Core Outcome

For any project repo, this skill should produce:

1. **Project state summary**
   - what changed
   - what exists now
   - what the project appears to be
   - current execution state

2. **Intent classification**
   - tool
   - website
   - blog/content system
   - workflow/automation
   - internal utility
   - agent/system component
   - hybrid

3. **Structured review**
   - product review
   - UX/front-end review
   - architecture review
   - repo health review
   - automation opportunities
   - monetization / leverage opportunities

4. **Build trajectory recommendation**
   - highest-probability next move
   - fastest path to usable value
   - best path to scalable value
   - risks and blockers
   - what to defer

5. **User decision prompt**
   - asks what to build next, but only after generating concrete informed options

6. **Execution hooks**
   - mass front-end update plan
   - image generation plan
   - asset insertion plan
   - repo change plan
   - branch / commit / PR workflow

---

## When to Use

Trigger this skill for requests like:

- review this repo
- summarize what happened in this project
- tell me what this product is
- what should we build next here
- audit this repo and recommend the next trajectory
- update the front end for this project
- create assets and wire them into the UI
- run this process across multiple repos

---

## Inputs

### Required
- `project_name`
- `repo_url` or `repo_full_name`

### Optional
- `project_goal`
- `target_user`
- `priority_mode` (`speed`, `quality`, `monetization`, `launch`, `repair`, `scale`)
- `front_end_only` (boolean)
- `include_image_generation` (boolean)
- `design_direction`
- `time_horizon` (`today`, `this week`, `this month`, `quarter`)

If the user leaves some fields implicit, infer the highest-probability interpretation and proceed.

---

## Workflow Phases

## Phase 1 — Repo Intake

Pull the project into a structured intake model.

### Capture
- repo metadata
- README and docs
- package manifests
- framework/runtime
- folder structure
- design system clues
- deployment clues
- image/asset directories
- existing UI components
- routes/pages
- recent commits / PRs / issues if accessible

### Output
- `PROJECT_IDENTITY`
- `TECH_STACK`
- `PROJECT_SURFACE_MAP`
- `CHANGE_CONTEXT`

### Guidance
Use GitHub-connected tooling first when the repo is available. Prefer reading:
- README
- package.json / pyproject.toml / Cargo.toml / similar manifests
- main app directories
- component directories
- routes/pages/app directory
- styles/theme/tokens/assets directories

---

## Phase 2 — What Happened Summary

Summarize what happened in plain operational language.

### Questions to answer
- What is this project?
- What was likely being built?
- What changed recently?
- Is the project active, stalled, broken, or partially complete?
- What are the strongest visible accomplishments?
- What is unfinished or incoherent?

### Output format
#### What this project is
A concise paragraph.

#### What happened recently
- notable additions
- edits
- removals
- unfinished work
- likely intention behind recent changes

#### Current status
Choose one:
- concept
- prototype
- working MVP
- production candidate
- production
- broken / drifted
- abandoned / unclear

---

## Phase 3 — Intent Classification

Classify the actual user intent behind the project.

### Primary classes
- Tool
- Website
- Blog / publishing system
- Workflow / automation
- Internal operating system
- Brand / landing page
- SaaS app
- AI agent / orchestrator
- Asset / media generator
- Hybrid

### Secondary traits
- informational
- transactional
- operational
- monetizable
- internal-facing
- customer-facing
- content-heavy
- image-heavy
- UI-heavy
- automation-heavy

### Output
#### Intent classification
- primary type
- secondary traits
- confidence
- why this classification fits

---

## Phase 4 — Full Review

Review the project from a systems perspective.

### 4A. Product Review
Evaluate:
- clarity of value proposition
- coherence of user journey
- clarity of CTA / next step
- likely user confusion
- monetization path

### 4B. Front-End Review
Evaluate:
- visual quality
- consistency
- responsiveness clues
- image quality
- component consistency
- missing polish
- opportunities for batch UI uplift

### 4C. Repo / Architecture Review
Evaluate:
- maintainability
- repeated patterns
- dead zones
- missing abstractions
- likely technical debt
- where automation can reduce ops

### 4D. Operating Review
Evaluate:
- what can be templatized
- what can be turned into a repeatable workflow
- what can become skills or automations

### Output
Produce:
- strengths
- weaknesses
- immediate fixes
- high-leverage improvements

---

## Phase 5 — Build Trajectory Engine

Recommend the best next trajectory.

### Decision frame
Choose the best path based on:
- current repo state
- likely user intent
- complexity vs payoff
- launchability
- leverage
- maintainability

### Trajectory types
- repair and stabilize
- package and launch
- redesign front end
- add monetization layer
- automate operations
- productize internal tool
- create content engine
- convert into multi-project template

### Output
#### Best trajectory now
One primary recommendation.

#### Why this is best
- reason 1
- reason 2
- reason 3

#### 3-path comparison
1. fastest path
2. best long-term path
3. highest upside path

#### Recommended next sprint
A small concrete plan.

---

## Phase 6 — User Build Prompt

After analysis, ask the user what to build next using informed options.

### Required behavior
Do **not** ask only a vague open-ended question.

Instead produce:

#### Suggested next build options
- Option A: stabilize and ship
- Option B: redesign UI and images
- Option C: add automation/workflows
- Option D: add monetization layer
- Option E: convert into reusable template/system

Then ask:

**What do you want to build from here?**

with this structure:
- choose one option
- combine options
- or specify a custom direction

---

## Phase 7 — Mass Front-End Update Mode

For projects with existing repos, support batch UI uplift.

### Scope
- hero sections
- typography
- spacing
- colors
- cards
- buttons
- iconography
- page shells
- dashboards
- navigation
- reusable components

### Batch workflow
1. detect framework and component structure
2. identify repeated visual primitives
3. define uplift rules
4. generate patch plan
5. apply updates per repo or across many repos
6. validate visually
7. commit in controlled batches

### Output artifacts
- `UI_UPLIFT_SPEC.md`
- `COMPONENT_REFACTOR_PLAN.md`
- `BATCH_CHANGESET_PLAN.md`

---

## Phase 8 — Image Generation and Asset Injection

Support generating and applying images directly from the workflow.

### Supported asset types
- hero images
- product mockups
- illustrations
- backgrounds
- blog cover images
- icons
- campaign graphics
- social cards
- avatars / branded characters

### Asset workflow
1. infer image needs from project review
2. define asset list
3. generate prompts by page/component/use case
4. generate images
5. map each image to target UI location
6. add assets to repo structure
7. update front-end references
8. create commit / PR

### Output artifacts
- `ASSET_PLAN.md`
- `IMAGE_PROMPTS.json`
- `ASSET_MAPPING.json`
- `UI_IMAGE_PATCH_PLAN.md`

---

## Phase 9 — Repo Execution Layer

This workflow can move from analysis into action.

### Execution modes
- review only
- plan only
- patch repo
- create branch
- commit changes
- open PR
- run batch mode across many repos

### Safe defaults
- default to branch-based changes for multi-file edits
- require confirmation before destructive or broad updates
- generate preview summary before commit

---

## Standard Final Output Template

## 1. Project Summary
Short description of what the project is and what happened.

## 2. Intent Classification
Primary type, secondary traits, confidence, rationale.

## 3. Review
### Strengths
### Weaknesses
### Gaps
### High-leverage improvements

## 4. Best Trajectory
### Recommended path
### Why
### What to avoid right now

## 5. Suggested Build Options
- Option A
- Option B
- Option C
- Option D

## 6. Build Prompt to User
**What do you want to build from here?**
Choose one of the options above, combine them, or give a custom direction and then convert it into the execution plan.

## 7. Optional Execution Plan
If requested:
- front-end patch plan
- asset generation plan
- repo update plan
- branch / PR plan

---

## Recommended Command Shape

### Master workflow command
`/project-review-build-trajectory <repo>`

### Sub-workflows
- `/repo-intake`
- `/intent-classifier`
- `/project-review`
- `/trajectory-planner`
- `/ui-uplift`
- `/asset-plan`
- `/repo-patch`

### Generated artifacts per project
- `PROJECT_REVIEW.md`
- `TRAJECTORY_PLAN.md`
- `UI_UPLIFT_SPEC.md`
- `ASSET_PLAN.md`
- `EXECUTION_PLAN.md`

---

## Studio-Level Recommendation

Use this as the **portfolio-wide operator workflow**:

1. ingest repos
2. classify each project
3. review each project
4. choose best trajectory
5. generate UI uplift + image plan
6. patch repos in batches
7. commit changes per project
8. optionally open PRs

This becomes the mass-upgrade layer for all project fronts.

---

## Versioning Recommendation

### V1
- single repo input
- summary of what happened
- intent classification
- project review
- best trajectory recommendation
- user prompt for what to build next
- optional image plan

### V2
- multi-repo mode
- front-end batch patching
- generated asset injection
- automatic branch + commit flow

---

## Operational Notes

- Bias toward digital-first execution.
- Reuse existing repo patterns rather than inventing new structures.
- Prefer scalable system decisions over one-off hacks.
- Distinguish clearly between analysis, plan, and execution.
- When facts are uncertain, state assumptions explicitly and proceed with the highest-probability path.
