# YouTube transcript and video-research workflow

Use this for a single video, a playlist item, channel research, or multi-video
comparison. Prefer subtitles over speech-to-text because subtitles are faster,
cheaper, and usually preserve timing more accurately.

## Reliability ladder

Use this order and stop at the first verified success:

1. uploader-provided subtitles;
2. YouTube automatic subtitles;
3. `agent-reach transcribe` from public video/audio;
4. report blocked with the exact failure.

Never call a title/description summary a transcript.

## One-command Hermes helper

The bundled script executes the reliability ladder, normalizes VTT text, and
writes a receipt:

```bash
bash "<skill-directory>/scripts/youtube-transcript.sh" "YOUTUBE_URL"
```

Optional output directory and language preference:

```bash
bash "<skill-directory>/scripts/youtube-transcript.sh" \
  "YOUTUBE_URL" \
  "/tmp/agent-reach-youtube-job" \
  "en.*,en,es.*,es,zh-Hans,zh"
```

The script never downloads the full video unless subtitles are unavailable and
transcription fallback is needed.

## Manual workflow

### 1. Health and URL validation

```bash
agent-reach doctor --json
yt-dlp --simulate --no-playlist --print "%(webpage_url)s" "URL"
```

For a playlist URL, ask whether the user wants one item or the entire playlist.
Do not accidentally process a full playlist.

### 2. Capture metadata

```bash
mkdir -p /tmp/agent-reach-youtube

yt-dlp --dump-single-json --skip-download --no-playlist "URL" \
  > /tmp/agent-reach-youtube/metadata.json
```

Preserve at minimum:

- canonical URL and video ID;
- title;
- channel/uploader;
- upload date;
- duration;
- description;
- chapter list when present;
- live/upcoming status.

### 3. Try uploader-provided subtitles

```bash
yt-dlp \
  --write-sub \
  --sub-lang "en.*,en,es.*,es,zh-Hans,zh" \
  --sub-format vtt \
  --skip-download \
  --no-playlist \
  -o "/tmp/agent-reach-youtube/%(id)s.%(ext)s" \
  "URL"
```

### 4. Try automatic subtitles

Only when no manual subtitle file was produced:

```bash
yt-dlp \
  --write-auto-sub \
  --sub-lang "en.*,en,es.*,es,zh-Hans,zh" \
  --sub-format vtt \
  --skip-download \
  --no-playlist \
  -o "/tmp/agent-reach-youtube/%(id)s.%(ext)s" \
  "URL"
```

Automatic captions commonly repeat partial phrases between cues. Normalize and
deduplicate consecutive lines before analysis, but preserve the original VTT.

### 5. No-subtitle fallback

```bash
agent-reach transcribe "URL" -o /tmp/agent-reach-youtube/transcript.txt
```

Transcription requires a configured Groq or OpenAI key. Agent Reach auto mode
tries Groq first and falls back to OpenAI. Do not reveal the key.

### 6. Verify transcript quality

Before summarizing:

- transcript is non-empty;
- language matches the video or requested language;
- duration is plausible for the amount of text;
- the first, middle, and final sections correspond to the video metadata;
- repeated auto-caption fragments were removed without deleting unique speech;
- timestamps remain available for important claims.

## Output format

```markdown
# Video title

- Channel:
- Published:
- Duration:
- URL:
- Transcript source: manual subtitles | automatic subtitles | ASR fallback
- Coverage: complete | partial | blocked

## Executive summary

## Key ideas

1. Idea — timestamp
2. Idea — timestamp

## Claims worth verifying

## Quotable excerpts

Use only short excerpts and include timestamps.

## Actions / implementation notes

## Transcript artifact

Path or attachment, only when the user requested the file.
```

## Multi-video research

1. Define topic, date range, languages, and result limit.
2. Search:

```bash
yt-dlp --dump-json "ytsearch10:QUERY"
```

3. Deduplicate by video ID and channel.
4. Prefer primary/expert sources over reaction compilations.
5. Process at most three transcripts concurrently to avoid resource spikes.
6. Store one source record per video.
7. Compare agreement, disagreement, dates, and evidence quality.
8. Do not merge several creators' claims into a single unattributed conclusion.

## Comments

Comments are best-effort and can be incomplete:

```bash
yt-dlp --write-comments --skip-download --write-info-json \
  --extractor-args "youtube:max_comments=20" \
  -o "/tmp/agent-reach-youtube/%(id)s" "URL"
```

Treat comments as audience sentiment, not factual verification.

## Common failures

| Failure | Response |
|---|---|
| Video unavailable/private/age restricted | Report blocked; do not bypass access controls. |
| No subtitles | Use `agent-reach transcribe`. |
| Transcript too large | Process in chunks and preserve section boundaries. |
| Auto-caption repetition | Normalize consecutive duplicate/overlapping cues. |
| Wrong language | Inspect available subtitles with `yt-dlp --list-subs`. |
| Playlist explosion | Always use `--no-playlist` unless full playlist was requested. |
| Live stream/upcoming video | Report status; do not claim a complete transcript. |
| Rate limit/network block | Retry once with backoff, then report and stop. |

## User-facing examples

- “Use Agent Reach to get the full transcript, timestamped summary, and action
  plan from this YouTube URL.”
- “Find the five strongest recent videos about this topic, transcribe them, and
  tell me where the experts agree and disagree.”
- “Turn this tutorial into an ICM learning module with source timestamps.”
