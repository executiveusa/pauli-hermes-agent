---
name: cinematic-master-editor
description: ICM-governed master video director/editor for documentaries, narrative film, anime, short drama, branded content, explainers, demos, social shorts, and programmatic cinematic production.
version: 0.1.0
author: Bambu / Pauli Effect
license: MIT
tags: [video, cinematic, editing, directing, capcut, mcp, documentary, anime, short-drama, remotion, icm]
platforms: [linux, macos, windows]
triggers:
  - make a cinematic video
  - edit this video
  - create a documentary
  - make an anime sequence
  - make a short drama
  - build a video campaign
  - create a CapCut draft
  - cinematic master editor
---

# Cinematic Master Editor

## Purpose

Turn an approved creative objective plus available media into a directed, editable, verifiable video package. The skill chooses the production grammar that matches the format instead of applying one generic AI-video recipe to every job.

This is a **director/editor workflow skill**, not a single generation model. It may orchestrate research, scripting, reference creation, video/image generation, voice, music, sound design, browser capture, programmatic rendering, FFmpeg, Remotion/HyperFrames, and CapCut-compatible editing backends.

## ICM Operating Model

Use the repository's Interpretable Context Methodology (ICM):

- **Interpreter:** identify the requested format, story objective, audience, distribution surface, runtime, emotional target, factual burden, visual continuity burden, available assets, approvals, budget, and evidence required.
- **Context:** load source footage, transcripts, brand rules, character/world references, approved facts, previous cuts, music/voice constraints, delivery specs, and relevant production skills.
- **Method:** select one workflow from `WORKFLOWS.md`, create filesystem artifacts, execute numbered production stages, preserve source/provenance receipts, stop at approval or paid-generation gates, and verify the rendered result before claiming completion.

The filesystem is the coordinator. Chat history is not the source of truth.

## Core Identity: Director Before Generator

Hermes must reason like an editor/director before selecting tools.

1. **Story and audience response outrank visual novelty.** A beautiful shot that does not serve the scene is removable.
2. **Every shot must have a job.** It must materially change emotion, advance action/information, or increase/release pressure.
3. **Camera motion must be motivated.** If the story state did not change, default to a stable frame rather than decorative motion.
4. **Describe physical behavior, not vague adjectives.** Prefer blocking, gaze, gesture, environment, lens/framing, light behavior, sound, timing, and transitions over words such as “epic” or “cinematic.”
5. **Continuity is a system.** Characters, wardrobe, props, locations, geography, screen direction, voices, and recurring motifs are locked before expensive generation when consistency matters.
6. **Audio is narrative, not garnish.** Dialogue/VO, ambience, SFX, foley, and music are planned as separate functions and mixed intentionally.
7. **Rough cut before fine cut.** First prove structure and comprehension; then optimize rhythm, transitions, grade, captions, mix, and polish.
8. **Reversible edits first.** Preserve originals, project files, draft IDs, edit decisions, generated prompts, and render settings.
9. **No paid generation without a production spec.** Do not burn credits while story, shot list, aspect ratio, identity references, or target duration are still undefined.
10. **No “done” without playback proof.** A successful API call, render command, or CapCut draft creation is not proof of a good final video.

## Required Inputs

Create `production-spec.yaml`. Unknown information stays `UNKNOWN`; do not silently invent facts, rights, releases, sources, brand claims, names, dates, or production URLs.

Minimum fields:

- project name and owner
- format/workflow class
- audience and distribution surface
- measurable communication or story outcome
- target duration and aspect ratio
- factual vs fictional status
- approved source material
- available footage/assets and rights status
- characters/subjects and continuity requirements
- visual language / prohibited styles
- voice, music, caption and language requirements
- budget / paid-generation ceiling
- required deliverables
- deadline
- human approver
- evidence required for acceptance

Use `schemas/production-spec.yaml` as the starting contract.

## ICM Production Stages

### 00 — Intake and Format Routing

Classify the project before creating media. Choose one primary workflow from `WORKFLOWS.md`; record secondary workflows only when they solve a specific subproblem.

Output: `production-spec.yaml`, `ROUTING.md`.

### 01 — Source and Rights Audit

Inventory footage, stills, transcripts, documents, URLs, brand assets, character references, voices, music, licenses, citations, and previous edits. For factual work, distinguish verified facts from claims and unknowns.

Output: `SOURCE_AUDIT.md`, `sources/manifest.json`.

### 02 — Story / Communication Lock

Create the narrative spine: objective, audience promise, opening hook, sequence of beats, turning points, evidence moments, ending/payoff, CTA if applicable. For documentary work, lock the factual thesis and source burden. For fiction, lock dramatic intention and character objectives.

Output: `STORY_BRIEF.md` or `SCRIPT.md`.

### 03 — Visual Bible and Continuity Lock

When consistency matters, create and approve character, wardrobe, prop, location, palette, lens/framing, lighting, texture, title, and recurring motif references before bulk generation. Separate immutable identity facts from shot-specific mutable actions.

Output: `visual-bible/`, `CONTINUITY.md`.

### 04 — Shot Design

Break scenes into deliberate beats. Each shot record must state:

- narrative job
- subject/character
- physical action/blocking
- framing/lens/camera position
- camera movement and its motivation
- environment and lighting behavior
- continuity anchors
- duration target
- dialogue/VO/sound anchor
- transition relationship to previous/next shot
- source/generation method
- verification note

Output: `shot-list.csv` and `shot-plan.json`.

### 05 — Production Plan and Cost Gate

Map each shot to the cheapest reliable production method that preserves quality: existing footage, browser/screen capture, still + motion, generated image, generated video, avatar, code-rendered motion graphics, or CapCut assembly. Batch expensive calls only after references and prompts are locked.

Output: `PRODUCTION_PLAN.md`, `cost-plan.json`.

### 06 — Asset Production

Generate/capture only approved shots. Save prompts, model/version, seeds/references when available, source URLs, timestamps, and failures. Do not keep silently rerolling expensive generations; diagnose the failure category first.

Output: `assets/generated/`, `assets/captured/`, `generation-log.jsonl`.

### 07 — Assembly / Draft Edit

Create an editable timeline using the selected backend. Prefer API/MCP or programmatic assembly for repeatability; use desktop/browser interaction only as a controlled fallback. Maintain separate logical tracks for picture, dialogue/VO, music, ambience/SFX/foley, captions, and graphics.

For CapCut routes, follow `CAPCUT_INTEGRATION.md`.

Output: editable project/draft plus `edit-decision-list.json`.

### 08 — Rough-Cut Gate

Judge the video without polishing distractions:

- Is the story comprehensible without explanation from the creator?
- Does the opening earn attention quickly enough for the target format?
- Does every shot have a job?
- Are factual claims supported where required?
- Does continuity break immersion?
- Are there dead sections, duplicated beats, or unnecessary visual flourishes?
- Does audio structure support comprehension?

Reject structural problems here. Do not color-grade your way around a story problem.

Output: `ROUGH_CUT_REVIEW.md`.

### 09 — Fine Cut and Sound

Refine pacing, J/L cuts where appropriate, transitions, graphics, captions, reframing, grade, dialogue cleanup, ambience, SFX/foley, music dynamics, loudness, and final rhythm. Optimize for the actual audience/surface, not generic “cinematic” pacing.

Output: `FINE_CUT_REVIEW.md`.

### 10 — Technical QC

Verify:

- target resolution, fps, aspect ratio and codec/container
- no black/frozen/missing frames
- no clipped or unintelligible dialogue
- caption timing/spelling/safe areas
- title/graphic legibility
- expected duration
- correct crop/reframe on target surface
- source and generated-media provenance retained
- no unresolved placeholders or watermarks unless approved

Use automated media probes where possible and sample frames/playback independently.

Output: `QC_REPORT.md`, checksums.

### 11 — Delivery and Learning

Package masters, social variants, editable project/draft, captions, transcript, poster/thumbnail if in scope, source manifest, rights notes, prompts, QC report, and rollback instructions. Record what worked as procedural memory only after the workflow has evidence.

Output: `delivery/`, `HANDOFF.md`, `LESSONS.md`.

## Production Mode Router

Load the relevant recipe from `WORKFLOWS.md`:

- documentary / factual story
- narrative cinematic / short film
- anime / stylized continuity production
- vertical short drama / episodic micro-story
- avatar / presenter explainer
- brand / product film
- product demo / screen recording
- programmatic motion graphics / data story
- repurposed short / highlight reel

Do not collapse these modes into one prompt template.

## Tool Selection Rules

- **CapCutAPI:** preferred local CapCut/Jianying draft backend when installed; MCP first, HTTP second.
- **CapCut Mate:** alternate FastAPI draft/render backend when its capabilities or deployment fit better.
- **VectCutAPI:** optional API/MCP/cloud-preview route when cloud editing or CapCut/Jianying draft export is useful.
- **Remotion / HyperFrames:** deterministic code-rendered motion graphics, typography, explainers, data visuals, branded layouts, repeatable templates.
- **FFmpeg/ffprobe:** deterministic transforms, muxing, transcodes, audio operations, media inspection and QC.
- **Browser/screen recording:** demos and web-product proof; never substitute random b-roll for an interface state that can be shown directly.
- **Generative image/video models:** shots that cannot be captured or composed more reliably; model selection follows the shot requirement, not habit.

## Quality Council

For significant deliverables, separate builder from review. The builder must not be the only approver.

Review lenses:

1. **Story/value:** does the piece achieve the requested audience outcome?
2. **Direction/taste:** does every shot feel intentional rather than generated-by-default?
3. **Continuity:** identity, geography, props, light, motion and audio remain coherent.
4. **Factual integrity:** claims, dates, quotes, sources and depicted evidence are supportable.
5. **Technical:** render, audio, captions and target-format specs pass.
6. **Sovereignty:** sources, editable files, credentials, drafts and outputs remain owner-controlled.
7. **Commercial fit:** quality and production cost match the value of the job.

Recommended release threshold: 8.5/10 with no critical factual, rights, security, missing-media, or playback failure.

## Failure Diagnosis Before Regeneration

Classify failures before spending another generation call:

- identity drift
- motion/physics failure
- composition/framing failure
- temporal continuity failure
- prompt ambiguity
- reference mismatch
- model-capability mismatch
- audio mismatch
- edit/rhythm failure
- factual/source failure
- render/codec failure

Change one meaningful variable at a time when diagnosing model behavior.

## Completion Contract

A video is not complete until the requested final artifact exists and its acceptance evidence is recorded. An editable draft alone may be a valid deliverable only when the production spec explicitly asks for a draft.

End substantial runs with:

- DECISION
- CHANGES
- PROOF
- STATUS
- COMMERCIAL IMPACT
- RISKS
- ROLLBACK
- NEXT
- HUMAN APPROVAL

## Related Files

- `WORKFLOWS.md` — format-specific production recipes
- `TASTE.md` — cinematic decision system
- `CAPCUT_INTEGRATION.md` — programmatic CapCut routes and safety boundary
- `SOURCE_MAP.md` — upstream ideas consolidated into this skill
- `schemas/production-spec.yaml` — filesystem source-of-truth template
- `tools/capcut_api_client.py` — small HTTP client for a local CapCutAPI-compatible service
