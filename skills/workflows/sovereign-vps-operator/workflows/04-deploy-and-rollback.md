# Workflow 04 — Deploy and Roll Back

## Human input at beginning

Confirm target environment, approved Git ref or image digest, maintenance window, acceptable downtime, database-change status, live acceptance criteria, rollback threshold, and release approver.

## Preflight

- verify repository, branch, commit, and artifact provenance;
- confirm native tests and independent review results;
- capture current image/config/database version and live health baseline;
- verify recent backup and tested rollback path;
- identify migrations, irreversible effects, DNS/cache impact, and customer communications;
- refuse release when target, authority, backup, or rollback is ambiguous.

## Gate

The Builder prepares the release. An independent reviewer evaluates evidence. The Judge returns `SHIP` or `HOLD`. A human authorizes production release.

## Bounded execution

1. Deploy immutable artifacts where possible.
2. Apply changes in the smallest reversible slice.
3. Preserve exact command/API receipts with secrets redacted.
4. Stop on unexpected drift, failed health checks, migration error, or exceeded authority.
5. Do not treat a successful build, CI run, merge, or provider deployment state as live proof.

## Live verification

Check:

- expected production URL and routes;
- representative user flow;
- authentication and authorization;
- database read/write behavior where approved;
- logs, latency, error rate, resource use, and external integrations;
- version/commit displayed by the live target;
- monitoring and backup continuity.

## Rollback

Trigger rollback when acceptance criteria fail or risk exceeds the approved threshold. Restore the previous immutable image/config; use database rollback only when explicitly designed and approved. Verify the rolled-back system live.

## Human review at end

Return release version, evidence, observed impact, unresolved risks, rollback readiness, and whether production is `PASS`, `PASS_WITH_DISPOSITIONS`, or `HOLD`.
