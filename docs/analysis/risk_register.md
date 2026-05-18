# Risk Register (Snapshot: 2026-04-23)

## Critical Risks
1. **Upstream divergence cannot be computed locally**
   - No configured git remotes in this environment (`git remote -v` returns empty), so upstream comparison cannot be performed.
2. **Credential-gated integrations are not verifiable end-to-end**
   - GitHub/Vercel/Infisical/Twilio production validation requires external credentials/accounts not present in this environment.
3. **Program scope exceeds single-pass safe delivery**
   - Requested workstreams span backend adapters, webhook security, voice stack, dashboard UX, CI, and deployment automation.
4. **Deployment target specificity is external**
   - Hostinger/Coolify tenant/project identifiers, domains, and secret mounts are required for full rollout validation.

## Mitigations Executed in This PR
- Updated discovery artifacts for repo/runtime/workflow/env/risk/merge status.
- Produced explicit architecture + PRD + runbook guidance for staged implementation.
- Documented blocker boundaries precisely to prevent false "green" claims.

## Remaining External Blockers
- Upstream remote URL/branch access for divergence and merge hygiene.
- Integration credentials and project identifiers for live end-to-end verification.
