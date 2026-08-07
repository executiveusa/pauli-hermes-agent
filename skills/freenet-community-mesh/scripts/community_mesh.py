#!/usr/bin/env python3
"""Deterministic two-node sovereign community mesh simulator.

This prototype proves the data boundary before any live Freenet integration:
- private records remain inside each node directory;
- only schema-approved CommunityResource records enter shared state;
- records are signed with a node-local HMAC key for tamper detection;
- revocations synchronize;
- no network access is performed.

The HMAC signature is a prototype mechanism, not a replacement for a Freenet delegate.
"""
from __future__ import annotations

import argparse
import hashlib
import hmac
import json
import re
import secrets
import shutil
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ALLOWED_FIELDS = {
    "resource_id",
    "organization_id",
    "category",
    "service",
    "location",
    "languages",
    "availability",
    "contact_method",
    "updated_at",
    "status",
    "signature",
}
REQUIRED_FIELDS = ALLOWED_FIELDS - {"signature"}
ALLOWED_CATEGORIES = {
    "food",
    "transport",
    "youth",
    "translation",
    "health-navigation",
    "housing-navigation",
    "jobs",
    "other",
}
ALLOWED_STATUS = {"active", "revoked"}
PROHIBITED_FIELD_FRAGMENTS = {
    "client",
    "patient",
    "donor",
    "student",
    "child",
    "beneficiary",
    "password",
    "secret",
    "token",
    "api_key",
    "private_key",
    "case_note",
    "medical",
    "immigration",
    "account_number",
}
SENSITIVE_PATTERNS = [
    re.compile(r"\b(?:sk|pk|ghp|github_pat|21st_sk)_[A-Za-z0-9_-]{8,}\b"),
    re.compile(r"\b\d{3}-\d{2}-\d{4}\b"),
    re.compile(r"\b(?:\d[ -]*?){13,19}\b"),
]


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def canonical_payload(record: dict[str, Any]) -> bytes:
    unsigned = {k: v for k, v in record.items() if k != "signature"}
    return json.dumps(unsigned, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")


def sign(record: dict[str, Any], key: bytes) -> str:
    return hmac.new(key, canonical_payload(record), hashlib.sha256).hexdigest()


def validate_record(record: dict[str, Any], *, require_signature: bool = True) -> None:
    if not isinstance(record, dict):
        raise ValueError("resource must be an object")
    unknown = set(record) - ALLOWED_FIELDS
    if unknown:
        raise ValueError(f"unknown fields: {sorted(unknown)}")
    missing = REQUIRED_FIELDS - set(record)
    if missing:
        raise ValueError(f"missing fields: {sorted(missing)}")
    if require_signature and not isinstance(record.get("signature"), str):
        raise ValueError("missing signature")
    if record["category"] not in ALLOWED_CATEGORIES:
        raise ValueError("unsupported category")
    if record["status"] not in ALLOWED_STATUS:
        raise ValueError("unsupported status")
    if not isinstance(record["languages"], list) or not all(isinstance(v, str) for v in record["languages"]):
        raise ValueError("languages must be a list of strings")
    for key, value in record.items():
        lowered = key.lower()
        if any(fragment in lowered for fragment in PROHIBITED_FIELD_FRAGMENTS):
            raise ValueError(f"prohibited field: {key}")
        text = json.dumps(value, ensure_ascii=False)
        for pattern in SENSITIVE_PATTERNS:
            if pattern.search(text):
                raise ValueError(f"sensitive pattern detected in {key}")


@dataclass(frozen=True)
class Node:
    root: Path

    @property
    def private_path(self) -> Path:
        return self.root / "private.json"

    @property
    def shared_path(self) -> Path:
        return self.root / "shared.json"

    @property
    def key_path(self) -> Path:
        return self.root / ".prototype-signing-key"

    def init(self) -> None:
        self.root.mkdir(parents=True, exist_ok=True)
        if not self.private_path.exists():
            self.private_path.write_text("[]\n", encoding="utf-8")
        if not self.shared_path.exists():
            self.shared_path.write_text("[]\n", encoding="utf-8")
        if not self.key_path.exists():
            self.key_path.write_text(secrets.token_hex(32), encoding="ascii")

    def key(self) -> bytes:
        return bytes.fromhex(self.key_path.read_text(encoding="ascii").strip())

    def load(self, path: Path) -> list[dict[str, Any]]:
        value = json.loads(path.read_text(encoding="utf-8"))
        if not isinstance(value, list):
            raise ValueError(f"expected list in {path}")
        return value

    def write_atomic(self, path: Path, value: Any) -> None:
        temp = path.with_suffix(path.suffix + ".tmp")
        temp.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        temp.replace(path)

    def add_private(self, record: dict[str, Any]) -> None:
        records = self.load(self.private_path)
        records.append(record)
        self.write_atomic(self.private_path, records)

    def publish(self, record: dict[str, Any]) -> dict[str, Any]:
        candidate = dict(record)
        validate_record(candidate, require_signature=False)
        candidate["signature"] = sign(candidate, self.key())
        validate_record(candidate)
        records = [item for item in self.load(self.shared_path) if item.get("resource_id") != candidate["resource_id"]]
        records.append(candidate)
        self.write_atomic(self.shared_path, records)
        return candidate

    def verify(self, record: dict[str, Any], source_key: bytes) -> None:
        validate_record(record)
        expected = sign(record, source_key)
        if not hmac.compare_digest(expected, record["signature"]):
            raise ValueError("invalid signature")


def sync(source: Node, destination: Node) -> int:
    source_key = source.key()
    incoming = source.load(source.shared_path)
    current = {item["resource_id"]: item for item in destination.load(destination.shared_path)}
    for item in incoming:
        source.verify(item, source_key)
        current[item["resource_id"]] = item
    destination.write_atomic(destination.shared_path, sorted(current.values(), key=lambda item: item["resource_id"]))
    return len(incoming)


def leakage_check(source: Node, destination: Node) -> None:
    private_bytes = source.private_path.read_bytes()
    if private_bytes and private_bytes != b"[]\n":
        shared_blob = destination.shared_path.read_bytes()
        private_records = json.loads(private_bytes)
        for record in private_records:
            serialized = json.dumps(record, sort_keys=True, ensure_ascii=False).encode("utf-8")
            if serialized in shared_blob:
                raise RuntimeError("private record leaked into destination shared state")


def demo(workspace: Path) -> dict[str, Any]:
    if workspace.exists():
        shutil.rmtree(workspace)
    a = Node(workspace / "seattle-node")
    b = Node(workspace / "everett-node")
    a.init()
    b.init()
    a.add_private({"case_id": "PRIVATE-A-001", "notes": "Never publish this local case note."})
    b.add_private({"internal_id": "PRIVATE-B-001", "notes": "Local-only operating memory."})
    record = {
        "resource_id": "resource-food-001",
        "organization_id": "org-seattle-demo",
        "category": "food",
        "service": "Weekly public food distribution",
        "location": "Seattle, WA",
        "languages": ["en", "es"],
        "availability": "Tuesdays, subject to confirmation",
        "contact_method": "public@example.org",
        "updated_at": utc_now(),
        "status": "active",
    }
    published = a.publish(record)
    sync(a, b)
    leakage_check(a, b)
    revoked = dict(record)
    revoked["status"] = "revoked"
    revoked["updated_at"] = utc_now()
    a.publish(revoked)
    sync(a, b)
    leakage_check(a, b)
    synced = {item["resource_id"]: item for item in b.load(b.shared_path)}
    if synced[published["resource_id"]]["status"] != "revoked":
        raise RuntimeError("revocation did not synchronize")
    result = {
        "status": "PASS",
        "transport": "deterministic-local-simulator",
        "nodes": [str(a.root), str(b.root)],
        "shared_resource_count": len(synced),
        "private_a_count": len(a.load(a.private_path)),
        "private_b_count": len(b.load(b.private_path)),
        "revocation_verified": True,
        "private_leakage_detected": False,
        "live_freenet_verified": False,
    }
    (workspace / "proof.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    return result


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=["demo"])
    parser.add_argument("--workspace", type=Path, required=True)
    args = parser.parse_args()
    result = demo(args.workspace)
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
