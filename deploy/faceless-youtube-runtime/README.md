# Faceless YouTube Runtime — VPS Deployment

This runtime is a sidecar capability inside Hermes, not a second daemon. Hermes' existing gateway process ticks the native cron scheduler every 60 seconds.

## Required server baseline

- Linux VPS with `git`, `uv`, Python 3.11-3.13, Node.js 22+, npm, and systemd.
- Hermes checkout, default `/opt/pauli-hermes-agent`.
- Internet egress to configured model/browser/Google providers.

## GitHub Actions deployment secrets

Configure repository Actions secrets (values never belong in Git):

- `HERMES_VPS_HOST` — required.
- `HERMES_VPS_USER` — required; must be able to install/restart Hermes gateway through sudo.
- `HERMES_VPS_SSH_KEY` — required private deployment key.
- `HERMES_VPS_PORT` — optional; defaults to 22.
- `HERMES_VPS_REPO_DIR` — optional; defaults to `/opt/pauli-hermes-agent`.

After these are present, `.github/workflows/deploy-faceless-youtube-vps.yml` deploys automatically when the runtime/Channel OS changes on `main`, or can be dispatched manually.

## Runtime/provider secrets on the VPS

Configure these in the Hermes service environment/approved secret store, never Git:

HyperAgent/Sollo:

- `HYPERAGENT_LLM_PROVIDER`
- `HYPERAGENT_LLM_MODEL`
- `HYPERAGENT_BROWSER_PROVIDER`
- `HYPERBROWSER_API_KEY` when Hyperbrowser is selected
- provider model API credential (for example `OPENAI_API_KEY`)

YouTube first-party data/publishing:

- `HERMES_YOUTUBE_CLIENT_SECRET_FILE`
- `HERMES_YOUTUBE_TOKEN_FILE`
- `HERMES_YOUTUBE_PUBLISH_ENABLED=0` by default

The OAuth token file is created with owner-only permissions after the normal Google consent flow. Publishing requires both an explicit runtime approval argument and `HERMES_YOUTUBE_PUBLISH_ENABLED=1`.

## First deployment

The workflow performs:

1. fetch/reset checkout to `origin/main`;
2. `uv sync --locked --extra google --extra youtube`;
3. install the isolated HyperAgent npm dependency;
4. run `python -m tools.faceless_youtube_runtime.cli doctor`;
5. idempotently register `faceless-youtube-channel-os` in Hermes cron (default every six hours);
6. install/refresh Hermes gateway as a system service;
7. print cron/runtime status as deployment evidence.

## Manual equivalent

```bash
cd /opt/pauli-hermes-agent
git fetch origin main && git reset --hard origin/main
uv sync --locked --extra google --extra youtube
(cd integrations/hyperagent && npm install --ignore-scripts --no-audit --no-fund)
uv run python -m tools.faceless_youtube_runtime.cli --repo-root "$PWD" doctor
uv run python scripts/install_faceless_youtube_runtime.py --repo "$PWD"
sudo .venv/bin/hermes gateway install --system
```

## Safe initial state

A healthy deployment does **not** imply Sollo or YouTube are authenticated. `doctor` reports provider readiness independently.

Autonomous stages may research, package, script, QA, prepare production, derive content, measure, and learn. Login/MFA/CAPTCHA, new spend, initial/material thesis changes, and public publishing remain owner gates.

## Rollback

Revert the runtime commit/PR, redeploy main, then pause/remove the `faceless-youtube-channel-os` cron job. Runtime state remains under `~/.hermes/faceless-youtube/` for forensic recovery unless deliberately removed.
