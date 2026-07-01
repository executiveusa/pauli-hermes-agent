# Stage 06_browser_verification: Browser Verification


## Inputs

- Deployment URL
- `skills/browser-smoke-check/SKILL.md`
- `config/production-policy.json`

## Process

1. Fetch URL.
2. Follow redirects.
3. Detect HTTP status.
4. Detect title/H1/hero-ish content.
5. Reject generic 404, blank shells, auth walls, and Vercel error pages.

## Outputs

- `runs/<run-id>/browser-check.json`
- `runs/<run-id>/browser-evidence.md`

## Gate

Complete when visible. If not visible, proceed to Stage 07 if the next action is no-code-safe.

