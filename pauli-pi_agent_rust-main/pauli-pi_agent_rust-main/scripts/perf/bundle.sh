#!/usr/bin/env bash
# scripts/perf/bundle.sh — Artifact bundling utility for perf orchestration runs.
#
# Creates versioned, checksummed artifact bundles from orchestration output
# directories. Bundles are portable tar.gz archives with manifest and integrity
# verification built in.
#
# Bead: bd-3ar8v.1.8
#
# Usage:
#   ./scripts/perf/bundle.sh <run-dir>                     # bundle a specific run
#   ./scripts/perf/bundle.sh --latest                      # bundle the latest run
#   ./scripts/perf/bundle.sh --verify <bundle.tar.gz>      # verify bundle integrity
#   ./scripts/perf/bundle.sh --extract <bundle.tar.gz>     # extract and verify
#   ./scripts/perf/bundle.sh --list <bundle.tar.gz>        # list bundle contents
#   ./scripts/perf/bundle.sh --inventory <run-dir>         # generate inventory without bundling
#
# Environment:
#   CARGO_TARGET_DIR     Cargo target directory (default: target/)
#   PERF_BUNDLE_DIR      Override bundle output directory (default: target/perf/bundles/)
#   PERF_BUNDLE_FORMAT   Archive format: tar.gz, tar.zst (default: tar.gz)
#   PERF_BUNDLE_KEEP     Number of bundles to keep (0=unlimited, default: 10)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# ─── Configuration ───────────────────────────────────────────────────────────

TARGET_DIR="${CARGO_TARGET_DIR:-$PROJECT_ROOT/target}"
BUNDLE_DIR="${PERF_BUNDLE_DIR:-$TARGET_DIR/perf/bundles}"
BUNDLE_FORMAT="${PERF_BUNDLE_FORMAT:-tar.gz}"
BUNDLE_KEEP="${PERF_BUNDLE_KEEP:-10}"

# ─── Helpers ─────────────────────────────────────────────────────────────────

red()    { printf '\033[0;31m%s\033[0m\n' "$*"; }
green()  { printf '\033[0;32m%s\033[0m\n' "$*"; }
yellow() { printf '\033[0;33m%s\033[0m\n' "$*"; }
bold()   { printf '\033[1m%s\033[0m\n' "$*"; }

die() { red "ERROR: $*" >&2; exit 1; }

sha256_file() {
  sha256sum "$1" 2>/dev/null | cut -d' ' -f1
}

human_size() {
  local bytes=$1
  if [[ "$bytes" -gt $((1024 * 1024 * 1024)) ]]; then
    echo "$(( bytes / 1024 / 1024 / 1024 ))GB"
  elif [[ "$bytes" -gt $((1024 * 1024)) ]]; then
    echo "$(( bytes / 1024 / 1024 ))MB"
  elif [[ "$bytes" -gt 1024 ]]; then
    echo "$(( bytes / 1024 ))KB"
  else
    echo "${bytes}B"
  fi
}

# ─── Commands ────────────────────────────────────────────────────────────────

cmd_bundle() {
  local run_dir="$1"

  if [[ ! -d "$run_dir" ]]; then
    die "Run directory not found: $run_dir"
  fi

  if [[ ! -f "$run_dir/manifest.json" ]]; then
    die "Not a valid orchestration run (missing manifest.json): $run_dir"
  fi

  # Extract metadata from manifest
  local correlation_id git_commit timestamp
  correlation_id=$(python3 -c "import json; print(json.load(open('$run_dir/manifest.json'))['correlation_id'])" 2>/dev/null || echo "unknown")
  git_commit=$(python3 -c "import json; print(json.load(open('$run_dir/manifest.json'))['git_commit'])" 2>/dev/null || echo "unknown")
  timestamp=$(python3 -c "import json; print(json.load(open('$run_dir/manifest.json'))['timestamp'])" 2>/dev/null || echo "unknown")

  mkdir -p "$BUNDLE_DIR"

  local bundle_name="perf-${timestamp}-${git_commit}"
  local bundle_path="$BUNDLE_DIR/${bundle_name}.${BUNDLE_FORMAT}"

  bold "Creating bundle: $bundle_name"
  echo "  Source: $run_dir"
  echo "  Output: $bundle_path"

  # Generate inventory before bundling
  cmd_inventory "$run_dir"

  # Verify checksums before bundling
  if [[ -f "$run_dir/checksums.sha256" ]]; then
    echo "  Verifying source checksums..."
    pushd "$run_dir" >/dev/null
    local checksum_errors=0
    while IFS= read -r line; do
      local expected_hash expected_file
      expected_hash=$(echo "$line" | cut -d' ' -f1)
      expected_file=$(echo "$line" | sed 's/^[a-f0-9]*  *//')
      if [[ -f "$expected_file" ]]; then
        actual_hash=$(sha256_file "$expected_file")
        if [[ "$actual_hash" != "$expected_hash" ]]; then
          red "  Checksum mismatch: $expected_file"
          checksum_errors=$((checksum_errors + 1))
        fi
      else
        yellow "  Missing file: $expected_file"
        checksum_errors=$((checksum_errors + 1))
      fi
    done < checksums.sha256
    popd >/dev/null

    if [[ "$checksum_errors" -gt 0 ]]; then
      die "Source integrity check failed ($checksum_errors error(s))"
    fi
    green "  Source checksums verified"
  fi

  # Create archive
  case "$BUNDLE_FORMAT" in
    tar.gz)
      tar -czf "$bundle_path" -C "$(dirname "$run_dir")" "$(basename "$run_dir")"
      ;;
    tar.zst)
      if command -v zstd >/dev/null 2>&1; then
        tar -cf - -C "$(dirname "$run_dir")" "$(basename "$run_dir")" | zstd -T0 -o "$bundle_path"
      else
        die "zstd not found; install or use tar.gz format"
      fi
      ;;
    *)
      die "Unsupported format: $BUNDLE_FORMAT (use tar.gz or tar.zst)"
      ;;
  esac

  local bundle_size bundle_sha
  bundle_size=$(stat -c%s "$bundle_path" 2>/dev/null || stat -f%z "$bundle_path" 2>/dev/null || echo "0")
  bundle_sha=$(sha256_file "$bundle_path")

  # Write sidecar metadata
  cat > "${bundle_path}.meta.json" <<EOF
{
  "schema": "pi.perf.bundle_meta.v1",
  "bundle_name": "$bundle_name",
  "bundle_format": "$BUNDLE_FORMAT",
  "bundle_size_bytes": $bundle_size,
  "bundle_sha256": "$bundle_sha",
  "source_dir": "$run_dir",
  "correlation_id": "$correlation_id",
  "git_commit": "$git_commit",
  "timestamp": "$timestamp",
  "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

  green "Bundle created: $bundle_path ($(human_size "$bundle_size"))"
  echo "  SHA-256: $bundle_sha"
  echo "  Metadata: ${bundle_path}.meta.json"

  # Prune old bundles if configured
  if [[ "$BUNDLE_KEEP" -gt 0 ]]; then
    prune_old_bundles
  fi
}

cmd_verify() {
  local bundle_path="$1"

  if [[ ! -f "$bundle_path" ]]; then
    die "Bundle not found: $bundle_path"
  fi

  bold "Verifying bundle: $bundle_path"

  # Check sidecar metadata
  local meta_path="${bundle_path}.meta.json"
  if [[ -f "$meta_path" ]]; then
    local expected_sha
    expected_sha=$(python3 -c "import json; print(json.load(open('$meta_path'))['bundle_sha256'])" 2>/dev/null || echo "")
    actual_sha=$(sha256_file "$bundle_path")

    if [[ -n "$expected_sha" && "$expected_sha" == "$actual_sha" ]]; then
      green "  Archive SHA-256 matches metadata"
    elif [[ -n "$expected_sha" ]]; then
      die "Archive SHA-256 mismatch: expected=$expected_sha actual=$actual_sha"
    fi
  else
    yellow "  No sidecar metadata found (${meta_path})"
  fi

  # Extract to temp dir and verify internal checksums
  local tmp_dir
  tmp_dir=$(mktemp -d)
  trap "rm -rf '$tmp_dir'" EXIT

  echo "  Extracting to verify internal integrity..."
  tar -xf "$bundle_path" -C "$tmp_dir"

  # Find the extracted directory
  local extracted_dir
  extracted_dir=$(find "$tmp_dir" -mindepth 1 -maxdepth 1 -type d | head -1)

  if [[ -z "$extracted_dir" ]]; then
    die "No directory found inside bundle"
  fi

  if [[ ! -f "$extracted_dir/manifest.json" ]]; then
    die "Missing manifest.json inside bundle"
  fi
  green "  manifest.json present"

  if [[ -f "$extracted_dir/checksums.sha256" ]]; then
    pushd "$extracted_dir" >/dev/null
    if sha256sum -c checksums.sha256 --quiet 2>/dev/null; then
      green "  All internal checksums verified"
    else
      die "Internal checksum verification failed"
    fi
    popd >/dev/null
  else
    yellow "  No internal checksums.sha256 found"
  fi

  green "Bundle verification passed."
}

cmd_extract() {
  local bundle_path="$1"
  local extract_dir="${2:-.}"

  if [[ ! -f "$bundle_path" ]]; then
    die "Bundle not found: $bundle_path"
  fi

  bold "Extracting: $bundle_path → $extract_dir"

  # Verify first
  cmd_verify "$bundle_path"

  # Extract
  tar -xf "$bundle_path" -C "$extract_dir"

  local extracted_dir
  extracted_dir=$(find "$extract_dir" -mindepth 1 -maxdepth 1 -type d -name "perf-*" | sort -r | head -1)
  green "Extracted to: ${extracted_dir:-$extract_dir}"
}

cmd_list() {
  local bundle_path="$1"

  if [[ ! -f "$bundle_path" ]]; then
    die "Bundle not found: $bundle_path"
  fi

  bold "Bundle contents: $bundle_path"
  tar -tf "$bundle_path" | head -100

  local file_count
  file_count=$(tar -tf "$bundle_path" | wc -l)
  if [[ "$file_count" -gt 100 ]]; then
    echo "  ... and $((file_count - 100)) more files"
  fi
  echo ""
  echo "  Total files: $file_count"
}

cmd_inventory() {
  local run_dir="$1"

  if [[ ! -d "$run_dir" ]]; then
    die "Run directory not found: $run_dir"
  fi

  local inventory_path="$run_dir/inventory.json"

  # Count files by type
  local json_count jsonl_count log_count total_size
  json_count=$(find "$run_dir" -name "*.json" 2>/dev/null | wc -l)
  jsonl_count=$(find "$run_dir" -name "*.jsonl" 2>/dev/null | wc -l)
  log_count=$(find "$run_dir" -name "*.log" 2>/dev/null | wc -l)
  total_size=$(du -sb "$run_dir" 2>/dev/null | cut -f1 || echo "0")

  # Build file list
  local files_json="["
  local first=true
  while IFS= read -r -d '' file; do
    local rel_path file_size file_sha
    rel_path="${file#$run_dir/}"
    file_size=$(stat -c%s "$file" 2>/dev/null || stat -f%z "$file" 2>/dev/null || echo "0")
    file_sha=$(sha256_file "$file")

    if [[ "$first" == "true" ]]; then
      first=false
    else
      files_json+=","
    fi
    files_json+="{\"path\":\"$rel_path\",\"size\":$file_size,\"sha256\":\"$file_sha\"}"
  done < <(find "$run_dir" -type f \( -name "*.json" -o -name "*.jsonl" \) -print0 2>/dev/null | sort -z)
  files_json+="]"

  cat > "$inventory_path" <<EOF
{
  "schema": "pi.perf.bundle_inventory.v1",
  "generated_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "source_dir": "$run_dir",
  "counts": {
    "json_files": $json_count,
    "jsonl_files": $jsonl_count,
    "log_files": $log_count,
    "total_size_bytes": $total_size
  },
  "files": $files_json
}
EOF

  echo "  Inventory written: $inventory_path ($json_count json, $jsonl_count jsonl, $log_count log)"
}

cmd_latest() {
  local runs_dir="$TARGET_DIR/perf/runs"
  if [[ ! -d "$runs_dir" ]]; then
    die "No runs directory found: $runs_dir"
  fi

  local latest
  latest=$(find "$runs_dir" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | sort -r | head -1)
  if [[ -z "$latest" ]]; then
    die "No runs found in: $runs_dir"
  fi

  echo "$latest"
}

prune_old_bundles() {
  if [[ ! -d "$BUNDLE_DIR" ]]; then
    return
  fi

  local count
  count=$(find "$BUNDLE_DIR" -name "perf-*.${BUNDLE_FORMAT}" 2>/dev/null | wc -l)

  if [[ "$count" -gt "$BUNDLE_KEEP" ]]; then
    local to_remove=$((count - BUNDLE_KEEP))
    echo "  Pruning $to_remove old bundle(s) (keeping $BUNDLE_KEEP)..."
    find "$BUNDLE_DIR" -name "perf-*.${BUNDLE_FORMAT}" 2>/dev/null \
      | sort \
      | head -"$to_remove" \
      | while IFS= read -r old_bundle; do
          rm -f "$old_bundle" "${old_bundle}.meta.json"
          echo "    Removed: $(basename "$old_bundle")"
        done
  fi
}

# ─── Main Dispatch ───────────────────────────────────────────────────────────

case "${1:-}" in
  --verify)
    [[ -n "${2:-}" ]] || die "Usage: $0 --verify <bundle.tar.gz>"
    cmd_verify "$2"
    ;;
  --extract)
    [[ -n "${2:-}" ]] || die "Usage: $0 --extract <bundle.tar.gz> [target-dir]"
    cmd_extract "$2" "${3:-.}"
    ;;
  --list)
    [[ -n "${2:-}" ]] || die "Usage: $0 --list <bundle.tar.gz>"
    cmd_list "$2"
    ;;
  --inventory)
    [[ -n "${2:-}" ]] || die "Usage: $0 --inventory <run-dir>"
    cmd_inventory "$2"
    ;;
  --latest)
    latest_dir="$(cmd_latest)"
    cmd_bundle "$latest_dir"
    ;;
  --help|-h)
    sed -n '2,/^$/p' "$0" | sed 's/^# \?//'
    exit 0
    ;;
  "")
    die "Usage: $0 <run-dir> | --latest | --verify <bundle> | --extract <bundle> | --list <bundle> | --inventory <run-dir>"
    ;;
  -*)
    die "Unknown flag: $1 (try --help)"
    ;;
  *)
    cmd_bundle "$1"
    ;;
esac
