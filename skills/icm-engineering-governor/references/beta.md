# Beta capability library

These capabilities are experimental. Do not auto-select them for production-critical work unless their value is clear, bounded, and reversible.

## loop-me
Use multi-session self-interrogation to turn a fuzzy workflow into an implementable specification. Persist decisions and unresolved branches in the workspace so a fresh session can continue without replaying all context.

## writing-beats
Shape an article as a sequence of beats. Choose the next beat based on what the reader now knows and needs, write one beat at a time, and stop when the argument/narrative reaches a natural end.

## writing-fragments
Interview for heterogeneous raw fragments: examples, claims, stories, observations, metaphors, objections, evidence, and questions. Preserve them as raw material without prematurely forcing a final structure.

## writing-shape
Transform a raw Markdown corpus into an article incrementally. Decide the role and format of each paragraph/section explicitly, preserving useful source material and removing repetition.

## claude-handoff
When a compatible Claude background-agent runtime is actually available, create a compact handoff and start a fresh background worker seeded with it. Otherwise fall back to the normal `handoff` capability; never pretend a background process was started.

## setup-ts-deep-modules
Use dependency-cruiser (or an existing equivalent) to enforce deep-module boundaries in TypeScript packages: public entry points are stable, internal implementation stays hidden, and tests exercise behavior through public seams. Inspect current package boundaries before introducing new tooling.

## Beta release gate
Before any beta capability changes production code or workflow, record: expected benefit, rollback, bounded experiment scope, proof signal, and stop condition.
