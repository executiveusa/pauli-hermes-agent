#!/usr/bin/env bash
# scripts/perf/capture_baseline.sh — Capture baseline measurements with variance analysis.
#
# Runs benchmark suites multiple times to produce baseline measurements with
# statistical confidence intervals. The output is a structured JSON baseline
# file that downstream consumers use for regression detection and truthful
# progress claims.
#
# Bead: bd-3ar8v.1.5
# Depends on: bd-3ar8v.1.8 (orchestration), bd-3ar8v.1.6 (SLI matrix),
#             bd-3ar8v.1.4 (observability), bd-3ar8v.1.1 (protocol)
#
# Usage:
#   ./scripts/perf/capture_baseline.sh                     # full baseline (5 rounds)
#   ./scripts/perf/capture_baseline.sh --rounds 10         # custom round count
#   ./scripts/perf/capture_baseline.sh --quick             # quick baseline (3 rounds)
#   ./scripts/perf/capture_baseline.sh --output <path>     # custom output path
#   ./scripts/perf/capture_baseline.sh --validate <path>   # validate existing baseline
#   ./scripts/perf/capture_baseline.sh --diagnose-env ci=tests/perf/reports/baseline_variance.json \
#       --diagnose-env canary=/tmp/baseline_variance_canary.json           # cross-env variance diagnosis
#   ./scripts/perf/capture_baseline.sh --require-rch        # require remote offload
#   ./scripts/perf/capture_baseline.sh --no-rch             # force local cargo execution
#
# Environment:
#   CARGO_TARGET_DIR          Cargo target directory
#   BASELINE_ROUNDS           Number of measurement rounds (default: 5)
#   BASELINE_WARMUP_ROUNDS    Warmup rounds discarded (default: 1)
#   BASELINE_OUTPUT            Output path (default: tests/perf/reports/baseline_variance.json)
#   BASELINE_MAX_CV            Maximum coefficient of variation for acceptance (default: 0.15)
#   BASELINE_DIAGNOSIS_OUTPUT  Cross-env diagnosis report path
#                              (default: tests/perf/reports/cross_env_variance_diagnosis.json)
#   BASELINE_VARIANCE_ALERT_PCT
#                              Spread threshold (percent) for fail/warn diagnostics (default: 10.0)
#   PERF_REGRESSION_FULL       Forward to perf_regression (default: 0)
#   BASELINE_CARGO_RUNNER      Cargo runner mode: rch | auto | local (default: rch)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

# ─── Configuration ───────────────────────────────────────────────────────────

ROUNDS="${BASELINE_ROUNDS:-5}"
WARMUP_ROUNDS="${BASELINE_WARMUP_ROUNDS:-1}"
OUTPUT="${BASELINE_OUTPUT:-$PROJECT_ROOT/tests/perf/reports/baseline_variance.json}"
MAX_CV="${BASELINE_MAX_CV:-0.15}"
TARGET_DIR="${CARGO_TARGET_DIR:-$PROJECT_ROOT/target}"
GIT_COMMIT="$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")"
TIMESTAMP="$(date -u +%Y%m%dT%H%M%SZ)"
VALIDATE_ONLY=""
QUICK=0
DIAGNOSE_OUTPUT="${BASELINE_DIAGNOSIS_OUTPUT:-$PROJECT_ROOT/tests/perf/reports/cross_env_variance_diagnosis.json}"
VARIANCE_ALERT_PCT="${BASELINE_VARIANCE_ALERT_PCT:-10.0}"
DIAGNOSE_ENVS=()
CARGO_RUNNER_REQUEST="${BASELINE_CARGO_RUNNER:-rch}" # rch | auto | local
CARGO_RUNNER_MODE="local"
declare -a CARGO_RUNNER_ARGS=("cargo")
SEEN_NO_RCH=false
SEEN_REQUIRE_RCH=false

# ─── Helpers ─────────────────────────────────────────────────────────────────

red()    { printf '\033[0;31m%s\033[0m\n' "$*"; }
green()  { printf '\033[0;32m%s\033[0m\n' "$*"; }
yellow() { printf '\033[0;33m%s\033[0m\n' "$*"; }
bold()   { printf '\033[1m%s\033[0m\n' "$*"; }

die() { red "ERROR: $*" >&2; exit 1; }

# ─── CLI Parsing ─────────────────────────────────────────────────────────────

while [[ $# -gt 0 ]]; do
  case "$1" in
    --rounds)  ROUNDS="$2"; shift 2 ;;
    --output)  OUTPUT="$2"; shift 2 ;;
    --quick)   QUICK=1; ROUNDS=3; WARMUP_ROUNDS=0; shift ;;
    --validate) VALIDATE_ONLY="$2"; shift 2 ;;
    --diagnose-env) DIAGNOSE_ENVS+=("$2"); shift 2 ;;
    --diagnose-output) DIAGNOSE_OUTPUT="$2"; shift 2 ;;
    --variance-alert-pct) VARIANCE_ALERT_PCT="$2"; shift 2 ;;
    --no-rch)
      if [[ "$SEEN_REQUIRE_RCH" == true ]]; then
        die "Cannot combine --no-rch and --require-rch"
      fi
      SEEN_NO_RCH=true
      CARGO_RUNNER_REQUEST="local"
      shift
      ;;
    --require-rch)
      if [[ "$SEEN_NO_RCH" == true ]]; then
        die "Cannot combine --require-rch and --no-rch"
      fi
      SEEN_REQUIRE_RCH=true
      CARGO_RUNNER_REQUEST="rch"
      shift
      ;;
    --help|-h)
      sed -n '2,/^$/p' "$0" | sed 's/^# \?//'
      exit 0
      ;;
    *) die "Unknown flag: $1 (try --help)" ;;
  esac
done

# ─── Validate-only mode ─────────────────────────────────────────────────────

if [[ -n "$VALIDATE_ONLY" ]]; then
  bold "Validating baseline: $VALIDATE_ONLY"

  if [[ ! -f "$VALIDATE_ONLY" ]]; then
    die "Baseline file not found: $VALIDATE_ONLY"
  fi

  python3 - "$VALIDATE_ONLY" "$MAX_CV" <<'PYEOF'
import json, sys

path, max_cv = sys.argv[1], float(sys.argv[2])
with open(path) as f:
    baseline = json.load(f)

schema = baseline.get("schema", "")
if schema != "pi.perf.baseline_variance.v1":
    print(f"FAIL: wrong schema: {schema}")
    sys.exit(1)
print(f"OK: schema={schema}")

metrics = baseline.get("metrics", [])
print(f"OK: {len(metrics)} metrics")

warnings = 0
for m in metrics:
    name = m["metric_name"]
    cv = m.get("coefficient_of_variation", 0)
    confidence = m.get("variance_class", "unknown")
    if cv > max_cv:
        print(f"WARN: {name} cv={cv:.4f} exceeds max_cv={max_cv}")
        warnings += 1
    else:
        print(f"OK: {name} cv={cv:.4f} class={confidence}")

if warnings > 0:
    print(f"\n{warnings} metric(s) exceed CV threshold")
else:
    print("\nAll metrics within acceptable variance")
PYEOF
  exit $?
fi

# ─── Cross-environment diagnosis mode ───────────────────────────────────────

if [[ "${#DIAGNOSE_ENVS[@]}" -gt 0 ]]; then
  bold "Cross-environment variance diagnosis (bd-3ar8v.5.7)"

  if [[ "${#DIAGNOSE_ENVS[@]}" -lt 2 ]]; then
    die "--diagnose-env requires at least two entries (label=path)"
  fi

  mkdir -p "$(dirname "$DIAGNOSE_OUTPUT")"

  python3 - "$DIAGNOSE_OUTPUT" "$VARIANCE_ALERT_PCT" "${DIAGNOSE_ENVS[@]}" <<'PYEOF'
import json
import math
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

output_path = Path(sys.argv[1])
alert_pct = float(sys.argv[2])
pairs = sys.argv[3:]

if len(pairs) < 2:
    raise SystemExit("need at least two --diagnose-env entries")

inputs = []
metrics_by_env = {}
for pair in pairs:
    if "=" not in pair:
        raise SystemExit(f"invalid --diagnose-env '{pair}'; expected label=path")
    label, path_str = pair.split("=", 1)
    label = label.strip()
    path = Path(path_str.strip())
    if not label:
        raise SystemExit(f"empty label in --diagnose-env '{pair}'")
    if not path.exists():
        raise SystemExit(f"baseline file not found for '{label}': {path}")

    payload = json.loads(path.read_text(encoding="utf-8"))
    if payload.get("schema") != "pi.perf.baseline_variance.v1":
        raise SystemExit(
            f"baseline schema mismatch for '{label}': {payload.get('schema')}"
        )

    metric_rows = payload.get("metrics", [])
    metric_map = {}
    for row in metric_rows:
        if not isinstance(row, dict):
            continue
        metric_name = str(row.get("metric_name", "")).strip()
        if not metric_name:
            continue
        mean_value = row.get("mean")
        cv_value = row.get("coefficient_of_variation")
        if isinstance(mean_value, (int, float)) and isinstance(cv_value, (int, float)):
            metric_map[metric_name] = {
                "mean": float(mean_value),
                "coefficient_of_variation": float(cv_value),
                "variance_class": str(row.get("variance_class", "unknown")),
            }

    metrics_by_env[label] = metric_map
    inputs.append(
        {
            "label": label,
            "path": str(path),
            "git_commit": str(payload.get("git_commit", "unknown")),
            "measurement_rounds": payload.get("measurement_rounds"),
            "warmup_rounds": payload.get("warmup_rounds"),
            "metric_count": len(metric_map),
        }
    )

common_metrics = sorted(
    set.intersection(*(set(metric_map.keys()) for metric_map in metrics_by_env.values()))
)

diagnostics_rows = []
alerts = []
for metric_name in common_metrics:
    env_rows = []
    values = []
    cvs_pct = []
    for label in sorted(metrics_by_env.keys()):
        row = metrics_by_env[label][metric_name]
        value = float(row["mean"])
        cv_pct = float(row["coefficient_of_variation"]) * 100.0
        values.append(value)
        cvs_pct.append(cv_pct)
        env_rows.append(
            {
                "label": label,
                "mean": value,
                "coefficient_of_variation_pct": round(cv_pct, 6),
                "variance_class": row.get("variance_class", "unknown"),
            }
        )

    n = len(values)
    mean_value = sum(values) / n if n else 0.0
    if n > 1:
        env_variance = sum((v - mean_value) ** 2 for v in values) / (n - 1)
        env_stddev = math.sqrt(env_variance)
    else:
        env_stddev = 0.0

    spread_pct = (env_stddev / abs(mean_value) * 100.0) if abs(mean_value) > 1e-9 else 0.0
    noise_floor_pct = sum(cvs_pct) / n if n else 0.0
    signal_to_noise = spread_pct / max(noise_floor_pct, 1e-9)

    environment_component_pct = max(0.0, spread_pct - noise_floor_pct)
    noise_component_pct = min(spread_pct, noise_floor_pct)
    build_component_pct = 0.0
    runtime_component_pct = noise_component_pct

    if environment_component_pct > max(build_component_pct, runtime_component_pct, noise_component_pct):
        dominant_source = "environment"
    elif runtime_component_pct > max(build_component_pct, noise_component_pct):
        dominant_source = "runtime"
    elif noise_component_pct > 0:
        dominant_source = "noise"
    else:
        dominant_source = "build"

    triggered = spread_pct >= alert_pct
    severity = "high" if triggered and signal_to_noise >= 1.5 else "medium" if triggered else "info"
    reason = (
        "cross-environment spread exceeds threshold"
        if triggered
        else "within configured cross-environment threshold"
    )

    diagnostic = {
        "schema": "pi.perf.cross_env_variance_diagnostic.v1",
        "metric_name": metric_name,
        "severity": severity,
        "event_code": "cross_env_variance_exceeds_threshold" if triggered else "cross_env_variance_within_threshold",
        "spread_pct": round(spread_pct, 6),
        "noise_floor_pct": round(noise_floor_pct, 6),
        "signal_to_noise": round(signal_to_noise, 6),
        "dominant_source": dominant_source,
        "message": reason,
        "recommended_action": "Stabilize environment/build controls before interpreting regression" if triggered else "No action required",
    }
    diagnostics_rows.append(diagnostic)

    if triggered:
        alerts.append(
            {
                "metric_name": metric_name,
                "severity": severity,
                "reason": reason,
                "spread_pct": round(spread_pct, 6),
                "threshold_pct": round(alert_pct, 6),
                "signal_to_noise": round(signal_to_noise, 6),
            }
        )

    env_rows_sorted = sorted(env_rows, key=lambda row: row["label"])
    metric_payload = {
        "metric_name": metric_name,
        "environment_values": env_rows_sorted,
        "aggregate": {
            "environment_count": n,
            "mean": round(mean_value, 6),
            "env_stddev": round(env_stddev, 6),
            "spread_pct": round(spread_pct, 6),
            "noise_floor_pct": round(noise_floor_pct, 6),
            "signal_to_noise": round(signal_to_noise, 6),
        },
        "variance_sources": {
            "environment_pct": round(environment_component_pct, 6),
            "build_pct": round(build_component_pct, 6),
            "runtime_pct": round(runtime_component_pct, 6),
            "noise_pct": round(noise_component_pct, 6),
            "dominant_source": dominant_source,
        },
        "alert": {
            "triggered": triggered,
            "severity": severity,
            "reason": reason,
            "threshold_pct": round(alert_pct, 6),
        },
    }
    diagnostics_rows[-1]["metric"] = metric_payload

diagnostics_jsonl_path = output_path.with_suffix(".jsonl")
diagnostics_jsonl_path.write_text(
    "".join(json.dumps(row, separators=(",", ":")) + "\n" for row in diagnostics_rows),
    encoding="utf-8",
)

report = {
    "schema": "pi.perf.cross_env_variance_diagnosis.v1",
    "version": "1.0.0",
    "bead_id": "bd-3ar8v.5.7",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "thresholds": {
        "variance_alert_pct": round(alert_pct, 6),
        "signal_to_noise_env_cutoff": 1.5,
    },
    "inputs": sorted(inputs, key=lambda row: row["label"]),
    "metrics": [entry["metric"] for entry in diagnostics_rows],
    "diagnostics_log": {
        "schema": "pi.perf.cross_env_variance_diagnostic.v1",
        "jsonl_path": str(diagnostics_jsonl_path),
        "entries": len(diagnostics_rows),
    },
    "summary": {
        "environment_count": len(inputs),
        "metric_count": len(common_metrics),
        "alert_count": len(alerts),
        "high_variance_metrics": [alert["metric_name"] for alert in alerts],
    },
    "alerts": alerts,
}

output_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

print(f"Cross-environment diagnosis written: {output_path}")
print(f"Diagnostic log written: {diagnostics_jsonl_path}")
print(f"Metrics analyzed: {len(common_metrics)}")
print(f"Alerts: {len(alerts)}")
PYEOF
  exit $?
fi

# ─── Cargo Runner Resolution ────────────────────────────────────────────────

if [[ "$CARGO_RUNNER_REQUEST" != "rch" && "$CARGO_RUNNER_REQUEST" != "auto" && "$CARGO_RUNNER_REQUEST" != "local" ]]; then
  die "Invalid BASELINE_CARGO_RUNNER value: $CARGO_RUNNER_REQUEST (expected: rch|auto|local)"
fi

if [[ "$CARGO_RUNNER_REQUEST" == "rch" ]]; then
  if ! command -v rch >/dev/null 2>&1; then
    die "BASELINE_CARGO_RUNNER=rch requested, but 'rch' is not available in PATH."
  fi
  if ! rch check --quiet >/dev/null 2>&1; then
    die "'rch check' failed; refusing heavy local cargo fallback. Fix rch or pass --no-rch."
  fi
  CARGO_RUNNER_MODE="rch"
  CARGO_RUNNER_ARGS=("rch" "exec" "--" "cargo")
elif [[ "$CARGO_RUNNER_REQUEST" == "auto" ]] && command -v rch >/dev/null 2>&1; then
  if rch check --quiet >/dev/null 2>&1; then
    CARGO_RUNNER_MODE="rch"
    CARGO_RUNNER_ARGS=("rch" "exec" "--" "cargo")
  else
    yellow "rch detected but unhealthy; auto mode will run cargo locally (set --require-rch to fail fast)."
  fi
fi

# ─── Capture baseline ───────────────────────────────────────────────────────

bold "═══ Baseline Capture (bd-3ar8v.1.5) ═══"
echo ""
echo "  Rounds:        $ROUNDS (+ $WARMUP_ROUNDS warmup)"
echo "  Output:        $OUTPUT"
echo "  Max CV:        $MAX_CV"
echo "  Git commit:    $GIT_COMMIT"
echo "  Cargo runner:  $CARGO_RUNNER_MODE (request=$CARGO_RUNNER_REQUEST)"
echo ""

TEMP_DIR=$(mktemp -d)
trap "rm -rf '$TEMP_DIR'" EXIT

total_rounds=$((WARMUP_ROUNDS + ROUNDS))

for i in $(seq 1 "$total_rounds"); do
  if [[ "$i" -le "$WARMUP_ROUNDS" ]]; then
    echo "  [Round $i/$total_rounds] Warmup (discarding)..."
  else
    echo "  [Round $i/$total_rounds] Measuring..."
  fi

  round_dir="$TEMP_DIR/round_$i"
  mkdir -p "$round_dir"

  PERF_REGRESSION_OUTPUT="$round_dir" \
  CARGO_TARGET_DIR="$TARGET_DIR" \
    "${CARGO_RUNNER_ARGS[@]}" test --test perf_regression -- --nocapture \
    >"$round_dir/stdout.log" 2>"$round_dir/stderr.log" || true
done

# ─── Aggregate results ───────────────────────────────────────────────────────

bold "Aggregating $ROUNDS measurement rounds..."

mkdir -p "$(dirname "$OUTPUT")"

python3 - "$TEMP_DIR" "$WARMUP_ROUNDS" "$ROUNDS" "$OUTPUT" "$MAX_CV" "$GIT_COMMIT" "$TIMESTAMP" <<'PYEOF'
import json, os, sys, math

temp_dir = sys.argv[1]
warmup = int(sys.argv[2])
rounds = int(sys.argv[3])
output_path = sys.argv[4]
max_cv = float(sys.argv[5])
git_commit = sys.argv[6]
timestamp = sys.argv[7]

# Collect measurements per metric from non-warmup rounds
metrics_data = {}  # metric_name -> list of values

for i in range(warmup + 1, warmup + rounds + 1):
    jsonl_path = os.path.join(temp_dir, f"round_{i}", "perf_regression.jsonl")
    if not os.path.exists(jsonl_path):
        continue
    with open(jsonl_path) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                record = json.loads(line)
            except json.JSONDecodeError:
                continue
            name = record.get("budget_name", "")
            value = record.get("actual_value")
            if name and value is not None:
                metrics_data.setdefault(name, []).append(float(value))

# t-distribution critical values for 95% CI (two-tailed)
# df -> t_critical (from standard t-tables)
T_CRITICAL_95 = {
    1: 12.706, 2: 4.303, 3: 3.182, 4: 2.776, 5: 2.571,
    6: 2.447, 7: 2.365, 8: 2.306, 9: 2.262, 10: 2.228,
    15: 2.131, 20: 2.086, 25: 2.060, 30: 2.042, 50: 2.009,
    100: 1.984,
}
T_CRITICAL_99 = {
    1: 63.657, 2: 9.925, 3: 5.841, 4: 4.604, 5: 4.032,
    6: 3.707, 7: 3.499, 8: 3.355, 9: 3.250, 10: 3.169,
    15: 2.947, 20: 2.845, 25: 2.787, 30: 2.750, 50: 2.678,
    100: 2.626,
}

def get_t_critical(df, table):
    if df in table:
        return table[df]
    # Linear interpolation between known values
    keys = sorted(table.keys())
    for i in range(len(keys) - 1):
        if keys[i] <= df <= keys[i+1]:
            frac = (df - keys[i]) / (keys[i+1] - keys[i])
            return table[keys[i]] + frac * (table[keys[i+1]] - table[keys[i]])
    return table[keys[-1]]  # fallback to largest

def compute_stats(values):
    n = len(values)
    if n == 0:
        return None
    values_sorted = sorted(values)
    mean = sum(values) / n
    if n > 1:
        variance = sum((x - mean) ** 2 for x in values) / (n - 1)  # sample variance
        stddev = math.sqrt(variance)
    else:
        variance = 0.0
        stddev = 0.0

    cv = stddev / mean if mean > 0 else 0.0

    # Percentiles
    def pct(p):
        idx = (p / 100.0) * (n - 1)
        lo = int(math.floor(idx))
        hi = min(lo + 1, n - 1)
        frac = idx - lo
        return values_sorted[lo] * (1 - frac) + values_sorted[hi] * frac

    # Confidence intervals
    df = n - 1
    se = stddev / math.sqrt(n) if n > 0 else 0.0

    t95 = get_t_critical(max(df, 1), T_CRITICAL_95)
    t99 = get_t_critical(max(df, 1), T_CRITICAL_99)

    ci_95_lower = mean - t95 * se
    ci_95_upper = mean + t95 * se
    ci_99_lower = mean - t99 * se
    ci_99_upper = mean + t99 * se

    # Variance classification
    if cv <= 0.05:
        var_class = "low"
    elif cv <= 0.15:
        var_class = "medium"
    else:
        var_class = "high"

    return {
        "count": n,
        "min": values_sorted[0],
        "max": values_sorted[-1],
        "mean": round(mean, 4),
        "stddev": round(stddev, 4),
        "coefficient_of_variation": round(cv, 6),
        "variance_class": var_class,
        "p50": round(pct(50), 4),
        "p95": round(pct(95), 4),
        "p99": round(pct(99), 4),
        "confidence_interval_95": {
            "lower": round(ci_95_lower, 4),
            "upper": round(ci_95_upper, 4),
            "t_critical": round(t95, 4),
            "standard_error": round(se, 4),
        },
        "confidence_interval_99": {
            "lower": round(ci_99_lower, 4),
            "upper": round(ci_99_upper, 4),
            "t_critical": round(t99, 4),
            "standard_error": round(se, 4),
        },
        "raw_values": [round(v, 4) for v in values],
    }

# Build output
metric_results = []
acceptable_count = 0
high_variance_count = 0

for name in sorted(metrics_data.keys()):
    values = metrics_data[name]
    stats = compute_stats(values)
    if stats is None:
        continue

    is_acceptable = stats["coefficient_of_variation"] <= max_cv
    if is_acceptable:
        acceptable_count += 1
    else:
        high_variance_count += 1

    metric_results.append({
        "metric_name": name,
        "sample_count": stats["count"],
        "mean": stats["mean"],
        "stddev": stats["stddev"],
        "coefficient_of_variation": stats["coefficient_of_variation"],
        "variance_class": stats["variance_class"],
        "percentiles": {
            "p50": stats["p50"],
            "p95": stats["p95"],
            "p99": stats["p99"],
        },
        "range": {
            "min": stats["min"],
            "max": stats["max"],
        },
        "confidence_interval_95": stats["confidence_interval_95"],
        "confidence_interval_99": stats["confidence_interval_99"],
        "raw_values": stats["raw_values"],
        "acceptable": is_acceptable,
    })

baseline = {
    "schema": "pi.perf.baseline_variance.v1",
    "version": "1.0.0",
    "bead_id": "bd-3ar8v.1.5",
    "generated_at": timestamp,
    "git_commit": git_commit,
    "measurement_rounds": rounds,
    "warmup_rounds": warmup,
    "max_cv_threshold": max_cv,
    "summary": {
        "total_metrics": len(metric_results),
        "acceptable": acceptable_count,
        "high_variance": high_variance_count,
        "overall_status": "PASS" if high_variance_count == 0 else "WARN",
    },
    "metrics": metric_results,
}

with open(output_path, "w") as f:
    json.dump(baseline, f, indent=2)

print(f"  Metrics analyzed: {len(metric_results)}")
print(f"  Acceptable (CV <= {max_cv}): {acceptable_count}")
print(f"  High variance (CV > {max_cv}): {high_variance_count}")
print(f"  Output: {output_path}")
PYEOF

green "Baseline capture complete."
