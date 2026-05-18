---
name: pauli-video-watch
description: Video ingestion and transcript workflow with yt-dlp, ffmpeg, caption extraction, and budget-aware analysis.
version: 1.0.0
required_environment_variables: []
---

# Pauli Video Watch

## triggers

- video
- watch this
- transcript
- captions
- youtube

## when_to_use

Use for transcript extraction, frame sampling, or low-cost video analysis.

## when_not_to_use

Do not run network-heavy or paid transcription workflows by default in CI.

## required_tools

- terminal
- yt-dlp
- ffmpeg

## required_env

- optional Whisper or provider keys only if explicitly configured

## context_budget

- 5 skills max with other video helpers

## safety_gates

- paid generation and paid transcription off by default
- sanitize temp paths and clean up outputs

## workflow

1. Prefer captions or transcript extraction.
2. Fall back to heavier analysis only if configured and approved.
3. Save summaries, timestamps, and extracted metadata.

## output_contract

- transcript path or blocker
- clip summary
- temp cleanup status

## tests

- local command availability check
- no paid call on default path
