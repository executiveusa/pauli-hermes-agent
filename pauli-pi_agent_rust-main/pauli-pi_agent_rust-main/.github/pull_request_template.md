## Summary
- What changed:
- Why:
- Risk level (low/medium/high):

## Definition of Done Evidence
- [ ] Unit evidence linked
- [ ] E2E evidence linked
- [ ] Extension evidence linked
- [ ] Failing paths include artifact links and repro commands

### Unit Evidence
- Primary run link(s):
- Notes:

### E2E Evidence
- Primary run link(s):
- Notes:

### Extension Evidence
- Primary run link(s):
- Notes:

## Reproduction Commands
```bash
cargo test --all-targets
cargo test --all-targets --features ext-conformance
./scripts/e2e/run_all.sh --profile ci
```

## Migration Guidance (Existing Feature Branches)
- If this branch was created before the DoD gate rollout, replace the PR body with this template before requesting merge.
- For historical failing runs, include direct artifact links plus the exact rerun command used to validate the fix.
