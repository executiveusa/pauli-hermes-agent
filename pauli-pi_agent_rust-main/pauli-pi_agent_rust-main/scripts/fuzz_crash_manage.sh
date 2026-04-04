#!/usr/bin/env bash
# fuzz_crash_manage.sh — Crash corpus management for cargo-fuzz.
#
# Provides subcommands for the crash lifecycle:
#   triage   — List and categorize unprocessed crash artifacts
#   minimize — Minimize a crash input via cargo fuzz tmin
#   store    — Move a processed crash to fuzz/crashes/<target>/
#   regress  — Move a fixed crash to fuzz/regression/<target>/
#   generate-tests — Generate regression manifest + cargo test cases from fuzz/regression/
#   report   — Emit a JSON summary of all stored crashes
#
# Usage:
#   ./scripts/fuzz_crash_manage.sh triage [--target=<name>]
#   ./scripts/fuzz_crash_manage.sh minimize <target> <artifact-path>
#   ./scripts/fuzz_crash_manage.sh store <target> <artifact-path> --category=<cat> [--description=<desc>]
#   ./scripts/fuzz_crash_manage.sh regress <target> <crash-name> [--bead=<id>]
#   ./scripts/fuzz_crash_manage.sh generate-tests [--output=<path>] [--manifest=<path>]
#   ./scripts/fuzz_crash_manage.sh report [--format=json|text]
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FUZZ_DIR="$REPO_ROOT/fuzz"
ARTIFACTS_DIR="$FUZZ_DIR/artifacts"
CRASHES_DIR="$FUZZ_DIR/crashes"
REGRESSION_DIR="$FUZZ_DIR/regression"

# Crash categories
VALID_CATEGORIES="oom,stack-overflow,panic-unwrap,panic-index,panic-assertion,timeout,logic-error,unknown"

die() { echo "ERROR: $*" >&2; exit 1; }

usage() {
    cat <<'USAGE'
fuzz_crash_manage.sh — Crash corpus management

Subcommands:
  triage   [--target=NAME]          List unprocessed crash artifacts
  minimize TARGET ARTIFACT_PATH     Minimize a crash via cargo fuzz tmin
  store    TARGET ARTIFACT --category=CAT [--description=DESC]
                                    Move crash to fuzz/crashes/TARGET/
  regress  TARGET CRASH_NAME [--bead=ID]
                                    Move fixed crash to fuzz/regression/TARGET/
  generate-tests [--output=PATH] [--manifest=PATH]
                                    Generate regression manifest + Rust tests
  report   [--format=json|text]     Summarize all stored crashes

Categories: oom, stack-overflow, panic-unwrap, panic-index, panic-assertion,
            timeout, logic-error, unknown
USAGE
    exit 1
}

# ── triage ──────────────────────────────────────────────────────────────────

cmd_triage() {
    local target_filter=""
    for arg in "$@"; do
        case "$arg" in
            --target=*) target_filter="${arg#--target=}" ;;
            *) die "Unknown option: $arg" ;;
        esac
    done

    echo "=== Crash Artifact Triage ==="
    echo ""

    local total=0
    local found=0

    for target_dir in "$ARTIFACTS_DIR"/*/; do
        [ -d "$target_dir" ] || continue
        local target
        target="$(basename "$target_dir")"

        if [ -n "$target_filter" ] && [ "$target" != "$target_filter" ]; then
            continue
        fi

        local crashes=()
        while IFS= read -r -d '' f; do
            crashes+=("$f")
        done < <(find "$target_dir" -maxdepth 1 -type f \( -name 'crash-*' -o -name 'oom-*' -o -name 'timeout-*' -o -name 'slow-unit-*' \) -print0 2>/dev/null)

        if [ ${#crashes[@]} -eq 0 ]; then
            continue
        fi

        found=1
        echo "Target: $target (${#crashes[@]} crash artifact(s))"
        for crash in "${crashes[@]}"; do
            local name size
            name="$(basename "$crash")"
            size="$(stat -c%s "$crash" 2>/dev/null || stat -f%z "$crash" 2>/dev/null || echo "?")"
            echo "  - $name (${size} bytes)"
            total=$((total + 1))
        done
        echo ""
    done

    if [ "$found" -eq 0 ]; then
        echo "No unprocessed crash artifacts found."
    else
        echo "Total: $total crash artifact(s) pending triage."
    fi
}

# ── minimize ────────────────────────────────────────────────────────────────

cmd_minimize() {
    [ $# -ge 2 ] || die "Usage: minimize TARGET ARTIFACT_PATH"
    local target="$1"
    local artifact="$2"

    [ -f "$artifact" ] || die "Artifact not found: $artifact"

    echo "Minimizing crash for target '$target': $artifact"

    local min_output
    min_output="${artifact}.minimized"

    # Determine runner prefix
    local runner=""
    if command -v rch &>/dev/null; then
        runner="env RCH_FORCE_REMOTE=${RCH_FORCE_REMOTE:-true} rch exec --"
    fi

    $runner cargo fuzz tmin "$target" "$artifact" -- 2>&1 | tee /dev/stderr

    if [ -f "$min_output" ]; then
        local orig_size min_size
        orig_size="$(stat -c%s "$artifact" 2>/dev/null || stat -f%z "$artifact")"
        min_size="$(stat -c%s "$min_output" 2>/dev/null || stat -f%z "$min_output")"
        echo ""
        echo "Minimized: $orig_size -> $min_size bytes ($(( (orig_size - min_size) * 100 / orig_size ))% reduction)"
        echo "Output: $min_output"
    else
        echo ""
        echo "Note: cargo fuzz tmin may have modified the input in-place."
        echo "Check the artifact at: $artifact"
    fi
}

# ── store ───────────────────────────────────────────────────────────────────

cmd_store() {
    local target="" artifact="" category="" description=""

    [ $# -ge 2 ] || die "Usage: store TARGET ARTIFACT --category=CAT [--description=DESC]"
    target="$1"; shift
    artifact="$1"; shift

    for arg in "$@"; do
        case "$arg" in
            --category=*) category="${arg#--category=}" ;;
            --description=*) description="${arg#--description=}" ;;
            *) die "Unknown option: $arg" ;;
        esac
    done

    [ -n "$category" ] || die "Required: --category=<$VALID_CATEGORIES>"
    echo "$VALID_CATEGORIES" | tr ',' '\n' | grep -qx "$category" || die "Invalid category '$category'. Valid: $VALID_CATEGORIES"
    [ -f "$artifact" ] || die "Artifact not found: $artifact"

    local dest_dir="$CRASHES_DIR/$target"
    mkdir -p "$dest_dir"

    # Generate sequential name: <category>-NNN.bin
    local seq=1
    while [ -f "$dest_dir/${category}-$(printf '%03d' $seq).bin" ]; do
        seq=$((seq + 1))
    done
    local dest_name="${category}-$(printf '%03d' $seq).bin"
    local dest_path="$dest_dir/$dest_name"

    cp "$artifact" "$dest_path"

    # Write metadata sidecar
    local meta_path="${dest_path%.bin}.json"
    cat > "$meta_path" <<EOF
{
  "schema": "pi.fuzz.crash_metadata.v1",
  "target": "$target",
  "category": "$category",
  "original_artifact": "$(basename "$artifact")",
  "stored_as": "$dest_name",
  "description": "$description",
  "stored_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "size_bytes": $(stat -c%s "$dest_path" 2>/dev/null || stat -f%z "$dest_path"),
  "status": "open"
}
EOF

    echo "Stored: $dest_path"
    echo "Metadata: $meta_path"
    echo "Category: $category"
    [ -n "$description" ] && echo "Description: $description"
}

# ── regress ─────────────────────────────────────────────────────────────────

generate_regression_tests() {
    local output="$1"
    local manifest="$2"
    python3 - "$REPO_ROOT" "$REGRESSION_DIR" "$output" "$manifest" <<'PY'
import json
import re
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

repo_root = Path(sys.argv[1]).resolve()
regression_dir = Path(sys.argv[2]).resolve()
output_path = Path(sys.argv[3]).resolve()
manifest_path = Path(sys.argv[4]).resolve()

def sanitize_ident(text: str) -> str:
    ident = re.sub(r"[^a-zA-Z0-9_]+", "_", text).strip("_").lower()
    if not ident:
        ident = "case"
    if ident[0].isdigit():
        ident = f"case_{ident}"
    return ident

cases = []
if regression_dir.exists():
    for target_dir in sorted(regression_dir.iterdir()):
        if not target_dir.is_dir():
            continue
        target = target_dir.name
        for bin_path in sorted(target_dir.glob("*.bin")):
            rel_path = bin_path.relative_to(repo_root).as_posix()
            stem = bin_path.stem
            meta_path = bin_path.with_suffix(".json")
            category = "unknown"
            if meta_path.exists():
                try:
                    meta = json.loads(meta_path.read_text())
                    category = str(meta.get("category", "unknown"))
                except Exception:
                    category = "unknown"
            cases.append({
                "target": target,
                "category": category,
                "path": rel_path,
                "stem": stem,
            })

manifest = {
    "schema": "pi.fuzz.regression_manifest.v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "case_count": len(cases),
    "cases": cases,
}
manifest_path.parent.mkdir(parents=True, exist_ok=True)
manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")

lines = []
lines.append("// AUTO-GENERATED by scripts/fuzz_crash_manage.sh generate-tests")
lines.append("// DO NOT EDIT BY HAND.")
lines.append("")
lines.append("use pi::sse::SseParser;")
lines.append("")
lines.append("fn assert_sse_chunking_invariant(data: &[u8]) {")
lines.append("    let input = String::from_utf8_lossy(data);")
lines.append("")
lines.append("    let mut parser_whole = SseParser::new();")
lines.append("    let events_whole = parser_whole.feed(&input);")
lines.append("    let flush_whole = parser_whole.flush();")
lines.append("")
lines.append("    let mut parser_char = SseParser::new();")
lines.append("    let mut events_char = Vec::new();")
lines.append("    for ch in input.chars() {")
lines.append("        let mut buf = [0u8; 4];")
lines.append("        let s = ch.encode_utf8(&mut buf);")
lines.append("        events_char.extend(parser_char.feed(s));")
lines.append("    }")
lines.append("    let flush_char = parser_char.flush();")
lines.append("")
lines.append("    if input.len() >= 2 {")
lines.append("        let mid = input.len() / 2;")
lines.append("        let mut split_at = mid;")
lines.append("        while !input.is_char_boundary(split_at) && split_at < input.len() {")
lines.append("            split_at += 1;")
lines.append("        }")
lines.append("        let (part1, part2) = input.split_at(split_at);")
lines.append("        let mut parser_split = SseParser::new();")
lines.append("        let mut events_split = parser_split.feed(part1);")
lines.append("        events_split.extend(parser_split.feed(part2));")
lines.append("        let flush_split = parser_split.flush();")
lines.append("")
lines.append("        assert_eq!(events_whole.len(), events_split.len(), \"whole/split event count mismatch\");")
lines.append("        for (idx, (whole, split)) in events_whole.iter().zip(events_split.iter()).enumerate() {")
lines.append("            assert_eq!(whole, split, \"whole/split event mismatch at index {idx}\");")
lines.append("        }")
lines.append("        assert_eq!(flush_whole, flush_split, \"whole/split flush mismatch\");")
lines.append("    }")
lines.append("")
lines.append("    assert_eq!(events_whole.len(), events_char.len(), \"whole/char event count mismatch\");")
lines.append("    for (idx, (whole, ch)) in events_whole.iter().zip(events_char.iter()).enumerate() {")
lines.append("        assert_eq!(whole, ch, \"whole/char event mismatch at index {idx}\");")
lines.append("    }")
lines.append("    assert_eq!(flush_whole, flush_char, \"whole/char flush mismatch\");")
lines.append("}")
lines.append("")
lines.append("fn run_regression_case(target: &str, data: &[u8], path: &str) {")
lines.append("    match target {")
lines.append("        \"fuzz_smoke\" => assert!(!data.is_empty(), \"regression input must not be empty: {path}\"),")
lines.append("        \"fuzz_sse_parser\" => assert_sse_chunking_invariant(data),")
lines.append("        _ => panic!(\"missing regression target handler for {target} (file {path})\"),")
lines.append("    }")
lines.append("}")
lines.append("")
if not cases:
    lines.append("#[test]")
    lines.append("fn generated_regression_manifest_not_empty() {")
    lines.append("    panic!(\"expected at least one regression corpus input in fuzz/regression\");")
    lines.append("}")
    lines.append("")

used_names = set()
for case in cases:
    base = sanitize_ident(f"{case['target']}_{case['stem']}")
    name = base
    suffix = 2
    while name in used_names:
        name = f"{base}_{suffix}"
        suffix += 1
    used_names.add(name)

    path = case["path"]
    target = case["target"]
    category = case["category"]
    lines.append("#[test]")
    lines.append(f"fn regression_{name}() {{")
    lines.append(f"    // target={target}, category={category}")
    lines.append(f"    let data = include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{path}\"));")
    lines.append(f"    run_regression_case(\"{target}\", data, \"{path}\");")
    lines.append("}")
    lines.append("")

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text("\n".join(lines).rstrip() + "\n")
rustfmt = shutil.which("rustfmt")
if rustfmt:
    subprocess.run([rustfmt, str(output_path)], check=True)
print(f"Generated {len(cases)} regression tests at {output_path}")
print(f"Wrote manifest to {manifest_path}")
PY
}

cmd_regress() {
    local target="" crash_name="" bead=""

    [ $# -ge 2 ] || die "Usage: regress TARGET CRASH_NAME [--bead=ID]"
    target="$1"; shift
    crash_name="$1"; shift

    for arg in "$@"; do
        case "$arg" in
            --bead=*) bead="${arg#--bead=}" ;;
            *) die "Unknown option: $arg" ;;
        esac
    done

    local src="$CRASHES_DIR/$target/$crash_name"
    [ -f "$src" ] || die "Crash file not found: $src"

    local dest_dir="$REGRESSION_DIR/$target"
    mkdir -p "$dest_dir"

    local dest="$dest_dir/$crash_name"
    mv "$src" "$dest"

    # Update metadata if present
    local meta_src="${src%.bin}.json"
    if [ -f "$meta_src" ]; then
        local meta_dest="${dest%.bin}.json"
        # Update status to resolved
        python3 -c "
import json, sys
from datetime import datetime, timezone
with open('$meta_src') as f:
    m = json.load(f)
m['status'] = 'resolved'
m['resolved_at'] = datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')
m['bead'] = '${bead}'
with open('$meta_dest', 'w') as f:
    json.dump(m, f, indent=2)
    f.write('\n')
" 2>/dev/null || mv "$meta_src" "${dest%.bin}.json"
        rm -f "$meta_src"
    fi

    echo "Moved to regression: $dest"
    [ -n "$bead" ] && echo "Linked bead: $bead"
    generate_regression_tests \
        "$REPO_ROOT/tests/fuzz_regression_generated.rs" \
        "$REGRESSION_DIR/regression_manifest.json"
}

# ── generate-tests ──────────────────────────────────────────────────────────

cmd_generate_tests() {
    local output="$REPO_ROOT/tests/fuzz_regression_generated.rs"
    local manifest="$REGRESSION_DIR/regression_manifest.json"

    for arg in "$@"; do
        case "$arg" in
            --output=*) output="${arg#--output=}" ;;
            --manifest=*) manifest="${arg#--manifest=}" ;;
            *) die "Unknown option: $arg" ;;
        esac
    done

    generate_regression_tests "$output" "$manifest"
}

# ── report ──────────────────────────────────────────────────────────────────

cmd_report() {
    local format="text"
    for arg in "$@"; do
        case "$arg" in
            --format=*) format="${arg#--format=}" ;;
            *) die "Unknown option: $arg" ;;
        esac
    done

    if [ "$format" = "json" ]; then
        python3 - "$CRASHES_DIR" "$REGRESSION_DIR" <<'PY'
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

crashes_dir = Path(sys.argv[1])
regression_dir = Path(sys.argv[2])

def scan_dir(base_dir, status_default):
    entries = []
    if not base_dir.exists():
        return entries
    for target_dir in sorted(base_dir.iterdir()):
        if not target_dir.is_dir():
            continue
        target = target_dir.name
        for f in sorted(target_dir.glob("*.bin")):
            meta_path = f.with_suffix(".json")
            meta = {}
            if meta_path.exists():
                try:
                    meta = json.loads(meta_path.read_text())
                except Exception:
                    pass
            entries.append({
                "target": target,
                "file": f.name,
                "category": meta.get("category", "unknown"),
                "status": meta.get("status", status_default),
                "description": meta.get("description", ""),
                "size_bytes": f.stat().st_size,
                "stored_at": meta.get("stored_at", ""),
                "resolved_at": meta.get("resolved_at", ""),
                "bead": meta.get("bead", ""),
            })
    return entries

open_crashes = scan_dir(crashes_dir, "open")
resolved = scan_dir(regression_dir, "resolved")

report = {
    "schema": "pi.fuzz.crash_report.v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "summary": {
        "open_count": len(open_crashes),
        "resolved_count": len(resolved),
        "total_count": len(open_crashes) + len(resolved),
        "categories": {},
    },
    "open": open_crashes,
    "resolved": resolved,
}

for entry in open_crashes + resolved:
    cat = entry["category"]
    report["summary"]["categories"][cat] = report["summary"]["categories"].get(cat, 0) + 1

print(json.dumps(report, indent=2))
PY
    else
        echo "=== Crash Corpus Report ==="
        echo ""

        local open_count=0 resolved_count=0

        if [ -d "$CRASHES_DIR" ]; then
            for target_dir in "$CRASHES_DIR"/*/; do
                [ -d "$target_dir" ] || continue
                local target
                target="$(basename "$target_dir")"
                local count
                count=$(find "$target_dir" -maxdepth 1 -name '*.bin' 2>/dev/null | wc -l)
                if [ "$count" -gt 0 ]; then
                    echo "Open crashes ($target): $count"
                    open_count=$((open_count + count))
                fi
            done
        fi

        if [ -d "$REGRESSION_DIR" ]; then
            for target_dir in "$REGRESSION_DIR"/*/; do
                [ -d "$target_dir" ] || continue
                local target
                target="$(basename "$target_dir")"
                local count
                count=$(find "$target_dir" -maxdepth 1 -name '*.bin' 2>/dev/null | wc -l)
                if [ "$count" -gt 0 ]; then
                    echo "Regression tests ($target): $count"
                    resolved_count=$((resolved_count + count))
                fi
            done
        fi

        echo ""
        echo "Total: $open_count open, $resolved_count resolved"
    fi
}

# ── main ────────────────────────────────────────────────────────────────────

[ $# -ge 1 ] || usage

cmd="$1"; shift
case "$cmd" in
    triage)   cmd_triage "$@" ;;
    minimize) cmd_minimize "$@" ;;
    store)    cmd_store "$@" ;;
    regress)  cmd_regress "$@" ;;
    generate-tests) cmd_generate_tests "$@" ;;
    report)   cmd_report "$@" ;;
    help|-h|--help) usage ;;
    *) die "Unknown subcommand: $cmd. Use 'help' for usage." ;;
esac
