---
name: pauli-course-generator
description: Graph-backed codebase-to-course workflow for turning a mapped repo into a single-page interactive onboarding course.
version: 1.0.0
required_environment_variables: []
---

# Pauli Course Generator

## triggers

- course
- teach this repo
- onboarding
- explain this codebase
- codebase to course

## when_to_use

Use when a repo, graph, or project needs a plain-English interactive course for onboarding or client education.

## when_not_to_use

Do not use before the repo has at least a minimal structural analysis or graph/inventory pass.

## required_tools

- terminal
- file read/write
- Graphify or equivalent structure map

## required_env

- none

## context_budget

- pair with one repo-analysis skill and one design/polish skill

## safety_gates

- do not claim the course is graph-backed if no repo map exists
- keep the artifact self-contained and readable without a build step

## workflow

1. Run a structural analysis of the repo or load an existing graph/inventory.
2. Extract actors, user journeys, APIs, and data flows.
3. Design 4 to 6 teaching modules with quizzes and diagrams.
4. Emit a single-page course artifact plus metadata.

## output_contract

- course readiness
- module outline
- artifact paths
- blockers

## tests

- `scripts/pauli/coursegen-status.ps1` returns valid JSON
- upstream `codebase-to-course` checkout and skill definition are present
