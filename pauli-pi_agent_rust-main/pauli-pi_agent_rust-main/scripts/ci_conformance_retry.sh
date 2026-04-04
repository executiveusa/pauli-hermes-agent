#!/usr/bin/env bash
# scripts/ci_conformance_retry.sh — CI retry wrapper for conformance tests.
#
# Wraps a single conformance test command with automatic retry on
# transient failures (oracle timeouts, resource exhaustion, etc.).
#
# Usage:
#   ./scripts/ci_conformance_retry.sh [--require-rch|--no-rch] <target_name> <test_command...>
#
# Example:
#   ./scripts/ci_conformance_retry.sh full-official \
#     cargo test --test ext_conformance_diff --features ext-conformance -- --nocapture
#
# Environment:
#   PI_CONFORMANCE_MAX_RETRIES   Max retries (default: 1)
#   PI_CONFORMANCE_RETRY_DELAY   Seconds between retries (default: 5)
#   PI_CONFORMANCE_CLASSIFY_ONLY Set to 1 to classify without retrying
#   PI_CONFORMANCE_CARGO_RUNNER  Cargo runner mode: rch | auto | local (default: rch)

set -euo pipefail

CARGO_RUNNER_REQUEST="${PI_CONFORMANCE_CARGO_RUNNER:-rch}" # rch | auto | local
CARGO_RUNNER_MODE="local"
SEEN_NO_RCH=false
SEEN_REQUIRE_RCH=false

while [[ $# -gt 0 ]]; do
    case "$1" in
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
            sed -n '2,/^$/p' "$0" | sed 's/^# \?//'
            exit 0
            ;;
        *)
            break
            ;;
    esac
done

TARGET="${1:?Usage: ci_conformance_retry.sh [--require-rch|--no-rch] <target> <command...>}"
shift
CMD=("$@")
if [[ "${#CMD[@]}" -eq 0 ]]; then
    echo "Usage: ci_conformance_retry.sh [--require-rch|--no-rch] <target> <command...>" >&2
    exit 1
fi

if [[ "$CARGO_RUNNER_REQUEST" != "rch" && "$CARGO_RUNNER_REQUEST" != "auto" && "$CARGO_RUNNER_REQUEST" != "local" ]]; then
    echo "Invalid PI_CONFORMANCE_CARGO_RUNNER value: $CARGO_RUNNER_REQUEST (expected: rch|auto|local)" >&2
    exit 2
fi

if [[ "$CARGO_RUNNER_REQUEST" == "rch" ]]; then
    if ! command -v rch >/dev/null 2>&1; then
        echo "PI_CONFORMANCE_CARGO_RUNNER=rch requested, but 'rch' is not available in PATH." >&2
        exit 2
    fi
    if ! rch check --quiet >/dev/null 2>&1; then
        echo "'rch check' failed; refusing heavy local cargo fallback. Fix rch or pass --no-rch." >&2
        exit 2
    fi
    CARGO_RUNNER_MODE="rch"
elif [[ "$CARGO_RUNNER_REQUEST" == "auto" ]] && command -v rch >/dev/null 2>&1; then
    if rch check --quiet >/dev/null 2>&1; then
        CARGO_RUNNER_MODE="rch"
    else
        echo "rch detected but unhealthy; auto mode will run cargo locally (set --require-rch to fail fast)." >&2
    fi
fi

if [[ "$CARGO_RUNNER_MODE" == "rch" ]] && [[ "${CMD[0]}" == "cargo" ]]; then
    CMD=("rch" "exec" "--" "${CMD[@]}")
fi

MAX_RETRIES="${PI_CONFORMANCE_MAX_RETRIES:-1}"
RETRY_DELAY="${PI_CONFORMANCE_RETRY_DELAY:-5}"
CLASSIFY_ONLY="${PI_CONFORMANCE_CLASSIFY_ONLY:-0}"
FLAKE_LOG="${PI_CONFORMANCE_FLAKE_LOG:-flake_events.jsonl}"

# ─── Known flake patterns (must match src/flake_classifier.rs) ──────────────

is_transient_failure() {
    local output_lower
    output_lower="$(echo "$1" | tr '[:upper:]' '[:lower:]')"

    # Oracle timeout
    if echo "$output_lower" | grep -qE '(oracle|bun).*(timed out|timeout)'; then
        echo "oracle_timeout"
        return 0
    fi
    # Resource exhaustion
    if echo "$output_lower" | grep -qE 'out of memory|enomem|cannot allocate'; then
        if echo "$output_lower" | grep -qE 'quickjs|allocation failed'; then
            echo "js_gc_pressure"
        else
            echo "resource_exhaustion"
        fi
        return 0
    fi
    # Filesystem contention
    if echo "$output_lower" | grep -qE 'ebusy|etxtbsy|resource busy'; then
        echo "fs_contention"
        return 0
    fi
    # Port conflict
    if echo "$output_lower" | grep -qE 'eaddrinuse|address already in use'; then
        echo "port_conflict"
        return 0
    fi
    # Temp directory race
    if echo "$output_lower" | grep -qE '(no such file or directory|enoent).*tmp'; then
        echo "tmpdir_race"
        return 0
    fi

    echo "deterministic"
    return 1
}

log_flake_event() {
    local target="$1"
    local category="$2"
    local attempt="$3"
    local timestamp
    timestamp="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

    printf '{"target":"%s","category":"%s","attempt":%d,"timestamp":"%s","retriable":true}\n' \
        "$target" "$category" "$attempt" "$timestamp" >> "$FLAKE_LOG"
}

# ─── Execute with retry logic ───────────────────────────────────────────────

attempt=0
while true; do
    attempt=$((attempt + 1))
    echo "=== [$TARGET] attempt $attempt ==="
    if [[ "$attempt" -eq 1 ]]; then
        echo "=== [$TARGET] cargo runner mode: $CARGO_RUNNER_MODE (request=$CARGO_RUNNER_REQUEST) ==="
    fi

    OUTPUT_FILE=$(mktemp)
    EXIT_CODE=0
    "${CMD[@]}" 2>&1 | tee "$OUTPUT_FILE" || EXIT_CODE=$?

    if [[ $EXIT_CODE -eq 0 ]]; then
        echo "=== [$TARGET] PASS (attempt $attempt) ==="
        rm -f "$OUTPUT_FILE"
        exit 0
    fi

    # Classify the failure.
    OUTPUT=$(cat "$OUTPUT_FILE")
    rm -f "$OUTPUT_FILE"

    CATEGORY=$(is_transient_failure "$OUTPUT" || true)

    if [[ "$CATEGORY" == "deterministic" ]]; then
        echo "=== [$TARGET] DETERMINISTIC FAILURE (attempt $attempt) ==="
        exit $EXIT_CODE
    fi

    echo "=== [$TARGET] TRANSIENT FAILURE: $CATEGORY (attempt $attempt) ==="
    log_flake_event "$TARGET" "$CATEGORY" "$attempt"

    if [[ "$CLASSIFY_ONLY" -eq 1 ]]; then
        echo "=== [$TARGET] CLASSIFY_ONLY mode — not retrying ==="
        exit $EXIT_CODE
    fi

    if [[ $attempt -gt $MAX_RETRIES ]]; then
        echo "=== [$TARGET] MAX RETRIES EXCEEDED ($MAX_RETRIES) ==="
        exit $EXIT_CODE
    fi

    echo "=== [$TARGET] Retrying in ${RETRY_DELAY}s... ==="
    sleep "$RETRY_DELAY"
done
