#![forbid(unsafe_code)]

use jsonschema::Validator;
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn schema_path() -> PathBuf {
    repo_root().join("docs/schema/session_store_v2_contract.json")
}

fn compiled_contract_schema() -> Validator {
    let path = schema_path();
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("Failed to read schema {}: {err}", path.display()));
    let schema: Value = serde_json::from_str(&raw)
        .unwrap_or_else(|err| panic!("Failed to parse schema {}: {err}", path.display()));

    jsonschema::draft202012::options()
        .should_validate_formats(true)
        .build(&schema)
        .unwrap_or_else(|err| panic!("Failed to compile schema {}: {err}", path.display()))
}

#[allow(clippy::too_many_lines)]
fn canonical_contract_bundle() -> Value {
    json!({
        "schema": "pi.session_store_v2.contract.v1",
        "manifest": {
            "schema": "pi.session_store_v2.manifest.v1",
            "store_version": 2,
            "session_id": "f4c03c8c-cf0a-4c90-9535-e95f2d02b393",
            "source_format": "jsonl_v3",
            "created_at": "2026-02-15T18:10:00Z",
            "updated_at": "2026-02-15T18:10:10Z",
            "head": {
                "segment_seq": 2,
                "entry_seq": 9,
                "entry_id": "entry_00000009"
            },
            "counters": {
                "entries_total": 9,
                "messages_total": 6,
                "branches_total": 1,
                "compactions_total": 1,
                "bytes_total": 4096
            },
            "files": {
                "segment_dir": "segments/",
                "segment_count": 2,
                "index_path": "index/offsets.jsonl",
                "checkpoint_dir": "checkpoints/",
                "migration_ledger_path": "migrations/ledger.jsonl"
            },
            "integrity": {
                "chain_hash": "f7a3b79f9d1b84444c34f6f6f1393ba55fba8c4d0868ac8f80a7b951907e8095",
                "manifest_hash": "7d6cf8f3ad3f8bc9f5a4191efebda5b2236a3a4387fa31dd619f024997f30871",
                "last_crc32c": "1A2B3C4D"
            },
            "invariants": {
                "parent_links_closed": true,
                "monotonic_entry_seq": true,
                "monotonic_segment_seq": true,
                "index_within_segment_bounds": true,
                "branch_heads_indexed": true,
                "checkpoints_monotonic": true,
                "hash_chain_valid": true
            }
        },
        "segments": [
            {
                "schema": "pi.session_store_v2.segment_frame.v1",
                "segment_seq": 1,
                "frame_seq": 1,
                "entry_seq": 1,
                "entry_id": "entry_00000001",
                "parent_entry_id": null,
                "entry_type": "message",
                "timestamp": "2026-02-15T18:10:00Z",
                "payload_sha256": "2d66234f0f7a6f4fcf5b37ab54fef9cb79373ca4ac75734f84f3f1a8ac26bf58",
                "payload_bytes": 128,
                "payload": {
                    "role": "user",
                    "text": "hello"
                }
            },
            {
                "schema": "pi.session_store_v2.segment_frame.v1",
                "segment_seq": 2,
                "frame_seq": 1,
                "entry_seq": 9,
                "entry_id": "entry_00000009",
                "parent_entry_id": "entry_00000008",
                "entry_type": "session_info",
                "timestamp": "2026-02-15T18:10:10Z",
                "payload_sha256": "4d94e98ec87dbb5e7fb5952a322f6303f65895d15fd8ff81a9f65ee31c6db331",
                "payload_bytes": 96,
                "payload": {
                    "name": "v2-session"
                }
            }
        ],
        "offset_index": [
            {
                "schema": "pi.session_store_v2.offset_index.v1",
                "entry_seq": 1,
                "entry_id": "entry_00000001",
                "segment_seq": 1,
                "frame_seq": 1,
                "byte_offset": 0,
                "byte_length": 256,
                "crc32c": "1A2B3C4D",
                "state": "active"
            },
            {
                "schema": "pi.session_store_v2.offset_index.v1",
                "entry_seq": 9,
                "entry_id": "entry_00000009",
                "segment_seq": 2,
                "frame_seq": 1,
                "byte_offset": 0,
                "byte_length": 192,
                "crc32c": "9ABCDEFF",
                "state": "active"
            }
        ],
        "checkpoints": [
            {
                "schema": "pi.session_store_v2.checkpoint.v1",
                "checkpoint_seq": 1,
                "at": "2026-02-15T18:10:10Z",
                "head_entry_seq": 9,
                "head_entry_id": "entry_00000009",
                "snapshot_ref": "checkpoints/0000000000000001.json",
                "compacted_before_entry_seq": 0,
                "chain_hash": "f7a3b79f9d1b84444c34f6f6f1393ba55fba8c4d0868ac8f80a7b951907e8095",
                "reason": "pre_migration"
            }
        ],
        "migration_events": [
            {
                "schema": "pi.session_store_v2.migration_event.v1",
                "migration_id": "4dbf9c6b-c165-4f28-a69a-91f8a8e388e2",
                "phase": "completed",
                "at": "2026-02-15T18:11:00Z",
                "source_path": "sessions/legacy.jsonl",
                "target_path": "sessions/f4c03c8c.v2/",
                "source_format": "jsonl_v3",
                "target_format": "native_v2",
                "verification": {
                    "entry_count_match": true,
                    "hash_chain_match": true,
                    "index_consistent": true
                },
                "outcome": "ok",
                "error_class": null,
                "correlation_id": "mig_20260215_181100_f4c03c8c"
            }
        ],
        "state_transitions": [
            {
                "from_state": "CLEAN",
                "to_state": "MIGRATION_STAGING",
                "reason": "begin migration",
                "at": "2026-02-15T18:10:59Z"
            },
            {
                "from_state": "MIGRATION_STAGING",
                "to_state": "MIGRATED",
                "reason": "cutover commit",
                "at": "2026-02-15T18:11:00Z"
            },
            {
                "from_state": "MIGRATED",
                "to_state": "DIRTY",
                "reason": "new append",
                "at": "2026-02-15T18:11:01Z"
            }
        ]
    })
}

#[test]
fn session_store_v2_contract_bundle_validates() {
    let validator = compiled_contract_schema();
    let bundle = canonical_contract_bundle();

    if let Err(err) = validator.validate(&bundle) {
        panic!("Canonical session store V2 contract bundle must validate: {err}");
    }
}

#[test]
fn contract_fails_closed_when_required_section_missing() {
    let validator = compiled_contract_schema();
    let mut bundle = canonical_contract_bundle();
    bundle
        .as_object_mut()
        .expect("bundle object")
        .remove("migration_events");

    assert!(
        validator.validate(&bundle).is_err(),
        "missing migration_events must fail validation"
    );
}

#[test]
fn invalid_transition_is_rejected() {
    let validator = compiled_contract_schema();
    let mut bundle = canonical_contract_bundle();
    let transitions = bundle["state_transitions"]
        .as_array_mut()
        .expect("state_transitions array");
    transitions[0]["from_state"] = json!("DIRTY");
    transitions[0]["to_state"] = json!("MIGRATED");

    assert!(
        validator.validate(&bundle).is_err(),
        "DIRTY -> MIGRATED must be rejected by transition rules"
    );
}

#[test]
fn manifest_store_version_must_remain_v2() {
    let validator = compiled_contract_schema();
    let mut bundle = canonical_contract_bundle();
    bundle["manifest"]["store_version"] = json!(3);

    assert!(
        validator.validate(&bundle).is_err(),
        "manifest.store_version != 2 must fail validation"
    );
}
