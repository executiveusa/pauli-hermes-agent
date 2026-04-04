#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

usage() {
    cat <<'EOF'
Usage: ./scripts/provider_registry_guardrails.sh [OPTIONS]

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

# Default to tmpfs-backed build/test paths to reduce disk pressure in multi-agent runs.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/data/tmp/pi_agent_rust/${USER:-agent}}"
export TMPDIR="${TMPDIR:-${CARGO_TARGET_DIR}/tmp}"
mkdir -p "$TMPDIR"

run_cargo test --test provider_registry_guardrails -- --nocapture
