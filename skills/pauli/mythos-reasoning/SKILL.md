---
name: pauli-mythos-reasoning
description: Reasoning harness for context normalization, expert routing, bounded iteration, and decision summaries without chain-of-thought leakage.
version: 1.0.0
required_environment_variables: []
---

# Pauli Mythos Reasoning

## triggers

- reasoning
- evaluate options
- council
- confidence
- synthesize

## when_to_use

Use when the task needs structured decision synthesis across multiple candidate approaches.

## when_not_to_use

Do not use when a simple direct answer or straightforward code change is enough.

## required_tools

- file read
- terminal

## required_env

- none

## context_budget

- one reasoning skill plus domain skill companions

## safety_gates

- do not expose hidden chain-of-thought
- produce decision summaries and trace notes only

## workflow

1. Normalize the problem and constraints.
2. Compare options with bounded loops.
3. Emit a clear decision record and confidence note.

## output_contract

- normalized problem statement
- compared options
- decision
- confidence

## tests

- output excludes chain-of-thought
- final recommendation is concrete
