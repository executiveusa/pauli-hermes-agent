---
name: strix-batch-security-testing
description: "Batch security testing for GitHub repos using Strix AI agents. Spawns parallel agents to detect and fix vulnerabilities across multiple applications."
version: 1.0.0
author: Pauli Hermes Agent + Strix
license: Apache 2.0
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [security-testing, penetration-testing, strix, parallel-agents, vulnerability-scanning, batch-testing, security-fixes, github-integration, owasp]
    related_skills: [red-teaming/godmode, security-review]
    capabilities: [multi-agent-orchestration, parallel-execution, dynamic-coordination, exploit-validation]
---

# Strix Batch Security Testing Skill

Automated security testing for your GitHub repositories using Strix autonomous AI pentesting agents. Spawns parallel agents to detect OWASP Top 10 vulnerabilities, validate exploits with working PoCs, and generate security patches.

**Key capabilities:**
- **Multi-agent orchestration** — parallel pentesting agents working on different repos simultaneously
- **Real exploit validation** — working proof-of-concepts, not false positives
- **Batch testing** — test multiple repos in a single run with automatic result aggregation
- **Auto-fix & patching** — AI-generated security patches as ready-to-merge PRs
- **Comprehensive reporting** — CVSS scoring, OWASP classification, compliance-ready reports
- **CI/CD integration** — GitHub Actions compatible for continuous security testing

## When to Use This Skill

Trigger when the user:
- Wants to "batch test apps for security flaws"
- Needs security audits across multiple GitHub repos
- Asks to "run security tests on my repos"
- Wants to find and fix OWASP vulnerabilities
- Requests penetration testing automation
- Wants continuous security scanning in CI/CD
- Needs compliance-ready security reports
- Asks for "parallel security testing"
- Wants to validate exploits with PoCs

## Architecture: Parallel Agent Teams

```
User Request
    ↓
Strix Coordinator Agent
    ├─→ Recon Agent (Subdomain enumeration, fingerprinting)
    ├─→ Exploitation Agent (Vulnerability discovery, PoC development)
    ├─→ Post-Exploitation Agent (Impact assessment)
    ├─→ API Testing Agent (OpenAPI/Swagger analysis)
    └─→ Fix Agent (Patch generation, PR creation)
```

Each agent specializes in a different phase of pentesting:
- **Recon** — Attack surface mapping, technology detection
- **Exploitation** — Vulnerability testing against OWASP Top 10
- **Validation** — PoC execution and impact measurement
- **Fix** — Patch generation with remediation guidance

## Vulnerability Coverage

Strix automatically detects and validates:

### Injection Attacks (OWASP A03)
- SQL injection (SQLi)
- NoSQL injection
- OS command injection
- Template injection (SSTI)

### Broken Access Control (OWASP A01)
- Insecure Direct Object Reference (IDOR)
- Privilege escalation
- Authentication bypass
- Session fixation

### Server-Side Vulnerabilities
- Server-Side Request Forgery (SSRF)
- XXE (XML External Entity)
- Insecure deserialization
- Remote Code Execution (RCE)

### Client-Side Attacks
- Cross-Site Scripting (XSS) — stored, reflected, DOM
- Cross-Site Request Forgery (CSRF)
- Prototype pollution
- Clickjacking

### Business Logic Flaws
- Race conditions
- Payment manipulation
- Workflow bypass
- Price manipulation

### API Security
- Broken authentication
- Mass assignment
- Rate limiting bypass
- Exposed internal APIs

### Infrastructure & Cloud
- Misconfiguration exposure
- Exposed services & ports
- Cloud bucket exposure
- Insecure cloud APIs

## Quick Start

### Option 1: Test a Single Repository

```bash
strix --target https://github.com/org/repo
```

### Option 2: Batch Test Multiple Repos

Create a file `targets.txt`:
```
https://github.com/org/app1
https://github.com/org/app2
https://github.com/org/api
```

Run batch testing:
```bash
strix --target-list ./targets.txt
```

### Option 3: Test Deployed Applications

```bash
strix --target https://app1.example.com --target https://app2.example.com
```

### Option 4: API Testing (OpenAPI/Swagger)

```bash
strix --target ./openapi.yaml --target https://api.your-app.com
```

## Step 1: Configuration

### Install Strix

```bash
curl -sSL https://strix.ai/install | bash
```

### Set LLM Provider

Strix supports multiple LLM providers:

```bash
# OpenAI GPT-5.4 (Recommended)
export STRIX_LLM="openai/gpt-5.4"
export LLM_API_KEY="sk-..."

# Anthropic Claude (via free proxy from CLAUDE.md)
export STRIX_LLM="anthropic/claude-sonnet-4-6"
export LLM_API_KEY="sk-ant-..."
export ANTHROPIC_BASE_URL="http://31.220.58.212:8082"

# Google Vertex AI
export STRIX_LLM="vertex_ai/gemini-3-pro-preview"
export LLM_API_KEY="your-json-key"

# Local Ollama
export STRIX_LLM="ollama/mistral"
export LLM_API_BASE="http://localhost:11434"
```

### Optional: Search Enhancement

For improved reconnaissance:

```bash
export PERPLEXITY_API_KEY="pplx-..."
```

## Step 2: Run Security Tests

### Headless Mode (Perfect for CI/CD)

```bash
strix -n --target ./repo --scan-mode quick
```

Exit codes:
- `0` — No vulnerabilities found
- `1` — Vulnerabilities discovered
- `2` — Error during scan

### Interactive Mode (Local Testing)

```bash
strix --target ./repo
```

Browse results in the dashboard:
```bash
strix view
```

### Custom Scan Instructions

Provide detailed testing guidance:

```bash
strix --target api.example.com \
  --instruction "Focus on authentication bypass and IDOR. Skip rate limiting tests."
```

Load instructions from file:

```bash
strix --target ./repo --instruction-file ./rules-of-engagement.md
```

### Authenticated Testing

Test with credentials:

```bash
strix --target https://app.com \
  --instruction "Use credentials: admin@example.com / password123. Test admin panel endpoints."
```

### Diff-Scoped Testing (PR Security Checks)

Test only changed files:

```bash
strix -n --target ./ --scan-mode quick --scope-mode diff --diff-base origin/main
```

## Step 3: Interpret Results

### Run Overview

Each run generates:

```
strix_runs/
└── my-app-2024-08-05-120000/
    ├── findings.json          # Machine-readable findings
    ├── report.html            # Compliance report
    ├── run-state.json         # Run metadata
    ├── agent-graph.json       # Multi-agent execution trace
    └── exploits/
        ├── sqli-poc.py        # Working SQL injection PoC
        ├── xss-poc.html       # XSS reproduction steps
        └── idor-poc.json      # IDOR test case
```

### Finding Structure

Each vulnerability includes:

```json
{
  "title": "SQL Injection in /api/search",
  "severity": "CRITICAL",
  "cvss_score": 9.8,
  "owasp": "A03:2021 - Injection",
  "description": "Unsanitized user input in search parameter",
  "reproduction_steps": ["Step 1: ...", "Step 2: ..."],
  "proof_of_concept": "SELECT * FROM users WHERE id=1 OR 1=1",
  "remediation": "Use parameterized queries..."
}
```

### Severity Levels

| Level | CVSS Score | Impact |
|-------|-----------|--------|
| **CRITICAL** | 9.0-10.0 | Immediate exploitation, data compromise |
| **HIGH** | 7.0-8.9 | Remote code execution, significant data exposure |
| **MEDIUM** | 4.0-6.9 | Potential access control bypass |
| **LOW** | 0.1-3.9 | Minor issues, limited impact |
| **INFO** | N/A | Informational findings |

## Step 4: Generate Patches (Auto-Fix)

### Via Strix Platform

1. Go to [app.strix.ai](https://app.strix.ai)
2. Select a finding
3. Click "Generate Patch"
4. Strix creates a pull request with the fix

### Via CLI (Local)

Use the platform API to generate patches:

```bash
curl -X POST https://api.strix.ai/v1/findings/{finding_id}/patch \
  -H "Authorization: Bearer $STRIX_API_TOKEN"
```

### Example Patch

```patch
--- a/api/search.py
+++ b/api/search.py
@@ -10,7 +10,7 @@ def search_users(query):
-    sql = f"SELECT * FROM users WHERE name LIKE '%{query}%'"
-    cursor.execute(sql)
+    sql = "SELECT * FROM users WHERE name LIKE ?"
+    cursor.execute(sql, (query,))
```

## Step 5: Automate in CI/CD

### GitHub Actions Workflow

Create `.github/workflows/strix-security-scan.yml`:

```yaml
name: Strix Security Scan

on:
  pull_request:
  schedule:
    - cron: "0 2 * * *"  # Daily at 2 AM UTC

jobs:
  security-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install Strix
        run: curl -sSL https://strix.ai/install | bash

      - name: Run Security Test
        env:
          STRIX_LLM: ${{ secrets.STRIX_LLM }}
          LLM_API_KEY: ${{ secrets.LLM_API_KEY }}
        run: strix -n -t ./ --scan-mode quick

      - name: Upload Results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: strix-results
          path: strix_runs/
```

### GitLab CI Pipeline

```yaml
security-scan:
  stage: security
  image: ubuntu:latest
  script:
    - curl -sSL https://strix.ai/install | bash
    - strix -n -t ./ --scan-mode quick
  artifacts:
    paths:
      - strix_runs/
    expire_in: 90 days
```

## Advanced: Custom Agents

### Add Custom Agent Behavior

Create `instructions.md`:

```markdown
# Custom Security Testing Instructions

## Scope
Test only these endpoints:
- /api/users
- /api/products
- /api/admin

## Focus Areas
1. Authentication mechanisms
2. Data validation
3. Business logic

## Exclusions
- Rate limiting tests (known to cause issues)
- Payment gateway testing (external service)

## Tools Available
- HTTP interception proxy
- Browser automation
- Custom Python exploits
- Shell access for enumeration
```

Run with custom instructions:

```bash
strix --target ./app --instruction-file ./instructions.md
```

## Multi-Agent Orchestration

Strix automatically distributes work across specialized agents:

```
Coordinator Agent (Main orchestrator)
├── Reconnaissance Agent (OSINT, fingerprinting)
│   ├── Service enumeration
│   ├── Technology detection
│   └── Surface mapping
├── Exploitation Agent (Vulnerability testing)
│   ├── OWASP Top 10 testing
│   ├── Custom exploits
│   └── PoC validation
├── Post-Exploitation Agent (Impact assessment)
│   ├── Data extraction
│   ├── Privilege escalation
│   └── Lateral movement
└── Reporting Agent (Documentation)
    ├── Finding aggregation
    ├── Patch generation
    └── Report creation
```

Each agent operates independently but coordinates findings through a shared graph database.

## Batch Testing Workflow

### Process Flow

```
1. Input: List of targets (repos, URLs, API specs)
   ↓
2. Validate targets and resolve resources
   ↓
3. Spawn coordinator agent for each target
   ↓
4. Coordinators spawn specialized agents (recon, exploit, etc.)
   ↓
5. Parallel execution with inter-agent communication
   ↓
6. Real-time finding aggregation
   ↓
7. Generate unified report across all targets
   ↓
8. Auto-create PRs for each fix
```

### Performance Metrics

- **Recon phase:** ~5-15 minutes per target
- **Exploitation phase:** ~30-60 minutes per target
- **Total time (5 repos):** ~2-3 hours with parallel execution
- **Typical findings per repo:** 5-20 vulnerabilities

## Troubleshooting

### Issue: Docker Not Running

```bash
# Start Docker
docker daemon

# Or use Docker Desktop
open -a Docker
```

### Issue: LLM Rate Limits

```bash
# Check current usage
strix auth status

# Reduce scan scope
strix -n -t ./ --scan-mode quick --instruction "Focus on critical issues only"
```

### Issue: Network Connectivity

```bash
# Test connectivity
curl https://api.openai.com/v1/models

# Use proxy if needed
export HTTP_PROXY=http://proxy.corp:8080
```

### Issue: Results in Wrong Directory

```bash
# Find all run results
find . -name "findings.json" -type f

# View specific run
strix view run-name-2024-08-05
```

## Performance Tuning

### Reduce Scan Time

```bash
# Quick scan (OWASP Top 3 only)
strix -n -t ./ --scan-mode quick

# Set reasoning effort
export STRIX_REASONING_EFFORT="medium"
```

### Increase Coverage

```bash
# Full comprehensive scan
strix -n -t ./ --scan-mode full

# Enable all modules
export STRIX_REASONING_EFFORT="high"
```

## Integration with Hermes Agent

### Automated Security Monitoring

```python
# In Hermes agent config
[[agents]]
name = "strix-monitor"
type = "batch-security-testing"
schedule = "daily"
targets = [
  "https://github.com/org/critical-app",
  "https://github.com/org/api",
  "https://prod.example.com"
]
on_findings = "create_pr"
```

### Slack Notifications

Configure Hermes MCP to notify on findings:

```yaml
security_alerts:
  channel: "#security-findings"
  on_critical: notify_team_lead
  on_high: notify_security_lead
  on_medium: log_to_channel
```

## Best Practices

1. **Always test responsibly** — only test apps you own or have explicit permission
2. **Start with quick scans** — validate setup before running full scans
3. **Review findings before auto-fix** — understand the vulnerability before applying patches
4. **Use authenticated testing** — some vulnerabilities only appear to authenticated users
5. **Run regular scans** — integrate into your deployment pipeline
6. **Monitor rate limits** — some providers have usage restrictions
7. **Keep LLM keys secure** — store in environment variables, not in code
8. **Review agent decisions** — don't blindly trust automated patches
9. **Test patches locally** — verify fixes before merging

## References

- [Strix Documentation](https://docs.strix.ai)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [CVSS Calculator](https://www.first.org/cvss/calculator/3.1)
- [CWE Top 25](https://cwe.mitre.org/top25/)
- [HackerOne Vulnerability Classification](https://www.hackerone.com/knowledge/vulnerability-classification)

## Support

- GitHub: [usestrix/strix](https://github.com/usestrix/strix)
- Discord: [Strix Community](https://discord.gg/strix-ai)
- Docs: [docs.strix.ai](https://docs.strix.ai)
- Platform: [app.strix.ai](https://app.strix.ai)

---

> [!WARNING]
> Only test applications you own or have explicit written permission to test. Unauthorized security testing is illegal. Strix users are responsible for compliance with applicable laws and regulations.
