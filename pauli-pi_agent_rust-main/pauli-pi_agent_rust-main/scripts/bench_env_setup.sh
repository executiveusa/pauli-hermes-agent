#!/usr/bin/env bash
# bench_env_setup.sh — Standardize benchmark execution environment for low-variance results.
#
# Usage:
#   sudo ./scripts/bench_env_setup.sh apply     # Apply benchmark-optimal settings
#   sudo ./scripts/bench_env_setup.sh restore    # Restore original settings
#   ./scripts/bench_env_setup.sh validate        # Check current settings (no root needed)
#   ./scripts/bench_env_setup.sh fingerprint     # Emit JSON environment fingerprint
#
# Environment variables:
#   BENCH_CORES       Comma-separated core list for affinity (default: "0,1")
#   BENCH_GOVERNOR    CPU frequency governor (default: "performance")
#   BENCH_NICE        Nice value for bench processes (default: "-20")
#   BENCH_STATE_FILE  Path to save/restore state (default: /tmp/bench_env_state.json)

set -euo pipefail

BENCH_CORES="${BENCH_CORES:-0,1}"
BENCH_GOVERNOR="${BENCH_GOVERNOR:-performance}"
BENCH_NICE="${BENCH_NICE:--20}"
BENCH_STATE_FILE="${BENCH_STATE_FILE:-/tmp/bench_env_state.json}"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

red()    { printf '\033[0;31m%s\033[0m\n' "$*"; }
green()  { printf '\033[0;32m%s\033[0m\n' "$*"; }
yellow() { printf '\033[0;33m%s\033[0m\n' "$*"; }
bold()   { printf '\033[1m%s\033[0m\n' "$*"; }

die() { red "ERROR: $*" >&2; exit 1; }

require_root() {
  if [[ $EUID -ne 0 ]]; then
    die "This operation requires root. Run with sudo."
  fi
}

# ---------------------------------------------------------------------------
# Detection
# ---------------------------------------------------------------------------

detect_governor() {
  local gov_path="/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor"
  if [[ -f "$gov_path" ]]; then
    cat "$gov_path"
  else
    echo "unavailable"
  fi
}

detect_available_governors() {
  local path="/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors"
  if [[ -f "$path" ]]; then
    cat "$path"
  else
    echo "unavailable"
  fi
}

detect_turbo() {
  # Intel: /sys/devices/system/cpu/intel_pstate/no_turbo
  # AMD: /sys/devices/system/cpu/cpufreq/boost
  if [[ -f /sys/devices/system/cpu/intel_pstate/no_turbo ]]; then
    local val
    val=$(cat /sys/devices/system/cpu/intel_pstate/no_turbo)
    if [[ "$val" == "1" ]]; then echo "disabled"; else echo "enabled"; fi
  elif [[ -f /sys/devices/system/cpu/cpufreq/boost ]]; then
    local val
    val=$(cat /sys/devices/system/cpu/cpufreq/boost)
    if [[ "$val" == "0" ]]; then echo "disabled"; else echo "enabled"; fi
  else
    echo "unavailable"
  fi
}

detect_aslr() {
  if [[ -f /proc/sys/kernel/randomize_va_space ]]; then
    local val
    val=$(cat /proc/sys/kernel/randomize_va_space)
    case "$val" in
      0) echo "disabled" ;;
      1) echo "partial" ;;
      2) echo "full" ;;
      *) echo "unknown($val)" ;;
    esac
  else
    echo "unavailable"
  fi
}

detect_thp() {
  if [[ -f /sys/kernel/mm/transparent_hugepage/enabled ]]; then
    # Extract the active setting (the one in brackets)
    sed -n 's/.*\[\(.*\)\].*/\1/p' /sys/kernel/mm/transparent_hugepage/enabled
  else
    echo "unavailable"
  fi
}

detect_cpu_freq_mhz() {
  if [[ -f /sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq ]]; then
    local khz
    khz=$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq)
    echo $(( khz / 1000 ))
  elif command -v lscpu >/dev/null 2>&1; then
    # Try "CPU MHz", then "CPU max MHz", then BogoMIPS as fallback
    local mhz
    mhz=$(lscpu | grep -i "CPU MHz" | head -1 | awk '{print int($NF)}')
    if [[ -z "$mhz" || "$mhz" == "0" ]]; then
      mhz=$(lscpu | grep -i "CPU max MHz" | head -1 | awk '{print int($NF)}')
    fi
    if [[ -z "$mhz" || "$mhz" == "0" ]]; then
      mhz=$(lscpu | grep -i "BogoMIPS" | head -1 | awk '{print int($NF / 2)}')
    fi
    echo "${mhz:-0}"
  else
    echo "0"
  fi
}

detect_cpu_max_freq_mhz() {
  if [[ -f /sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq ]]; then
    local khz
    khz=$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq)
    echo $(( khz / 1000 ))
  elif command -v lscpu >/dev/null 2>&1; then
    local mhz
    mhz=$(lscpu | grep -i "CPU max MHz" | head -1 | awk '{print int($NF)}')
    if [[ -z "$mhz" || "$mhz" == "0" ]]; then
      mhz=$(detect_cpu_freq_mhz)
    fi
    echo "${mhz:-0}"
  else
    echo "0"
  fi
}

detect_io_scheduler() {
  # Check first block device
  for dev in /sys/block/sd*/queue/scheduler /sys/block/nvme*/queue/scheduler /sys/block/vd*/queue/scheduler; do
    if [[ -f "$dev" ]]; then
      sed -n 's/.*\[\(.*\)\].*/\1/p' "$dev"
      return
    fi
  done
  echo "unavailable"
}

core_count() {
  nproc 2>/dev/null || echo "1"
}

# ---------------------------------------------------------------------------
# State save/restore
# ---------------------------------------------------------------------------

save_state() {
  local governor turbo aslr thp
  governor=$(detect_governor)
  turbo=$(detect_turbo)
  aslr=$(cat /proc/sys/kernel/randomize_va_space 2>/dev/null || echo "-1")
  thp=$(detect_thp)

  cat > "$BENCH_STATE_FILE" <<STATEJSON
{
  "saved_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "governor": "$governor",
  "turbo": "$turbo",
  "aslr": $aslr,
  "thp": "$thp"
}
STATEJSON
  green "Saved original state to $BENCH_STATE_FILE"
}

# ---------------------------------------------------------------------------
# Apply
# ---------------------------------------------------------------------------

cmd_apply() {
  require_root
  bold "=== Applying benchmark environment settings ==="

  # Save original state first
  save_state

  # 1. CPU governor → performance
  local avail
  avail=$(detect_available_governors)
  if echo "$avail" | grep -qw "$BENCH_GOVERNOR"; then
    for gov_file in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
      if [[ -f "$gov_file" ]]; then
        echo "$BENCH_GOVERNOR" > "$gov_file"
      fi
    done
    green "  CPU governor: set to $BENCH_GOVERNOR"
  else
    yellow "  CPU governor: '$BENCH_GOVERNOR' not available (have: $avail)"
  fi

  # 2. Disable turbo boost
  if [[ -f /sys/devices/system/cpu/intel_pstate/no_turbo ]]; then
    echo 1 > /sys/devices/system/cpu/intel_pstate/no_turbo
    green "  Turbo boost: disabled (Intel)"
  elif [[ -f /sys/devices/system/cpu/cpufreq/boost ]]; then
    echo 0 > /sys/devices/system/cpu/cpufreq/boost
    green "  Turbo boost: disabled (AMD)"
  else
    yellow "  Turbo boost: cannot control (no sysfs interface)"
  fi

  # 3. Disable ASLR for reproducible memory layouts
  echo 0 > /proc/sys/kernel/randomize_va_space
  green "  ASLR: disabled"

  # 4. Disable transparent hugepages (can cause latency spikes)
  if [[ -f /sys/kernel/mm/transparent_hugepage/enabled ]]; then
    echo never > /sys/kernel/mm/transparent_hugepage/enabled
    green "  THP: disabled"
  fi
  if [[ -f /sys/kernel/mm/transparent_hugepage/defrag ]]; then
    echo never > /sys/kernel/mm/transparent_hugepage/defrag
    green "  THP defrag: disabled"
  fi

  bold "=== Environment configured for benchmarking ==="
  echo "Run benchmarks with:"
  echo "  taskset -c $BENCH_CORES nice -n $BENCH_NICE cargo bench"
  echo ""
  echo "Restore with: sudo $0 restore"
}

# ---------------------------------------------------------------------------
# Restore
# ---------------------------------------------------------------------------

cmd_restore() {
  require_root
  bold "=== Restoring original environment settings ==="

  if [[ ! -f "$BENCH_STATE_FILE" ]]; then
    die "No saved state file at $BENCH_STATE_FILE"
  fi

  # Parse saved state (simple grep-based, no jq dependency required)
  local saved_governor saved_aslr saved_thp
  saved_governor=$(grep -o '"governor": *"[^"]*"' "$BENCH_STATE_FILE" | cut -d'"' -f4)
  saved_aslr=$(grep -o '"aslr": *[0-9]*' "$BENCH_STATE_FILE" | grep -o '[0-9]*$')
  saved_thp=$(grep -o '"thp": *"[^"]*"' "$BENCH_STATE_FILE" | cut -d'"' -f4)

  # Restore governor
  if [[ -n "$saved_governor" && "$saved_governor" != "unavailable" ]]; then
    for gov_file in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
      if [[ -f "$gov_file" ]]; then
        echo "$saved_governor" > "$gov_file" 2>/dev/null || true
      fi
    done
    green "  CPU governor: restored to $saved_governor"
  fi

  # Restore turbo (re-enable)
  if [[ -f /sys/devices/system/cpu/intel_pstate/no_turbo ]]; then
    echo 0 > /sys/devices/system/cpu/intel_pstate/no_turbo
    green "  Turbo boost: re-enabled (Intel)"
  elif [[ -f /sys/devices/system/cpu/cpufreq/boost ]]; then
    echo 1 > /sys/devices/system/cpu/cpufreq/boost
    green "  Turbo boost: re-enabled (AMD)"
  fi

  # Restore ASLR
  if [[ -n "$saved_aslr" && "$saved_aslr" != "-1" ]]; then
    echo "$saved_aslr" > /proc/sys/kernel/randomize_va_space
    green "  ASLR: restored to $saved_aslr"
  fi

  # Restore THP
  if [[ -n "$saved_thp" && "$saved_thp" != "unavailable" ]]; then
    if [[ -f /sys/kernel/mm/transparent_hugepage/enabled ]]; then
      echo "$saved_thp" > /sys/kernel/mm/transparent_hugepage/enabled 2>/dev/null || true
      green "  THP: restored to $saved_thp"
    fi
  fi

  rm -f "$BENCH_STATE_FILE"
  bold "=== Environment restored ==="
}

# ---------------------------------------------------------------------------
# Validate
# ---------------------------------------------------------------------------

cmd_validate() {
  bold "=== Benchmark Environment Validation ==="
  local issues=0

  # Governor
  local gov
  gov=$(detect_governor)
  if [[ "$gov" == "performance" ]]; then
    green "  [PASS] CPU governor: $gov"
  elif [[ "$gov" == "unavailable" ]]; then
    yellow "  [SKIP] CPU governor: not available (VM or container)"
  else
    yellow "  [WARN] CPU governor: $gov (recommend: performance)"
    issues=$((issues + 1))
  fi

  # Turbo
  local turbo
  turbo=$(detect_turbo)
  if [[ "$turbo" == "disabled" ]]; then
    green "  [PASS] Turbo boost: disabled"
  elif [[ "$turbo" == "unavailable" ]]; then
    yellow "  [SKIP] Turbo boost: not available"
  else
    yellow "  [WARN] Turbo boost: $turbo (recommend: disabled for stable results)"
    issues=$((issues + 1))
  fi

  # Frequency
  local cur_freq max_freq
  cur_freq=$(detect_cpu_freq_mhz)
  max_freq=$(detect_cpu_max_freq_mhz)
  if [[ "$max_freq" -gt 0 && "$cur_freq" -gt 0 ]]; then
    local pct=$(( cur_freq * 100 / max_freq ))
    if [[ $pct -ge 95 ]]; then
      green "  [PASS] CPU frequency: ${cur_freq}MHz / ${max_freq}MHz (${pct}%)"
    else
      yellow "  [WARN] CPU frequency: ${cur_freq}MHz / ${max_freq}MHz (${pct}% — not at max)"
      issues=$((issues + 1))
    fi
  else
    yellow "  [SKIP] CPU frequency: cannot read"
  fi

  # ASLR
  local aslr
  aslr=$(detect_aslr)
  if [[ "$aslr" == "disabled" ]]; then
    green "  [PASS] ASLR: disabled"
  elif [[ "$aslr" == "unavailable" ]]; then
    yellow "  [SKIP] ASLR: not available"
  else
    yellow "  [INFO] ASLR: $aslr (disable for reproducible address layouts)"
    # ASLR is informational, not a hard requirement
  fi

  # THP
  local thp
  thp=$(detect_thp)
  if [[ "$thp" == "never" ]]; then
    green "  [PASS] THP: disabled"
  elif [[ "$thp" == "unavailable" ]]; then
    yellow "  [SKIP] THP: not available"
  else
    yellow "  [WARN] THP: $thp (recommend: never — THP can cause latency spikes)"
    issues=$((issues + 1))
  fi

  # I/O scheduler
  local iosched
  iosched=$(detect_io_scheduler)
  echo "  [INFO] I/O scheduler: $iosched"

  # Core count
  local cores
  cores=$(core_count)
  if [[ "$cores" -ge 4 ]]; then
    green "  [PASS] CPU cores: $cores (sufficient for isolated benchmarking)"
  else
    yellow "  [INFO] CPU cores: $cores (4+ recommended for core isolation)"
  fi

  echo ""
  if [[ $issues -eq 0 ]]; then
    green "Environment is well-configured for benchmarking."
  else
    yellow "$issues issue(s) found. Run 'sudo $0 apply' to fix."
  fi
  return $issues
}

# ---------------------------------------------------------------------------
# Fingerprint (JSON)
# ---------------------------------------------------------------------------

cmd_fingerprint() {
  local governor turbo aslr thp iosched cur_freq max_freq cores
  governor=$(detect_governor)
  turbo=$(detect_turbo)
  aslr=$(detect_aslr)
  thp=$(detect_thp)
  iosched=$(detect_io_scheduler)
  cur_freq=$(detect_cpu_freq_mhz)
  max_freq=$(detect_cpu_max_freq_mhz)
  cores=$(core_count)

  local cpu_brand="unknown"
  if command -v lscpu >/dev/null 2>&1; then
    cpu_brand=$(lscpu | grep "Model name" | head -1 | sed 's/^.*: *//')
  fi

  local os_version
  os_version=$(uname -r)

  local mem_total_mb=0
  if [[ -f /proc/meminfo ]]; then
    mem_total_mb=$(awk '/MemTotal/ {print int($2/1024)}' /proc/meminfo)
  fi

  local git_sha="unknown"
  if command -v git >/dev/null 2>&1; then
    git_sha=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
  fi

  cat <<FPJSON
{
  "schema": "pi.bench.env.v1",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "os": "$(uname -s)",
  "os_version": "$os_version",
  "arch": "$(uname -m)",
  "cpu_brand": "$cpu_brand",
  "cpu_cores": $cores,
  "cpu_freq_mhz": $cur_freq,
  "cpu_max_freq_mhz": $max_freq,
  "mem_total_mb": $mem_total_mb,
  "governor": "$governor",
  "turbo_boost": "$turbo",
  "aslr": "$aslr",
  "thp": "$thp",
  "io_scheduler": "$iosched",
  "git_sha": "$git_sha",
  "bench_cores": "$BENCH_CORES",
  "bench_nice": "$BENCH_NICE"
}
FPJSON
}

# ---------------------------------------------------------------------------
# Taskset wrapper (convenience)
# ---------------------------------------------------------------------------

cmd_run() {
  shift  # Remove "run" from args
  if [[ $# -eq 0 ]]; then
    die "Usage: $0 run <command> [args...]"
  fi

  # Validate environment first (non-fatal)
  cmd_validate 2>/dev/null || true
  echo ""
  bold "Running with core affinity ($BENCH_CORES) and nice ($BENCH_NICE):"
  echo "  taskset -c $BENCH_CORES nice -n $BENCH_NICE $*"
  echo ""

  exec taskset -c "$BENCH_CORES" nice -n "$BENCH_NICE" "$@"
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

case "${1:-help}" in
  apply)      cmd_apply ;;
  restore)    cmd_restore ;;
  validate)   cmd_validate ;;
  fingerprint) cmd_fingerprint ;;
  run)        cmd_run "$@" ;;
  help|--help|-h)
    cat <<USAGE
Usage: $0 <command>

Commands:
  apply        Apply benchmark-optimal OS settings (requires root)
  restore      Restore original OS settings (requires root)
  validate     Check current environment suitability (no root needed)
  fingerprint  Emit JSON environment fingerprint to stdout
  run <cmd>    Run a command with CPU affinity and nice priority

Environment:
  BENCH_CORES=$BENCH_CORES
  BENCH_GOVERNOR=$BENCH_GOVERNOR
  BENCH_NICE=$BENCH_NICE
  BENCH_STATE_FILE=$BENCH_STATE_FILE
USAGE
    ;;
  *)
    die "Unknown command: $1. Run '$0 help' for usage."
    ;;
esac
