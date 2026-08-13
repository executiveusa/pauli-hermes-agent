# Cinematic Master Editor — Workflow Router

Choose one primary workflow based on the audience outcome and production constraints. Hybrid jobs may borrow a bounded stage from another workflow, but must keep one source-of-truth production spec.

## 1. Documentary / Factual Story

**Use when:** documentary, case study, nonprofit impact story, founder story, investigative/explanatory factual video.

**Order:**
1. Define thesis/question and audience promise.
2. Build a source ledger: documents, interviews, footage, dates, statistics, rights, unresolved claims.
3. Transcribe/index interviews and source footage.
4. Build a paper edit before expensive visuals: strongest verified statements, counterpoints/context, narration gaps, evidence moments.
5. Design sequences around evidence and real environments; use generated imagery only when clearly appropriate and not misleading.
6. Record narration only after factual/script lock.
7. Rough cut for argument and emotional arc.
8. Add maps, diagrams, archive labels, lower thirds, captions, music and sound design.
9. Fact-check the cut, not only the script.
10. Deliver a source manifest with the master.

**Quality trap:** mood footage replacing evidence.

## 2. Narrative Film / Cinematic Short

**Use when:** short film, trailer, cinematic scene, music-led narrative, fictional branded film.

**Order:**
1. Lock dramatic objective, obstacle, turning point and payoff.
2. Break script into characters, locations, props and scene beats.
3. Create visual bible and identity references before generation.
4. Convert scenes to short shot beats with explicit physical blocking.
5. Mark immutable continuity facts separately from per-shot action.
6. Select image/video models per shot requirement.
7. Generate in continuity-aware batches; use reference images and frame chaining where useful.
8. Assemble picture before elaborate sound/music.
9. Rough-cut for emotional causality and rhythm.
10. Fine-cut, sound design, grade and titles.

**Quality trap:** impressive isolated clips that do not cut together.

## 3. Anime / Stylized Continuity Production

**Use when:** anime, comic-to-motion, stylized episodic world, recurring avatars/characters.

**Order:**
1. Lock style bible: line, shading, palette, proportions, lens language, motion density.
2. Lock character sheets: front/side/back, expressions, costume variants, key props.
3. Lock locations and recurring establishing views.
4. Define episode/scene beats and shot continuity.
5. Separate identity/style constraints from motion prompts.
6. Generate keyframes/storyboards before motion shots.
7. Approve consistency contact sheet before bulk motion generation.
8. Produce motion shots, preserving screen direction and scene geography.
9. Assemble dialogue/VO and effects; use animation-appropriate timing rather than live-action assumptions.
10. QC character identity, costume, hand/prop continuity, subtitle timing and style drift.

**Quality trap:** changing art direction between shots because each prompt is authored independently.

## 4. Vertical Short Drama / Episodic Micro-Story

**Use when:** serialized 9:16 drama, cliffhanger short, character-led social fiction.

**Order:**
1. Validate concept for repeatable conflict and episode engine.
2. Define world, core relationships, motivations and secrets.
3. Create season/arc outline before isolated episodes.
4. Write short episodes with fast orientation, conflict escalation and a clean turn/cliffhanger.
5. Lock recurring characters and locations.
6. Storyboard for vertical framing and close-readable performance.
7. Produce shots with continuity references.
8. Cut aggressively; eliminate setup that the image already communicates.
9. Use captions/sound hooks appropriate to mobile viewing without destroying dramatic tone.
10. Preserve episode-state memory for the next installment.

**Quality trap:** treating a short drama as unrelated viral clips instead of a continuity system.

## 5. Avatar / Presenter Explainer

**Use when:** spokesperson video, AI avatar explainer, educational presenter, talking-head campaign.

**Order:**
1. Lock audience question and one clear promise.
2. Write for spoken language and time the actual narration.
3. Decide where the presenter must be visible and where visuals should carry meaning.
4. Lock avatar/voice identity and pronunciation list.
5. Produce presenter segments.
6. Create explanatory visuals tied to specific narration claims/state changes.
7. Assemble with purposeful b-roll, diagrams, screenshots or examples.
8. Caption from the final audio timing.
9. QC lip sync, pronunciation, eye line, pacing, visual relevance and CTA.

**Quality trap:** wall-to-wall avatar plus generic stock/AI b-roll.

## 6. Brand / Product Film

**Use when:** launch film, campaign spot, product story, premium social ad.

**Order:**
1. Lock the commercial objective and single audience action.
2. Translate product facts into visual proof moments.
3. Define brand visual language and prohibited claims.
4. Build a beat sheet: hook → tension/problem → proof/transformation → payoff/CTA.
5. Capture or generate hero product/people/environment assets.
6. Design typography, graphics and sonic identity as one system.
7. Rough cut for comprehension without copy overload.
8. Fine cut for rhythm, product visibility and brand recognition.
9. Verify every factual/comparative claim.
10. Export surface-specific variants from the same locked master story.

**Quality trap:** production value without product proof.

## 7. Product Demo / Screen Recording

**Use when:** software demo, tutorial, feature proof, walkthrough.

**Order:**
1. Define the exact user task and successful end state.
2. Prepare a clean demo account/state and remove private data.
3. Script interaction checkpoints, not every mouse movement.
4. Record real UI states whenever possible.
5. Capture narration separately or after a clean interaction pass.
6. Edit pauses/errors while preserving truthful behavior.
7. Add zooms, callouts and highlights only where they improve comprehension.
8. Verify the demonstrated flow against the current product.
9. Caption and export target aspect ratios.

**Quality trap:** simulated interface visuals when working software can be demonstrated directly.

## 8. Programmatic Motion Graphics / Data Story

**Use when:** kinetic typography, charts, repeatable branded videos, data explainers, template-driven content.

**Preferred tools:** Remotion, HyperFrames, SVG/Canvas/WebGL where needed, FFmpeg for deterministic post.

**Order:**
1. Lock data/copy and visual hierarchy.
2. Define reusable components and timing tokens.
3. Build one verified scene slice.
4. Validate typography, safe areas, motion readability and render performance.
5. Extend scene system without duplicating animation logic.
6. Render deterministic preview/master.
7. Inspect sampled frames and full playback.
8. Export variants from parameters rather than hand-editing duplicates.

**Quality trap:** coding a custom renderer when a simpler timeline edit would be faster and cheaper.

## 9. Repurposed Short / Highlight Reel

**Use when:** podcast clip, interview highlight, event recap, long-to-short conversion.

**Order:**
1. Transcribe/index source material.
2. Select clips by complete idea/emotional turn, not keyword alone.
3. Preserve enough context to avoid changing meaning.
4. Build hook without fabricating a claim.
5. Reframe for target aspect ratio and keep faces/text safe.
6. Add captions from final timing.
7. Add contextual b-roll/graphics only when they clarify.
8. Verify quote fidelity and source timestamp.
9. Export variants and preserve source references.

**Quality trap:** maximizing retention by distorting the speaker's meaning.

## Shared Deliverables

Every workflow should produce the smallest useful subset of:

```text
project/
├── production-spec.yaml
├── ROUTING.md
├── SOURCE_AUDIT.md
├── STORY_BRIEF.md or SCRIPT.md
├── visual-bible/
├── shot-list.csv
├── shot-plan.json
├── PRODUCTION_PLAN.md
├── assets/
├── edit-decision-list.json
├── ROUGH_CUT_REVIEW.md
├── FINE_CUT_REVIEW.md
├── QC_REPORT.md
└── delivery/
```
