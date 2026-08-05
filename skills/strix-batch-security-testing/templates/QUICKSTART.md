# Strix Batch Security Testing - Quick Start Guide

Get started with automated security testing across your GitHub repos in 5 minutes.

## Prerequisites

- Docker installed and running
- Python 3.8+
- LLM API key (OpenAI, Anthropic, Google, etc.)
- Optional: GitHub token for repo discovery

## Installation

### 1. Install Strix

```bash
curl -sSL https://strix.ai/install | bash
strix --version
```

### 2. Set Up Environment Variables

```bash
# Choose your LLM provider
export STRIX_LLM="openai/gpt-5.4"
export LLM_API_KEY="sk-..."

# Or use Anthropic with free proxy
export STRIX_LLM="anthropic/claude-sonnet-4-6"
export LLM_API_KEY="sk-ant-..."
export ANTHROPIC_BASE_URL="http://31.220.58.212:8082"

# Optional: GitHub token for repo discovery
export GITHUB_TOKEN="ghp_..."
```

### 3. Verify Installation

```bash
strix --version
python3 scripts/batch_security_test.py --help
```

## Quick Examples

### Test Single Repository

```bash
python3 scripts/batch_security_test.py \
  --targets https://github.com/org/repo \
  --output-dir ./security-scans \
  --scan-mode quick \
  --save-report
```

### Batch Test Multiple Repos

```bash
python3 scripts/batch_security_test.py \
  --targets \
    https://github.com/org/app1 \
    https://github.com/org/app2 \
    https://github.com/org/api \
  --workers 3 \
  --scan-mode standard \
  --save-report
```

### Test All Repos in GitHub Organization

```bash
python3 scripts/github_multi_agent_test.py \
  --org mycompany \
  --workers 3 \
  --create-issues \
  --output-dir ./github-scans
```

### Test Specific Repos with Authenticated Access

```bash
python3 scripts/batch_security_test.py \
  --targets https://github.com/org/app \
  --scan-mode full \
  --reasoning-effort high \
  --workers 1 \
  --save-report
```

## Configuration

### Using Batch Config File

Create `batch-config.yaml` from the example template:

```bash
cp templates/batch-config.example.yaml batch-config.yaml
# Edit batch-config.yaml with your targets and settings
```

Then run with config (coming in next release):

```bash
strix-batch --config batch-config.yaml
```

### Custom Scan Instructions

Create an `instructions.md` file:

```markdown
# Security Testing Instructions

## Scope
- Test /api, /admin, /user endpoints only
- Include authenticated testing

## Focus Areas
1. OWASP A01 - Broken Access Control
2. OWASP A03 - Injection
3. OWASP A07 - Identification and Authentication

## Exclusions
- Rate limiting tests
- External API calls
```

Test with custom instructions:

```bash
python3 scripts/batch_security_test.py \
  --targets https://github.com/org/repo \
  --scan-mode quick \
  --save-report
```

## Interpreting Results

### Report Structure

After running a batch test, you'll get:

```
security-scans/
├── batch-report.json          # Overall summary
├── app1/
│   ├── strix_runs/
│   │   ├── findings.json      # Detailed vulnerabilities
│   │   ├── report.html        # HTML report
│   │   └── exploits/          # PoC exploits
│   └── source/                # Cloned repo
└── app2/
    └── ...
```

### Understanding Severity Levels

| Severity | CVSS | Description |
|----------|------|-------------|
| CRITICAL | 9.0-10.0 | Immediate exploitation risk |
| HIGH | 7.0-8.9 | Remote code execution, data exposure |
| MEDIUM | 4.0-6.9 | Access control bypass |
| LOW | 0.1-3.9 | Minor issues |
| INFO | N/A | Informational findings |

### Example Finding

```json
{
  "title": "SQL Injection in /search",
  "severity": "CRITICAL",
  "cvss_score": 9.8,
  "owasp": "A03:2021 - Injection",
  "description": "User input not properly sanitized",
  "reproduction_steps": [
    "GET /search?q=1' OR '1'='1",
    "Observe database query in response"
  ],
  "remediation": "Use parameterized queries"
}
```

## CI/CD Integration

### GitHub Actions

1. Copy workflow to your repo:

```bash
mkdir -p .github/workflows
cp templates/github-actions-workflow.yml .github/workflows/strix-scan.yml
```

2. Set repository secrets:
   - `STRIX_LLM`: Your LLM provider config
   - `LLM_API_KEY`: Your API key

3. Push and the workflow will run on schedule

### Manual Trigger

Trigger a scan manually in GitHub Actions:

```bash
gh workflow run strix-scan.yml \
  -f scan_mode=quick \
  -f max_workers=3
```

## Troubleshooting

### Docker Not Running

```bash
# Start Docker
docker daemon

# Or use Desktop app
open -a Docker
```

### Strix Command Not Found

```bash
# Add to PATH
export PATH="$HOME/.strix/bin:$PATH"

# Or reinstall
curl -sSL https://strix.ai/install | bash
```

### Rate Limits

If you hit LLM rate limits:

```bash
# Use quick scan with reduced reasoning
strix -n -t ./repo --scan-mode quick --instruction "Find critical issues only"
```

### Finding Not Reproducible

Some findings might be environment-specific. Review:

1. The PoC (proof-of-concept) in `exploits/`
2. Reproduction steps in the finding
3. Test the same endpoint in your local environment

## Performance Tips

### Faster Scans

```bash
# Use quick scan mode
--scan-mode quick

# Reduce reasoning effort
export STRIX_REASONING_EFFORT=medium

# Focus on specific areas
--instruction "Test authentication endpoints only"
```

### Better Coverage

```bash
# Use full scan mode
--scan-mode full

# Increase reasoning effort
export STRIX_REASONING_EFFORT=high

# Test authenticated and public endpoints
--instruction "Include both authenticated and unauthenticated testing"
```

## Next Steps

1. **Run your first scan**: Use the quick examples above
2. **Review findings**: Check the generated reports
3. **Understand results**: Read the security fundamentals
4. **Integrate with CI/CD**: Add the GitHub Actions workflow
5. **Auto-fix vulnerabilities**: Use the Strix platform for patch generation

## Support & Resources

- **Strix Docs**: https://docs.strix.ai
- **GitHub Repo**: https://github.com/usestrix/strix
- **Discord Community**: https://discord.gg/strix-ai
- **Platform**: https://app.strix.ai (enterprise features)

## Security Best Practices

✅ **DO:**
- Test only applications you own or have permission to test
- Review and understand findings before applying fixes
- Use this in your CI/CD pipeline
- Keep your LLM API keys secure
- Integrate with your security team's workflow

❌ **DON'T:**
- Test unauthorized applications
- Blindly apply auto-fixes without review
- Commit API keys to version control
- Disable security checks to speed up scanning
- Run exploits on production without approval

---

**Ready to secure your apps?** Start with the quick examples above!
