#!/usr/bin/env bash
set -euo pipefail

PIN="b4d52c46c9113cb0f653d6df4cf71ebadf4930ac"
MODE="check"
CHANNELS=""

usage() {
  cat <<'EOF'
Usage: bootstrap.sh [--check|--apply] [--channels name1,name2]

  --check      Inspect requirements and run doctor if already installed.
  --apply      Install the pinned Agent Reach build, run its installer, then doctor.
  --channels   Optional Agent Reach channels, for example opencli,twitter.

This script never runs sudo.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --check) MODE="check"; shift ;;
    --apply) MODE="apply"; shift ;;
    --channels)
      [[ $# -ge 2 ]] || { echo "--channels requires a value" >&2; exit 2; }
      CHANNELS="$2"; shift 2 ;;
    --channels=*) CHANNELS="${1#*=}"; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
  esac
done

if ! command -v python3 >/dev/null 2>&1; then
  echo "Python 3.10+ is required. Install it manually; no elevated action was attempted." >&2
  exit 3
fi

python3 - <<'PY'
import sys
if sys.version_info < (3, 10):
    raise SystemExit(f"Python 3.10+ required; found {sys.version.split()[0]}")
print(f"Python: {sys.version.split()[0]}")
PY

find_agent_reach() {
  if command -v agent-reach >/dev/null 2>&1; then
    command -v agent-reach
    return 0
  fi
  if [[ -x "$HOME/.agent-reach-venv/bin/agent-reach" ]]; then
    printf '%s\n' "$HOME/.agent-reach-venv/bin/agent-reach"
    return 0
  fi
  return 1
}

if [[ "$MODE" == "check" ]]; then
  if AR_BIN="$(find_agent_reach)"; then
    echo "Agent Reach: $AR_BIN"
    "$AR_BIN" doctor --json
  else
    echo "Agent Reach is not installed. Run this script with --apply."
    echo "Pinned upstream commit: $PIN"
    exit 4
  fi
  exit 0
fi

SOURCE="https://github.com/Panniantong/Agent-Reach/archive/${PIN}.zip"

if command -v pipx >/dev/null 2>&1; then
  echo "Installing Agent Reach with pipx from pinned commit $PIN"
  pipx install --force "$SOURCE"
else
  echo "pipx not found; installing into dedicated venv ~/.agent-reach-venv"
  python3 -m venv "$HOME/.agent-reach-venv"
  "$HOME/.agent-reach-venv/bin/python" -m pip install --upgrade pip
  "$HOME/.agent-reach-venv/bin/python" -m pip install --upgrade --force-reinstall "$SOURCE"

  mkdir -p "$HOME/.local/bin"
  cat > "$HOME/.local/bin/agent-reach" <<EOF
#!/usr/bin/env sh
exec "$HOME/.agent-reach-venv/bin/agent-reach" "\$@"
EOF
  chmod 700 "$HOME/.local/bin/agent-reach"
fi

AR_BIN="$(find_agent_reach)" || {
  echo "Installation completed but agent-reach was not found on PATH." >&2
  echo "Ensure $HOME/.local/bin is on PATH." >&2
  exit 5
}

INSTALL_ARGS=(install --env=auto)
if [[ -n "$CHANNELS" ]]; then
  INSTALL_ARGS+=(--channels "$CHANNELS")
fi

"$AR_BIN" "${INSTALL_ARGS[@]}"
"$AR_BIN" doctor --json

echo "Agent Reach installation and health check completed."
