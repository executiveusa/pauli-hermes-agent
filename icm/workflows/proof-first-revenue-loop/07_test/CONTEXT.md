# 07_test — run the authorized real test

## Inputs
- Working: `../06_approve/output/approval-packet.md` plus required approval receipt
- Reference: `../../../instructions/PROOF_FIRST_REVENUE_LOOP.md`

## Process
1. Use an owned/authorized environment.
2. Record baseline, variant ID, attribution method, window/budget, primary KPI, threshold and stop condition before launch.
3. Execute only inside the approved action envelope.

## Outputs
- `output/test-record.md`

## Human check
Require the applicable approval for send, publish, spend, contract or production actions.
