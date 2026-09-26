# Riverside Flow — API + Data Contract

## Principle
Manual-first, API-ready. Prove the workflow before paying for enterprise/API access or building custom integration.

## Riverside API boundary
Riverside currently lists API access under Business. Treat API availability, scopes, endpoints, rate limits, and authentication as plan-dependent and re-verify official documentation before implementation.

Future adapter name: `riverside_provider`

The rest of Hermes must depend on provider-neutral contracts rather than Riverside-specific payloads.

## Environment-only credentials
Future secrets must be referenced only through environment variables or the existing secret manager. Suggested names:

- `RIVERSIDE_API_KEY`
- `RIVERSIDE_API_SECRET` only if required by current auth scheme
- `RIVERSIDE_WEBHOOK_SECRET` only if webhooks are supported/used

Never commit real values, sample live tokens, cookies, bearer headers, or exported session credentials.

## Minimal future provider interface

### list_recordings
Input: studio/project identifier, time window.
Output: normalized recording metadata.

### get_recording
Output must include when available:
- provider recording id
- project/studio id
- title
- participants
- created_at
- duration
- source URLs or provider file handles
- processing state

### get_transcript
Normalize:
- speaker
- start timestamp
- end timestamp when available
- text
- transcript version

### get_tracks
Normalize available high-quality participant audio/video tracks without assuming permanent public URLs.

### get_exports
Return available long-form edits/clips/export metadata.

### create_export / request_edit
Only implement if current Riverside API explicitly supports the operation and the active account is authorized.

## Derivative manifest
Every generated asset should be able to point back to:

- source_provider: riverside
- source_recording_id
- source_project/studio
- transcript_version
- source timestamps
- guest(s)
- consent boundary
- narrative_spine_version
- derivative_type
- derivative_version
- creator agent/tool
- approval status
- published destination(s)
- analytics ids where available

## Relationship / Opportunity Graph schema
Do not bury commercial and mission outcomes inside content analytics.

Suggested normalized record:

- person_id
- public/provided name
- organization
- role
- relationship type
- expertise
- resources offered or discussed
- needs/problems explicitly stated
- interests explicitly stated
- funding/sponsorship interests explicitly stated
- mentor/volunteer capability explicitly stated
- opportunities discussed
- introductions offered/requested
- geography
- source recording + timestamps
- evidence label
- consent/privacy level
- next action
- owner
- due date
- outcome

Do not infer sensitive personal traits. Do not publish or share private relationship intelligence outside its authorized context.

## Event model for future automation

Possible internal events:

- `riverside.recording.ready`
- `riverside.transcript.ready`
- `riverside.derivatives.ready`
- `riverside.experiment.started`
- `riverside.metrics.checkpoint_due`
- `riverside.relationship.followup_due`

These are internal contract names only. Do not claim Riverside emits matching webhooks until current docs prove it.

## Manual bridge before API
Until Riverside API is wired, Hermes may operate from:
- exported TXT/SRT transcripts;
- downloaded MP4/WAV tracks;
- Riverside-generated clips/exports;
- operator-provided recording/project links or identifiers;
- manually exported analytics screenshots/CSV/values;
- connected publishing-platform analytics.

Manual import must preserve provenance and never be represented as live API automation.

## Build trigger
Only implement Riverside API integration after at least one of these is true:
1. manual workflow repeatedly costs enough labor to justify integration;
2. a paying client requires API-scale delivery;
3. the media operation exceeds manual reliability;
4. Riverside Business is already justified for independent reasons.

Before coding, SPEC the exact endpoints, authentication, scopes, webhook behavior, rate limits, retention, and rollback.
