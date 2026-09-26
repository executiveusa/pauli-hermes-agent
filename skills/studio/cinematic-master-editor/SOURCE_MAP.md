# Source Consolidation Map

This file records **what capability was extracted and why**. Hermes should absorb reusable production principles and interface patterns, not copy whole upstream repositories into its own context.

Sweep date: 2026-08-09.

## CapCutAPI — ashreo/CapCutAPI

**Core intent:** expose CapCut/Jianying editing as programmable draft operations through HTTP and MCP.

**Hermes adoption:**
- primary local CapCut adapter pattern;
- draft-first editing contract;
- multi-track media/text/subtitle/effect operations;
- MCP-first orchestration, HTTP fallback;
- machine acceptance test before claiming CapCut is connected.

**Not copied:** upstream implementation internals and local installation-specific config.

## CapCut Mate — Hommy-master/capcut-mate

**Core intent:** FastAPI-based Jianying/CapCut draft automation with validated web API and render-oriented operations.

**Hermes adoption:**
- alternate backend for local/network deployments;
- schema-validation mindset;
- API-doc inspection before production requests;
- optional render route when supported by installed backend.

## VectCutAPI — sun-guannan/VectCutAPI

**Core intent:** agent-facing editing API/MCP with local draft export plus optional cloud preview/render.

**Hermes adoption:**
- optional cloud-assisted editing route;
- backend abstraction: editing intent should survive switching between MCP/HTTP/cloud implementations;
- keep cloud dependency optional for sovereign/private work.

## visual-skills — smixs/visual-skills

**Core intent:** teach agents film direction and shot reasoning rather than prompt decoration.

**Hermes adoption:**
- story/emotion and rhythm precede visual novelty;
- every shot needs a narrative job;
- physical blocking, gaze, environment and sound anchors beat vague adjectives;
- camera movement requires narrative motivation;
- editorial priority stack encoded in `TASTE.md`.

## Hyperreal AI Video System (HAVS) — geniusdapeng-collab/hyperreality-system

**Core intent:** industrialized, multi-stage AI video pre-production and generation with structured cinematic shot language and agent orchestration.

**Hermes adoption:**
- decouple script/story, production/generation, rendering and post-production;
- structured shot records instead of monolithic prompts;
- continuity references and quality gates before bulk generation;
- checkpointable filesystem artifacts for long-running productions.

## super-video-maker-skill — Bomx/super-video-maker-skill

**Core intent:** end-to-end agent video workflow spanning scripting, source proof, avatar/generated/captured visuals, real audio timing, programmatic rendering and QC.

**Hermes adoption:**
- source/provenance deck for factual work;
- narration timing before final visual timing;
- screen capture when showing actual software is more truthful/useful than generic b-roll;
- technical plus visual QC gates;
- explicit paid-generation gate.

## awesome-ai-short-drama-tools — clipcurator/awesome-ai-short-drama-tools

**Core intent:** map the AI short-drama ecosystem and show that serialized drama needs story/world/character/continuity/storyboard control, not only text-to-video generation.

**Hermes adoption:**
- dedicated vertical short-drama workflow;
- concept → world → character relationships → episode arc → script → consistency → storyboard → video;
- evaluate tools by continuity and full-chain needs, not popularity alone.

**Classification:** research/taxonomy source, not a runtime dependency.

## AI Cinematic Pipeline — billpar/ai-cinematic-pipeline

**Core intent:** production-tested, tool-agnostic narrative workflow focused on consistent characters/settings, short shot beats, prompt structure, frame chaining and audio post.

**Hermes adoption:**
- lock character/location/prop/voice references before expensive generation;
- split scenes into short directed beats;
- separate immutable identity constraints from mutable shot action;
- use frame chaining when it improves continuity;
- keep dialogue/VO, SFX, music, ambience/foley as distinguishable audio functions.

## Owner Avatar / Higgsfield Prompt Skill — mahshid1378/Owner-avatar-higgsfield-ai-prompt-skill

**Core intent:** model-aware prompting, camera/subject/look/action decomposition, identity consistency and systematic failure diagnosis for modern image/video models.

**Hermes adoption:**
- select model by shot requirement rather than habit;
- separate identity/style constraints from motion/action;
- verify current model/API schema before calls;
- diagnose identity, motion, composition and model-capability failures before rerolling;
- change one meaningful variable at a time during debugging.

## FilMaster — arXiv:2506.18899

**Core intent from the abstract inspected:** bridge cinematic principles and generative film systems through reference-guided camera-language design and audience-oriented post-production rhythm, with explicit rough-cut/fine-cut stages.

**Hermes adoption:**
- reference retrieval should answer a camera/story problem, not merely provide aesthetic inspiration;
- rough cut and fine cut are separate reasoning stages;
- audience response and audiovisual rhythm are first-class post-production constraints.

**Evidence boundary:** this sweep used the arXiv abstract/metadata. The full PDF was not successfully retrieved into the available PDF-analysis path, so no claim in this skill depends on uninspected paper details beyond the abstract.

## Capability Synthesis

The combined system is deliberately layered:

```text
REQUEST
  -> ICM interpreter + workflow router
  -> story/source lock
  -> visual/continuity bible
  -> structured shot plan
  -> model/capture/render routing
  -> generated/captured assets
  -> CapCut/API or programmatic timeline assembly
  -> rough-cut council
  -> fine cut + sound
  -> technical/factual/taste QC
  -> editable master + final delivery + receipts
```

This is the intended Hermes upgrade: **reason about filmmaking first, then call editing/generation tools as replaceable execution backends.**
