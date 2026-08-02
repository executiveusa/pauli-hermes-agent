# Phase 3 — Local QVAC Transcription

## Purpose

Add searchable local speech evidence without weakening silent-footage support. Clips with no audio stream are completed as `no-audio-track`; they are not treated as failures and remain available for vision-only analysis.

## Configuration

Set the exact loaded transcription model alias in the workspace `config.json`:

```json
{
  "qvac_base_url": "http://127.0.0.1:11434/v1",
  "transcription_model": "<exact-loaded-alias>"
}
```

Verify aliases first:

```powershell
curl.exe http://127.0.0.1:11434/v1/models
```

## One-clip proof

```powershell
python skills/local-footage-studio/scripts/transcribe_qvac.py `
  --workspace "D:\PauliFootageAI" `
  --clip-id "<CLIP_ID>"
```

## Acceptance criteria

- Spoken clip creates transcript JSON and text files.
- Transcript becomes searchable in the existing FTS evidence index.
- A clip without an audio stream becomes `no-audio-track` without calling QVAC.
- Re-running replaces the transcript evidence block rather than duplicating it.
- Temporary WAV files are deleted automatically.
- Original media size and modification time remain unchanged.

## Limitations

This slice does not perform speaker diarization, word-level alignment, translation, audio event classification, or loudness-quality scoring. Those require separate proof slices.
