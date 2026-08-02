#!/usr/bin/env bash
set -euo pipefail

URL="${1:-}"
OUT_DIR="${2:-/tmp/agent-reach-youtube-$(date +%Y%m%d-%H%M%S)}"
LANGS="${3:-en.*,en,es.*,es,zh-Hans,zh}"

if [[ -z "$URL" ]]; then
  echo "Usage: youtube-transcript.sh YOUTUBE_URL [OUTPUT_DIR] [LANGS]" >&2
  exit 2
fi

if ! command -v yt-dlp >/dev/null 2>&1; then
  echo "yt-dlp is required. Run the Agent Reach bootstrap first." >&2
  exit 3
fi

mkdir -p "$OUT_DIR"
META="$OUT_DIR/metadata.json"
TRANSCRIPT="$OUT_DIR/transcript.txt"
RECEIPT="$OUT_DIR/receipt.json"
SOURCE=""
VTT=""

# Guard against accidental playlist expansion.
yt-dlp --dump-single-json --skip-download --no-playlist --no-warnings "$URL" > "$META"

# Prefer uploader-provided subtitles.
set +e
yt-dlp \
  --write-sub \
  --sub-lang "$LANGS" \
  --sub-format vtt \
  --skip-download \
  --no-playlist \
  --no-warnings \
  -o "$OUT_DIR/%(id)s.%(ext)s" \
  "$URL" >/dev/null 2>&1
set -e

VTT="$(find "$OUT_DIR" -maxdepth 1 -type f -name '*.vtt' -print | sort | head -n 1 || true)"
if [[ -n "$VTT" ]]; then
  SOURCE="manual_subtitles"
else
  # Fall back to YouTube automatic captions.
  set +e
  yt-dlp \
    --write-auto-sub \
    --sub-lang "$LANGS" \
    --sub-format vtt \
    --skip-download \
    --no-playlist \
    --no-warnings \
    -o "$OUT_DIR/%(id)s.%(ext)s" \
    "$URL" >/dev/null 2>&1
  set -e
  VTT="$(find "$OUT_DIR" -maxdepth 1 -type f -name '*.vtt' -print | sort | head -n 1 || true)"
  if [[ -n "$VTT" ]]; then
    SOURCE="automatic_subtitles"
  fi
fi

if [[ -n "$VTT" ]]; then
  python3 - "$VTT" "$TRANSCRIPT" <<'PY'
import html
import re
import sys
from pathlib import Path

src = Path(sys.argv[1])
dst = Path(sys.argv[2])
lines = []
for raw in src.read_text(encoding="utf-8", errors="replace").splitlines():
    line = raw.strip()
    if not line:
        continue
    if line == "WEBVTT" or line.startswith(("Kind:", "Language:", "NOTE", "X-TIMESTAMP-MAP")):
        continue
    if "-->" in line or line.isdigit():
        continue
    line = re.sub(r"<[^>]+>", "", line)
    line = html.unescape(line).strip()
    if not line:
        continue
    if lines and line == lines[-1]:
        continue
    lines.append(line)

dst.write_text("\n".join(lines).strip() + "\n", encoding="utf-8")
PY
else
  if ! command -v agent-reach >/dev/null 2>&1; then
    echo "No subtitles were available and agent-reach is not on PATH for ASR fallback." >&2
    exit 4
  fi
  SOURCE="speech_to_text"
  agent-reach transcribe "$URL" -o "$TRANSCRIPT"
fi

if [[ ! -s "$TRANSCRIPT" ]]; then
  echo "Transcript extraction produced an empty file." >&2
  exit 5
fi

python3 - "$META" "$TRANSCRIPT" "$RECEIPT" "$SOURCE" "$VTT" <<'PY'
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

meta_path = Path(sys.argv[1])
transcript_path = Path(sys.argv[2])
receipt_path = Path(sys.argv[3])
source = sys.argv[4]
vtt = sys.argv[5]
meta = json.loads(meta_path.read_text(encoding="utf-8"))
text = transcript_path.read_text(encoding="utf-8", errors="replace")
receipt = {
    "status": "verified" if text.strip() else "blocked",
    "retrieved_at": datetime.now(timezone.utc).isoformat(),
    "video_id": meta.get("id"),
    "title": meta.get("title"),
    "channel": meta.get("channel") or meta.get("uploader"),
    "upload_date": meta.get("upload_date"),
    "duration_seconds": meta.get("duration"),
    "canonical_url": meta.get("webpage_url") or meta.get("original_url"),
    "transcript_source": source,
    "transcript_path": str(transcript_path),
    "original_vtt_path": vtt or None,
    "character_count": len(text),
    "line_count": len(text.splitlines()),
}
receipt_path.write_text(json.dumps(receipt, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
print(json.dumps(receipt, indent=2, ensure_ascii=False))
PY

echo "Artifacts: $OUT_DIR"
