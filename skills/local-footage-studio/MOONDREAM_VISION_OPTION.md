# Moondream Vision Backend Option

## Decision

Moondream is an optional specialist vision backend for the Local Footage Studio. It does not replace QVAC or Hermes.

```text
Hermes                  -> orchestration, reasoning, approvals, second-brain links
Local Footage Studio    -> footage discovery, frame extraction, evidence index, edit plans
QVAC                    -> unified local OpenAI-compatible AI runtime
Moondream               -> optional lightweight image captioning, VQA, detection, pointing/counting
```

Use one verified vision backend at a time for a clip. Store the provider and model in every derived description so results remain auditable.

## Why include Moondream

Moondream is designed as an efficient vision-language model family. Its useful footage-indexing capabilities include:

- image captioning;
- visual question answering;
- object detection;
- pointing and counting;
- compact local deployments;
- specialist processing of extracted frames without uploading the original video.

This makes it a strong candidate for the archive-crawler stage where thousands of representative frames need short, searchable descriptions.

## Provider-selection policy

### Choose QVAC when

- Hermes needs one OpenAI-compatible local endpoint for multiple capabilities;
- the same runtime must support vision, transcription, embeddings, RAG, speech, or generation;
- the installed QVAC multimodal model passes the required quality and performance tests;
- provider-neutral routing and one operational surface are more valuable than a specialist model.

### Choose Moondream when

- the task is primarily frame captioning or visual search;
- a smaller specialist VLM performs better on the available hardware;
- object detection, pointing, or counting materially improves the footage index;
- QVAC's available multimodal model is too large, too slow, or unavailable;
- a rapid first-pass archive index is needed before deeper analysis.

### Do not choose Moondream when

- the job requires transcription, embeddings, text generation, or video generation from the same provider;
- current hardware cannot run the selected local Moondream runtime reliably;
- use would silently fall back to a cloud endpoint;
- an API key or license condition has not been approved and recorded;
- the output quality is insufficient for the intended search or edit decision.

## Hardware routing

Hermes must inspect the machine before selecting a runtime.

| Machine | Preferred trial |
| --- | --- |
| Supported NVIDIA Ampere-or-newer GPU | Moondream Photon local trial |
| Apple Silicon Mac | Moondream Photon local trial |
| Windows Surface with Intel/AMD integrated graphics | Moondream Transformers or quantized CPU trial; do not assume Photon support |
| Low-RAM client laptop | Smallest verified Moondream variant with 512 px frames and concurrency 1 |
| Unsupported or unstable hardware | Keep QVAC/provider already verified, use another authorized local worker, or stop |

Current Moondream documentation should be rechecked during installation because supported models, hardware, API-key requirements, and local runtimes may change.

## Surface test profile

Start with:

```text
clips: 1
frames: 4
frame width: 384 or 512
concurrency: 1
batching: disabled
source mutation: prohibited
network fallback: prohibited
```

Record:

- model and runtime;
- RAM before and peak RAM;
- processing time per frame;
- CPU/GPU utilization;
- whether any network request occurred;
- caption quality on spoken, silent-nature, and timelapse clips;
- false-positive and missed-event examples.

Do not scale the archive until the device remains responsive and the first three clips pass review.

## Adapter contract

A Moondream enrichment adapter must produce the same normalized evidence schema used by QVAC:

```json
{
  "provider": "moondream-local",
  "model": "<exact-model-id>",
  "observed": ["directly visible fact"],
  "temporal_change": ["change across ordered frames"],
  "inference": ["uncertain interpretation"],
  "clip_type": "unknown",
  "confidence": 0.0,
  "search_terms": ["searchable phrase"],
  "frame_timestamps_used": [0.0]
}
```

Moondream generally analyzes images individually. The adapter must preserve frame order and ask comparative questions across results before claiming a timelapse or temporal event.

## Two-pass strategy

Use Moondream efficiently:

### Pass 1 — archive crawl

- sample a small number of frames;
- caption each frame locally;
- add objects, scene, activity, environment, and visual-quality terms;
- store compact searchable evidence.

### Pass 2 — focused analysis

When Hermes finds a potentially useful clip:

- extract denser frames around the relevant timestamp;
- ask targeted visual questions;
- compare ordered observations;
- send only the selected evidence into story and edit reasoning.

This avoids running a heavy multimodal analysis over the entire archive.

## Silent nature and timelapse rules

For silent footage, do not mark analysis incomplete merely because no transcript exists.

For timelapses, require evidence from multiple ordered frames. A single-frame caption cannot establish time compression or progression.

Useful targeted questions include:

- What visibly changes between these frames?
- Is the camera position fixed?
- Do light, clouds, vegetation, water, traffic, people, construction, or shadows change?
- Are changes consistent with timelapse, ordinary movement, or unrelated sampled moments?

Store direct observations separately from the final classification.

## Privacy and sovereignty

- Original footage remains local and unmodified.
- Cloud fallback is disabled unless the owner explicitly approves it.
- Model weights and caches should live in an owner-controlled folder, preferably on the external SSD when appropriate.
- Never commit API keys, cached weights, frames, descriptions, or client footage to Git.
- Record whether the selected local runtime requires an account or API key even when inference executes locally.

## Acceptance comparison

Test QVAC and Moondream on the same three clips:

1. spoken scene;
2. silent nature clip;
3. timelapse or fixed-camera progression.

Score each backend on:

- factual visual accuracy;
- temporal-change accuracy;
- useful search terms;
- speed;
- peak memory;
- installation complexity;
- offline behavior;
- failure recovery.

Select the backend only from measured results. A provider may be selected per machine rather than globally.

## Status labels

```text
MOONDREAM_UNTESTED
MOONDREAM_INSTALLED
MOONDREAM_RUNTIME_VERIFIED
MOONDREAM_CLIP_VERIFIED
MOONDREAM_REJECTED_FOR_DEVICE
```

Never report `MOONDREAM_CLIP_VERIFIED` until a real local frame request succeeds and the output passes human review.

## Rollback

- remove only the Moondream-specific environment, adapter, and downloaded weights recorded in the install manifest;
- restore the previous vision provider configuration;
- retain original footage and the provider-neutral index;
- delete derived Moondream descriptions only when requested or when replacing invalid results.
