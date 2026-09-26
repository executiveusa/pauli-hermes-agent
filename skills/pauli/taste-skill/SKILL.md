---
name: pauli-taste-skill
description: Taste and visual-direction skill for deliberate, non-generic interfaces with strong hierarchy and editorial restraint.
version: 1.0.0
required_environment_variables: []
---

# Pauli Taste Skill

## triggers

- taste
- refresh UI
- make it better
- premium design

## when_to_use

Use when the interface needs a more intentional visual language without drifting into clutter.

## when_not_to_use

Do not use if the task is purely backend or if the existing system must stay unchanged pixel-for-pixel.

## required_tools

- file read
- browser-harness

## required_env

- none

## context_budget

- 2 to 3 design skills max

## safety_gates

- avoid generic template styling
- keep primary actions obvious

## workflow

1. Choose a clear visual direction.
2. Improve typography, spacing, and contrast.
3. Keep labels and actions obvious.

## output_contract

- visual direction summary
- notable component changes
- rationale in plain language

## tests

- primary action is obvious
- hierarchy is scannable at a glance

## Design Intelligence Feed Integration (v2.0 upgrade)

When user requests a visual build for a specific niche:
1. Load `skills/design-intelligence/SKILL.md`
2. Identify the niche from `_shared/niches.md`
3. Load `_feeds/{niche}-latest.md`
4. Use the extracted mechanisms as the visual target — not training-data taste

Taste is calibrated against what's actually winning in the niche today,
not against what Claude last saw in training data. Feed file is truth.
