---
name: pauli-jcodemunch
description: Large-repo compression and context-map workflow that summarizes before deep reading while preserving source-of-truth discipline.
version: 1.0.0
required_environment_variables: []
---

# JcodeMunch

## triggers

- large repo
- inventory
- scan codebase
- context bloat
- summarize project

## when_to_use

Use before loading many files from a large repo or package set.

## when_not_to_use

Do not use as a replacement for reading the exact source file you are editing.

## required_tools

- terminal
- file search

## required_env

- none

## context_budget

- run before reading more than 20 repo files on one task

## safety_gates

- summaries are indexes, not facts
- source files remain the source of truth

## workflow

1. Build a compact map of repo areas, hot files, and likely edit surfaces.
2. Save compact maps under `docs/context-maps/` when the task is large enough to benefit from reuse.
3. Use the map to narrow the next file reads.

## output_contract

- repo slice summary
- hot files
- unanswered questions

## tests

- generated summaries omit secrets
- summary points to actual source files
