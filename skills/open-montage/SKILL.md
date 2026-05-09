---
name: open-montage
description: >
  OpenMontage™ — AI-powered end-to-end video production. Trigger when user says
  things like: "make a video", "create a montage", "produce a trailer",
  "edit this footage", "animated explainer", "podcast clip", "documentary",
  "cinematic trailer", "talking head video", "repurpose this content",
  "create a video from", "video about".
version: 1.0.0
author: Pauli Second Brain™ | Kupuri Media™
invocation:
  model: default
metadata:
  hermes:
    tags: [video, production, filmmaking, montage, editing, media, AI, animation]
    related_skills: []
---

# OpenMontage™

Describe your vision. AI handles the rest — research, scripting, assets, editing, composition.

## When to Activate

Activate this skill when the user:
- Says **"make a video"** or **"create a montage"**
- Asks for a **cinematic trailer**, **animated explainer**, or **documentary**
- Wants to **repurpose** a podcast, interview, or existing footage
- Mentions **talking head**, **B-roll**, or **video editing**
- Asks to **analyze a reference video** and create a variant
- Says **"video about"** followed by any topic

## 12 Production Pipelines

| Pipeline | Use When |
|----------|----------|
| `animated_explainer` | Concept or product needs visual explanation |
| `cinematic_trailer` | Hype/promo for product, event, or project |
| `documentary_montage` | Archive footage + narration storytelling |
| `talking_head` | Single-speaker scripted or interview format |
| `podcast_repurpose` | Convert audio podcast to video clip |
| `social_short` | 15–60s vertical or square social content |
| `brand_story` | Company/founder narrative |
| `tutorial_walkthrough` | Step-by-step instructional |
| `news_recap` | Research-driven news summary video |
| `music_video` | AI-generated visual + audio sync |
| `product_showcase` | E-commerce or SaaS product demo |
| `event_highlight` | Conference, meetup, or launch recap |

## Core Workflow

### New video from description
1. Ask user: topic, duration target, style, platform (YouTube / Instagram / LinkedIn)
2. Select pipeline based on format
3. Run `python -m open_montage.run --pipeline <name> --prompt "<description>"`
4. Report: budget estimate → approval → render → deliver

### Reference-driven variant
1. User provides a video URL or file path
2. Run analysis: `python -m open_montage.analyze --input <path>`
3. Propose 3 differentiated variants
4. User selects → full production run

### Repurpose existing content
1. Identify source type (podcast, interview, article)
2. Run `python -m open_montage.repurpose --source <path> --format <pipeline>`
3. Auto-generate script, B-roll selection, captions

## Budget & Approval

- Every run produces a cost estimate before rendering
- Default approval threshold: **$5.00** (configurable via `MONTAGE_SPEND_CAP`)
- Free-tier path available: Archive.org + NASA footage + Piper TTS + local Stable Diffusion

## Quality Gates

OpenMontage validates automatically:
- Pre-render: script coherence, asset resolution, audio sync
- Post-render: frame inspection, audio levels, duration check
- Audit trail: full JSON checkpoint of every creative decision

## Asset Sources (Free Tier)

- **Video**: Archive.org, NASA, Wikimedia Commons
- **Images**: Stable Diffusion (local), Wikimedia Commons
- **TTS**: Piper TTS (local, zero cost)
- **Music**: Royalty-free archive

## Installation

```bash
git clone https://github.com/calesthio/OpenMontage
cd OpenMontage
pip install -r requirements.txt
npm install  # Remotion compositor
```

Set `OPEN_MONTAGE_PATH` in `~/.hermes/.env` to the install directory.
