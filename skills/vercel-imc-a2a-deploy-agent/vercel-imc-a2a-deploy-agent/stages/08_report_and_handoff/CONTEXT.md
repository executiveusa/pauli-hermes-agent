# Stage 08_report_and_handoff: Report and Handoff


## Inputs

- All run JSON artifacts
- `subagents/report-writer/AGENT.md`

## Process

1. Render human-readable report.
2. Render machine-readable summary.
3. Include next manual approval required, if any.
4. Do not include secret values.

## Outputs

- `runs/<run-id>/report.md`
- `runs/<run-id>/summary.json`
- `stages/08_report_and_handoff/output/latest-report.md`

