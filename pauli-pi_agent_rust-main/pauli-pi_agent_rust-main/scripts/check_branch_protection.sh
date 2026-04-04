#!/usr/bin/env bash
# scripts/check_branch_protection.sh — Validate GitHub branch protection settings.
#
# Checks that the main branch has the required protection rules configured
# to prevent quality gate bypass.
#
# Usage:
#   ./scripts/check_branch_protection.sh                # interactive
#   ./scripts/check_branch_protection.sh --report       # JSON output
#   ./scripts/check_branch_protection.sh --repo owner/repo  # specific repo
#
# Requires: gh (GitHub CLI) authenticated with repo access.

set -euo pipefail

BRANCH="main"
REPO=""
REPORT_JSON=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --repo) REPO="$2"; shift 2 ;;
        --branch) BRANCH="$2"; shift 2 ;;
        --report) REPORT_JSON=1; shift ;;
        --help|-h)
            sed -n '2,/^$/p' "$0" | sed 's/^# \?//'
            exit 0
            ;;
        *) echo "Unknown flag: $1"; exit 1 ;;
    esac
done

# Auto-detect repo from git remote if not specified.
if [[ -z "$REPO" ]]; then
    REPO=$(gh repo view --json nameWithOwner -q '.nameWithOwner' 2>/dev/null || true)
    if [[ -z "$REPO" ]]; then
        echo "Could not detect repo. Use --repo owner/name."
        exit 1
    fi
fi

# ─── Required status checks ─────────────────────────────────────────────────

REQUIRED_CHECKS=(
    "rust (ubuntu-latest)"
    "rust (macos-latest)"
    "rust (windows-latest)"
    "conformance (fast-official)"
    "conformance (fast-generated)"
    "conformance (fast-negative)"
    "conformance (fast-capability-matrix)"
)

# ─── State tracking ─────────────────────────────────────────────────────────

PASS_COUNT=0
FAIL_COUNT=0
WARN_COUNT=0
declare -a CHECKS=()

log() {
    if [[ "$REPORT_JSON" -eq 0 ]]; then
        echo "[$1] $2"
    fi
}

check_pass() {
    local name="$1"
    local detail="$2"
    log "PASS" "$name: $detail"
    PASS_COUNT=$((PASS_COUNT + 1))
    CHECKS+=("{\"name\":\"$name\",\"status\":\"pass\",\"detail\":\"$detail\"}")
}

check_fail() {
    local name="$1"
    local detail="$2"
    log "FAIL" "$name: $detail"
    FAIL_COUNT=$((FAIL_COUNT + 1))
    CHECKS+=("{\"name\":\"$name\",\"status\":\"fail\",\"detail\":\"$detail\"}")
}

check_warn() {
    local name="$1"
    local detail="$2"
    log "WARN" "$name: $detail"
    WARN_COUNT=$((WARN_COUNT + 1))
    CHECKS+=("{\"name\":\"$name\",\"status\":\"warn\",\"detail\":\"$detail\"}")
}

# ─── Fetch protection rules ─────────────────────────────────────────────────

PROTECTION_JSON=$(gh api "repos/$REPO/branches/$BRANCH/protection" 2>/dev/null || echo "NONE")

if [[ "$PROTECTION_JSON" == "NONE" ]]; then
    check_fail "protection_enabled" "No branch protection rules found for $BRANCH"

    # Output early and exit.
    if [[ "$REPORT_JSON" -eq 1 ]]; then
        echo '{"schema":"pi.branch_protection.v1","repo":"'"$REPO"'","branch":"'"$BRANCH"'","verdict":"fail","counts":{"pass":0,"fail":1,"warn":0},"checks":[{"name":"protection_enabled","status":"fail","detail":"No branch protection rules found"}]}'
    else
        echo ""
        echo "VERDICT: FAIL — no branch protection on $BRANCH"
    fi
    exit 1
fi

check_pass "protection_enabled" "Branch protection is enabled for $BRANCH"

# ─── Check required status checks ───────────────────────────────────────────

STATUS_CHECKS=$(echo "$PROTECTION_JSON" | python3 -c "
import json, sys
data = json.load(sys.stdin)
checks = data.get('required_status_checks', {})
if checks:
    strict = checks.get('strict', False)
    contexts = checks.get('contexts', [])
    print(f'{strict}')
    for c in contexts:
        print(c)
else:
    print('NONE')
" 2>/dev/null || echo "NONE")

if [[ "$STATUS_CHECKS" == "NONE" ]]; then
    check_fail "required_status_checks" "No required status checks configured"
else
    STRICT=$(echo "$STATUS_CHECKS" | /usr/bin/head -1)
    CONFIGURED_CHECKS=$(echo "$STATUS_CHECKS" | /usr/bin/tail -n +2)

    if [[ "$STRICT" == "True" ]]; then
        check_pass "strict_mode" "Branches must be up-to-date before merging"
    else
        check_fail "strict_mode" "Strict mode disabled — stale branches can merge"
    fi

    for required in "${REQUIRED_CHECKS[@]}"; do
        if echo "$CONFIGURED_CHECKS" | grep -qF "$required"; then
            check_pass "check_$required" "Required check present"
        else
            check_fail "check_$required" "Missing required status check: $required"
        fi
    done
fi

# ─── Check admin enforcement ────────────────────────────────────────────────

ENFORCE_ADMINS=$(echo "$PROTECTION_JSON" | python3 -c "
import json, sys
data = json.load(sys.stdin)
ea = data.get('enforce_admins', {})
print(ea.get('enabled', False) if isinstance(ea, dict) else False)
" 2>/dev/null || echo "False")

if [[ "$ENFORCE_ADMINS" == "True" ]]; then
    check_pass "enforce_admins" "Admins cannot bypass protection"
else
    check_fail "enforce_admins" "Admins can bypass protection rules"
fi

# ─── Check force push / deletion ────────────────────────────────────────────

FORCE_PUSH=$(echo "$PROTECTION_JSON" | python3 -c "
import json, sys
data = json.load(sys.stdin)
fp = data.get('allow_force_pushes', {})
print(fp.get('enabled', True) if isinstance(fp, dict) else True)
" 2>/dev/null || echo "True")

if [[ "$FORCE_PUSH" == "False" ]]; then
    check_pass "no_force_push" "Force pushes are disabled"
else
    check_fail "no_force_push" "Force pushes are allowed — history can be rewritten"
fi

ALLOW_DELETE=$(echo "$PROTECTION_JSON" | python3 -c "
import json, sys
data = json.load(sys.stdin)
ad = data.get('allow_deletions', {})
print(ad.get('enabled', True) if isinstance(ad, dict) else True)
" 2>/dev/null || echo "True")

if [[ "$ALLOW_DELETE" == "False" ]]; then
    check_pass "no_deletions" "Branch deletion is disabled"
else
    check_warn "no_deletions" "Branch deletion is allowed"
fi

# ─── Check PR reviews ───────────────────────────────────────────────────────

PR_REVIEWS=$(echo "$PROTECTION_JSON" | python3 -c "
import json, sys
data = json.load(sys.stdin)
pr = data.get('required_pull_request_reviews', {})
if pr:
    count = pr.get('required_approving_review_count', 0)
    dismiss = pr.get('dismiss_stale_reviews', False)
    print(f'{count} {dismiss}')
else:
    print('0 False')
" 2>/dev/null || echo "0 False")

read -r REVIEW_COUNT DISMISS_STALE <<< "$PR_REVIEWS"

if [[ "$REVIEW_COUNT" -ge 1 ]]; then
    check_pass "pr_reviews" "Requires $REVIEW_COUNT approving review(s)"
else
    check_fail "pr_reviews" "No pull request reviews required"
fi

if [[ "$DISMISS_STALE" == "True" ]]; then
    check_pass "dismiss_stale" "Stale approvals are dismissed on new pushes"
else
    check_warn "dismiss_stale" "Stale approvals are not dismissed"
fi

# ─── Summary ────────────────────────────────────────────────────────────────

TOTAL_CHECKS=$((PASS_COUNT + FAIL_COUNT + WARN_COUNT))

if [[ "$REPORT_JSON" -eq 1 ]]; then
    JSON_CHECKS=""
    for c in "${CHECKS[@]}"; do
        if [[ -n "$JSON_CHECKS" ]]; then
            JSON_CHECKS="$JSON_CHECKS,$c"
        else
            JSON_CHECKS="$c"
        fi
    done

    VERDICT="pass"
    if [[ $FAIL_COUNT -gt 0 ]]; then
        VERDICT="fail"
    fi

    cat <<EOF
{
  "schema": "pi.branch_protection.v1",
  "repo": "$REPO",
  "branch": "$BRANCH",
  "verdict": "$VERDICT",
  "counts": {
    "pass": $PASS_COUNT,
    "fail": $FAIL_COUNT,
    "warn": $WARN_COUNT,
    "total": $TOTAL_CHECKS
  },
  "checks": [$JSON_CHECKS]
}
EOF
else
    echo ""
    echo "═══════════════════════════════════════════════════════════"
    echo "  Branch Protection Audit — $REPO ($BRANCH)"
    echo "═══════════════════════════════════════════════════════════"
    echo "  Pass: $PASS_COUNT  Fail: $FAIL_COUNT  Warn: $WARN_COUNT  Total: $TOTAL_CHECKS"
    echo "═══════════════════════════════════════════════════════════"

    if [[ $FAIL_COUNT -gt 0 ]]; then
        echo "  VERDICT: FAIL — branch protection is insufficient"
        exit 1
    else
        echo "  VERDICT: PASS — branch protection is properly configured"
    fi
fi
