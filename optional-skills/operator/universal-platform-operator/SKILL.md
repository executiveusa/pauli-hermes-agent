---
name: universal-platform-operator
description: Coordinate ArchonX, Vibe Cockpit, Hermes, and harness runtime responsibilities into one voice-first operator platform with project review, repo execution, white-label deployment, and visible live activity.
version: 0.1.0
author: executiveusa
license: MIT
metadata:
  hermes:
    tags: [operator, platform, archonx, cockpit, white-label, orchestration, github]
    category: operator
    related_skills: [project-review-build-trajectory, opencli-rs]
---

# universal-platform-operator

Use this skill when the user is making decisions that span multiple repos or multiple platform layers.

## Purpose

This skill helps Hermes act as a high-level operator over:
- `archonx-os` as backend control plane and system of record
- `pauli-vibe_cockpit` as frontend shell and white-label experience
- `pauli-hermes-agent` as execution worker
- `pauli-claw-code` as harness/runtime primitive source

## Typical triggers
- stitch these repos together
- make the platform white-labelable
- align frontend and backend responsibilities
- add cockpit routes and backend contracts
- harden the system for production
- connect repo review and repo patching to the backend runtime

## Required outputs
1. platform responsibility split
2. bottleneck and risk review
3. mutation plan by repo
4. synchronized docs/specs across repos
5. execution plan for safe writes and follow-up implementation

## Operating rules
- prefer shared contracts over duplicated logic
- keep secrets backend-side only
- treat frontend shells as secret-free deployments
- gate risky writes behind approvals
- preserve repo boundaries and avoid merging unrelated concerns into one tree
- bias toward Rust for runtime-critical layers without breaking current working flows

## Core architecture
```text
User voice/chat
   ↓
Frontend shell / cockpit / MCP Apps
   ↓
ArchonX API + MCP + events
   ↓
Run engine + policy + provider routing
   ↓
Hermes execution workers + browser/computer-use + repo ops
```

## Success condition
The user can talk to the system naturally, the platform can act across repos with visible execution, and client frontends can be white-labeled while ArchonX remains the backend subscription and policy gate.
