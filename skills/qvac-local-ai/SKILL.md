---
name: qvac-local-ai
description: |
  Operate QVAC as Hermes' local-first, OpenAI-compatible AI runtime. Use this skill to
  inspect, install, configure, start, verify, and safely connect QVAC on the owner's laptop,
  a client workstation, or a consented remote machine. Includes browser-assisted setup,
  visible approval gates, rollback, and support handoff. Prefer the OpenAI-compatible HTTP
  boundary over copying QVAC internals.

  Triggers: "use qvac", "connect qvac", "install qvac", "set up qvac on this laptop",
  "install qvac for a client", "remote qvac install", "browser assisted install",
  "run local ai", "local model provider", "private inference", "offline ai",
  "qvac server", "qvac embeddings", "qvac transcription", "qvac image generation",
  "qvac video generation"
triggers:
  - "use qvac"
  - "connect qvac"
  - "install qvac"
  - "set up qvac on this laptop"
  - "install qvac for a client"
  - "remote qvac install"
  - "browser assisted install"
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
  - windows
  - client-install
  - remote-support
  - browser-assistance
---

# QVAC Local AI Skill

## Purpose

Use QVAC as a separate local AI runtime behind its OpenAI-compatible HTTP API. Hermes
remains the orchestrator. QVAC owns model loading and inference. Do not vendor, fork, or
copy the QVAC monorepo into Hermes merely to use it.

Upstream source: `https://github.com/tetherto/qvac`

This skill supports three installation modes:

1. **Owner-local** — Hermes guides installation on the machine where it is running.
2. **Client-assisted** — Hermes produces a machine-specific runbook while the client or
   authorized technician performs privileged actions.
3. **Consented remote support** — Hermes may operate an approved browser or remote-support
   session while the machine owner remains present and approves sensitive actions.

A browser by itself cannot reliably install desktop software, approve operating-system
security dialogs, or type administrator credentials. Remote installation therefore requires
an authorized remote-control channel or a human executing terminal steps. Never claim that
browser access alone provides full machine control.

## Operating contract

1. **Inspect before changing.** Detect OS, architecture, RAM, free disk, GPU/runtime,
   Node.js, npm, and any existing QVAC installation.
2. **Obtain authorization.** Before touching a client or friend's computer, record who owns
   the device, who authorized the work, the approved scope, and whether the owner is present.
3. **Do not expose secrets.** Never print, record, paste into chat, or commit passwords,
   recovery codes, private keys, API keys, or remote-support access codes.
4. **Bind locally by default.** Use `127.0.0.1`, not `0.0.0.0`, unless the owner explicitly
   approves network exposure and authentication/firewall controls are in place.
5. **Verify every capability.** A running process is not proof. Test `/v1/models` and the
   exact endpoint needed by the task.
6. **Preserve rollback.** Record changed files, processes, packages, ports, services, model
   storage, and previous provider configuration before editing.
7. **Use bounded resources.** Do not download or preload large models until hardware and
   disk capacity are checked and the approved task requires them.
8. **Require visible approval for privileged actions.** Administrator prompts, firewall
   changes, startup services, remote access, model downloads, and provider replacement must
   be approved by the device owner at the moment of action.
9. **No unsupported claims.** Report a capability as available only after its model is
   loaded and its endpoint passes a smoke test.
10. **Leave no covert access.** Never create hidden users, persistent tunnels, unattended
    remote-control access, unknown startup tasks, or credentials retained by the operator.

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

## Phase 0 — Authorization and install mode

Before a remote or client installation, produce this gate:

```text
DEVICE OWNER: <name or organization>
AUTHORIZED OPERATOR: <person performing setup>
INSTALL MODE: owner-local | client-assisted | consented-remote
APPROVED SCOPE: inspect | install | configure | model-download | Hermes-connect
PRIVILEGED ACTIONS: list each expected admin/firewall/service change
DATA BOUNDARY: files/directories Hermes may access
REMOTE ACCESS: temporary only | not used
OWNER PRESENT: yes | no
ROLLBACK OWNER: person responsible for reversal
```

Stop when ownership or authorization is unclear. Do not ask the user to send passwords or
remote-access codes through chat. The owner enters credentials directly into their device.

## Phase 1 — Inspect

Run read-only checks first.

### Cross-platform

```bash
node --version
npm --version
git --version
python --version || python3 --version
```

### Windows PowerShell

```powershell
Get-ComputerInfo | Select-Object WindowsProductName, WindowsVersion, OsArchitecture
Get-CimInstance Win32_ComputerSystem | Select-Object TotalPhysicalMemory
Get-Volume | Select-Object DriveLetter, SizeRemaining, Size
Get-CimInstance Win32_VideoController | Select-Object Name, AdapterRAM, DriverVersion
Get-NetTCPConnection -LocalPort 11434 -ErrorAction SilentlyContinue
Get-Command node,npm,git,python -ErrorAction SilentlyContinue
```

### Linux

```bash
uname -a
free -h
lsblk
lspci | grep -Ei 'vga|3d|display' || true
ss -ltnp | grep ':11434' || true
```

### macOS

```bash
sw_vers
system_profiler SPHardwareDataType SPDisplaysDataType
sysctl -n hw.memsize
df -h
lsof -nP -iTCP:11434 -sTCP:LISTEN || true
```

Also inspect:

- existing Hermes provider/base-URL configuration;
- existing QVAC processes, packages, directories, configuration, and model cache;
- antivirus or endpoint-management restrictions on client devices;
- whether the device is personally owned or organization-managed;
- whether installation violates an employer, school, or client IT policy.

Return:

```text
MODE: brownfield
TARGET: local QVAC provider for Hermes
BASELINE: hardware, software, port, current provider
BLAST RADIUS: config files, packages, process/service, model storage
ROLLBACK: restore provider config; stop QVAC; remove only newly installed assets
FIT: PASS | PASS WITH SMALL MODEL | FAIL
```

Do not proceed on a managed client machine without the required IT approval.

## Phase 2 — Select the smallest viable installation

Choose the minimum capability and model needed for the approved outcome.

```text
chat only              -> one small verified chat model
private document search -> chat + one embedding model
meeting transcription   -> one transcription model
voice assistant          -> chat + transcription + TTS
image/video workstation  -> only after GPU, VRAM, disk, and cooling checks pass
```

For low-power Windows laptops and tablets, start with chat or embeddings and a small
quantized model. Do not begin with image or video generation merely to demonstrate QVAC.

## Phase 3 — Install or update safely

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
- prefer a dedicated directory and reversible local installation;
- capture the exact upstream commit or package version installed;
- create no desktop shortcut, startup service, firewall rule, or PATH change without approval.

Never commit QVAC source, downloaded model weights, caches, generated media, or local
configuration into the Hermes repository.

## Phase 4 — Start the OpenAI-compatible server

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
startup persistence: disabled until separately approved
```

Do not invent model aliases. Read the active QVAC configuration and `/v1/models` output.

## Phase 5 — Verify the server

Health and inventory smoke test:

```bash
curl --fail --silent --show-error http://127.0.0.1:11434/v1/models
```

PowerShell equivalent:

```powershell
Invoke-RestMethod http://127.0.0.1:11434/v1/models
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
- no secret or model file was committed;
- resource usage remains within the agreed machine budget.

## Phase 6 — Connect Hermes

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

## Browser-assisted remote installation protocol

Use this protocol when Hermes has an approved browser/computer-use capability or when a
human operator is following Hermes' instructions through a remote-support session.

### What the browser may do

- open official QVAC and Hermes documentation;
- download an approved installer or source archive from the verified upstream location;
- inspect visible system-information pages;
- open a web terminal or approved remote-support portal;
- copy non-secret commands into a terminal after presenting them for review;
- observe command output and collect redacted verification evidence;
- guide the owner through operating-system dialogs.

### What requires the owner or authorized technician

- entering administrator credentials;
- accepting UAC, sudo, macOS security, antivirus, or firewall prompts;
- approving remote-control permissions;
- authorizing model downloads and storage consumption;
- changing startup behavior or network exposure;
- approving installation on an organization-managed machine.

### Required browser loop

For every state-changing action:

```text
1. OBSERVE — capture the current visible state.
2. EXPLAIN — state exactly what will change and why.
3. APPROVE — obtain owner approval for that single action.
4. ACT — perform only the approved action.
5. VERIFY — inspect the resulting state and command output.
6. RECORD — add evidence and rollback instruction to the session log.
```

Never batch multiple privileged actions behind one vague approval.

### Browser and remote-session safety

- Use a temporary, owner-approved session.
- Do not save passwords in the browser or password manager.
- Do not enable unattended access.
- Do not copy the client's personal files, browser history, cookies, or credentials.
- Close unrelated tabs and applications where practical before screen sharing.
- Redact email addresses, usernames, IP addresses, license keys, and access codes from logs.
- Keep a visible action log the owner can review.
- End the remote session after verification and have the owner confirm it is disconnected.
- Remove temporary downloads or support tools only when the owner approves and removal does
  not damage an existing support arrangement.

### Remote support stop conditions

Stop immediately when:

- the owner withdraws consent;
- an unexpected administrator, security, payment, or data-access prompt appears;
- the device appears organization-managed without authorization;
- antivirus or endpoint protection blocks the action;
- disk, memory, temperature, or stability becomes unsafe;
- the requested action would expose QVAC beyond localhost;
- the browser or remote tool cannot prove what command actually ran.

## Client handoff package

After a client installation, generate a client-owned handoff containing no secrets:

```text
INSTALL DATE
DEVICE / OS
QVAC VERSION OR COMMIT
HERMES VERSION
INSTALLED CAPABILITIES
MODEL ALIASES AND STORAGE LOCATIONS
LOCAL BASE URL
START COMMAND
STOP COMMAND
VERIFICATION COMMANDS
RESOURCE LIMITS
FILES AND SETTINGS CHANGED
ROLLBACK PROCEDURE
SUPPORT CONTACT / RESPONSIBILITY
REMOTE SESSION DISCONNECTED: YES | NO
CLIENT ACCEPTANCE: name + date
```

The client must retain ownership of the machine, configuration, model files, accounts, and
support relationship. Do not make continued operation depend on the installer's personal
credentials or private infrastructure unless that commercial arrangement is explicit.

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
remote state unclear -> stop; return control to owner; re-establish observable state
admin prompt         -> owner enters credentials and approves locally
managed device       -> stop until authorized IT approval is documented
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

`STATUS` may be `AUTHORIZED`, `INSPECTED`, `INSTALLED`, `SERVER VERIFIED`,
`HERMES VERIFIED`, or `CLIENT ACCEPTED`.

Never use `DONE` unless Hermes successfully completes a real request through QVAC, the
rollback path is recorded, the remote session is disconnected, and the device owner accepts
the result.
