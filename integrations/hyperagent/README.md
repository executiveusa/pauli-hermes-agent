# Hermes HyperAgent Adapter

This directory is the boundary between Hermes/Cosmos and the external `@hyperbrowser/agent` runtime.

## Why an adapter

Hermes is MIT licensed. HyperAgent is AGPL-3.0. This integration does not copy HyperAgent source code into Hermes. It invokes the published package as an external dependency while Hermes retains its own ICM governance, approvals, routine naming, evidence, and rollback policy.

## First-use setup

```bash
cd integrations/hyperagent
npm install --ignore-scripts --no-audit --no-fund
npm run self-test
```

Expected self-test shape:

```json
{
  "ok": true,
  "package": "@hyperbrowser/agent",
  "expected_version": "1.1.2",
  "capabilities": {
    "constructor": true,
    "executeTask": true,
    "newPage": true,
    "closeAgent": true
  }
}
```

## Request protocol

`runner.cjs` accepts one JSON object on stdin and returns one JSON object on stdout.

Supported actions:

- `task`: full HyperAgent `executeTask` workflow.
- `perform`: one granular page action.
- `ai`: multi-step browser task; optionally save its action cache as a named routine.
- `extract`: semantic page extraction.
- `replay`: deterministic action-cache replay with bounded XPath retries and upstream fallback behavior.

## Routine storage

Named routines are stored under `integrations/hyperagent/routines/` and filenames are restricted to `[A-Za-z0-9._-]`.

Do not persist:

- API keys;
- passwords;
- session cookies;
- access tokens;
- payment data;
- private form values that are not safe to retain.

A cache is not automatically trustworthy just because HyperAgent generated it. The ICM skill requires target proof and an independent result check before a routine becomes reusable.

## Environment

The adapter reads provider configuration from environment variables instead of task payloads:

```text
HYPERAGENT_LLM_PROVIDER
HYPERAGENT_LLM_MODEL
HYPERAGENT_BROWSER_PROVIDER
HYPERBROWSER_API_KEY
OPENAI_API_KEY or provider-specific credential
```

`HYPERAGENT_BROWSER_PROVIDER=Hyperbrowser` enables the upstream cloud-browser path. Otherwise the adapter leaves browser-provider selection local/default.

## Pilot status

This is a Cosmos/Bambu pilot. It must not be promoted into Agent Max until runtime tests and a real browser workflow have passed and the owner approves promotion.
