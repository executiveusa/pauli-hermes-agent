#!/usr/bin/env bash
# fuzz_e2e.sh — FUZZ-V3 end-to-end orchestration runner
#
# Runs Phase 1 (proptest) and Phase 2 (libFuzzer) validators, emits a unified
# report, and writes a machine-readable JSONL event stream for each step.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPORT_DIR="$PROJECT_ROOT/fuzz/reports"

usage() {
    cat <<'EOF'
Usage: ./scripts/fuzz_e2e.sh [OPTIONS]

Modes:
  (default)            Full run: P1 + P2 (60s per fuzz target)
  --quick              Quick run: P1 + P2 (10s, defaults P2 to fuzz_smoke)
  --deep               Deep run: P1 + P2 (1800s per fuzz target)
  --report             Regenerate unified report/events from existing P1/P2 reports

Options:
  --p1-min-cases=N     Override P1 aggregate case threshold (default: 2000)
  --p2-time=SECONDS    Override P2 per-target time budget (mode default otherwise)
  --target=NAME        Restrict P2 to target(s); repeatable
  --p1-report=PATH     Existing P1 report for --report mode (default: latest)
  --p2-report=PATH     Existing P2 report for --report mode (default: latest)
  --output=PATH        Unified JSON output path (default: fuzz/reports/fuzz_e2e_*.json)
  --events=PATH        JSONL event output path (default: fuzz/reports/fuzz_e2e_*.jsonl)
  --no-rch             Forward local execution mode to phase scripts
  --require-rch        Require rch in phase scripts
  -h, --help           Show help

Exit codes:
  0: all phases pass
  1: P1 failures
  2: P2 crashes/failures
  3: both P1 and P2 failures
  4: P2 build/infrastructure failure

Examples:
  ./scripts/fuzz_e2e.sh
  ./scripts/fuzz_e2e.sh --quick
  ./scripts/fuzz_e2e.sh --deep --target=fuzz_sse_parser
  ./scripts/fuzz_e2e.sh --report
EOF
}

supports_color() {
    [ -t 1 ] && [ "${NO_COLOR:-}" = "" ]
}

if supports_color; then
    C_RESET=$'\033[0m'
    C_RED=$'\033[31m'
    C_GREEN=$'\033[32m'
    C_YELLOW=$'\033[33m'
else
    C_RESET=""
    C_RED=""
    C_GREEN=""
    C_YELLOW=""
fi

is_positive_int() {
    case "$1" in
        ''|*[!0-9]*)
            return 1
            ;;
        *)
            [ "$1" -gt 0 ]
            ;;
    esac
}

latest_report() {
    local pattern="$1"
    find "$REPORT_DIR" -maxdepth 1 -type f -name "$pattern" | sort | tail -n 1
}

extract_report_path_from_log() {
    local log_path="$1"
    local report_path
    report_path="$(grep -E '^Report: ' "$log_path" | tail -n 1 | sed 's/^Report: //')"
    printf '%s' "$report_path"
}

emit_event() {
    local event_name="$1"
    local extra="${2:-}"
    printf '{"ts":"%s","event":"%s"%s}\n' \
        "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
        "$event_name" \
        "$extra" >> "$EVENTS_FILE"
}

MODE="full" # full|quick|deep
REPORT_ONLY=0
P1_MIN_CASES=2000
P2_TIME_OVERRIDE=""
P1_REPORT_INPUT=""
P2_REPORT_INPUT=""
OUTPUT_PATH=""
EVENTS_PATH=""
RCH_FORWARD="" # --no-rch|--require-rch
declare -a P2_TARGETS=()

for arg in "$@"; do
    case "$arg" in
        --quick)
            MODE="quick"
            ;;
        --deep)
            MODE="deep"
            ;;
        --report)
            REPORT_ONLY=1
            ;;
        --p1-min-cases=*)
            P1_MIN_CASES="${arg#--p1-min-cases=}"
            ;;
        --p2-time=*)
            P2_TIME_OVERRIDE="${arg#--p2-time=}"
            ;;
        --target=*)
            P2_TARGETS+=("${arg#--target=}")
            ;;
        --p1-report=*)
            P1_REPORT_INPUT="${arg#--p1-report=}"
            ;;
        --p2-report=*)
            P2_REPORT_INPUT="${arg#--p2-report=}"
            ;;
        --output=*)
            OUTPUT_PATH="${arg#--output=}"
            ;;
        --events=*)
            EVENTS_PATH="${arg#--events=}"
            ;;
        --no-rch)
            if [ "$RCH_FORWARD" = "--require-rch" ]; then
                echo "Cannot combine --no-rch and --require-rch" >&2
                exit 2
            fi
            RCH_FORWARD="--no-rch"
            ;;
        --require-rch)
            if [ "$RCH_FORWARD" = "--no-rch" ]; then
                echo "Cannot combine --no-rch and --require-rch" >&2
                exit 2
            fi
            RCH_FORWARD="--require-rch"
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

if ! is_positive_int "$P1_MIN_CASES"; then
    echo "Invalid --p1-min-cases value: '$P1_MIN_CASES'" >&2
    exit 2
fi

if [ -n "$P2_TIME_OVERRIDE" ] && ! is_positive_int "$P2_TIME_OVERRIDE"; then
    echo "Invalid --p2-time value: '$P2_TIME_OVERRIDE'" >&2
    exit 2
fi

mkdir -p "$REPORT_DIR"

TIMESTAMP_UTC="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
STAMP="$(date +%Y%m%d_%H%M%S)"
UNIFIED_REPORT="${OUTPUT_PATH:-$REPORT_DIR/fuzz_e2e_${STAMP}.json}"
EVENTS_FILE="${EVENTS_PATH:-$REPORT_DIR/fuzz_e2e_${STAMP}.jsonl}"
PIPELINE_LOG="$REPORT_DIR/fuzz_e2e_${STAMP}.log"
P1_LOG="$REPORT_DIR/fuzz_e2e_p1_${STAMP}.log"
P2_LOG="$REPORT_DIR/fuzz_e2e_p2_${STAMP}.log"

case "$MODE" in
    full)
        MODE_P2_TIME=60
        ;;
    quick)
        MODE_P2_TIME=10
        ;;
    deep)
        MODE_P2_TIME=1800
        ;;
    *)
        echo "Internal error: invalid MODE '$MODE'" >&2
        exit 2
        ;;
esac

P2_TIME="${P2_TIME_OVERRIDE:-$MODE_P2_TIME}"
if ! is_positive_int "$P2_TIME"; then
    echo "Internal error: invalid P2_TIME '$P2_TIME'" >&2
    exit 2
fi

if [ "$MODE" = "quick" ] && [ "${#P2_TARGETS[@]}" -eq 0 ]; then
    P2_TARGETS=("fuzz_smoke")
fi

printf '=== FUZZ-V3 E2E Orchestration ===\n' | tee "$PIPELINE_LOG"
printf 'Mode: %s\n' "$MODE" | tee -a "$PIPELINE_LOG"
printf 'Report-only: %s\n' "$REPORT_ONLY" | tee -a "$PIPELINE_LOG"
printf 'P1 min cases: %s\n' "$P1_MIN_CASES" | tee -a "$PIPELINE_LOG"
printf 'P2 time: %ss\n' "$P2_TIME" | tee -a "$PIPELINE_LOG"
printf 'RCH forward: %s\n' "${RCH_FORWARD:-auto}" | tee -a "$PIPELINE_LOG"
printf 'Unified report: %s\n' "$UNIFIED_REPORT" | tee -a "$PIPELINE_LOG"
printf 'Events JSONL: %s\n' "$EVENTS_FILE" | tee -a "$PIPELINE_LOG"
printf '\n' | tee -a "$PIPELINE_LOG"

: > "$EVENTS_FILE"
emit_event "pipeline_start" ",\"mode\":\"$MODE\",\"report_only\":$REPORT_ONLY"

P1_REPORT=""
P2_REPORT=""
P1_EXIT=0
P2_EXIT=0
P1_TIME_MS=0
P2_TIME_MS=0
RUN_P1=1
RUN_P2=1

if [ "$REPORT_ONLY" -eq 1 ]; then
    RUN_P1=0
    RUN_P2=0
fi

if [ "$RUN_P1" -eq 1 ]; then
    declare -a p1_cmd=("./scripts/validate_fuzz_p1.sh" "--min-cases=$P1_MIN_CASES")
    if [ -n "$RCH_FORWARD" ]; then
        p1_cmd+=("$RCH_FORWARD")
    fi

    emit_event "phase_start" ",\"phase\":\"P1\",\"min_cases\":$P1_MIN_CASES"
    printf '>>> Running P1: %s\n' "${p1_cmd[*]}" | tee -a "$PIPELINE_LOG"
    P1_START_NS=$(date +%s%N)
    "${p1_cmd[@]}" 2>&1 | tee "$P1_LOG"
    P1_EXIT=${PIPESTATUS[0]}
    P1_END_NS=$(date +%s%N)
    P1_TIME_MS=$(( (P1_END_NS - P1_START_NS) / 1000000 ))

    P1_REPORT="$(extract_report_path_from_log "$P1_LOG")"
    if [ -z "$P1_REPORT" ] || [ ! -f "$P1_REPORT" ]; then
        P1_REPORT="$(latest_report "p1_validation_*.json")"
    fi
    if [ -z "$P1_REPORT" ] || [ ! -f "$P1_REPORT" ]; then
        echo "Unable to locate P1 report after phase execution." >&2
        emit_event "phase_end" ",\"phase\":\"P1\",\"status\":\"fail\",\"exit_code\":$P1_EXIT,\"time_ms\":$P1_TIME_MS,\"report_found\":false"
        exit 2
    fi
    emit_event "phase_end" ",\"phase\":\"P1\",\"status\":\"done\",\"exit_code\":$P1_EXIT,\"time_ms\":$P1_TIME_MS,\"report_path\":\"$P1_REPORT\""
    printf 'P1 exit: %s | report: %s\n\n' "$P1_EXIT" "$P1_REPORT" | tee -a "$PIPELINE_LOG"
fi

if [ "$RUN_P2" -eq 1 ]; then
    declare -a p2_cmd=("./scripts/validate_fuzz_p2.sh" "--time=$P2_TIME")
    for target in "${P2_TARGETS[@]}"; do
        p2_cmd+=("--target=$target")
    done
    if [ -n "$RCH_FORWARD" ]; then
        p2_cmd+=("$RCH_FORWARD")
    fi

    emit_event "phase_start" ",\"phase\":\"P2\",\"time_s\":$P2_TIME,\"target_count\":${#P2_TARGETS[@]}"
    printf '>>> Running P2: %s\n' "${p2_cmd[*]}" | tee -a "$PIPELINE_LOG"
    P2_START_NS=$(date +%s%N)
    "${p2_cmd[@]}" 2>&1 | tee "$P2_LOG"
    P2_EXIT=${PIPESTATUS[0]}
    P2_END_NS=$(date +%s%N)
    P2_TIME_MS=$(( (P2_END_NS - P2_START_NS) / 1000000 ))

    P2_REPORT="$(extract_report_path_from_log "$P2_LOG")"
    if [ -z "$P2_REPORT" ] || [ ! -f "$P2_REPORT" ]; then
        P2_REPORT="$(latest_report "p2_validation_*.json")"
    fi
    if [ -z "$P2_REPORT" ] || [ ! -f "$P2_REPORT" ]; then
        echo "Unable to locate P2 report after phase execution." >&2
        emit_event "phase_end" ",\"phase\":\"P2\",\"status\":\"fail\",\"exit_code\":$P2_EXIT,\"time_ms\":$P2_TIME_MS,\"report_found\":false"
        exit 2
    fi
    emit_event "phase_end" ",\"phase\":\"P2\",\"status\":\"done\",\"exit_code\":$P2_EXIT,\"time_ms\":$P2_TIME_MS,\"report_path\":\"$P2_REPORT\""
    printf 'P2 exit: %s | report: %s\n\n' "$P2_EXIT" "$P2_REPORT" | tee -a "$PIPELINE_LOG"
fi

if [ "$REPORT_ONLY" -eq 1 ]; then
    if [ -n "$P1_REPORT_INPUT" ]; then
        P1_REPORT="$P1_REPORT_INPUT"
    else
        P1_REPORT="$(latest_report "p1_validation_*.json")"
    fi
    if [ -n "$P2_REPORT_INPUT" ]; then
        P2_REPORT="$P2_REPORT_INPUT"
    else
        P2_REPORT="$(latest_report "p2_validation_*.json")"
    fi

    if [ -z "$P1_REPORT" ] || [ ! -f "$P1_REPORT" ]; then
        echo "--report mode requires a P1 report (pass --p1-report=PATH or generate one first)." >&2
        exit 2
    fi
    if [ -z "$P2_REPORT" ] || [ ! -f "$P2_REPORT" ]; then
        echo "--report mode requires a P2 report (pass --p2-report=PATH or generate one first)." >&2
        exit 2
    fi

    emit_event "report_mode_inputs" ",\"p1_report\":\"$P1_REPORT\",\"p2_report\":\"$P2_REPORT\""
fi

PIPELINE_EXIT="$(
python3 - "$P1_REPORT" "$P2_REPORT" "$UNIFIED_REPORT" "$MODE" "$TIMESTAMP_UTC" "$P1_EXIT" "$P2_EXIT" "$P1_TIME_MS" "$P2_TIME_MS" <<'PY'
import json
import pathlib
import sys

p1_path = pathlib.Path(sys.argv[1])
p2_path = pathlib.Path(sys.argv[2])
out_path = pathlib.Path(sys.argv[3])
mode = sys.argv[4]
timestamp = sys.argv[5]
p1_phase_exit = int(sys.argv[6])
p2_phase_exit = int(sys.argv[7])
p1_time_ms = int(sys.argv[8])
p2_time_ms = int(sys.argv[9])

with p1_path.open("r", encoding="utf-8") as fh:
    p1 = json.load(fh)
with p2_path.open("r", encoding="utf-8") as fh:
    p2 = json.load(fh)

p1_failed = not (bool(p1.get("all_pass")) and bool(p1.get("case_target_met")))
p2_summary = p2.get("summary") or {}
p2_build_fail = p2.get("build_status") != "pass"
p2_fail_count = int(p2_summary.get("failed", 0) or 0)
p2_crash_count = int(p2_summary.get("crashed", 0) or 0)
p2_failed = p2_build_fail or p2_fail_count > 0 or p2_crash_count > 0

if p2_build_fail:
    pipeline_exit = 4
elif p1_failed and p2_failed:
    pipeline_exit = 3
elif p1_failed:
    pipeline_exit = 1
elif p2_failed:
    pipeline_exit = 2
else:
    pipeline_exit = 0

targets = p2.get("targets") or []
corpus_total = 0
for target in targets:
    try:
        corpus_total += int(target.get("corpus_size", 0) or 0)
    except Exception:
        pass

proptest_functions = int(p1.get("total_proptest_functions", 0) or 0)
proptest_cases = int(p1.get("total_cases_generated", 0) or 0)
fuzz_targets = int(p2_summary.get("total_targets", 0) or 0)
fuzz_total_time_ms = int(p2_summary.get("total_time_ms", 0) or 0)
fuzz_build_time_ms = int(p2.get("build_time_ms", 0) or 0)
fuzz_run_time_ms = max(0, fuzz_total_time_ms - fuzz_build_time_ms)
bugs_found = int(p2_summary.get("total_new_artifacts", 0) or 0)

unified = {
    "pipeline_version": "1.0",
    "timestamp": timestamp,
    "mode": mode,
    "phases": {
        "P1_proptest": p1,
        "P2_libfuzzer": p2,
    },
    "phase_runs": {
        "P1_proptest": {
            "exit_code": p1_phase_exit,
            "time_ms": p1_time_ms,
            "report_file": str(p1_path),
        },
        "P2_libfuzzer": {
            "exit_code": p2_phase_exit,
            "time_ms": p2_time_ms,
            "report_file": str(p2_path),
        },
    },
    "summary": {
        "overall_status": "pass" if pipeline_exit == 0 else "fail",
        "proptest_functions": proptest_functions,
        "proptest_cases": proptest_cases,
        "fuzz_targets": fuzz_targets,
        "fuzz_time_s": fuzz_run_time_ms // 1000,
        "total_crashes": p2_crash_count,
        "corpus_total": corpus_total,
        "bugs_found": bugs_found,
        "bugs_fixed": 0,
        "total_time_ms": p1_time_ms + p2_time_ms,
        "exit_code": pipeline_exit,
    },
}

out_path.parent.mkdir(parents=True, exist_ok=True)
with out_path.open("w", encoding="utf-8") as fh:
    json.dump(unified, fh, indent=2, sort_keys=False)
    fh.write("\n")

print(pipeline_exit)
PY
)"

case "$PIPELINE_EXIT" in
    ''|*[!0-9]*)
        echo "Failed to compute pipeline exit code." >&2
        exit 2
        ;;
esac

emit_event "report_generated" ",\"report\":\"$UNIFIED_REPORT\",\"pipeline_exit\":$PIPELINE_EXIT"
emit_event "pipeline_end" ",\"status\":\"$([ "$PIPELINE_EXIT" -eq 0 ] && echo pass || echo fail)\",\"exit_code\":$PIPELINE_EXIT,\"report\":\"$UNIFIED_REPORT\""

printf '=== FUZZ-V3 Summary ===\n' | tee -a "$PIPELINE_LOG"
printf 'Unified report: %s\n' "$UNIFIED_REPORT" | tee -a "$PIPELINE_LOG"
printf 'Events JSONL: %s\n' "$EVENTS_FILE" | tee -a "$PIPELINE_LOG"
printf 'Pipeline exit: %s\n' "$PIPELINE_EXIT" | tee -a "$PIPELINE_LOG"

if [ "$PIPELINE_EXIT" -eq 0 ]; then
    printf '%sRESULT: PASS%s\n' "$C_GREEN" "$C_RESET" | tee -a "$PIPELINE_LOG"
else
    case "$PIPELINE_EXIT" in
        1)
            FAIL_REASON="P1 failures"
            ;;
        2)
            FAIL_REASON="P2 crashes/failures"
            ;;
        3)
            FAIL_REASON="both P1 and P2 failures"
            ;;
        4)
            FAIL_REASON="P2 infrastructure/build failure"
            ;;
        *)
            FAIL_REASON="unexpected pipeline failure"
            ;;
    esac
    printf '%sRESULT: FAIL%s (%s%s%s)\n' \
        "$C_RED" \
        "$C_RESET" \
        "$C_YELLOW" \
        "$FAIL_REASON" \
        "$C_RESET" | tee -a "$PIPELINE_LOG"
fi

exit "$PIPELINE_EXIT"
