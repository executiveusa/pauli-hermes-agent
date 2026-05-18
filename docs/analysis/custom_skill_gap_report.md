# Custom Skill Gap Report

## Gaps Closed In This Patch

- Added Pauli-specific skill documents for engineering, secrets, deployment, memory, design, voice, video, reasoning, and studio workflows.
- Added router and profile manifests for bounded skill selection.
- Added a router adapter for task-to-skill matching and budget enforcement.
- Added repo-local custom skill resolution through absolute skill paths.

## Remaining External Dependencies

- `pauli-coolify-ops` needs valid `COOLIFY_BASE_URL` and `COOLIFY_API_KEY`.
- `pauli-vercel-ops` needs `VERCEL_TOKEN` for live inspection.
- `pauli-infisical-secrets` needs working Infisical auth for source-of-truth mode.
- `pauli-twilio-voice` needs `TWILIO_ACCOUNT_SID` and `TWILIO_AUTH_TOKEN`.
- `pauli-supabase-memory` needs a real Supabase project URL and token or service key.
- `pauli-video-watch` needs local `yt-dlp` and `ffmpeg`.
- `pauli-openmontage-studio` and `pauli-fal-ai` are dry-run first and stay gated from paid calls.

## Integration Notes

- Repo-local custom skills now have a viable loading path through absolute-path skill resolution.
- The Pauli router can preload skill payloads for single-query CLI runs.
- Interactive routing is still profile-led or explicit-skill-led because there is no task text yet at startup.
