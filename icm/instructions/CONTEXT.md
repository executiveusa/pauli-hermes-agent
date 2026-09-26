# ICM Instructions Contract

## Job

Stable, model-agnostic operating policy for standalone Hermes.

## Reads

Instructions may read named files from `../context/`, explicit user/mission inputs, and relevant prior memory. Do not load the whole repository by default.

## Writes

Runtime agents never rewrite instructions. Policy changes are reviewed repository changes.

## Requirements

Each executable instruction states:
- inputs;
- ordered process;
- outputs;
- evidence/validation;
- failure and stop conditions;
- risk tier / human gate;
- rollback or compensation where action is reversible.

Use task profiles/capability requirements instead of hardcoding models. Put boundary facts in context and run results in memory.