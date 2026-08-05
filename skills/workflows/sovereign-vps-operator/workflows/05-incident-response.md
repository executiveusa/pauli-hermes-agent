# Workflow 05 — Incident Response

## Human input at beginning

Confirm incident owner, communication channel, affected customers, business priority, allowed containment actions, evidence-retention needs, regulator or client notification requirements, and recovery authority.

When the incident is active, do not delay immediate low-risk observation while waiting for nonessential context.

## Detect and preserve

1. Record detection time, reporter, symptoms, affected targets, and current business impact.
2. Capture volatile evidence: processes, listeners, connections, containers, resource state, authentication events, recent deployments, and relevant logs.
3. Protect evidence from modification; redact secrets in shared reports.
4. Establish an incident timeline and confidence level.

## Classify

```text
SEV-1: active compromise, broad outage, or material data risk
SEV-2: major degraded service or contained security event
SEV-3: limited failure with workaround
SEV-4: anomaly requiring investigation
```

Classify impact separately from certainty. Unknown cause does not lower severity.

## Contain

Hermes may execute only pre-authorized reversible containment. Network isolation, account revocation, credential rotation, firewall modification, destructive process termination, or customer-impacting shutdown requires human approval unless an emergency policy explicitly delegates it.

Preserve owner access and an out-of-band recovery route.

## Diagnose and recover

- correlate logs, changes, dependencies, capacity, and external provider events;
- identify confirmed cause, contributing conditions, and unproven hypotheses;
- select the smallest recovery action with rollback;
- restore from known-good artifacts or backups only under the required gate;
- invoke Workflow 03 for recovery validation and Workflow 04 for controlled release when applicable.

## Independent verification

A verifier confirms service recovery, security posture, data integrity, customer flows, monitoring, backup continuity, and containment effectiveness. Continue observation for a defined stabilization window.

## Human review at end

Deliver an incident report containing timeline, impact, cause confidence, actions, evidence, customer/data implications, recovery proof, unresolved risk, required notifications, and prevention proposals. Human closes or reopens the incident.
