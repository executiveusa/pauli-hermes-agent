#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

usage() {
    cat <<'EOF'
Usage: ./scripts/e2e/run_runtime_risk_telemetry.sh [OPTIONS]

Options:
  --no-rch        Run cargo locally (explicit override)
  --require-rch   Fail if rch is unavailable
  -h, --help      Show help
EOF
}

RCH_REQUEST="auto" # auto | always | never
for arg in "$@"; do
    case "$arg" in
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
            RCH_MODE="disabled"
        fi
        ;;
    *)
        echo "Internal error: invalid RCH_REQUEST='$RCH_REQUEST'" >&2
        exit 2
        ;;
esac

run_cargo() {
    if [ "$RCH_MODE" = "enabled" ]; then
        RCH_FORCE_REMOTE="${RCH_FORCE_REMOTE:-true}" rch exec -- cargo "$@"
    else
        cargo "$@"
    fi
}

STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
ARTIFACT_DIR="${E2E_ARTIFACT_DIR:-$PROJECT_ROOT/tests/e2e_results/runtime-risk-telemetry/$STAMP}"
mkdir -p "$ARTIFACT_DIR"

export CI_CORRELATION_ID="${CI_CORRELATION_ID:-runtime-risk-telemetry-$STAMP}"
export TEST_LOG_JSONL_PATH="$ARTIFACT_DIR/test-log.jsonl"
export TEST_ARTIFACT_INDEX_PATH="$ARTIFACT_DIR/artifact-index.jsonl"

run_cargo test --test e2e_runtime_risk_telemetry -- --nocapture

echo "Runtime-risk telemetry E2E completed"
echo "Artifacts: $ARTIFACT_DIR"
