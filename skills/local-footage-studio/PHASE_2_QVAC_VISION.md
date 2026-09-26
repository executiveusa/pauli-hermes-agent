# Phase 2 — QVAC Vision Enrichment

## Outcome

Prove that Hermes can use a locally loaded QVAC multimodal model to describe ordered footage frames, including silent nature clips and timelapses, without modifying original media.

## Preconditions

1. PR #61 is merged to `main`.
2. The Local Footage Studio workspace has been initialized and test footage indexed.
3. QVAC is running on `127.0.0.1`.
4. `GET /v1/models` returns a loaded model that has passed a multimodal image test.
5. `workspace/config.json` contains the exact returned alias:

```json
{
  "qvac_base_url": "http://127.0.0.1:11434/v1",
  "vision_model": "<verified-loaded-multimodal-alias>"
}
```

Do not guess the alias and do not mark a text-only model as vision capable.

## Test batch

Use 3–10 non-critical clips:

1. one spoken or interview clip;
2. one silent or low-audio nature clip;
3. one fixed-camera timelapse or visibly changing sequence.

Keep the first pass at eight 512-pixel frames per clip on a Surface-class laptop.

## Run

Index the batch first:

```powershell
python skills/local-footage-studio/scripts/footage_studio.py index `
  --workspace "D:\PauliFootageAI" `
  --source "D:\Footage\TestBatch" `
  --max-frames 8 `
  --frame-width 512
```

Enrich one clip only:

```powershell
python skills/local-footage-studio/scripts/enrich_qvac.py `
  --workspace "D:\PauliFootageAI" `
  --clip-id "<CLIP_ID>" `
  --max-frames 8
```

After the single-clip result is reviewed, process at most three pending clips:

```powershell
python skills/local-footage-studio/scripts/enrich_qvac.py `
  --workspace "D:\PauliFootageAI" `
  --limit 3 `
  --max-frames 8
```

Search the resulting evidence:

```powershell
python skills/local-footage-studio/scripts/footage_studio.py search `
  --workspace "D:\PauliFootageAI" `
  --query "sunset OR vegetation OR timelapse"
```

## Pass criteria

- QVAC is reachable only through the approved local endpoint.
- The configured model appears in `/v1/models`.
- A description JSON file is created under `descriptions/<clip_id>.json`.
- `vision_status` becomes `verified-local` only after a valid model response.
- Direct observations, temporal changes, and inferences remain separate.
- Silent footage receives meaningful visual evidence without depending on a transcript.
- Timelapse classification is supported by visible changes across ordered frames.
- Original source file size and modification time remain unchanged.
- Search returns the enriched clip using words not present in its filename.

## Failure rules

- `model_not_found`: refresh `/v1/models`; do not invent an alias.
- `invalid_model_type` or image rejection: stop and choose a verified multimodal model.
- out of memory: stop QVAC, unload the model, and choose a smaller approved model.
- malformed JSON: keep the clip pending and record the model response; do not silently accept it.
- weak visual evidence: re-index the selected clip with a focused or denser frame strategy rather than raising the whole archive budget.

## Rollback

Delete the generated description JSON and re-index the affected clip, or restore the workspace from its pre-test copy. Original media is never edited by this phase.

## Completion report

```text
DECISION
CHANGES
PROOF
STATUS: VISION VERIFIED or VISION FAILED
COMMERCIAL IMPACT
RISKS
ROLLBACK
NEXT
HUMAN APPROVAL
```
