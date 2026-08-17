# Sollo Browser Provider

## Role

Sollo is an execution provider for the Faceless YouTube Channel OS. It is not the strategist, governor, evidence store, or approval authority.

Use Sollo when an authenticated session already exists or after the owner completes login.

## Observed benchmark workflow

The One Person Business corpus shows Sollo being used for YouTube production by:

1. opening the Content area;
2. opening a YouTube Script Writer;
3. describing the video/topic;
4. supplying a title;
5. adding subject-specific details and research;
6. adding reference YouTube links when modeling structural writing style;
7. selecting a monetization goal;
8. selecting/increasing target video length;
9. generating the script;
10. continuing into voiceover and other production tools.

Hermes must use those capabilities to produce original work, not closely imitate another creator's expression.

## Browser control contract

Preferred execution order:

1. HyperAgent browser runtime;
2. Playwright/CDP deterministic control;
3. semantic/vision recovery when selectors drift.

Authentication is human-assisted:

- owner enters credentials;
- owner handles CAPTCHA/MFA;
- session artifacts stay outside Git and outside receipts;
- never print or persist raw cookies/tokens.

## Provider interface

Hermes should expose Sollo internally through a CLI/API adapter with these logical calls:

- `auth_status()`
- `open_content_studio()`
- `create_youtube_script(brief)`
- `create_voiceover(script, voice_settings)`
- `create_video(production_manifest)` when supported/proven
- `download_artifact(id)`
- `screenshot_proof()`
- `resume_job(job_id)`

## Script brief

```json
{
  "topic": "",
  "title": "",
  "research": {
    "text": "",
    "source_urls": [],
    "source_manifest": ""
  },
  "style_references": [
    {"youtube_url": "", "reason": "structural_reference_only"}
  ],
  "monetization_goal": "ad_revenue|affiliate|product|community|lead_generation|other",
  "target_length_minutes": 12,
  "channel_context": {
    "niche": "",
    "audience": "",
    "voice": "",
    "content_pillars": []
  }
}
```

## Required receipt

Every Sollo action returns:

```json
{
  "job_id": "",
  "stage": "script|voice|video",
  "status": "completed|blocked|failed",
  "input_hash": "",
  "artifact_paths": [],
  "artifact_hashes": [],
  "page_url": "",
  "screenshots": [],
  "started_at": "",
  "completed_at": "",
  "blocker": null
}
```

Never store credentials or raw session material in the receipt.

## Idempotency

Before spending generation credits, compute an idempotency key from:

`channel + workflow_stage + package_lock_hash + input_hash`

Reuse an identical verified artifact unless `force=true` or the critic requires regeneration.

## Autonomous behavior

Hermes may automatically run Sollo for draft/script/voice/video generation when:

- the account is already authenticated;
- the operation is within an approved plan/budget;
- the input package passed the prior workflow gate;
- no account/settings mutation is required.

Interrupt the owner only for:

- login/CAPTCHA/MFA;
- new paid plan/credit purchase;
- unexpected account permission changes;
- material provider failure requiring a strategic choice.

## Failure policy

If the expected UI changes:

1. capture screenshot;
2. capture safe DOM/semantic snapshot;
3. retry stable accessibility locators;
4. run HyperAgent semantic recovery;
5. retry once;
6. return `UI_CONTRACT_CHANGED` and route to an alternate approved provider when possible.

Do not repeatedly click coordinates or spend generation credits blindly.

## Publishing boundary

Sollo-generated artifacts are drafts until they pass Hermes independent QA. Sollo must never be allowed to bypass the Hermes publish-approval policy.
