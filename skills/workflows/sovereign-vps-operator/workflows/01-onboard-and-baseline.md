# Workflow 01 — Onboard and Baseline

## Human input at beginning

Collect or resolve:

- owner and human approver;
- exact VPS/account target;
- approved access method;
- known domains, applications, databases, and customers;
- maximum acceptable downtime;
- systems that must not change;
- current backup claims;
- data sensitivity and jurisdiction;
- desired proof and commercial objective.

Do not request secret values in chat. Use the approved secret store or local environment.

## Read-only discovery

1. Query `scripts/hostinger_bridge.py` for domains, websites, VPS, and subscriptions.
2. Record provider, region, plan, IP metadata, renewal, and ownership.
3. Inspect OS/version, uptime, users, groups, SSH configuration, sudo access, listening ports, firewall, reverse proxy, containers, services, scheduled jobs, storage, memory, CPU, and network exposure.
4. Map every domain to DNS, proxy, application, database, storage, certificate, repository, and owner.
5. Locate source manifests, deployment configuration, secrets locations, backups, logs, monitoring, and restore instructions.
6. Identify public admin panels, shared credentials, unsupported packages, unencrypted services, and single-provider failure points.

## Classification

Return:

```text
CONFIRMED FACTS
REASONABLE INFERENCES
UNKNOWN / UNVERIFIED
CRITICAL EXPOSURES
OWNERSHIP GAPS
BACKUP GAPS
LOCK-IN RISKS
```

Assign severity and blast radius. Do not change the server during discovery.

## Plan and gate

Prepare a smallest-safe-change plan containing:

- immediate containment, if necessary;
- target architecture;
- exact files/services permitted to change;
- maintenance window;
- rollback and recovery;
- required approvals;
- verification commands;
- unresolved risks.

A human approves the bounded remediation packet before execution.

## Bounded execution

Use an isolated change sequence. Preserve before-state configuration and package lists. Apply one reversible slice at a time. Stop on unexpected output, authority ambiguity, missing backup, or rollback failure.

## Independent verification

A verifier that did not execute the changes confirms:

- only approved ports are exposed;
- required services are healthy;
- owner access remains functional;
- no client data changed unexpectedly;
- monitoring and backups report correctly;
- rollback artifacts exist;
- inventory matches the live target.

## Human review at end

Present the baseline, changes, unresolved risks, ownership map, recovery path, and recommended next workflow. Human verdict: `ACCEPT`, `REMEDIATE`, or `HOLD`.
