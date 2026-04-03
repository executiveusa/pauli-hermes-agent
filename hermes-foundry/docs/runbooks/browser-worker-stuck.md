# Runbook: Browser Worker Stuck
1. Check run timeline (`/events`).
2. If CAPTCHA/MFA, keep paused and request human intervention.
3. If no progress 10m, stop sub-agent and restart run safely.
