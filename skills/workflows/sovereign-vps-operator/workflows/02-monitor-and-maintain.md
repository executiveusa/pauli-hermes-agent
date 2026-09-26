# Workflow 02 — Monitor and Maintain

## Human input at beginning

Confirm service criticality, alert recipients, maintenance window, automatic-remediation allowlist, spending limits, patch policy, and acceptable resource thresholds.

## Scheduled observation

Use Hermes cron and read-only commands to check:

- service and container health;
- HTTP/TLS availability and certificate renewal state;
- CPU, memory, disk, inode, network, and database capacity;
- backup completion and repository reachability;
- failed logins, privilege changes, new listeners, and firewall drift;
- pending security updates and required reboots;
- application error rates and repeated failures;
- provider subscription, renewal, and cost anomalies;
- Git/deployment drift from approved versions.

## Triage

Classify each finding:

```text
HEALTHY
WATCH
ACTION_REQUIRED
INCIDENT
UNKNOWN
```

Attach evidence, confidence, blast radius, and the relevant runbook. Never convert missing telemetry into a healthy result.

## Action gate

- `HEALTHY` and `WATCH`: report and continue.
- Allowlisted reversible remediation: execute with before/after receipts.
- Package, network, identity, database, billing, or customer-impacting change: propose and request human approval.
- `INCIDENT`: invoke Workflow 05.

## Verification

After any action, a separate verifier checks service health, live endpoints, resource trends, backup status, and configuration drift. Failed verification triggers rollback or incident escalation.

## Human review at end

Deliver a plain-language operations report with uptime evidence, actions taken, actions awaiting approval, capacity forecast, cost risks, and recommended next maintenance decision.
