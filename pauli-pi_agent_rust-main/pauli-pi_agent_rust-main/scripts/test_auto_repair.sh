#!/bin/bash
# E2E auto-repair test runner with human-readable reporting.
#
# Usage:
#   ./scripts/test_auto_repair.sh          # run full corpus
#   ./scripts/test_auto_repair.sh --quick  # run report-structure test only
#   ./scripts/test_auto_repair.sh --require-rch  # require remote offload
#   ./scripts/test_auto_repair.sh --no-rch       # force local cargo execution
#
# Outputs:
#   tests/ext_conformance/reports/auto_repair_report.md
#   tests/ext_conformance/reports/auto_repair_summary.json
# Environment:
#   AUTO_REPAIR_CARGO_RUNNER  Cargo runner mode: rch | auto | local (default: rch)
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPORT_DIR="$PROJECT_ROOT/tests/ext_conformance/reports"
TIMESTAMP=$(date -u +%Y%m%dT%H%M%SZ)
LOG_FILE="${REPORT_DIR}/auto_repair_${TIMESTAMP}.log"
CARGO_RUNNER_REQUEST="${AUTO_REPAIR_CARGO_RUNNER:-rch}" # rch | auto | local
CARGO_RUNNER_MODE="local"
declare -a CARGO_RUNNER_ARGS=("cargo")
QUICK=0
SEEN_NO_RCH=false
SEEN_REQUIRE_RCH=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --quick)
            QUICK=1
            shift
            ;;
        --no-rch)
            if [[ "$SEEN_REQUIRE_RCH" == true ]]; then
                echo "Cannot combine --no-rch and --require-rch" >&2
                exit 1
            fi
            SEEN_NO_RCH=true
            CARGO_RUNNER_REQUEST="local"
            shift
            ;;
        --require-rch)
            if [[ "$SEEN_NO_RCH" == true ]]; then
                echo "Cannot combine --require-rch and --no-rch" >&2
                exit 1
            fi
            SEEN_REQUIRE_RCH=true
            CARGO_RUNNER_REQUEST="rch"
            shift
            ;;
        --help|-h)
            sed -n '2,/^set -euo pipefail/{/^#/p}' "$0" | sed 's/^# \?//'
            exit 0
            ;;
        *)
            echo "Unknown flag: $1" >&2
            exit 1
            ;;
    esac
done

if [[ "$CARGO_RUNNER_REQUEST" != "rch" && "$CARGO_RUNNER_REQUEST" != "auto" && "$CARGO_RUNNER_REQUEST" != "local" ]]; then
    echo "Invalid AUTO_REPAIR_CARGO_RUNNER value: $CARGO_RUNNER_REQUEST (expected: rch|auto|local)" >&2
    exit 2
fi

if [[ "$CARGO_RUNNER_REQUEST" == "rch" ]]; then
    if ! command -v rch >/dev/null 2>&1; then
        echo "AUTO_REPAIR_CARGO_RUNNER=rch requested, but 'rch' is not available in PATH." >&2
        exit 2
    fi
    if ! rch check --quiet >/dev/null 2>&1; then
        echo "'rch check' failed; refusing heavy local cargo fallback. Fix rch or pass --no-rch." >&2
        exit 2
    fi
    CARGO_RUNNER_MODE="rch"
    CARGO_RUNNER_ARGS=("rch" "exec" "--" "cargo")
elif [[ "$CARGO_RUNNER_REQUEST" == "auto" ]] && command -v rch >/dev/null 2>&1; then
    if rch check --quiet >/dev/null 2>&1; then
        CARGO_RUNNER_MODE="rch"
        CARGO_RUNNER_ARGS=("rch" "exec" "--" "cargo")
    else
        echo "rch detected but unhealthy; auto mode will run cargo locally (set --require-rch to fail fast)." >&2
    fi
fi

cd "$PROJECT_ROOT"

echo "=== Auto-Repair E2E Test ==="
echo "Started: $(date -u)"
echo "Log: ${LOG_FILE}"
echo "Cargo runner: ${CARGO_RUNNER_MODE} (request=${CARGO_RUNNER_REQUEST})"
echo ""

if [[ "$QUICK" -eq 1 ]]; then
    echo "Running quick validation (report structure only)..."
    TMPDIR="$PROJECT_ROOT/target/tmp" "${CARGO_RUNNER_ARGS[@]}" test --test e2e_auto_repair \
        report_structure_is_valid \
        -- --nocapture 2>&1 | tee "${LOG_FILE}"
else
    echo "Running full corpus with auto-repair..."
    TMPDIR="$PROJECT_ROOT/target/tmp" "${CARGO_RUNNER_ARGS[@]}" test --test e2e_auto_repair \
        full_corpus_with_auto_repair \
        -- --nocapture --test-threads=1 2>&1 | tee "${LOG_FILE}"
fi

echo ""
echo "=== Done ==="

# Print summary from generated files
if [[ -f "${REPORT_DIR}/auto_repair_summary.json" ]]; then
    echo ""
    echo "--- Summary from auto_repair_summary.json ---"
    python3 -c "
import json, sys
d = json.load(open('${REPORT_DIR}/auto_repair_summary.json'))
print(f\"Total: {d['total']} | Clean: {d['clean_pass']} | Repaired: {d['repaired_pass']} | Failed: {d['failed']} | Skipped: {d['skipped']}\")
if d.get('repairs_by_pattern'):
    print('Repairs by pattern:')
    for pat, cnt in d['repairs_by_pattern'].items():
        print(f'  {pat}: {cnt}')
" 2>/dev/null || true
fi

if [[ -f "${REPORT_DIR}/auto_repair_report.md" ]]; then
    echo ""
    echo "Markdown report: ${REPORT_DIR}/auto_repair_report.md"
fi

echo "JSON summary: ${REPORT_DIR}/auto_repair_summary.json"
echo "Full log: ${LOG_FILE}"
