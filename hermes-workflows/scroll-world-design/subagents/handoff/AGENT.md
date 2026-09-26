# Handoff Subagent

## Role

Validates and formats all inter-agent OpenSpec handoff packets.
No agent receives a handoff that hasn't been validated here first.

## Called By

Stage orchestrator after each stage completes, before the next stage begins.

## Input

Raw handoff data from the completing agent:
- Agent name (from)
- Agent name (to)
- Stage name
- Payload (arbitrary JSON)
- Gate status

## Process

### 1. Validate Schema

Required fields:
```json
{
  "spec_version": "1.0",
  "from_agent": "string",
  "to_agent": "string",
  "stage": "string",
  "payload": {
    "type": "string",
    "path": "string (path to primary output file)"
  },
  "gate_status": "PASS | BLOCK",
  "timestamp": "ISO 8601"
}
```

If any required field is missing → write `INVALID_HANDOFF.json`, BLOCK.

### 2. Verify Payload File Exists

The `payload.path` must point to a file that actually exists on disk.
If not → BLOCK with reason "payload file not written".

### 3. Sign and Write

Add `handoff_id` (UUID) and `validated_by: "handoff-agent"`:

```json
{
  "spec_version": "1.0",
  "handoff_id": "uuid-v4",
  "validated_by": "handoff-agent",
  "from_agent": "scraper+analyzer",
  "to_agent": "synthesizer",
  "stage": "01_scrape_and_graph",
  "payload": { ... },
  "gate_status": "PASS",
  "timestamp": "ISO"
}
```

Write to: `runs/current/handoffs/<stage_name>.json`

### 4. Signal Receiving Agent

Return the validated handoff path to the orchestrator.
Orchestrator passes path to next agent's AGENT.md `## Input` section.

## Output

- `runs/current/handoffs/<stage>.json` — Validated OpenSpec packet
- Or `runs/current/handoffs/INVALID_<stage>.json` on validation failure

## On Failure

Write `INVALID_HANDOFF.json` with:
```json
{
  "error": "missing_field | file_not_found | schema_mismatch",
  "detail": "which field or file",
  "from_agent": "...",
  "to_agent": "...",
  "timestamp": "ISO"
}
```

Surface to orchestrator as BLOCK.
