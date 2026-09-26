---
name: local-footage-studio
description: |
  Build and operate a private, local-first AI footage studio on Windows, Linux, or macOS.
  Index local video archives with FFmpeg, transcripts, sampled frames, motion evidence,
  and QVAC-compatible vision analysis; search clips through SQLite FTS; create reversible
  edit plans before controlling an NLE. Designed for Surface-class laptops, external drives,
  client installations, and consented remote support.

  Triggers: "index my footage", "understand these videos", "search my archive",
  "local ai video editor", "find clips", "silent nature footage", "timelapse",
  "make an edit plan", "install footage studio"
triggers:
  - "index my footage"
  - "understand these videos"
  - "search my archive"
  - "local ai video editor"
  - "find clips"
  - "silent nature footage"
  - "timelapse"
  - "make an edit plan"
  - "install footage studio"
tags:
  - video
  - local-ai
  - qvac
  - ffmpeg
  - transcription
  - vision
  - semantic-search
  - editing
  - davinci-resolve
---

# Local Footage Studio

## Outcome

Turn folders of local footage into a compact, searchable evidence layer that Hermes can
query and reason over without repeatedly decoding full videos. The first verified slice is:

```text
video files -> metadata + audio transcript + sampled frames + motion evidence
            -> local descriptions -> SQLite/FTS index -> search results -> edit plan
```

Hermes remains the orchestrator. QVAC or another verified OpenAI-compatible local runtime
provides model inference. FFmpeg/ffprobe provide deterministic media inspection and frame
extraction. The original footage remains untouched.

## Architecture decision

Reuse ideas from `bradautomates/claude-video` without copying its product identity or
assuming Claude-specific image reading. Its useful pattern is: captions/transcript first,
scene-aware or keyframe extraction, frame deduplication, timestamp grounding, and focused
re-analysis. This skill extends that pattern from one-video question answering into a
persistent footage archive and editing workflow.

The archive crawler pattern is:

- extract a few representative frames instead of watching every frame;
- convert visual evidence into compact text;
- retain filename and timestamps so results map back to source media;
- search the text rather than repeatedly scanning terabytes of video.

## Non-negotiable safety rules

1. Never modify, rename, move, transcode, or delete source footage during indexing.
2. Write indexes, thumbnails, proxies, transcripts, and edit plans to a separate workspace.
3. Record source file size, modification time, and SHA-256 only when explicitly requested;
   full hashing can be expensive on large archives.
4. Bind local model servers to `127.0.0.1` unless network exposure is separately approved.
5. Never claim an edit is complete because an EDL, timeline, or render command was produced.
6. NLE control is approval-gated. Create an edit plan and preview first.
7. On a client machine, the client owns the footage, index, models, credentials, and exports.
8. Remote setup requires visible consent. The owner enters passwords and approves UAC/sudo.

## Storage layout

Default to an external drive when footage is large:

```text
<workspace>/
  footage.db
  manifests/
  frames/<clip-id>/
  audio/<clip-id>/
  transcripts/<clip-id>.json
  descriptions/<clip-id>.json
  edit-plans/
  logs/
  config.json
```

Do not place model weights or extracted frames inside the Hermes repository.

## Surface-class operating profile

Start with a low-resource profile:

```text
concurrency: 1
frame width: 384-512 px
sample policy: start + end + interval + scene-change cap
max frames per clip: 12
transcription: audio only; skip silent tracks
vision: smallest verified local model
proxy generation: off by default
background indexing: paused on battery
```

Escalate only after measuring RAM, disk, thermals, and processing rate.

## Silent clips, nature footage, and timelapses

Transcript absence is not failure. For low-audio or silent footage, prioritize:

- beginning, middle, and ending frames;
- scene-change frames;
- regular interval frames;
- visual difference between adjacent samples;
- average motion score from FFmpeg scene metrics;
- day/night and lighting changes;
- camera movement versus subject movement;
- weather, water, vegetation, animals, people, vehicles, and landmarks;
- timelapse indicators: rapid lighting shift, cloud displacement, plant movement,
  construction progression, traffic trails, tides, shadows, or repeated static framing.

Descriptions must separate direct evidence from inference:

```json
{
  "observed": ["dense green vegetation", "camera remains fixed"],
  "temporal_change": ["sky darkens across sampled frames"],
  "inference": ["likely sunset timelapse"],
  "confidence": 0.78
}
```

Never infer a species, person identity, location, or event without sufficient evidence.

## Installation workflow

### Phase 1 — inspect

Run read-only checks for OS, CPU, RAM, free disk, GPU, battery state, FFmpeg, ffprobe,
Python, Hermes, QVAC, and ports. Identify the footage root and separate workspace root.

### Phase 2 — dependencies

Minimum prototype dependencies:

- Python 3.11+
- FFmpeg and ffprobe on PATH
- SQLite with FTS5 support
- Hermes
- optional QVAC local OpenAI-compatible server for model descriptions
- optional local Whisper-compatible transcription endpoint

Use the `qvac-local-ai` skill for QVAC installation and verification.

### Phase 3 — initialize

Run:

```bash
python skills/local-footage-studio/scripts/footage_studio.py init \
  --workspace "D:/PauliFootageAI"
```

### Phase 4 — index a test folder

Start with 3-10 copied or non-critical clips, not the entire archive:

```bash
python skills/local-footage-studio/scripts/footage_studio.py index \
  --workspace "D:/PauliFootageAI" \
  --source "D:/Footage/TestBatch" \
  --max-frames 8
```

The prototype performs deterministic metadata inspection and frame extraction. When a local
vision endpoint is configured, Hermes may enrich each clip description through QVAC. Until
that path is live-tested, the index must report `vision_status: pending` rather than inventing
descriptions.

### Phase 5 — search

```bash
python skills/local-footage-studio/scripts/footage_studio.py search \
  --workspace "D:/PauliFootageAI" \
  --query "sunset timelapse over trees"
```

Search results must include source path, duration, evidence timestamps, transcript snippets,
and confidence/status fields.

### Phase 6 — create an edit plan

Hermes converts selected evidence into a reversible JSON plan:

```json
{
  "project": "nature-test-01",
  "source_clips": [
    {
      "path": "D:/Footage/TestBatch/clip01.mp4",
      "in": 4.2,
      "out": 11.8,
      "reason": "cloud movement and lighting transition",
      "evidence": ["00:04.2", "00:10.0"]
    }
  ],
  "music": null,
  "transitions": "cuts-only",
  "status": "PROPOSED"
}
```

Do not touch an NLE until the owner approves the plan.

## NLE control path

Preferred progression:

1. Generate and review a JSON edit plan.
2. Generate an EDL/FCPXML/OTIO timeline when the target editor supports it.
3. Import into DaVinci Resolve or another NLE.
4. Use the editor's official scripting API when available.
5. Use screen/computer control only for gaps not covered by an API.
6. Require a visible preview and explicit approval before rendering or overwriting files.

For UI control, Hermes follows:

```text
OBSERVE -> EXPLAIN -> APPROVE -> ACT -> VERIFY -> RECORD
```

Never rely only on screen coordinates when accessibility trees, application APIs, keyboard
commands, or deterministic project files are available.

## Second-brain integration

The footage index is a derived evidence source, not the source of truth. Hermes may send
compact clip summaries, transcript excerpts, tags, and edit decisions to the user's second
brain. Do not upload original footage by default.

Each exported note should retain:

```text
source_path
clip_id
captured_at when known
evidence timestamps
observed facts
inferences with confidence
people/project/client associations approved by the owner
```

This allows cross-project discovery while keeping the media local.

## Client installation mode

For Antone or another client:

1. Inspect the machine and obtain written/visible authorization.
2. Select a client-owned workspace and model directory.
3. Install only minimum dependencies.
4. Index a small acceptance batch.
5. Demonstrate one silent clip, one spoken clip, and one timelapse if available.
6. Verify search returns correct source timestamps.
7. Produce rollback and uninstall instructions.
8. Disconnect remote support and remove temporary access.

## Acceptance test

A prototype passes only when all are true:

- source files are unchanged;
- one spoken clip is indexed with metadata and transcript status;
- one silent or low-audio clip has representative visual frames;
- one timelapse or changing scene has temporal evidence from multiple timestamps;
- search returns the correct source path and timestamps;
- an edit plan is generated without modifying source footage;
- workspace can be deleted without affecting originals;
- Hermes records limitations and rollback.

## Completion report

End substantial work with:

```text
DECISION
CHANGES
PROOF
STATUS
COMMERCIAL IMPACT
RISKS
ROLLBACK
NEXT
HUMAN APPROVAL
```

Allowed statuses:

```text
DESIGNED
PROTOTYPE INSTALLED
INDEX VERIFIED
SEARCH VERIFIED
EDIT PLAN VERIFIED
NLE IMPORT VERIFIED
RENDER VERIFIED
```

Do not use `DONE` without a verified rendered output, preserved originals, and recorded rollback.
