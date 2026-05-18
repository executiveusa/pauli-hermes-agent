# Twilio Voice Setup Runbook (2026-04-23)

## Purpose
Run Hermes as a talk-first assistant via Twilio voice.

## Required Inputs (External)
- Twilio account SID/auth credentials.
- Twilio voice-capable phone number.
- Public webhook URL for inbound voice callbacks.
- STT/TTS provider credentials and model config.

## Required Runtime Capabilities
1. Outbound calling trigger.
2. Inbound call handler.
3. Speech-to-text and text-to-speech bridge.
4. Session persistence mapped to Hermes session IDs.
5. Transcript + summary storage.
6. Safe fallback messaging for voice errors/timeouts.

## Validation Checklist
- Outbound call reaches target and receives synthesized greeting.
- Inbound call routes into agent loop.
- Transcript and summary are persisted.
- Failures degrade safely without crashing the runtime.

## External Blockers for Full Validation
- No Twilio voice credentials/phone number/webhook endpoint are configured in this environment.
