---
name: avatar-world-builder
description: Assemble a personalized interactive avatar world from reusable scene parts, compatible open-source or open-asset options, and a reproducible scene manifest. Use for 3D avatar, virtual-world, or digital-twin experiences; do not use for a one-off image alone.
version: 1.0.0
author: Pauli Hermes Agent
license: MIT
triggers:
  - build an avatar world
  - make me an avatar site
  - make this avatar reusable
  - avatar world builder
  - create a 3D avatar
  - /avatar-world-builder
entry_point: /avatar-world-builder [brief]
---

# Avatar World Builder

Turn a person’s brief into a scene manifest that can be assembled again for another person without losing provenance or control.

## Canonical resources

- Parts, compatible slots, factory template, and product references live in StarNet Creative District: `districts/creative/avatar-world-builder/`.
- Use `_shared/PARTS_CATALOG.yaml` slot IDs exactly. Build from `factory/skin.template.yaml`.
- Present two or three compatible candidate choices for each part the human has not selected. Include source, license, commercial-use status, attribution, estimated performance, and trade-offs.

## Operating flow

1. Translate the brief into required slots: shell, world, avatar, identity, navigation, budgets, and license.
2. Determine truthful avatar capability: `none`, `rigged`, or `vrm-humanoid`. Do not promise articulated behavior to a static model.
3. Screen candidates and create a license ledger before import.
4. Obtain choices, write the manifest, build the scene, and validate desktop, mobile, reduced motion, and non-WebGL fallback.
5. Return a compact proof package: selected manifest, license ledger, test results, preview, rollback pointer.

## Approved release boundary

The owner approved normal Avatar World Builder logic and its documented internal progression on 2026-09-16. Do not ask again for routine composition, manifest work, or non-destructive checks.

This approval never substitutes for: an asset license ledger; paid-provider approval (including Meshy credits); valid credentials; or explicit authority to publish a production deployment. Stop at any missing boundary and state the exact blocker.
