---
name: qvac-local-ai
description: |
  Operate QVAC as Hermes' local-first, OpenAI-compatible AI runtime. Use this skill to
  inspect, install, configure, start, verify, and safely connect QVAC for local chat,
  embeddings, RAG/vector search, transcription, speech, image generation, and video
  generation. Prefer the OpenAI-compatible HTTP boundary over copying QVAC internals.

  Triggers: "use qvac", "connect qvac", "run local ai", "local model provider",
  "private inference", "offline ai", "qvac server", "qvac embeddings",
  "qvac transcription", "qvac image generation", "qvac video generation"
triggers:
  - "use qvac"
  - "connect qvac"
  - "install qvac"
  - "run local ai"
  - "local model provider"
  - "private inference"
  - "offline ai"
  - "qvac server"
  - "qvac embeddings"
  - "qvac transcription"
  - "qvac image generation"
  - "qvac video generation"
tags:
  - qvac
  - local-ai
  - openai-compatible
  - privacy
  - inference
  - rag
  - speech
  - image-generation
  - video-generation
---

# QVAC Local AI Skill

## Purpose

Use QVAC as a separate local AI runtime behind its OpenAI-compatible HTTP API. Hermes
remains the orchestrator. QVAC owns model loading and inference. Do not vendor, fork, or
copy the QVAC monorepo into Hermes merely to use it.

Upstream source: `https://github.com/tetherto/qvac`

## Operating contract

1. **Inspect before changing.** Detect OS, architecture, RAM, free disk, GPU/runtime,
   Node.js, npm, and any existing QVAC installation.
2. **Do not expose secrets.** Never print or commit credentials. Local QVAC normally does
   not require a third-party model API key.
3. **Bind locally by default.** Use `127.0.0.1`, not `0.0.0.0`, unless the user explicitly
   approves network exposure and authentication/firewall controls are in place.
4. **Verify every capability.** A running process is not proof. Test `/v1/models` and the
   exact endpoint needed by the task.
5. **Preserve rollback.** Record changed files, processes, packages, ports, and previous
   provider configuration before editing.
6. **Use bounded resources.** Do not download or preload large models until hardware and
   disk capacity are checked and the user-approved task requires them.
7. **No unsupported claims.** Report a capability as available only after its model is
   loaded and its endpoint passes a smoke test.

## Decision rule

Choose QVAC when the user needs one or more of:

- local/private inference;
- reduced dependency on SaaS APIs;
- OpenAI-compatible local endpoints;
- local embeddings or RAG;
- speech-to-text or text-to-speech;
- local image or video generation;
- eventual peer-to-peer inference.

Do not choose QVAC merely because it is available. Keep the existing provider when it is
already verified, materially faster, cheaper for the workload, or required by a customer.

## Phase 1 — Inspect

Run read-only checks first:

```bash
uname -a || ver
node --version
npm --version
git --version
python --version || python3 --version
```

Also inspect:

- available RAM and free disk;
- GPU vendor, VRAM, and drivers where applicable;
- whether ports such as `11434` are already occupied;
- existing Hermes provider/base-URL configuration;
- existing QVAC processes, packages, directories, and configuration.

Return:

```text
MODE: brownfield
TARGET: local QVAC provider for Hermes
BASELINE: hardware, software, port, current provider
BLAST RADIUS: config files, packages, process/service, model storage
ROLLBACK: restore provider config; stop QVAC; remove only newly installed assets
```

## Phase 2 — Install or update safely

Prefer official QVAC documentation and released packages. Do not assume commands from
memory when upstream files or docs can be inspected.

When source installation is genuinely required:

```bash
git clone https://github.com/tetherto/qvac.git
cd qvac
```

Before installing dependencies:

- inspect the repository README and CLI package documentation;
- inspect package-manager lockfiles and required Node version;
- avoid changing global packages unless necessary;
- prefer a dedicated directory and reversible local installation.

Never commit QVAC source, downloaded model weights, caches, generated media, or local
configuration into the Hermes repository.

## Phase 3 — Start the OpenAI-compatible server

QVAC's integration boundary is:

```bash
qvac serve openai
```

Use upstream CLI help to resolve current flags:

```bash
qvac serve openai --help
```

Default safety posture:

```text
host: 127.0.0.1
base URL: http://127.0.0.1:11434/v1
network exposure: disabled
process supervision: explicit and reversible
```

Do not invent model aliases. Read the active QVAC configuration and `/v1/models` output.

## Phase 4 — Verify the server

Health and inventory smoke test:

```bash
curl --fail --silent --show-error http://127.0.0.1:11434/v1/models
```

Chat smoke test after identifying a loaded chat-model alias:

```bash
curl --fail --silent --show-error \
  http://127.0.0.1:11434/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{"model":"<loaded-chat-alias>","messages":[{"role":"user","content":"Reply with exactly: QVAC_OK"}],"max_tokens":16}'
```

Pass criteria:

- HTTP request succeeds;
- selected model exists and is ready;
- response contains `QVAC_OK` without an upstream/provider error;
- no public bind was introduced;
- no secret or model file was committed.

## Phase 5 — Connect Hermes

QVAC should be registered as an OpenAI-compatible provider using the verified local base
URL and a model alias returned by `/v1/models`.

Because Hermes configuration may change across versions:

1. inspect `hermes model --help`, `hermes config set --help`, and current config;
2. preserve the previous provider and model values;
3. set the OpenAI-compatible base URL to `http://127.0.0.1:11434/v1`;
4. select a verified QVAC model alias;
5. restart only the minimum required Hermes process;
6. run one Hermes conversation smoke test;
7. restore the old provider immediately if verification fails.

Do not hard-code an API key into the repository. If a client library insists on a key for
a local endpoint, use a non-secret runtime placeholder supplied through environment/config,
not committed source.

## Capability routing

Use only endpoints supported by the running QVAC version and loaded model category:

| Task | Endpoint |
| --- | --- |
| Chat | `POST /v1/chat/completions` |
| Legacy completion | `POST /v1/completions` |
| Responses API | `POST /v1/responses` |
| Embeddings | `POST /v1/embeddings` |
| Transcription | `POST /v1/audio/transcriptions` |
| Speech translation | `POST /v1/audio/translations` |
| Text-to-speech | `POST /v1/audio/speech` |
| Image generation | `POST /v1/images/generations` |
| Image edit | `POST /v1/images/edits` |
| Video generation | `POST /v1/videos` |
| Vector search | `POST /v1/vector_stores/{id}/search` |

Treat QVAC Responses API storage as volatile unless upstream documentation proves a durable
configuration is active. Do not use in-memory response IDs as durable project memory.

## Failure handling

When a request fails, capture:

- command and endpoint without secrets;
- HTTP status and stable QVAC error code;
- selected model alias and category;
- whether the model is loaded/ready;
- relevant process logs;
- available memory/disk;
- exact rollback action.

Common branches:

```text
model_not_found      -> refresh /v1/models; correct alias
model_not_ready      -> wait only for an active load or load the approved model
invalid_model_type   -> choose a model matching the endpoint category
connection refused   -> inspect process, bind address, port conflict
out of memory        -> stop, unload model, choose smaller approved model
unsupported_*        -> do not silently degrade; explain the unsupported field
```

## Completion report

Always end a substantial QVAC task with:

```text
DECISION
CHANGES
PROOF
STATUS
COMMERCIAL IMPACT
RISKS
ROLLBACK
NEXT
HUMAN APPROVAL
```

`STATUS` may be `INSPECTED`, `INSTALLED`, `SERVER VERIFIED`, or `HERMES VERIFIED`.
Never use `DONE` unless Hermes successfully completes a real request through QVAC and the
rollback path is recorded.
