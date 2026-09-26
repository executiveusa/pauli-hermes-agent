---
name: secret-safety-policy
description: Security policy for preventing credential leakage in files, prompts, logs, and exports.
---

# Secret Safety Policy

- Never commit secrets.
- Never echo secrets into logs.
- Use explicit redaction for sensitive values.
- Treat .env-style files as sensitive inputs, not shareable artifacts.
