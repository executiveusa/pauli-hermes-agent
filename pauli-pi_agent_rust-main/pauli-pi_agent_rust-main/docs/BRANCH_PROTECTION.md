# Branch Protection and Merge Policy

## Purpose

Quality gates (conformance, clippy, tests, coverage) only protect the
codebase if they cannot be bypassed during the merge workflow. This
document specifies the required branch protection rules for the `main`
branch and documents how each CI gate maps to a required status check.

## Required Status Checks

The following checks must pass before a PR can be merged to `main`:

| CI Job | Workflow | Required | Blocks Merge |
|--------|----------|----------|--------------|
| `rust (ubuntu-latest)` | `ci.yml` | Yes | Yes |
| `rust (macos-latest)` | `ci.yml` | Yes | Yes |
| `rust (windows-latest)` | `ci.yml` | Yes | Yes |
| `conformance (fast-official)` | `conformance.yml` | Yes | Yes |
| `conformance (fast-generated)` | `conformance.yml` | Yes | Yes |
| `conformance (fast-negative)` | `conformance.yml` | Yes | Yes |
| `conformance (fast-capability-matrix)` | `conformance.yml` | Yes | Yes |

## What Each Gate Enforces

### CI Pipeline (`ci.yml`)

Per-platform (Linux, macOS, Windows):

1. **No-mock dependency guard** — Blocks `mockall`, `mockito`, `wiremock` in `Cargo.toml`/`Cargo.lock`.
2. **No-mock code guard** — Blocks `Mock*/Fake*/Stub*` identifiers in test code (allowlisted: `MockHttp{Server,Request,Response}`).
3. **Traceability matrix guard** — Validates `docs/traceability_matrix.json` consistency.
4. **Suite classification guard** — Every `tests/*.rs` must appear in `tests/suite_classification.toml`.
5. **VCR leak guard** — Unit-suite files must not reference VCR infrastructure.
6. **PR Definition-of-Done evidence guard** — For feature-surface PRs, blocks merge unless the PR body includes checked unit/e2e/extension evidence links and reproduction commands (Linux PR lane only).
7. **`cargo fmt --check`** — Format compliance.
8. **`cargo clippy -D warnings`** — Zero clippy warnings.
9. **`cargo doc --no-deps`** — Documentation builds cleanly.
10. **`cargo test --all-targets`** — All tests pass.
11. **Unified verification runner** — `scripts/e2e/run_all.sh --profile ci` (Linux only).
12. **CI gate promotion** — Conformance thresholds enforced (Linux only).
13. **Conformance regression gate** — No pass-rate regressions (Linux only).
14. **Coverage gate** — Line coverage >= 50% (Linux only).

### Conformance Pipeline (`conformance.yml`)

On PRs, four fast conformance checks run:

1. **fast-official** — Sample of official extensions (max 5).
2. **fast-generated** — Generated tier 1-2 scenarios.
3. **fast-negative** — Negative policy tests.
4. **fast-capability-matrix** — Capability denial matrix.

## GitHub Branch Protection Settings

### Recommended Configuration for `main`

```
Settings → Branches → Branch protection rules → main
```

| Setting | Value | Rationale |
|---------|-------|-----------|
| Require a pull request before merging | Enabled | No direct pushes to main |
| Required approvals | 1 | Minimum review gate |
| Dismiss stale pull request approvals | Enabled | Re-review after force-push |
| Require status checks to pass before merging | Enabled | CI gates are mandatory |
| Require branches to be up to date before merging | Enabled | Prevents stale merges |
| Required status checks | See [Required Status Checks](#required-status-checks) | All listed checks |
| Require conversation resolution before merging | Enabled | No unresolved threads |
| Require signed commits | Recommended | Commit provenance |
| Include administrators | Enabled | No admin bypass |
| Allow force pushes | Disabled | Prevents history rewriting |
| Allow deletions | Disabled | Prevents branch deletion |

### Applying via GitHub CLI

```bash
# Set required status checks (adjust repo owner/name):
gh api repos/{owner}/{repo}/branches/main/protection \
  --method PUT \
  --field required_status_checks='{"strict":true,"contexts":["rust (ubuntu-latest)","rust (macos-latest)","rust (windows-latest)","conformance (fast-official)","conformance (fast-generated)","conformance (fast-negative)","conformance (fast-capability-matrix)"]}' \
  --field enforce_admins=true \
  --field required_pull_request_reviews='{"required_approving_review_count":1,"dismiss_stale_reviews":true}' \
  --field restrictions=null \
  --field allow_force_pushes=false \
  --field allow_deletions=false
```

## Validation Script

Run `scripts/check_branch_protection.sh` to validate that branch
protection is correctly configured. This checks:

1. Required status checks are present.
2. `strict` mode (up-to-date branches) is enabled.
3. Admin enforcement is enabled.
4. Force pushes are disabled.
5. Deletions are disabled.
6. Pull request reviews are required.

## Migration Guidance for Existing Feature Branches

When this DoD gate rolls out, open feature branches created before rollout may be missing
the required PR evidence sections. Migrate those branches before merge:

1. Rebase on the latest `main`.
2. Replace the PR body with `.github/pull_request_template.md`.
3. Add direct links to unit, e2e, and extension evidence artifacts.
4. Add exact reproduction commands used for validation and for the most recent failing path.
5. Re-run CI and confirm the DoD evidence guard passes.

## Release Workflow Integration

The release workflow (`release.yml`) triggers on version tags (`v*`).
Because releases are created from `main`, the branch protection rules
ensure that only code that passed all CI gates can be released.

The `scripts/release_gate.sh` script provides an additional local or CI
pre-release check that validates the conformance evidence bundle meets
minimum thresholds before a release tag is created.

### Pre-Release Checklist

1. All CI checks pass on `main`.
2. `scripts/release_gate.sh --report` returns `verdict: pass`.
3. Conformance pass rate >= 80% (configurable via `RELEASE_GATE_MIN_PASS_RATE`).
4. Conformance failures <= 36 (configurable via `RELEASE_GATE_MAX_FAIL_COUNT`).
5. Tag follows semver: `vMAJOR.MINOR.PATCH[-prerelease]`.

## Bypass Prevention

### What Cannot Be Bypassed

- Status checks: Required for all users including administrators.
- PR requirement: Direct pushes to `main` are blocked.
- Format and lint: `cargo fmt --check` and `cargo clippy -D warnings`.

### Emergency Procedures

In genuine emergencies (e.g., security patches), a repository admin can
temporarily disable branch protection. This must be:

1. Documented in a GitHub issue with justification.
2. Re-enabled immediately after the emergency merge.
3. Reviewed in the next team sync.

## Monitoring

### CI Health Dashboard

Track these metrics weekly:

- **Flake rate**: Transient failures / total runs (target: < 5%).
- **Mean CI duration**: Average wall-clock time for the `ci` workflow.
- **Coverage trend**: Line coverage over time (floor: 50%).
- **Conformance pass rate**: Extension conformance trend.

### Alerts

- CI flake rate exceeds 5% → investigate per-target flake budgets.
- Coverage drops below 50% → `cargo llvm-cov` gate will block merge.
- Conformance pass rate drops → `CI_GATE_PROMOTION_MODE=strict` blocks merge.
