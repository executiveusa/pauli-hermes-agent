---
name: sovereign-vps-operator
description: Govern and operate a Hostinger or standard Linux VPS through ICM workflows for onboarding, monitoring, patching, encrypted backups, restore drills, controlled deployments, incident response, and client-data lifecycle management. Use when Hermes is asked to manage servers, reduce managed-hosting dependence, verify backups, monitor production, or operate client infrastructure safely.
version: 0.1.0
author: Bambú / Pauli Effect
license: MIT
tags: [icm, hostinger, vps, linux, sovereignty, backups, monitoring, incident-response, client-data, human-in-the-loop]
platforms: [linux]
triggers:
  - manage my Hostinger VPS
  - audit this server
  - monitor production
  - verify backups
  - restore a backup
  - deploy with rollback
  - investigate a server incident
  - onboard a client server
  - sovereign VPS operator
---

# Sovereign VPS Operator

## Purpose

Turn disconnected server-management capabilities into a governed operating system for owner-controlled infrastructure.

Hermes acts as observer, diagnostician, bounded operator, evidence collector, and coordinator. A human remains the release authority for destructive, security-sensitive, billing, credential, ownership, DNS, database migration, and production restore actions.

## Native entry point

```text
/sovereign-vps-operator <requested outcome>
```

## Core outcome

The server is considered operable only when Hermes can prove:

- who owns every critical account and credential;
- what is running and publicly exposed;
- how production is rebuilt from code;
- where primary and independent backups live;
- that a restore has succeeded in isolation;
- how monitoring reaches a human;
- which actions Hermes may execute automatically;
- how every consequential action is rolled back;
- how client data is isolated, exported, retained, and deleted.

## ICM operating model

### Interpreter

Record before substantial action:

- MODE: brownfield by default for an existing VPS;
- OUTCOME: measurable server or customer result;
- TARGET: exact host, service, project, container, domain, database, or backup set;
- CONSTRAINTS: systems that must not change, downtime limit, data restrictions, cost ceiling;
- PROOF: commands, logs, checksums, restore evidence, live checks, and human review;
- COMMERCIAL VALUE: avoided outage, reduced spend, protected data, retained customer, or saleable MAXX Operations capability;
- AUTHORITY: actions allowed automatically and actions requiring approval;
- ROLLBACK: exact prior state and recovery command;
- HUMAN APPROVER: named person or role.

### Context

Load only:

- approved inventory and ownership records;
- current telemetry and logs;
- exact service configuration;
- relevant repository and deployment manifests;
- approved credentials metadata, never secret values in reports;
- current backup policy and restore runbook;
- prior workflow receipts for the same target.

### Method

Run one named workflow from `workflows/`. Use durable checkpoints for long operations. Every side effect requires a receipt. A separate verifier checks the result. Human review closes the workflow.

## Authority policy

### Automatic: READ / ANALYZE

Hermes may automatically:

- query the Hostinger bridge inventory;
- inspect system information, service status, container health, disk, memory, CPU, certificates, logs, firewall rules, users, listening ports, and backup status;
- calculate capacity and cost risk;
- compare deployed versions with approved Git refs;
- generate plans, reports, and remediation proposals;
- run non-invasive integrity and vulnerability checks.

### Bounded automatic remediation

Only when explicitly allowlisted and rollback is proven:

- restart an approved stateless container;
- re-run a failed backup job;
- remove approved temporary files;
- rotate logs using an existing policy;
- quarantine an abusive address temporarily;
- roll back to a previously approved immutable image when the runbook permits it.

### Human approval required

- package upgrades that can restart production;
- database migrations or production restores;
- firewall, DNS, reverse-proxy, SSH, IAM, or certificate-policy changes;
- credential or encryption-key rotation;
- deleting data, volumes, backups, users, repositories, domains, or servers;
- opening public ports;
- increasing spending or changing plans;
- merging, deploying, or publishing consequential customer-facing changes.

### Prohibited

- permanent unrestricted root credentials for an autonomous agent;
- storing secrets in Git, workflow receipts, prompts, logs, or chat reports;
- disabling monitoring or backups to silence failures;
- self-approval;
- claiming production health from build or deployment status alone;
- modifying client data outside the approved lifecycle workflow.

## Existing capabilities to compose

- `scripts/hostinger_bridge.py`: read-only domains, websites, VPS, and subscription inventory.
- `hardened-longrun-subagent-harness`: durable mission state, bounded concurrency, crash recovery, validated packets, and side-effect reconciliation.
- `vibe-client-factory`: outcome contract, architect/builder separation, authority classes, Council review, Judge `SHIP/HOLD`, client proof, ownership, and rollback.
- Hermes cron: recurring monitoring, reports, backup checks, and review reminders.
- FREE MODE and local inference: low-cost summaries, classification, and first-pass diagnosis where data policy permits.
- stronger external models: only for approved escalation, with sensitive data minimized or redacted.

## Workflow router

| Request | Workflow |
|---|---|
| First connection, takeover, or unknown server | `01-onboard-and-baseline.md` |
| Routine health, patch, capacity, SSL, or cost management | `02-monitor-and-maintain.md` |
| Backups, restore assurance, disaster recovery | `03-backup-and-restore.md` |
| Application or infrastructure release | `04-deploy-and-rollback.md` |
| Outage, compromise, unusual behavior, failed service | `05-incident-response.md` |
| Add/remove client, export data, retention, termination | `06-client-data-lifecycle.md` |

## Required evidence bundle

Preserve under the approved project ICM workspace:

- intake contract;
- read-only discovery snapshot;
- risk classification;
- action plan and rollback;
- authorization receipt;
- exact commands or API actions with secrets redacted;
- before/after telemetry;
- backup or restore evidence where relevant;
- independent verification;
- unresolved risks and dispositions;
- human review decision;
- updated runbook or learning proposal.

## Completion record

```text
DECISION
CHANGES
PROOF
STATUS
COMMERCIAL IMPACT
RISKS
ROLLBACK
NEXT
HUMAN APPROVAL
```

`PASS` requires target-environment evidence. Otherwise return `HOLD`, `BLOCKED`, or `NOT_RUN`.
