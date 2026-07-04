# Stage 00: Project Intake

## Input
User provides:
- GitHub repository URL
- Project name
- Brief description of work needed (optional)

## Output
- `input.json` - Structured intake data
- `repo-link.txt` - Cleaned repository URL

## Process

Save the following to `input.json`:
```json
{
  "repo_url": "https://github.com/owner/repo",
  "repo_name": "repo",
  "project_name": "Project Display Name",
  "user_request": "What needs to be done",
  "submitted_at": "ISO-8601 timestamp"
}
```

Then move to **Stage 01**.

## Human Decision Point
✅ Ready to proceed? Run: `node ../01-scan/scan.js < input.json`
