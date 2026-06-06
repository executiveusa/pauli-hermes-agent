# Infisical Bootstrap Runbook (2026-04-23)

## Bootstrap Reality (Must Be Accurate)
A bootstrap credential is required. There is no secure runtime path without one initial trust anchor.

## Required Inputs (External)
- Infisical project + environment IDs.
- Machine identity (or equivalent) bootstrap token.
- Secret path policy defining readable keys.

## Bootstrap Procedure
1. Create machine identity with least-privilege secret access.
2. Store bootstrap token in deployment platform secret store (not repo).
3. On startup, exchange bootstrap token for scoped runtime access.
4. Hydrate required runtime secrets into process env/memory.
5. Run secret health checks (presence + scope), never log values.

## Separation Rules
- Bootstrap secret: one credential solely for Infisical auth bootstrap.
- Managed secrets: all operational tokens/keys delivered by Infisical.

## External Blockers for Full Validation
- No Infisical tenant/project credentials available in this environment.
