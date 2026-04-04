#!/usr/bin/env bash
# generate-coverage.sh — FUZZ-P3.3 coverage dashboard generator
#
# Runs cargo-fuzz coverage for one or more targets and emits:
# - per-run machine-readable summary JSON
# - append-only history JSONL for trend tracking
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FUZZ_DIR="$PROJECT_ROOT/fuzz"
REPORT_DIR="$FUZZ_DIR/reports"

usage() {
    cat <<'EOF'
Usage: ./fuzz/generate-coverage.sh [OPTIONS]

Options:
  --target=NAME        Restrict to one target (repeatable); default is all targets
  --runs=N             Extra libFuzzer run budget passed as -runs=N (default: 0)
  --output=PATH        JSON report output path (default: fuzz/reports/fuzz_coverage_*.json)
  --history=PATH       JSONL trend/history path (default: fuzz/reports/fuzz_coverage_history.jsonl)
  --list-targets       Print available fuzz targets and exit
  --no-rch             Do not use rch
  --require-rch        Fail if rch is unavailable
  -h, --help           Show help

Notes:
- Heavy coverage runs should be executed through rch in this repository.
- This script emits best-effort coverage percentages when llvm-cov summary export is available.
EOF
}

is_positive_int() {
    case "$1" in
        ''|*[!0-9]*)
            return 1
            ;;
        *)
            [ "$1" -ge 0 ]
            ;;
    esac
}

run_cmd() {
    if [ "$RCH_MODE" = "enabled" ]; then
        rch exec -- "$@"
    else
        "$@"
    fi
}

resolve_host_triple() {
    local triple
    triple="$(rustc -vV 2>/dev/null | awk '/^host: / { print $2; exit }')"
    if [ -n "$triple" ]; then
        printf '%s' "$triple"
    else
        printf 'x86_64-unknown-linux-gnu'
    fi
}

resolve_llvm_tool() {
    local tool="$1"
    local host_triple sysroot candidate

    if command -v "$tool" >/dev/null 2>&1; then
        command -v "$tool"
        return 0
    fi

    host_triple="$(resolve_host_triple)"
    sysroot="$(rustc --print sysroot 2>/dev/null || true)"
    if [ -n "$sysroot" ]; then
        candidate="$sysroot/lib/rustlib/$host_triple/bin/$tool"
        if [ -x "$candidate" ]; then
            printf '%s' "$candidate"
            return 0
        fi
    fi

    return 1
}

extract_report_from_log() {
    local log_file="$1"
    local prefix="$2"
    grep -E "^${prefix}: " "$log_file" | tail -n 1 | sed "s/^${prefix}: //"
}

RCH_REQUEST="auto" # auto|always|never
RUNS=0
LIST_TARGETS=0
OUTPUT_PATH=""
HISTORY_PATH=""
declare -a TARGET_FILTERS=()

for arg in "$@"; do
    case "$arg" in
        --target=*)
            TARGET_FILTERS+=("${arg#--target=}")
            ;;
        --runs=*)
            RUNS="${arg#--runs=}"
            ;;
        --output=*)
            OUTPUT_PATH="${arg#--output=}"
            ;;
        --history=*)
            HISTORY_PATH="${arg#--history=}"
            ;;
        --list-targets)
            LIST_TARGETS=1
            ;;
        --no-rch)
            RCH_REQUEST="never"
            ;;
        --require-rch)
            RCH_REQUEST="always"
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            echo "Unknown argument: $arg" >&2
            usage >&2
            exit 2
            ;;
    esac
done

if ! is_positive_int "$RUNS"; then
    echo "Invalid --runs value: '$RUNS' (must be integer >= 0)" >&2
    exit 2
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo is required but was not found on PATH." >&2
    exit 2
fi

RCH_AVAILABLE=0
if command -v rch >/dev/null 2>&1; then
    RCH_AVAILABLE=1
fi

case "$RCH_REQUEST" in
    always)
        if [ "$RCH_AVAILABLE" -eq 0 ]; then
            echo "--require-rch was set but rch is unavailable on PATH" >&2
            exit 2
        fi
        RCH_MODE="enabled"
        ;;
    never)
        RCH_MODE="disabled"
        ;;
    auto)
        if [ "$RCH_AVAILABLE" -eq 1 ]; then
            RCH_MODE="enabled"
        else
            RCH_MODE="fallback"
        fi
        ;;
    *)
        echo "Internal error: invalid RCH_REQUEST '$RCH_REQUEST'" >&2
        exit 2
        ;;
esac

cd "$FUZZ_DIR"
mapfile -t ALL_TARGETS < <(cargo fuzz list 2>/dev/null | sed '/^[[:space:]]*$/d')
if [ "${#ALL_TARGETS[@]}" -eq 0 ]; then
    echo "No fuzz targets found under fuzz/." >&2
    exit 1
fi

if [ "$LIST_TARGETS" -eq 1 ]; then
    printf '%s\n' "${ALL_TARGETS[@]}"
    exit 0
fi

declare -a TARGETS=()
if [ "${#TARGET_FILTERS[@]}" -eq 0 ]; then
    TARGETS=("${ALL_TARGETS[@]}")
else
    for wanted in "${TARGET_FILTERS[@]}"; do
        found=0
        for target in "${ALL_TARGETS[@]}"; do
            if [ "$wanted" = "$target" ]; then
                TARGETS+=("$target")
                found=1
                break
            fi
        done
        if [ "$found" -eq 0 ]; then
            echo "Warning: requested target '$wanted' not found in cargo fuzz list" >&2
        fi
    done
fi

if [ "${#TARGETS[@]}" -eq 0 ]; then
    echo "No runnable targets after applying --target filters." >&2
    exit 2
fi

mkdir -p "$REPORT_DIR"

STAMP="$(date +%Y%m%d_%H%M%S)"
TIMESTAMP_UTC="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
REPORT_FILE="${OUTPUT_PATH:-$REPORT_DIR/fuzz_coverage_${STAMP}.json}"
HISTORY_FILE="${HISTORY_PATH:-$REPORT_DIR/fuzz_coverage_history.jsonl}"
SUITE_LOG="$REPORT_DIR/fuzz_coverage_${STAMP}.log"

LLVM_COV=""
if LLVM_COV="$(resolve_llvm_tool llvm-cov 2>/dev/null)"; then
    HAVE_LLVM_COV=1
else
    HAVE_LLVM_COV=0
fi

HOST_TRIPLE="$(resolve_host_triple)"
COVERAGE_TARGET_ROOT="$FUZZ_DIR/target/$HOST_TRIPLE/coverage"

echo "=== FUZZ Coverage Dashboard Run ===" | tee "$SUITE_LOG"
echo "Targets: ${#TARGETS[@]}" | tee -a "$SUITE_LOG"
echo "Runs per target: $RUNS" | tee -a "$SUITE_LOG"
echo "RCH mode: $RCH_MODE (request=$RCH_REQUEST, available=$RCH_AVAILABLE)" | tee -a "$SUITE_LOG"
echo "llvm-cov available: $HAVE_LLVM_COV" | tee -a "$SUITE_LOG"
echo "Report: $REPORT_FILE" | tee -a "$SUITE_LOG"
echo "History: $HISTORY_FILE" | tee -a "$SUITE_LOG"
echo "" | tee -a "$SUITE_LOG"

TOTAL_TARGETS="${#TARGETS[@]}"
PASSED=0
FAILED=0
RESULTS_JSON=""
RESULTS_SEP=""
LINE_PCT_SUM=0
LINE_PCT_COUNT=0

for target in "${TARGETS[@]}"; do
    TARGET_LOG="$REPORT_DIR/fuzz_coverage_${target}_${STAMP}.log"
    TARGET_SUMMARY_JSON="$REPORT_DIR/fuzz_coverage_${target}_${STAMP}_llvm_summary.json"
    TARGET_START_NS="$(date +%s%N)"

    echo ">>> Running coverage for $target (runs=$RUNS)" | tee -a "$SUITE_LOG"
    run_cmd cargo fuzz coverage "$target" -- "-runs=$RUNS" 2>&1 | tee "$TARGET_LOG"
    TARGET_EXIT=${PIPESTATUS[0]}
    TARGET_END_NS="$(date +%s%N)"
    TARGET_TIME_MS=$(( (TARGET_END_NS - TARGET_START_NS) / 1000000 ))

    COVERAGE_BINARY="$COVERAGE_TARGET_ROOT/$HOST_TRIPLE/release/$target"
    if [ ! -x "$COVERAGE_BINARY" ]; then
        COVERAGE_BINARY="$(find "$COVERAGE_TARGET_ROOT" -type f -path "*/$HOST_TRIPLE/release/$target" -perm -u+x 2>/dev/null | head -n 1)"
    fi

    PROFDATA_PATH="$(find "$COVERAGE_TARGET_ROOT" -type f -name '*.profdata' 2>/dev/null | sort | tail -n 1)"
    COVERAGE_INDEX_HTML="$(find "$COVERAGE_TARGET_ROOT" -type f -name 'index.html' 2>/dev/null | head -n 1)"

    LINES_PERCENT_JSON="null"
    FUNCTIONS_PERCENT_JSON="null"
    REGIONS_PERCENT_JSON="null"
    SUMMARY_PATH_JSON="null"

    if [ "$HAVE_LLVM_COV" -eq 1 ] && [ -n "$PROFDATA_PATH" ] && [ -n "$COVERAGE_BINARY" ] && [ -x "$COVERAGE_BINARY" ]; then
        if "$LLVM_COV" export -summary-only -instr-profile="$PROFDATA_PATH" "$COVERAGE_BINARY" > "$TARGET_SUMMARY_JSON" 2>>"$TARGET_LOG"; then
            if METRICS_JSON="$(
                python3 - "$TARGET_SUMMARY_JSON" <<'PY'
import json
import sys

path = sys.argv[1]
with open(path, "r", encoding="utf-8") as fh:
    payload = json.load(fh)

totals = {}
if isinstance(payload, dict):
    data = payload.get("data")
    if isinstance(data, list) and data:
        entry = data[0]
        if isinstance(entry, dict):
            maybe_totals = entry.get("totals")
            if isinstance(maybe_totals, dict):
                totals = maybe_totals

def metric(name):
    m = totals.get(name, {})
    if not isinstance(m, dict):
        return None
    pct = m.get("percent")
    try:
        return float(pct)
    except Exception:
        return None

lines = metric("lines")
functions = metric("functions")
regions = metric("regions")

print(json.dumps({
    "lines_percent": lines,
    "functions_percent": functions,
    "regions_percent": regions,
}, separators=(",", ":")))
PY
            )"; then
                LINES_PERCENT_JSON="$(printf '%s' "$METRICS_JSON" | jq -r '.lines_percent // "null"')"
                FUNCTIONS_PERCENT_JSON="$(printf '%s' "$METRICS_JSON" | jq -r '.functions_percent // "null"')"
                REGIONS_PERCENT_JSON="$(printf '%s' "$METRICS_JSON" | jq -r '.regions_percent // "null"')"
                SUMMARY_PATH_JSON="\"fuzz/reports/$(basename "$TARGET_SUMMARY_JSON")\""
                if [ "$LINES_PERCENT_JSON" != "null" ]; then
                    LINE_PCT_SUM="$(python3 - "$LINE_PCT_SUM" "$LINES_PERCENT_JSON" <<'PY'
import sys
print(float(sys.argv[1]) + float(sys.argv[2]))
PY
)"
                    LINE_PCT_COUNT=$((LINE_PCT_COUNT + 1))
                fi
            fi
        fi
    fi

    if [ "$TARGET_EXIT" -eq 0 ]; then
        STATUS="pass"
        PASSED=$((PASSED + 1))
    else
        STATUS="fail"
        FAILED=$((FAILED + 1))
    fi

    echo "    Status: $STATUS (exit=$TARGET_EXIT, time_ms=$TARGET_TIME_MS)" | tee -a "$SUITE_LOG"
    if [ "$LINES_PERCENT_JSON" != "null" ]; then
        echo "    Lines coverage: ${LINES_PERCENT_JSON}%" | tee -a "$SUITE_LOG"
    fi
    echo "" | tee -a "$SUITE_LOG"

    if [ -n "$COVERAGE_INDEX_HTML" ]; then
        COVERAGE_INDEX_JSON="\"${COVERAGE_INDEX_HTML#$PROJECT_ROOT/}\""
    else
        COVERAGE_INDEX_JSON="null"
    fi

    if [ -n "$PROFDATA_PATH" ]; then
        PROFDATA_JSON="\"${PROFDATA_PATH#$PROJECT_ROOT/}\""
    else
        PROFDATA_JSON="null"
    fi

    RESULTS_JSON="${RESULTS_JSON}${RESULTS_SEP}
    {
      \"target\": \"$target\",
      \"status\": \"$STATUS\",
      \"exit_code\": $TARGET_EXIT,
      \"time_ms\": $TARGET_TIME_MS,
      \"line_percent\": $LINES_PERCENT_JSON,
      \"functions_percent\": $FUNCTIONS_PERCENT_JSON,
      \"regions_percent\": $REGIONS_PERCENT_JSON,
      \"llvm_summary_file\": $SUMMARY_PATH_JSON,
      \"coverage_index_html\": $COVERAGE_INDEX_JSON,
      \"profdata\": $PROFDATA_JSON,
      \"log_file\": \"fuzz/reports/$(basename "$TARGET_LOG")\"
    }"
    RESULTS_SEP=","
done

AVG_LINE_PERCENT="null"
if [ "$LINE_PCT_COUNT" -gt 0 ]; then
    AVG_LINE_PERCENT="$(python3 - "$LINE_PCT_SUM" "$LINE_PCT_COUNT" <<'PY'
import sys
print(float(sys.argv[1]) / float(sys.argv[2]))
PY
)"
fi

cat > "$REPORT_FILE" <<EOFJSON
{
  "schema": "pi.fuzz.coverage_report.v1",
  "timestamp": "$TIMESTAMP_UTC",
  "rch_mode": "$RCH_MODE",
  "runs_per_target": $RUNS,
  "targets": [${RESULTS_JSON}
  ],
  "summary": {
    "total_targets": $TOTAL_TARGETS,
    "passed": $PASSED,
    "failed": $FAILED,
    "targets_with_line_coverage": $LINE_PCT_COUNT,
    "average_line_percent": $AVG_LINE_PERCENT
  }
}
EOFJSON

mkdir -p "$(dirname "$HISTORY_FILE")"
cat >> "$HISTORY_FILE" <<EOFJSONL
{"schema":"pi.fuzz.coverage_history.v1","timestamp":"$TIMESTAMP_UTC","report_file":"${REPORT_FILE#$PROJECT_ROOT/}","total_targets":$TOTAL_TARGETS,"passed":$PASSED,"failed":$FAILED,"average_line_percent":$AVG_LINE_PERCENT}
EOFJSONL

echo "=== Coverage Summary ===" | tee -a "$SUITE_LOG"
echo "Targets: $TOTAL_TARGETS | Passed: $PASSED | Failed: $FAILED" | tee -a "$SUITE_LOG"
echo "Average line coverage: $AVG_LINE_PERCENT" | tee -a "$SUITE_LOG"
echo "Report: $REPORT_FILE" | tee -a "$SUITE_LOG"
echo "History JSONL: $HISTORY_FILE" | tee -a "$SUITE_LOG"

if [ "$FAILED" -ne 0 ]; then
    echo "RESULT: FAIL (one or more targets failed coverage run)" | tee -a "$SUITE_LOG"
    exit 1
fi

echo "RESULT: PASS" | tee -a "$SUITE_LOG"
exit 0
