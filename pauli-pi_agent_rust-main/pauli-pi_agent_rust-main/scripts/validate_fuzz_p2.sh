#!/usr/bin/env bash
# validate_fuzz_p2.sh — Phase 2 Fuzz Validation Suite
#
# Builds all fuzz harnesses, runs each for a configurable duration (default 60s),
# and emits a machine-readable JSON report under fuzz/reports/.
#
# CPU-heavy operations run through `rch exec --` by default when `rch` is present.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FUZZ_DIR="$PROJECT_ROOT/fuzz"
REPORT_DIR="$FUZZ_DIR/reports"

usage() {
    cat <<'EOF'
Usage: ./scripts/validate_fuzz_p2.sh [OPTIONS]

Options:
  --time=SECONDS       Max fuzz run time per target (default: 60)
  --target=NAME        Run only the named target (repeatable)
  --no-rch             Never use rch; run cargo-fuzz locally
  --require-rch        Fail if rch is unavailable
  -h, --help           Show help

Examples:
  ./scripts/validate_fuzz_p2.sh
  ./scripts/validate_fuzz_p2.sh --time=15
  ./scripts/validate_fuzz_p2.sh --target=fuzz_sse_parser --target=fuzz_tool_paths --time=30
  ./scripts/validate_fuzz_p2.sh --require-rch
EOF
}

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

count_files() {
    local dir="$1"
    if [ ! -d "$dir" ]; then
        echo 0
        return
    fi
    find "$dir" -maxdepth 1 -type f 2>/dev/null | wc -l | tr -d ' '
}

run_cmd() {
    if [ "$RCH_MODE" = "enabled" ]; then
        env "RCH_FORCE_REMOTE=${RCH_FORCE_REMOTE:-true}" rch exec -- "$@"
    else
        "$@"
    fi
}

resolve_target_triple() {
    local host_triple=""
    host_triple="$(rustc -vV 2>/dev/null | awk '/^host: / { print $2; exit }')"
    if [ -n "$host_triple" ]; then
        echo "$host_triple"
    else
        echo "x86_64-unknown-linux-gnu"
    fi
}

resolve_target_binary() {
    local bin_name="$1"
    local primary="$FUZZ_BIN_DIR/$bin_name"
    local fallback=""

    if [ -x "$primary" ]; then
        echo "$primary"
        return 0
    fi

    fallback="$(find "$CARGO_TARGET_DIR" -type f -path "*/release/$bin_name" -perm -u+x 2>/dev/null | head -n 1)"
    if [ -n "$fallback" ] && [ -x "$fallback" ]; then
        echo "$fallback"
        return 0
    fi

    return 1
}

choose_target_root() {
    local local_root="$PROJECT_ROOT/.tmp/${USER:-agent}"
    local shm_root="/data/tmp/pi_agent_rust/${USER:-agent}"
    local min_shm_free_kb="${FUZZ_VALIDATE_MIN_SHM_KB:-8388608}" # 8 GiB default
    local shm_free_kb=""

    if ! is_positive_int "$min_shm_free_kb"; then
        min_shm_free_kb=8388608
    fi

    if [ -d /dev/shm ] && [ -w /dev/shm ]; then
        shm_free_kb="$(df -Pk /dev/shm 2>/dev/null | awk 'NR==2 {print $4}')"
        if is_positive_int "$shm_free_kb" && [ "$shm_free_kb" -ge "$min_shm_free_kb" ]; then
            echo "$shm_root"
            return
        fi
    fi

    echo "$local_root"
}

# -------------------------------------------------------------------
# Parse arguments
# -------------------------------------------------------------------
MAX_TIME=60
RCH_REQUEST="auto" # auto | always | never
declare -a TARGET_FILTERS=()

for arg in "$@"; do
    case "$arg" in
        --time=*)
            MAX_TIME="${arg#--time=}"
            ;;
        --target=*)
            TARGET_FILTERS+=("${arg#--target=}")
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

if ! is_positive_int "$MAX_TIME"; then
    echo "Invalid --time value: '$MAX_TIME' (must be positive integer seconds)" >&2
    exit 2
fi

if [ ! -d "$FUZZ_DIR" ]; then
    echo "Missing fuzz directory: $FUZZ_DIR" >&2
    exit 2
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo is required but was not found on PATH" >&2
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
        echo "Internal error: invalid RCH_REQUEST='$RCH_REQUEST'" >&2
        exit 2
        ;;
esac

mkdir -p "$REPORT_DIR"

TIMESTAMP_UTC="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
STAMP="$(date +%Y%m%d_%H%M%S)"
REPORT_FILE="$REPORT_DIR/p2_validation_${STAMP}.json"
BUILD_LOG="$REPORT_DIR/build_${STAMP}.log"

echo "=== FUZZ-V2 Phase 2 Validation Suite ==="
echo "Time per target: ${MAX_TIME}s"
echo "RCH mode: $RCH_MODE (request=$RCH_REQUEST, available=$RCH_AVAILABLE)"
echo "Report: $REPORT_FILE"
echo ""

cd "$FUZZ_DIR"

if [ -z "${CARGO_TARGET_DIR:-}" ]; then
    TARGET_ROOT="$(choose_target_root)"
    export CARGO_TARGET_DIR="$TARGET_ROOT/fuzz_validate_${STAMP}_$$/target"
fi

if [ -z "${TMPDIR:-}" ]; then
    export TMPDIR="$(dirname "$CARGO_TARGET_DIR")/tmp"
fi

mkdir -p "$CARGO_TARGET_DIR" "$TMPDIR"

echo "CARGO_TARGET_DIR: $CARGO_TARGET_DIR"
echo "TMPDIR: $TMPDIR"
echo ""

# -------------------------------------------------------------------
# Step 1: Build all fuzz targets
# -------------------------------------------------------------------
echo ">>> Building all fuzz targets..."
BUILD_START_NS=$(date +%s%N)
run_cmd cargo fuzz build 2>&1 | tee "$BUILD_LOG"
BUILD_EXIT=${PIPESTATUS[0]}
BUILD_END_NS=$(date +%s%N)
BUILD_TIME_MS=$(( (BUILD_END_NS - BUILD_START_NS) / 1000000 ))

if [ "$BUILD_EXIT" -eq 0 ]; then
    BUILD_STATUS="pass"
else
    BUILD_STATUS="fail"
fi

echo ""
echo "Build status: $BUILD_STATUS (${BUILD_TIME_MS}ms)"
echo ""

if [ "$BUILD_STATUS" = "fail" ]; then
    cat > "$REPORT_FILE" <<EOFJSON
{
  "phase": "P2",
  "timestamp": "$TIMESTAMP_UTC",
  "rch_mode": "$RCH_MODE",
  "rch_request": "$RCH_REQUEST",
  "build_status": "fail",
  "build_exit_code": $BUILD_EXIT,
  "build_time_ms": $BUILD_TIME_MS,
  "build_log_file": "fuzz/reports/$(basename "$BUILD_LOG")",
  "max_time_per_target_s": $MAX_TIME,
  "targets_requested": [],
  "targets": [],
  "summary": {
    "total_targets": 0,
    "passed": 0,
    "failed": 0,
    "crashed": 0,
    "total_corpus_growth": 0,
    "total_new_artifacts": 0,
    "total_time_ms": $BUILD_TIME_MS
  }
}
EOFJSON
    echo "Build failed. Report saved to $REPORT_FILE"
    exit 1
fi

TARGET_TRIPLE="$(resolve_target_triple)"
FUZZ_BIN_DIR="$CARGO_TARGET_DIR/$TARGET_TRIPLE/release"

echo "Target triple: $TARGET_TRIPLE"
echo "Fuzz binary directory: $FUZZ_BIN_DIR"
echo ""

# -------------------------------------------------------------------
# Step 2: List/filter fuzz targets
# -------------------------------------------------------------------
mapfile -t ALL_TARGETS < <(cargo fuzz list 2>/dev/null | sed '/^[[:space:]]*$/d')

if [ "${#ALL_TARGETS[@]}" -eq 0 ]; then
    echo "No fuzz targets found." >&2
    exit 1
fi

declare -a TARGETS=()
if [ "${#TARGET_FILTERS[@]}" -eq 0 ]; then
    TARGETS=("${ALL_TARGETS[@]}")
else
    for wanted in "${TARGET_FILTERS[@]}"; do
        found=0
        for candidate in "${ALL_TARGETS[@]}"; do
            if [ "$candidate" = "$wanted" ]; then
                TARGETS+=("$candidate")
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

TOTAL_TARGETS="${#TARGETS[@]}"
echo "Found ${#ALL_TARGETS[@]} total fuzz targets; running $TOTAL_TARGETS target(s)."
echo ""

# -------------------------------------------------------------------
# Step 3: Run each target
# -------------------------------------------------------------------
OVERALL_EXIT=0
TARGET_RESULTS=""
TARGET_RESULTS_SEP=""
PASSED=0
FAILED=0
CRASHED=0
TOTAL_CORPUS_GROWTH=0
TOTAL_NEW_ARTIFACTS=0
TOTAL_RUN_TIME_MS=0
TARGET_IDX=0

for target in "${TARGETS[@]}"; do
    TARGET_IDX=$((TARGET_IDX + 1))
    echo ">>> [$TARGET_IDX/$TOTAL_TARGETS] Running $target for ${MAX_TIME}s..."

    TARGET_LOG="$REPORT_DIR/${target}_${STAMP}.log"
    CORPUS_DIR="$FUZZ_DIR/corpus/$target"
    ARTIFACT_DIR="$FUZZ_DIR/artifacts/$target"

    mkdir -p "$CORPUS_DIR" "$ARTIFACT_DIR"

    INITIAL_CORPUS="$(count_files "$CORPUS_DIR")"
    INITIAL_ARTIFACTS="$(count_files "$ARTIFACT_DIR")"
    SEED_CORPUS="$INITIAL_CORPUS"

    RUN_START_NS=$(date +%s%N)
    TARGET_BIN=""
    if TARGET_BIN="$(resolve_target_binary "$target")"; then
        "$TARGET_BIN" \
            -max_total_time="$MAX_TIME" \
            -artifact_prefix="$ARTIFACT_DIR/" \
            "$CORPUS_DIR" \
            2>&1 | tee "$TARGET_LOG"
        TARGET_EXIT=${PIPESTATUS[0]}
    else
        echo "Missing executable for target '$target' under $CARGO_TARGET_DIR" | tee "$TARGET_LOG"
        TARGET_EXIT=127
    fi
    RUN_END_NS=$(date +%s%N)
    RUN_TIME_MS=$(( (RUN_END_NS - RUN_START_NS) / 1000000 ))
    TOTAL_RUN_TIME_MS=$((TOTAL_RUN_TIME_MS + RUN_TIME_MS))

    FINAL_CORPUS="$(count_files "$CORPUS_DIR")"
    FINAL_ARTIFACTS="$(count_files "$ARTIFACT_DIR")"

    NEW_CORPUS=$((FINAL_CORPUS - INITIAL_CORPUS))
    if [ "$NEW_CORPUS" -lt 0 ]; then
        NEW_CORPUS=0
    fi
    TOTAL_CORPUS_GROWTH=$((TOTAL_CORPUS_GROWTH + NEW_CORPUS))

    NEW_ARTIFACTS=$((FINAL_ARTIFACTS - INITIAL_ARTIFACTS))
    if [ "$NEW_ARTIFACTS" -lt 0 ]; then
        NEW_ARTIFACTS=0
    fi
    TOTAL_NEW_ARTIFACTS=$((TOTAL_NEW_ARTIFACTS + NEW_ARTIFACTS))

    if [ "$TARGET_EXIT" -eq 0 ] && [ "$NEW_ARTIFACTS" -eq 0 ]; then
        TARGET_STATUS="pass"
        PASSED=$((PASSED + 1))
    elif [ "$NEW_ARTIFACTS" -gt 0 ]; then
        TARGET_STATUS="crashed"
        CRASHED=$((CRASHED + 1))
        OVERALL_EXIT=1
    else
        TARGET_STATUS="fail"
        FAILED=$((FAILED + 1))
        OVERALL_EXIT=1
    fi

    echo "    Status: $TARGET_STATUS | Exit: $TARGET_EXIT | Time: ${RUN_TIME_MS}ms"
    echo "    Corpus: $INITIAL_CORPUS -> $FINAL_CORPUS (+$NEW_CORPUS)"
    echo "    Artifacts: $INITIAL_ARTIFACTS -> $FINAL_ARTIFACTS (+$NEW_ARTIFACTS)"
    echo ""

    TARGET_RESULTS="${TARGET_RESULTS}${TARGET_RESULTS_SEP}
    {
      \"name\": \"$target\",
      \"status\": \"$TARGET_STATUS\",
      \"exit_code\": $TARGET_EXIT,
      \"time_ms\": $RUN_TIME_MS,
      \"corpus_size\": $FINAL_CORPUS,
      \"new_corpus_entries\": $NEW_CORPUS,
      \"seed_corpus_size\": $SEED_CORPUS,
      \"artifacts_total\": $FINAL_ARTIFACTS,
      \"new_artifacts\": $NEW_ARTIFACTS,
      \"crashes_found\": $NEW_ARTIFACTS,
      \"log_file\": \"fuzz/reports/$(basename "$TARGET_LOG")\"
    }"
    TARGET_RESULTS_SEP=","
done

# -------------------------------------------------------------------
# Step 4: Generate JSON report
# -------------------------------------------------------------------
TOTAL_TIME_MS=$((BUILD_TIME_MS + TOTAL_RUN_TIME_MS))

TARGETS_REQUESTED_JSON=""
TARGETS_REQUESTED_SEP=""
for target in "${TARGET_FILTERS[@]}"; do
    TARGETS_REQUESTED_JSON="${TARGETS_REQUESTED_JSON}${TARGETS_REQUESTED_SEP}\"$target\""
    TARGETS_REQUESTED_SEP=", "
done

cat > "$REPORT_FILE" <<EOFJSON
{
  "phase": "P2",
  "timestamp": "$TIMESTAMP_UTC",
  "rch_mode": "$RCH_MODE",
  "rch_request": "$RCH_REQUEST",
  "build_status": "$BUILD_STATUS",
  "build_exit_code": $BUILD_EXIT,
  "build_time_ms": $BUILD_TIME_MS,
  "build_log_file": "fuzz/reports/$(basename "$BUILD_LOG")",
  "max_time_per_target_s": $MAX_TIME,
  "targets_requested": [${TARGETS_REQUESTED_JSON}],
  "targets": [${TARGET_RESULTS}
  ],
  "summary": {
    "total_targets": $TOTAL_TARGETS,
    "passed": $PASSED,
    "failed": $FAILED,
    "crashed": $CRASHED,
    "total_corpus_growth": $TOTAL_CORPUS_GROWTH,
    "total_new_artifacts": $TOTAL_NEW_ARTIFACTS,
    "total_time_ms": $TOTAL_TIME_MS
  }
}
EOFJSON

echo ""
echo "=== Summary ==="
echo "Total targets: $TOTAL_TARGETS"
echo "Passed: $PASSED | Failed: $FAILED | Crashed: $CRASHED"
echo "Total corpus growth: $TOTAL_CORPUS_GROWTH"
echo "Total new artifacts: $TOTAL_NEW_ARTIFACTS"
echo "Total time: $((TOTAL_TIME_MS / 1000))s"
echo "Report: $REPORT_FILE"
echo ""

if [ "$OVERALL_EXIT" -ne 0 ]; then
    echo "RESULT: FAIL (some targets crashed or failed)"
else
    echo "RESULT: PASS (all targets ran without crashes)"
fi

exit "$OVERALL_EXIT"
