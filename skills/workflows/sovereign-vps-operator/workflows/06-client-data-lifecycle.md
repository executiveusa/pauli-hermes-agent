# Workflow 06 — Client Data Lifecycle

## Human input at beginning

Confirm client legal/operational owner, approved processors, data classes, collection purpose, residency constraints, retention schedule, export format, access roles, deletion authority, and termination obligations.

## Onboard client data

1. Minimize collection to the stated purpose.
2. Assign a separate client project, database/schema, storage prefix or bucket, credentials, encryption context, backup scope, logs, and ownership record appropriate to risk.
3. Apply least privilege and prohibit shared human accounts.
4. Document every external processor and model provider that may receive client data.
5. Verify production and staging separation.
6. Test client export and recovery before declaring onboarding complete.

## Operate

- log administrative access and consequential actions;
- review stale users and service credentials;
- enforce retention and backup policy;
- prevent cross-client retrieval and accidental model/API disclosure;
- use local/private inference for sensitive data unless the client-approved policy permits external processing;
- verify row-level security or equivalent isolation where infrastructure is shared.

## Export or transfer

1. Confirm requester identity and authority.
2. Produce a portable, documented export with checksums.
3. Include schema, files, metadata, and restore/import instructions.
4. Transfer through an approved encrypted channel.
5. Record acceptance by the receiving owner.

## Revoke or offboard

1. Freeze new writes if required by the contract.
2. Export and verify client-owned data.
3. Transfer repositories, domains, credentials, documentation, and backup custody.
4. Revoke studio and automation access.
5. Apply retention holds before deletion.
6. Deletion from primary systems, backups, logs, and replicas requires explicit human authority and a recorded scope.
7. Produce a deletion or retention certificate without exposing protected contents.

## Independent verification

Verify access revocation, tenant isolation, export completeness, retained obligations, deletion scope, recovery implications, and absence of cross-client impact.

## Human review at end

Return ownership state, data locations, processors, access list, backup status, export proof, retention/deletion status, unresolved obligations, and final client approval.
