# Extension Lockfile Format (`pi.package_lock.v1`)

This document specifies the deterministic extension lockfile format used by Pi
to verify package integrity and track trust state across install, update, and
load operations.

## Schema

```json
{
  "schema": "pi.package_lock.v1",
  "entries": [
    {
      "identity": "npm:my-extension",
      "source": "npm:my-extension@1.2.3",
      "source_kind": "npm",
      "resolved": {
        "kind": "npm",
        "name": "my-extension",
        "requested_spec": "npm:my-extension@1.2.3",
        "requested_version": "1.2.3",
        "installed_version": "1.2.3",
        "pinned": true
      },
      "digest_sha256": "a1b2c3d4e5f6...",
      "trust_state": "trusted"
    }
  ]
}
```

## Fields

### Top-level

| Field     | Type   | Description                          |
|-----------|--------|--------------------------------------|
| `schema`  | string | Always `"pi.package_lock.v1"`        |
| `entries` | array  | Ordered list of `PackageLockEntry`   |

### `PackageLockEntry`

| Field           | Type   | Description                                       |
|-----------------|--------|---------------------------------------------------|
| `identity`      | string | Stable identity key (`npm:{name}`, `git:{repo}`, `local:{path}`) |
| `source`        | string | Original source specification                     |
| `source_kind`   | enum   | `"npm"`, `"git"`, or `"local"`                    |
| `resolved`      | object | Resolved provenance (tagged union, see below)     |
| `digest_sha256` | string | Lowercase hex SHA-256 digest of package contents  |
| `trust_state`   | enum   | `"trusted"` or `"rejected"`                       |

### `PackageResolvedProvenance` (tagged by `kind`)

**NPM:**

| Field                | Type           | Description                        |
|----------------------|----------------|------------------------------------|
| `kind`               | `"npm"`        | Discriminator                      |
| `name`               | string         | NPM package name                   |
| `requested_spec`     | string         | Original install spec              |
| `requested_version`  | string or null | Semver spec if provided            |
| `installed_version`  | string         | Actual installed version           |
| `pinned`             | bool           | Whether version is exact-pinned    |

**Git:**

| Field              | Type           | Description                          |
|--------------------|----------------|--------------------------------------|
| `kind`             | `"git"`        | Discriminator                        |
| `repo`             | string         | `owner/repo` slug                    |
| `host`             | string         | Git host (e.g., `github.com`)        |
| `path`             | string         | Relative path within repo            |
| `requested_ref`    | string or null | Branch, tag, or commit ref           |
| `resolved_commit`  | string         | Full resolved commit SHA             |
| `origin_url`       | string or null | Git remote URL                       |
| `pinned`           | bool           | Whether ref is an exact commit SHA   |

**Local:**

| Field           | Type      | Description                    |
|-----------------|-----------|--------------------------------|
| `kind`          | `"local"` | Discriminator                  |
| `resolved_path` | string    | Canonicalized absolute path    |

## Digest Computation

SHA-256 digests are computed deterministically:

1. **Single file**: `SHA256("file\0" + filename + "\0" + content_without_CR + "\0")`
2. **Directory**: Files are collected recursively (excluding `.git/`), sorted
   by relative POSIX path, then hashed as a stream of
   `"file\0" + relative_path + "\0" + content_without_CR + "\0"`.
3. **CR normalization**: All `\r` bytes are stripped before hashing, ensuring
   cross-platform determinism (Windows CRLF vs Unix LF).

The output is lowercase hex-encoded (64 characters for SHA-256).

## Verification Semantics

### Install (first-time)

When no prior entry exists for the identity, the package is recorded as
`trust_state: "trusted"` with reason code `first_seen`.

### Install (already tracked)

If a lockfile entry already exists:

- **Digest match + provenance match**: Passes with reason code `verified`.
- **Digest mismatch**: **Fails closed** with code `digest_mismatch`.
  Remediation: `pi remove <source> && pi install <source>`.
- **Provenance mismatch**: **Fails closed** with code `provenance_mismatch`.
  Remediation: `pi remove <source> && pi install <source>`.

### Update

For unpinned sources (non-exact semver for NPM, non-commit-SHA for Git):

- Provenance and digest changes are **allowed** (the update is the intent).
- Pinned sources reject any mutation.

### Fail-Closed Guarantees

1. Any verification failure blocks the operation and emits a structured error
   with `code`, `reason`, and `remediation` fields.
2. Rejections are recorded in the trust audit log before returning.
3. No partial state: either the lockfile is updated atomically or the
   operation fails without side effects.

## Trust Audit Log

Every lockfile transition (success or failure) is appended to
`package-trust-audit.jsonl` in the appropriate scope directory:

- **Project**: `.pi/package-trust-audit.jsonl`
- **User (global)**: `~/.pi/package-trust-audit.jsonl`

### Audit Event Schema (`pi.package_trust_audit.v1`)

```json
{
  "schema": "pi.package_trust_audit.v1",
  "timestamp": "2026-02-14T08:00:00.000Z",
  "action": "install",
  "scope": "project",
  "source": "npm:my-extension@1.2.3",
  "identity": "npm:my-extension",
  "from_state": "untracked",
  "to_state": "trusted",
  "reason_codes": ["first_seen"],
  "remediation": null,
  "details": { "...full PackageLockEntry..." }
}
```

### Reason Codes

| Code                  | Meaning                                      |
|-----------------------|----------------------------------------------|
| `first_seen`          | No prior entry; package is new                |
| `verified`            | Digest and provenance match existing entry    |
| `provenance_changed`  | Resolved provenance differs (allowed updates) |
| `digest_changed`      | Content digest differs (allowed updates)      |
| `provenance_mismatch` | Provenance changed during install (blocked)   |
| `digest_mismatch`     | Digest changed during install (blocked)       |

## Entry Ordering

Entries are sorted deterministically by `(identity, source)` using lexicographic
comparison. This guarantees that identical inputs produce byte-identical lockfile
JSON output.

## File Locations

| Scope     | Lockfile path                | Audit log path                           |
|-----------|------------------------------|------------------------------------------|
| Project   | `.pi/packages.lock.json`     | `.pi/package-trust-audit.jsonl`          |
| User      | `~/.pi/packages.lock.json`   | `~/.pi/package-trust-audit.jsonl`        |
| Temporary | *(no lockfile)*              | *(no audit log)*                         |

## Determinism Guarantee

The lockfile format is designed for reproducible builds:

- Same source + same filesystem content = same `digest_sha256`.
- Same entries in any insertion order = same sorted output.
- Same input conditions = identical JSON output (no timestamps in lockfile).
- Audit events include timestamps but the lockfile itself is timestamp-free.
