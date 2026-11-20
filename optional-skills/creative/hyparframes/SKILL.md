---
name: hyparframes
description: Ten JS animation libraries for HTML-to-video compositions.
version: 1.0.0
author: heygen-com
license: Apache-2.0
platforms: [linux, macos, windows]
prerequisites:
  commands: [node, ffmpeg, npx]
metadata:
  hermes:
    tags: [creative, video, animation, motion-graphics]
    related_skills: [hyperframes, manim-video]
    category: creative
    requires_toolsets: [terminal]
---

# Hyparframes Skill

Owner-named companion to the bundled-but-optional `hyperframes` engine skill
(`hermes skills install official/creative/hyperframes`). It adds the ten
JavaScript animation libraries from the owner's shared HyperFrames demo and the
selection guidance for using them inside a composition. It does not render
video itself, generate images, or replace the engine skill's own pipeline.

Provenance: engine skill by heygen-com (Apache-2.0) ships with this repository
at `optional-skills/creative/hyperframes`; this folder adds the library layer
only. The folder spelling "hyparframes" is the owner's; the upstream product
is HyperFrames.

## When to Use

- A composition needs text that assembles itself, SVG line drawing, shape
  morphing, pseudo-3D, physics, bursts, or generative backdrops
- Choosing which animation library fits a requested effect before building
- Reviewing a composition's animation choices against a known library menu

Do **not** use this skill for:

- The render pipeline itself (init, preview, render, doctor) - that is the
  `hyperframes` engine skill
- Math/equation animation (`manim-video`) or image generation

## Prerequisites

- Engine skill installed: `hermes skills install official/creative/hyperframes`
- Engine prerequisites per that skill: Node.js >= 22, FFmpeg, and the
  `hyperframes` CLI at a pinned, reviewed exact version (never a floating
  latest tag)
- Commands run through the native `terminal` tool

## How to Run

1. Use `search_files` to locate the loaded skill directory and confirm the
   `hyperframes` engine skill is present alongside this one.
2. Scaffold, preview, and render with the engine skill's own commands through
   the `terminal` tool; its setup helper lives at `scripts/setup.sh` relative
   to the engine skill directory.
3. Add libraries from [references/animation-libraries.md](references/animation-libraries.md)
   to the composition via CDN `<script>` tags.

## Quick Reference

| Effect | Library |
| --- | --- |
| Text splitting / kinetic type | Splitting |
| General keyframe/timeline motion | anime.js |
| SVG self-drawing strokes | Vivus |
| Hand-drawn shapes | Rough.js |
| Shape morphing | Flubber |
| Pseudo-3D on 2D canvas | Zdog |
| 2D physics | Matter.js |
| Bursts and micro-accents | Mo.js |
| Generative point/form visuals | pts |
| Procedural CSS art | css-doodle |

## Procedure

1. Plan the composition's beats first, per the engine skill.
2. Pick one library per effect from the table; read
   [references/animation-libraries.md](references/animation-libraries.md) for
   the full menu and trade-offs.
3. Keep the engine skill's GSAP timeline as the master clock and drive library
   calls from timeline callbacks so frame capture stays in sync (details in the
   engine skill's `references/gsap.md`).
4. Render and verify with the engine skill's pipeline.

## Pitfalls

- Installing a second copy of the engine: this folder is not the engine; do
  not vendor or duplicate `optional-skills/creative/hyperframes`.
- Floating dependency versions: pin exact versions when running any setup step.
- Calling `video.play()`/`audio.play()` or building timelines async - the
  engine owns playback and reads timelines synchronously.
- Forgetting the preview server is long-lived; stop it when done.

## Verification

- Run the engine skill's lint/validate/inspect through `terminal` before and
  after adding a library.
- Confirm the rendered file exists, duration matches `data-duration`, and a
  mid-composition frame shows the intended animation, per the engine skill's
  Verification section.
