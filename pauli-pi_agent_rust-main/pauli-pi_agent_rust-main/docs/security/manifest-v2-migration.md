# Extension Capability Manifest v2 Migration Notes

Status: Draft  
Primary bead: `bd-f0huc`  
Last updated: 2026-02-14

## 1. What Changes in v2

`capability_manifest.schema` adds `pi.ext.cap.v2` with stricter per-capability metadata:

- `intents`: declared behavioral intent classes
- `connector_classes`: expected connector families used by this capability
- `hostcall_classes`: expected hostcall classes
- `provenance`: source + integrity + publisher attestations
- `risk_tier`: optional explicit risk label

Schema source of truth: `docs/schema/extension_manifest.json`.

## 2. Field Mapping (v1 -> v2)

| v1 field | v2 field(s) | Notes |
|---|---|---|
| `capability` | `capability` | unchanged, now enum-constrained |
| `methods` | `connector_classes`, `hostcall_classes` | split by abstraction layer |
| `scope.paths` | `scope.paths` | unchanged |
| `scope.hosts` | `scope.hosts` | unchanged |
| `scope.env` | `scope.env` | unchanged |
| none | `intents` | new required field |
| none | `provenance` | new required field |
| none | `risk_tier` | new optional field |

## 3. Fail-Closed Parser Rules (Target Runtime Behavior)

Runtime parser/validator for v2 should fail closed when:

1. `schema` is unsupported.
2. Any required v2 field is missing.
3. `capability`, `intents`, `connector_classes`, or `hostcall_classes` includes unknown values.
4. `provenance.integrity.digest` is not valid SHA-256 hex.
5. Unknown keys appear inside v2 manifest objects (`additionalProperties: false`).

## 4. Example v2 Capability Entry

```json
{
  "capability": "http",
  "intents": ["network_egress"],
  "connector_classes": ["http"],
  "hostcall_classes": ["http"],
  "risk_tier": "high",
  "scope": {
    "hosts": ["api.example.com"]
  },
  "provenance": {
    "source": "registry",
    "integrity": {
      "algorithm": "sha256",
      "digest": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    },
    "publisher": {
      "id": "example-org",
      "verification": "registry_attested"
    }
  }
}
```

## 5. Rollout Sequence

1. Add schema + docs (this change).
2. Add runtime parser/validation for v2 in `src/extensions.rs`.
3. Add strict negative tests for malformed/unknown critical fields.
4. Add extension package migration guide updates once parser enforcement lands.
