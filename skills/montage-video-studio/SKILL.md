# MONTAGE Video Studio

Use the `montage_studio` tool when the user asks Hermes to inspect footage, operate a Montage project, create/select/edit clips, inspect capabilities, or dispatch a video-production action.

## Authority boundary

MONTAGE is the source of truth for StudioProject state, source media references, timelines, versions, approvals, render evidence, and exports. Hermes is the operator/orchestrator. Do not recreate Montage project state in Hermes memory or mutate video state outside the Montage action contract.

## Required sequence

1. Call `montage_studio(operation="health")` before meaningful work.
2. If the exact capability is unclear, call `capabilities` or `describe`.
3. Dispatch the smallest named action with structured inputs.
4. Keep `approved=false` unless the user explicitly approved a consequential/paid stage or Montage's capability contract requires approval.
5. Return Montage's evidence/status. Do not claim a render, export, or publish step succeeded without the returned proof.

## Documentary footage

For footage-led work, prefer visual/source analysis over transcript-only assumptions. Silent footage, B-roll, timelapse, drone footage, and ambient scenes can still be valuable. Ask Montage to analyze/index source media and return exact source ranges before proposing an edit.

## Failure behavior

If Montage is unreachable, report the runtime boundary clearly. Do not substitute a different editor or fabricate project state. If an action is unavailable, inspect capabilities and present the closest verified Montage path.
