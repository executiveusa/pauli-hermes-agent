# Workflow 03 — Backup and Restore

## Human input at beginning

Confirm protected systems, recovery point objective, recovery time objective, retention, encryption-key owner, independent storage provider, offline-copy policy, restore target, and deletion authority.

## Discover and classify

Inventory databases, volumes, object storage, application configuration, DNS exports, repositories, secrets metadata, and provider snapshots. Distinguish:

- fast provider snapshot;
- independent encrypted backup;
- immutable or offline copy;
- unverified backup claim.

## Backup method

1. Quiesce or use application-consistent database backup methods.
2. Create logical database dumps where portable recovery is required.
3. Back up required files and configuration with Restic, Borg, Kopia, or the approved equivalent.
4. Encrypt before data leaves the server.
5. Send at least one copy outside the primary provider/account.
6. Record repository ID, timestamp, scope, checksum/integrity result, retention class, and key custodian.
7. Never place plaintext secrets in the archive manifest or report.

## Restore drill

1. Create an isolated recovery target.
2. Restore infrastructure configuration, application, database, and files in documented order.
3. Run schema, integrity, authentication, and representative application checks.
4. Confirm the restored system does not write to production services.
5. Measure actual recovery time and data freshness.
6. Destroy the temporary environment only after evidence is preserved and human approval is received when deletion is consequential.

## Gate

A backup cannot be marked verified until an isolated restore succeeds. Production overwrite or destructive restoration always requires explicit human approval.

## Independent verification

The verifier confirms backup completeness, encryption, provider separation, retention, restore functionality, checksum/integrity results, and absence of production side effects.

## Human review at end

Return actual RPO/RTO, restore evidence, missing datasets, key-custody risks, retention cost, and a `PASS`, `PASS_WITH_DISPOSITIONS`, or `HOLD` recommendation.
