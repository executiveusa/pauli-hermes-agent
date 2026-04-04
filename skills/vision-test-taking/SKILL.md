# Vision Online Test-Taking — Research Skill

## Purpose

Research capability for studying AI model performance on online tests. Uses Playwright + vision LLM to:
1. Navigate to a test/quiz page
2. Screenshot and analyze each question
3. Generate answers with reasoning
4. Log every step for analysis

## When to Use

- "Take this online test for me"
- "Research how well AI can answer [subject] questions"
- "Screenshot and analyze the questions on this page"
- "Compare model accuracy on multiple choice vs free-form questions"

## Quick Start

### Via Agent MAXX API
```bash
curl -X POST http://localhost:8642/api/browser-runs \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{
    "start_url": "https://quiz-site.com/test/123",
    "steps": [
      {
        "action": "vision_analyze",
        "prompt": "You are analyzing an online test. Screenshot each page, identify every question and answer choice. For each question: state the question, list the choices, select the correct answer with a 1-sentence reason. Format as JSON.",
        "save_as": "test_analysis"
      }
    ]
  }'
```

### Follow Up
```bash
# Check run status
curl http://localhost:8642/api/browser-runs/{run_id}

# Get artifacts (answers, screenshots)
curl http://localhost:8642/api/browser-runs/{run_id}/artifacts
```

## Technical Pipeline

```
Browser Run Created
       ↓
worker-browser picks up from Redis queue
       ↓
Playwright launches Chromium (headless)
       ↓
Navigate to start_url
       ↓
For vision_analyze step:
  → Take full-page screenshot
  → Send to provider-router (research_browser profile → Gemini Flash)
  → Parse vision LLM response
  → Save as browser_artifact (kind=vision_analysis)
       ↓
Continue to next step / next page
       ↓
Save video, trace, all artifacts
       ↓
Update BrowserRun.state = 'completed'
```

## Prompt Patterns

### Multiple Choice Test
```
Analyze this test page screenshot. For each question:
1. Question number and full text
2. All answer choices (A, B, C, D)
3. Selected answer
4. Brief reason (1 sentence)

Output as JSON array: [{question_num, text, choices, answer, reason}]
```

### Fill-in-the-Blank
```
This is a test page. Identify every blank or text input field.
For each: describe what it's asking, provide the correct answer.
Format: [{field_label, context, answer}]
```

### Research Mode (compare models)
Run the same test with different provider profiles and compare:
- `balanced` → Claude Sonnet 4
- `premium_reasoning` → Claude Opus 4  
- `research_browser` → Gemini 2.5 Flash

Compare artifacts to measure accuracy differences.

## Artifacts Saved

Each vision session saves:
| Kind | Description |
|------|-------------|
| `screenshot` | Full-page PNG of each analyzed page |
| `vision_analysis` | Raw LLM response (JSON or text) |
| `video` | Full session recording |
| `trace` | Playwright trace (open with `playwright show-trace`) |

## Viewing Results

```bash
# List artifacts for a run
curl http://localhost:8642/api/browser-runs/{run_id}/artifacts

# Download a specific artifact file
# (artifacts are in /data/artifacts/{run_id}/ on the VPS)
```

## Rate Limits

- Max concurrent browser sessions: 3 (configurable via `BROWSER_MAX_CONCURRENT`)
- Vision LLM: subject to Gemini / GPT-4V rate limits
- Screenshots: full page captured (not cropped)

## Privacy Note

All screenshots and vision responses are stored locally on the VPS in `/data/artifacts/`. No data is sent anywhere except to the configured vision LLM provider.
