---
name: pauli-infisical-secrets
description: Secret inventory, secret health, and Infisical or env-source routing without exposing secret values.
version: 1.0.0
required_environment_variables: []
---

# Pauli Infisical Secrets

## triggers

- secret
- token
- credential
- .env
- infisical

## when_to_use

Use for redacted env audits, missing-secret reports, secret routing, and safe sync planning.

## when_not_to_use

Do not use when the task would require printing or logging raw secret values.

## required_tools

- terminal
- file read

## required_env

- optional Infisical credentials if live sync is required

## context_budget

- 2 companion skills max

## safety_gates

- never print values
- report only key names and presence states

## workflow

1. Scan env sources without echoing values.
2. Produce redacted presence maps.
3. Identify missing required keys for the target workflow.

## output_contract

- redacted inventory
- leak check status
- missing required keys

## tests

- outputs show `present` or `missing` only
