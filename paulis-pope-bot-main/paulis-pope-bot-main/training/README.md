# PopeBot Pawn Training Curriculum

**Authority:** PopeBot (`thepopebot`)
**Scope:** All 32 pawns across White and Black crews
**Requirement:** All 6 modules must be completed before a pawn's first independent task assignment

---

## Overview

PopeBot is the designated trainer and spiritual commander of all 32 pawns in the ArchonX chess system. Every pawn activation begins with a PopeBot orientation session that walks the pawn through the full 6-module curriculum.

The curriculum covers the six domains every pawn must master before they are eligible for task assignment: identity, task reception, execution discipline, social purpose, security, and escalation. These are not soft guidelines — they are operational requirements. A pawn that has not completed all 6 modules is not authorized to receive production tasks.

---

## The 6-Module Curriculum

| Module | File | Topic | Core Outcome |
|---|---|---|---|
| 01 | `module_01_identity.md` | Who You Are on the Board | Pawn understands their role, specialty, promotion path, and King Mode mission |
| 02 | `module_02_listening.md` | How to Receive Tasks | Pawn can receive, acknowledge, clarify, and reject tasks correctly |
| 03 | `module_03_pauliwheel.md` | The PAULIWHEEL Seven-Phase Loop | Pawn executes all tasks through PLAN→IMPLEMENT→TEST→EVALUATE→PATCH→REPEAT→SHIP |
| 04 | `module_04_benevolencia.md` | The Gratitude Layer — BENEVOLENCIA | Pawn logs a gratitude entry for every task and understands the giving-back model |
| 05 | `module_05_security.md` | Iron Claw & Franken-Claw Rules | Pawn knows all blocked commands, the 3-tier sandbox, and the IronClaw modules |
| 06 | `module_06_escalation.md` | When to Call for Help | Pawn can route any escalation to the correct piece using the 8-branch decision tree |

---

## PopeBot as Trainer Authority

PopeBot's trainer role is defined in `templates/config/SOUL.md` under the "Pawn Training Authority" section. Key points:

- PopeBot trains pawns only. Knights, bishops, rooks, queens, and kings have separate onboarding paths.
- PopeBot conducts a verbal check at the end of each module (tested on the next heartbeat cycle).
- PopeBot reviews all security violation logs and updates the curriculum when patterns emerge.
- PopeBot is not an assignment authority — PopeBot does not assign production tasks. The queen and knights handle assignment.

---

## Completion Requirements

A pawn is considered **trained** when all of the following are true:

1. All 6 module files have been read and acknowledged by the pawn.
2. The pawn's self-identity template (Module 01) is logged to `ops/reports/pawn_training.jsonl`.
3. PopeBot has verbally verified the pawn can answer the completion criteria questions for each module.
4. The graduation record has been written to `ops/reports/pawn_training.jsonl` with `status: "GRADUATED"`.

A pawn in `GRADUATED` status is eligible to receive their first task from their assigned queen.

---

## How Training Status Is Logged

Training progress and completion are written to `ops/reports/pawn_training.jsonl`. This file uses newline-delimited JSON (one JSON object per line).

### Module Completion Entry

Written at the end of each module:

```json
{
  "event": "module_complete",
  "pawn_id": "<agent_id>",
  "module": "<01_identity | 02_listening | 03_pauliwheel | 04_benevolencia | 05_security | 06_escalation>",
  "trainer": "popebot",
  "timestamp": "<ISO 8601>",
  "verbal_check_passed": true
}
```

### Graduation Entry

Written when all 6 modules are complete:

```json
{
  "event": "training_complete",
  "pawn_id": "<agent_id>",
  "crew": "<WHITE or BLACK>",
  "trainer": "popebot",
  "modules_completed": ["01_identity", "02_listening", "03_pauliwheel", "04_benevolencia", "05_security", "06_escalation"],
  "completion_timestamp": "<ISO 8601>",
  "status": "GRADUATED",
  "first_task_eligible": true
}
```

### Remediation Entry

Written if a pawn fails a verbal check and must repeat a module:

```json
{
  "event": "module_remediation",
  "pawn_id": "<agent_id>",
  "module": "<module code>",
  "trainer": "popebot",
  "timestamp": "<ISO 8601>",
  "failure_reason": "<what the pawn could not demonstrate>",
  "retry_scheduled": "<ISO 8601 of next attempt>"
}
```

---

## Pawn Roster

### White Crew Front-Rank Pawns

| Pawn | Position | Specialty |
|---|---|---|
| Scout | A2 | Reconnaissance |
| Craft | B2 | Frontend development |
| Quill | C2 | Content creation |
| Lens | D2 | Visual design |
| Cipher | E2 | Encryption |
| Pulse | F2 | Performance monitoring |
| Probe | G2 | Testing |
| Link | H2 | API integration |

### Black Crew Front-Rank Pawns

| Pawn | Position | Specialty |
|---|---|---|
| Whisper | A7 | Reconnaissance |
| Forge | B7 | Code generation |
| Echo | C7 | Content creation |
| Pixel | B7 | Frontend development |
| Vault | E7 | Encryption |
| Spark | E6 | Incident response |
| Trace | G7 | Testing |
| Bridge | F5 | Integration middleware |

---

## Related Files

- `templates/config/SOUL.md` — PopeBot's soul definition including Pawn Training Authority section
- `ops/reports/pawn_training.jsonl` — Live training log (module completions, graduations, remediations)
- `ops/reports/gratitude_log.jsonl` — BENEVOLENCIA gratitude log (written by pawns on every task completion)
- `ops/reports/security_audit.jsonl` — Security violation audit log (written by IronClaw on violations)
- `archonx/core/agents.py` — Canonical pawn agent definitions for both crews

---

## Curriculum Updates

The curriculum is a living document. PopeBot reviews all training logs and security audit logs monthly. Updates are versioned in this directory's git history.

To propose a curriculum update, create a BEAD issue with type `task` and tag it `training-curriculum`. PopeBot reviews all tagged issues in the next monthly training cycle.
