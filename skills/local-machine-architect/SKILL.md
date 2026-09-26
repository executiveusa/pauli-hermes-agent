---
name: local-machine-architect
description: Read-only hardware, storage, model-fit, cost, and installation planning for owner and client machines before local AI setup.
version: "0.1.0"
platforms: [windows, linux, macos]
---

# Local Machine Architect

## Purpose

Use this skill before installing QVAC, Moondream, local language models, transcription models, footage tooling, browser-control software, or client-side agents.

The skill converts a machine into an evidence-backed deployment profile. It does not delete files, download models, change startup settings, expose ports, or install software during audit mode.

## Required outcome

Produce:

1. a machine inventory;
2. a storage map;
3. a model-fit recommendation;
4. an install location plan;
5. a recurring-cost summary;
6. a staged installation proposal;
7. rollback instructions;
8. unresolved risks and owner approvals.

## Workflow

### 1. Establish authority

Record:

- device owner;
- whether the device is personal, client-owned, or employer-managed;
- authorized directories;
- approved external drives;
- actions that must remain read-only;
- whether remote support is authorized;
- whether the owner is present.

Never install on an employer-managed device without IT authorization.

### 2. Run the read-only audit

```powershell
python skills/local-machine-architect/scripts/audit_machine.py `
  --output "$env:USERPROFILE\PauliMachineAudit\machine-audit.json"
```

Optional bounded storage scan:

```powershell
python skills/local-machine-architect/scripts/audit_machine.py `
  --scan-root "$env:USERPROFILE" `
  --max-depth 3 `
  --top 30 `
  --output "$env:USERPROFILE\PauliMachineAudit\machine-audit.json"
```

The script may read file metadata. It must not open personal documents, delete files, move files, or upload data.

### 3. Classify the machine

Use measured usable RAM, free disk, CPU architecture, GPU/VRAM evidence, and thermal/power constraints.

| Profile | Typical local role |
| --- | --- |
| `CONTROL_ONLY` | Hermes orchestration, browser control, search, remote worker routing |
| `MICRO_LOCAL` | 0.5B–3B quantized text models, small embeddings, bounded transcription |
| `SMALL_LOCAL` | 3B–8B quantized text models, lightweight vision trials, single-job transcription |
| `MEDIUM_LOCAL` | 7B–14B quantized models, larger embeddings, sustained media indexing |
| `GPU_WORKER` | verified GPU-accelerated vision, transcription, generation, and batch processing |

Do not assign a profile from marketing specifications. Assign it from the audit report and a real benchmark.

### 4. Model-fit policy

Approximate model weight size is not total runtime memory. Keep operating-system and application headroom.

- Never plan to consume more than 70% of measured usable RAM.
- On a Windows Surface, default to concurrency 1.
- Reserve at least 20 GB free on the system drive after installation.
- Keep model weights, frame caches, transcripts, and proxies on an owner-controlled external SSD when practical.
- Test one model and one clip before batch work.
- Prefer a smaller verified model over a larger model that causes paging or thermal instability.

Suggested trial order:

1. 0.5B–1.5B quantized model;
2. 2B–4B quantized model;
3. 7B–8B quantized model only when measured RAM and benchmark evidence support it;
4. specialist vision model such as Moondream for frame indexing;
5. QVAC as the unified local endpoint when its selected models pass device tests.

### 5. Storage architecture

Preferred layout:

```text
ExternalSSD/
├── AI-Models/
├── PauliFootageAI/
│   ├── footage.db
│   ├── frames/
│   ├── transcripts/
│   ├── descriptions/
│   ├── edit-plans/
│   └── benchmarks/
├── Agent-Workspaces/
├── Install-Manifests/
└── Backups/
```

Original footage remains separate and immutable. Derived caches must be rebuildable.

### 6. Space-recovery policy

The audit may identify candidates such as caches, duplicate installers, model caches, stale build output, package-manager caches, temporary media proxies, and oversized logs.

Every candidate must be classified:

- `SAFE_TO_REBUILD`
- `REVIEW_REQUIRED`
- `PRESERVE`
- `UNKNOWN`

No deletion or movement occurs without a manifest, owner approval, destination verification, and rollback path.

### 7. Cost policy

Use `references/pricing-and-costs.md` as a dated snapshot, not permanent truth. Re-check official pricing before purchase.

Separate:

- one-time hardware/storage cost;
- recurring memberships;
- optional AI credits;
- cloud fallback cost;
- electricity and time cost;
- client support and maintenance price.

Default commercial package:

```text
Local AI Audit
→ Sovereign Local Install
→ Footage Intelligence Setup
→ Client Handoff
→ MAXX Operations
```

### 8. Installation proposal

Before installing, report:

```text
MODE
OUTCOME
TARGET
CONSTRAINTS
PROOF
COMMERCIAL VALUE
MACHINE PROFILE
MODEL PLAN
STORAGE PLAN
COST
RISKS
ROLLBACK
HUMAN APPROVAL
```

### 9. Improvement loop

The agent improves the plan from evidence, not by autonomously rewriting itself.

After every benchmark, append a record containing:

- machine fingerprint without secrets;
- model and quantization;
- task;
- elapsed time;
- peak RAM/VRAM when available;
- output quality score;
- failure reason;
- recommendation for the next bounded trial.

The agent may propose updates to model-fit rules. A separate reviewer and human must approve changes before they enter the shared skill.

## Remote client setup

Follow:

```text
OBSERVE → EXPLAIN → APPROVE → ACT → VERIFY → RECORD → DISCONNECT
```

The client types passwords and administrator credentials. Hermes must not request, record, or retain them.

## Completion states

```text
MACHINE_AUDIT_ONLY
MACHINE_PROFILE_VERIFIED
INSTALL_PLAN_APPROVED
LOCAL_RUNTIME_VERIFIED
CLIENT_HANDOFF_COMPLETE
```

Do not report `LOCAL_RUNTIME_VERIFIED` until a real local model request succeeds on the target machine.
