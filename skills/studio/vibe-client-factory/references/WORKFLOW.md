# Vibe Client Factory — Execution Runbook

## 1. Outcome-first entry

Start every run by writing a small control packet:

```text
MODE
OUTCOME
TARGET
AUDIENCE
CONSTRAINTS
PROOF
COMMERCIAL VALUE
ROLLBACK
HUMAN APPROVER
```

Do not begin implementation until the outcome can be explained in one plain-language sentence.

## 2. Brownfield inspection order

For an existing project, inspect in this order:

1. repository metadata, current branch, open pull requests, and latest verified commit;
2. project governance files and existing workflow contracts;
3. package manager, framework, runtime, tests, and deployment configuration;
4. public product behavior and mobile experience;
5. database, authentication, forms, analytics, and external integrations;
6. client-approved content, brand assets, and prior decision history;
7. ownership, credentials, domains, hosting, billing, and rollback records;
8. current commercial or mission objective.

Record conflicts rather than silently choosing one source.

## 3. Source-of-truth order

Use this default precedence unless the project defines a stricter order:

1. applicable `AGENTS.md` or constitutional policy;
2. explicit current human approval;
3. signed contract or approved specification;
4. current project control record;
5. accepted decision receipt;
6. verified production evidence;
7. repository implementation;
8. historical notes and chat transcripts;
9. inference.

Inference can propose a decision. It cannot create authority.

## 4. Information reduction

Convert all discovered information into four queues:

### Confirmed facts

Facts with a named source and current evidence.

### Reasonable inferences

Likely interpretations that are safe for planning but not for consequential action.

### Unresolved decisions

Choices with real consequences that require an owner.

### Missing critical information

Unknowns that block safe progress.

Do not show the client the entire research record. Surface only the smallest decision set required to continue.

## 5. Decision-card procedure

Each decision card must:

- ask one question;
- contain exactly two real options by default;
- explain why the decision matters;
- include Hermes' recommendation and rationale;
- state the consequence of each option;
- link to evidence or a visual preview;
- identify the authority needed;
- define the next bounded action the selection authorizes;
- state what the selection does not authorize.

### Decision statuses

```text
DRAFT
READY_FOR_CLIENT
ANSWERED
SUPERSEDED
CANCELLED
```

### Client interaction rule

The preferred sequence is:

```text
notify -> open card -> review recommendation -> choose A/B -> receipt -> factory continues
```

A voice note may be accepted as a response, but Hermes must translate it into a structured decision and request confirmation when the meaning is ambiguous.

## 6. Client app specification

### Product promise

"Make a clear decision. Receive verified progress. Own the result."

### Primary navigation

```text
TODAY
PROGRESS
PROOF
OWNERSHIP
HELP
```

### Today

Show no more than three items. Prioritize the single blocking decision.

### Progress

Show the active factory station, plain-language status, and expected next gate. Do not expose fabricated percentages.

### Proof

Show completed results, checks performed, live verification state, known limitations, and rollback availability.

### Ownership

Show the owner of the domain, repository, data, hosting, credentials, assets, and documentation. Unknown ownership must be displayed as unknown.

### Help

Allow a voice note, image, file, or short question. Do not use an unbounded chat surface as the primary workflow.

### Access default

1. send a short-lived secure invitation;
2. bind the accepted invitation to the intended project and recipient;
3. enroll the device;
4. use a passkey, biometric unlock, or secure device credential for later access;
5. allow revocation and recovery;
6. require stronger confirmation for sensitive actions.

"No login" means no remembered password, not no security.

## 7. Factory stations

### Outcome

Create the contract and commercial classification.

### Investigate

Build the evidence-backed project baseline.

### Decide

Resolve only the decisions blocking the next bounded action.

### Architect

Create the directive, acceptance tests, authority envelope, and rollback.

### Build

Implement in isolation. Preserve evidence and avoid scope expansion.

### Verify

Run native checks, specialist review, usability evidence, and independent code review.

### Judge

Return only `SHIP` or `HOLD`.

### Release

Perform the separately approved state-changing action.

### Prove

Verify the live target and return a client-facing receipt.

### Improve

Propose workflow changes through the same governance system.

## 8. Authority enforcement

Hermes must classify every intended action before executing it.

### Automatic

- read approved files and public sources;
- analyze and compare;
- draft clearly labeled proposals;
- create reversible local artifacts;
- run non-destructive tests.

### Logged and bounded

- create a branch or worktree;
- modify files inside an approved scope;
- create preview deployments when permitted;
- generate client review artifacts.

### Approval required

- public copy or design changes;
- merge to a protected or production branch;
- production deployment;
- database migration;
- publishing;
- paid-credit consumption;
- new persistent integration.

### Dual confirmation required

- billing or purchases;
- credential rotation or disclosure;
- ownership transfer;
- destructive data changes;
- permanent remote access;
- legal or compliance representation.

### Never

- self-approve;
- fabricate proof;
- conceal failed checks;
- weaken authority rules to finish faster;
- use one client's data to train or improve another client's system without authorization;
- leave secrets in chat, logs, screenshots, commits, or generated packages.

## 9. Independent review contract

The Builder's completion message is evidence, not approval.

Review order:

1. inspect the directive;
2. inspect the diff and changed dependencies;
3. run native tests;
4. run security, privacy, accessibility, and architecture review where relevant;
5. run OpenCodeReview when available;
6. inspect client-facing behavior with browser or vision evidence;
7. confirm rollback;
8. classify findings;
9. produce dispositions;
10. pass the packet to a separate Judge.

## 10. Release receipt

A valid release receipt contains:

- project and release identifier;
- approved decision receipts;
- reviewed commit SHA;
- changed systems;
- test commands and results;
- review findings and dispositions;
- approver identity and timestamp;
- deployment identifier;
- live target checks;
- production status;
- rollback point;
- client-facing summary;
- unresolved limitations;
- commercial or mission result, marked verified or unverified.

Use `templates/run-receipt.schema.json`.

## 11. Commercial packaging

### Vibe Audit

Inspect ownership, architecture, risk, deployment, evidence, usability, and decision backlog. Deliver a recovery map and Vibe Score.

### Vibe Rescue Sprint

Repair one bounded, high-value defect with proof and rollback.

### Sovereign Launch

Deliver the client-owned application, repository, data, deployment, documentation, and handoff.

### MAXX Operations

Operate recurring bounded workflows, reviews, maintenance, content, monitoring, and client decisions.

Do not convert an unproven internal workflow into a SaaS platform before repeated paid delivery proves stable inputs and outputs.

## 12. Pilot procedure

Use one real project.

1. establish the brownfield baseline without changing production;
2. create one to three real decision cards;
3. have the client answer from a phone;
4. convert one answer into an Architect directive;
5. implement one bounded branch;
6. verify and review it;
7. obtain `SHIP` or `HOLD`;
8. obtain release approval;
9. deploy and verify live behavior;
10. return the proof to the client surface;
11. measure comprehension, decision time, defects, delivery time, and value;
12. improve the workflow only through a reviewed change.

## 13. Success measures

- client opens the app without password friction;
- decision completion takes less than one minute for a normal card;
- client correctly explains the consequence of the selected option;
- no agent exceeds its authority envelope;
- every production release maps to a reviewed commit and approval receipt;
- live verification is distinct from CI;
- ownership is complete or truthfully marked unknown;
- rollback is executable;
- the work creates or protects measurable value.
