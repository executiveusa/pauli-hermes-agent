---
name: pauli-supabase-memory
description: Supabase and pgvector-backed second-brain retrieval, writeback discipline, and search-first memory workflows.
version: 1.0.0
required_environment_variables:
  - SUPABASE_URL
  - SUPABASE_ACCESS_TOKEN
---

# Pauli Supabase Memory

## triggers

- second brain
- remember
- memory
- notes
- knowledge

## when_to_use

Use for durable project-memory retrieval and writeback planning.

## when_not_to_use

Do not dump the full knowledge base into context.

## required_tools

- terminal
- database or API client

## required_env

- `SUPABASE_URL`
- `SUPABASE_ACCESS_TOKEN`

## context_budget

- search-first
- retrieval snippets only

## safety_gates

- no full vault dump
- write structured summaries, not chain-of-thought

## workflow

1. Search memory with a narrow query.
2. Pull only the relevant snippets.
3. Write back durable facts or decisions after verification.

## output_contract

- retrieved memory summary
- candidate writebacks
- missing infra blockers

## tests

- retrieval path returns focused snippets
- no whole-vault injection
