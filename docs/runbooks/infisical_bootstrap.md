# Infisical Bootstrap Runbook

## Bootstrap secret requirement
A bootstrap credential is required to authenticate Hermes runtime to Infisical (machine identity token or equivalent).

## Steps
1. Provision Infisical project/env.
2. Create machine identity + scoped secret access policy.
3. Inject bootstrap credential into deployment platform secret store.
4. Sync runtime secrets at boot; never log secret values.
