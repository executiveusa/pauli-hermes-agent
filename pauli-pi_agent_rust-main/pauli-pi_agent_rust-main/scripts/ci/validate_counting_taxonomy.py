#!/usr/bin/env python3
"""Validate counting taxonomy metadata in evidence artifacts.

This checker enforces the Phase-0 counting taxonomy contract:
- Every count must carry an explicit granularity label.
- LOC/provider/extension dimensions must include required side-by-side labels.
- Every metric must include tool provenance and command signature.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def project_root_from_script() -> Path:
    # scripts/ci/validate_counting_taxonomy.py -> repo root is parents[2]
    return Path(__file__).resolve().parents[2]


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def validate_artifact(artifact: dict, contract: dict) -> list[str]:
    errors: list[str] = []

    required_dimensions = contract.get("required_dimensions", {})
    required_metric_fields = contract.get("required_metric_fields", [])
    required_provenance_fields = contract.get("required_tool_provenance_fields", [])
    expected_taxonomy_schema = contract.get("taxonomy_schema")

    taxonomy = artifact.get("counting_taxonomy")
    if not isinstance(taxonomy, dict):
        return ["missing top-level counting_taxonomy object"]

    schema = taxonomy.get("schema")
    if schema != expected_taxonomy_schema:
        errors.append(
            f"counting_taxonomy.schema must be {expected_taxonomy_schema!r}, got {schema!r}"
        )

    dimensions = taxonomy.get("dimensions")
    if not isinstance(dimensions, dict):
        return errors + ["counting_taxonomy.dimensions must be an object"]

    for dim_name, dim_contract in required_dimensions.items():
        dim = dimensions.get(dim_name)
        if not isinstance(dim, dict):
            errors.append(f"missing counting_taxonomy.dimensions.{dim_name}")
            continue

        metrics = dim.get("metrics")
        if not isinstance(metrics, list):
            errors.append(f"counting_taxonomy.dimensions.{dim_name}.metrics must be an array")
            continue

        required_labels = set(dim_contract.get("required_granularity_labels", []))
        seen_labels: set[str] = set()

        for idx, metric in enumerate(metrics):
            metric_path = f"counting_taxonomy.dimensions.{dim_name}.metrics[{idx}]"
            if not isinstance(metric, dict):
                errors.append(f"{metric_path} must be an object")
                continue

            for field in required_metric_fields:
                if field not in metric:
                    errors.append(f"{metric_path} missing field {field!r}")

            label = metric.get("granularity_label")
            if isinstance(label, str):
                seen_labels.add(label)
            else:
                errors.append(f"{metric_path}.granularity_label must be a non-empty string")

            value = metric.get("value")
            if not isinstance(value, (int, float)):
                errors.append(f"{metric_path}.value must be numeric")

            provenance = metric.get("tool_provenance")
            if not isinstance(provenance, dict):
                errors.append(f"{metric_path}.tool_provenance must be an object")
            else:
                for field in required_provenance_fields:
                    val = provenance.get(field)
                    if not isinstance(val, str) or not val.strip():
                        errors.append(f"{metric_path}.tool_provenance.{field} must be non-empty")

        missing_labels = sorted(required_labels - seen_labels)
        if missing_labels:
            errors.append(
                f"counting_taxonomy.dimensions.{dim_name} missing granularity labels: "
                + ", ".join(missing_labels)
            )

    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--artifact", required=True, help="Evidence artifact JSON to validate")
    parser.add_argument(
        "--contract",
        default=str(project_root_from_script() / "docs/counting-taxonomy-contract.json"),
        help="Counting taxonomy contract JSON",
    )
    args = parser.parse_args()

    artifact_path = Path(args.artifact)
    contract_path = Path(args.contract)

    if not artifact_path.exists():
        print(f"ERROR: artifact not found: {artifact_path}", file=sys.stderr)
        return 2
    if not contract_path.exists():
        print(f"ERROR: contract not found: {contract_path}", file=sys.stderr)
        return 2

    artifact = load_json(artifact_path)
    contract = load_json(contract_path)
    errors = validate_artifact(artifact, contract)

    if errors:
        print("Counting taxonomy validation: FAIL", file=sys.stderr)
        for err in errors:
            print(f"  - {err}", file=sys.stderr)
        return 1

    print("Counting taxonomy validation: PASS")
    print(f"  Artifact: {artifact_path}")
    print(f"  Contract: {contract_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
