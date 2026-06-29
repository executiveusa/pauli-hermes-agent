---
name: pauli-fal-ai
description: FAL.ai provider adapter policy for dry-run generation planning, key presence checks, and cost-guarded invocation.
version: 1.0.0
required_environment_variables:
  - FAL_KEY
---

# Pauli FAL.ai

## triggers

- fal
- image generation
- video generation
- render provider

## when_to_use

Use when the studio pipeline needs to evaluate or prepare a FAL-backed generation step.

## when_not_to_use

Do not make paid generation calls by default.

## required_tools

- terminal
- HTTP client

## required_env

- `FAL_KEY`

## context_budget

- use only on active media-generation tasks

## safety_gates

- dry-run first
- approval for paid generation

## workflow

1. Check key presence.
2. Validate model/provider choice.
3. Prepare dry-run payload or blocker report.

## output_contract

- provider status
- dry-run readiness
- paid-call gate status

## tests

- key presence check works
- default path makes no paid call
