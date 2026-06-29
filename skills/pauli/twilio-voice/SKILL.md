---
name: pauli-twilio-voice
description: Twilio voice status, webhook checks, transcript-oriented debugging, and allowlisted test-call procedures.
version: 1.0.0
required_environment_variables:
  - TWILIO_ACCOUNT_SID
  - TWILIO_AUTH_TOKEN
---

# Pauli Twilio Voice

## triggers

- twilio
- voice
- phone
- transcript
- webhook

## when_to_use

Use for Twilio health checks, webhook validation, and safe voice workflow debugging.

## when_not_to_use

Do not place calls to non-allowlisted numbers.

## required_tools

- terminal
- HTTP client

## required_env

- `TWILIO_ACCOUNT_SID`
- `TWILIO_AUTH_TOKEN`

## context_budget

- 3 skills max

## safety_gates

- no outbound calls without allowlist
- redact phone-sensitive metadata where possible

## workflow

1. Validate credentials and webhook configuration.
2. Check number/app status.
3. Run allowlisted non-production diagnostics only.

## output_contract

- voice health summary
- webhook health
- safe next step

## tests

- credentials presence check
- no unauthorized dial actions
