# Agent Reach upstream provenance

This Hermes skill integrates with, but does not vendor or fork, Agent Reach.

- Repository: https://github.com/Panniantong/Agent-Reach
- License: MIT
- Integrated version: `1.5.0`
- Integrated commit: `b4d52c46c9113cb0f653d6df4cf71ebadf4930ac`
- Integration date: 2026-07-28

## What is upstream

Agent Reach provides the CLI, installer, doctor, backend routing, transcription
entrypoint, and platform guidance. Its upstream tools include `yt-dlp`, Jina
Reader, Exa through `mcporter`, `gh`, OpenCLI, and platform-specific CLIs.

## What this repo adds

The Hermes integration adds:

- skill activation and routing instructions;
- a governed cross-platform research workflow;
- a reliable YouTube subtitle/ASR ladder;
- ICM/second-brain packaging guidance;
- a no-sudo pinned bootstrap script;
- a deterministic YouTube transcript helper;
- read-only, credential, evidence, and workspace boundaries.

## Update procedure

1. Review Agent Reach changelog and security notes.
2. Update the pinned commit in `SKILL.md`, `bootstrap.sh`, and this file.
3. Run the skill-pack test.
4. Run `bootstrap.sh --apply` in an isolated environment.
5. Run `agent-reach doctor --json`.
6. Verify one YouTube video with manual captions, one with automatic captions,
   and one no-caption ASR fallback.
7. Verify no cookie or token values appear in output or logs.
8. Submit the update as a separate PR with rollback to the previous pin.
