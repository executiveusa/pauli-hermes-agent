# Phase 4 — Multimodal Story Ranking

## Outcome

Combine verified visual evidence and local transcript evidence into a deterministic, human-reviewed ranking of potentially useful clips.

This phase does not edit footage, create a timeline, or let an agent control an NLE. It creates a proposed shortlist with explicit reasons.

## Run

```powershell
python skills/local-footage-studio/scripts/rank_story_moments.py `
  --workspace "D:\PauliFootageAI" `
  --limit 20 `
  --minimum 0.35
```

## Ranking inputs

- verified local vision status;
- verified local transcript status;
- visible temporal changes;
- evidence specificity;
- clip duration suitability;
- explicit uncertainty indicators.

The first ranking method is deterministic and intentionally simple. It is a baseline for evaluation, not a claim of editorial taste.

## Acceptance criteria

- Enriched clips rank above metadata-only clips.
- Silent vision-enriched footage remains eligible.
- Transcript evidence improves ranking without becoming mandatory.
- Scores remain between 0 and 1.
- Every result lists reasons.
- Output status remains `PROPOSED` and requires human review.
- Original source media remains unchanged.

## Next gate

Only after the shortlist is reviewed should Hermes create clip-level in/out proposals and export an OpenTimelineIO, EDL, or FCPXML draft. Screen control remains a later fallback after official editor APIs are tested.
