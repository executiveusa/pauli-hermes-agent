# Releasing pi_agent_rust
This repo ships:
- A crates.io package: `pi_agent_rust` (Cargo `[package].name`)
- A library crate: `pi` (Cargo `[lib].name`)
- A binary: `pi` (Cargo `[[bin]].name`)

## Versioning + tags (source of truth)
**Source of truth:** `Cargo.toml` `[package].version`.

- **Tag format:** `vX.Y.Z` (SemVer). Example: `v0.2.0`.
- **Pre-releases:** `vX.Y.Z-rc.1` (or similar). Example: `v0.2.0-rc.1`.
- **Coupling:** `pi_agent_rust` (crate), `pi` (lib), and `pi` (binary) are all built from the same package, so they share one version number.
- **Sibling repos:** `asupersync`, `rich_rust`, `charmed_rust`, `sqlmodel_rust` are versioned independently in their own repos.

### Publishing to crates.io
`.github/workflows/publish.yml` is triggered on tag pushes matching `v*` and will:
1) validate the tag is SemVer
2) verify `Cargo.toml` version matches the tag version
3) run `cargo publish --dry-run --locked`
4) publish to crates.io **only** when:
   - the tag is **not** a pre-release (workflow checks `tag` does **not** contain `-`)
   - `CARGO_REGISTRY_TOKEN` is configured

Note: dependencies that specify both `version` and `path` are expected to publish using the `version` constraint; ensure those versions exist on crates.io before tagging.

### Publishing GitHub Releases binaries
`.github/workflows/release.yml` is triggered on tag pushes matching `v*` and will:
- build `pi` for Linux/macOS/Windows (release profile)
- attach binaries, per-target build manifests, and `SHA256SUMS` to a GitHub Release
- mark the GitHub Release as a pre-release if the tag contains `-` (e.g. `-rc.1`)

Release notes are extracted from `CHANGELOG.md` on a best-effort basis; ensure the changelog contains a `##` heading with the version string for the tag you are cutting.

## Distribution compatibility strategy (DROPIN-146)
Goal: keep packaging and invocation ergonomics compatible enough for frictionless migration from upstream Pi.

### Supported distribution paths
- **Installer path (`install.sh`)**: default channel for end users; installs GitHub release binary, verifies checksums, and manages migration state.
- **Release artifact path (GitHub Releases)**: direct binary download per OS/arch with `SHA256SUMS` verification.
- **Source path (`cargo build --release`)**: deterministic fallback for constrained/air-gapped environments.

### Executable compatibility path
- Canonical command is `pi`.
- If TypeScript `pi` already exists, installer supports in-place migration and preserves old command as `legacy-pi`.
- If migration is declined (`--keep-existing-pi`), Rust Pi installs as `pi-rust` so both CLIs remain callable.
- Pinned rollout is supported by `install.sh --version vX.Y.Z`.

### Representative validation matrix
Run this matrix before declaring distribution parity complete for a release candidate:

1. Fresh Linux/macOS install (no prior `pi`):
   - `curl .../install.sh | bash`
   - `command -v pi && pi --version && pi --help >/dev/null`
2. Migration host with existing TypeScript `pi`:
   - `install.sh --adopt` (or interactive adopt path)
   - `pi --version` returns Rust build
   - `legacy-pi --version` still resolves to preserved TypeScript CLI
3. Keep-existing path:
   - `install.sh --keep-existing-pi`
   - `pi` remains TypeScript CLI, `pi-rust --version` resolves to Rust build
4. Pinned enterprise/CI rollout:
   - `install.sh --version vX.Y.Z`
   - binary checksum validation passes against release `SHA256SUMS`

## Perf-vs-size artifact policy (bd-3ar8v.5.5)

Release operations must keep benchmark evidence and shipping artifacts distinct.

- **Shipping/distribution artifacts**: built with Cargo `release` profile and published via
  `release.yml` + installer flows (`pi` binaries + `SHA256SUMS`).
- **Benchmark evidence artifacts**: produced by PERF-3X lanes (`scripts/perf/orchestrate.sh`,
  `scripts/bench_extension_workloads.sh`) using benchmark profile labeling (typically `perf`)
  with run-level provenance (`correlation_id`, build/profile metadata, allocator/PGO metadata).

Policy constraints:

1. Performance and certification claims must cite benchmark evidence artifacts, not release-only binaries.
2. Release binaries remain the deployment target and may be used to validate size/startup/install behavior.
3. Any release note claiming performance gains should include correlation-linked evidence references from benchmark artifact bundles.
4. If profile labels/provenance are missing or contradictory, treat the performance claim as invalid until regenerated.

## When do we call it 1.0?
We call it `1.0.0` when:
- CI is green on Linux/macOS/Windows (`.github/workflows/ci.yml`)
- Required execution surfaces are parity-stable (interactive + print + JSON mode + RPC + SDK contract) with conformance evidence green
- Extension runtime surface and security policy are stable enough that we can commit to not breaking users without an intentional SemVer bump
- Drop-in certification artifacts report `CERTIFIED` for strict replacement claims

Until then, `0.x` releases may still change behavior to improve correctness/parity, and release messaging must not claim strict drop-in replacement.

## Cutting a release (patch/minor)
1) **Pick version** (SemVer):
   - patch: bugfixes / internal refactors
   - minor: new user-facing features
2) **Update version** in `Cargo.toml` (`[package].version`).
3) **Run quality gates locally**:
   - `cargo fmt --check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test --all-targets`
4) **Update changelog**:
   - `br changelog --since-tag vX.Y.Z` (or use `--since YYYY-MM-DD` if no prior tags)
   - paste the output into `CHANGELOG.md` under a new version heading
5) **Commit** (`git commit`).
6) **Tag**:
   - `git tag vX.Y.Z`
   - `git push origin vX.Y.Z`
7) **Verify** GitHub Actions:
   - `Publish` workflow (crates.io publish) behaves as expected
   - `Release (GitHub binaries)` workflow creates a GitHub Release with binaries + `SHA256SUMS`

## Pre-release flow (rc)
Use a pre-release tag to exercise CI/publish validation without publishing to crates.io:
- `git tag vX.Y.Z-rc.1 && git push origin vX.Y.Z-rc.1`

This should run the `Publish` workflow planning step and skip the crates publish step.

## Merge-Gate DoD Policy
Feature-surface pull requests must satisfy the Definition-of-Done evidence checklist before merge:
- Unit evidence link(s)
- E2E evidence link(s)
- Extension evidence link(s)
- Reproduction commands for pass/fail validation paths

CI enforces this via `.github/workflows/ci.yml` using `.github/pull_request_template.md` as the
canonical checklist format.

### Migration Guidance for Existing Feature Branches
For branches opened before this gate was introduced:
1. Rebase onto latest `main`.
2. Replace the PR body with `.github/pull_request_template.md`.
3. Backfill links to current evidence artifacts.
4. Include exact rerun commands used to validate fixes for the most recent failing path.
5. Re-run CI and merge only after the DoD evidence guard passes.

## Pre-release checklist
- CI is green on `main` (Linux/macOS/Windows).
- Local gates are green:
  - `cargo fmt --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test --all-targets`
- Feature PRs merged since the previous tag satisfy the DoD evidence checklist (unit + e2e + extension + repro commands).
- `CHANGELOG.md` updated for the version you’re tagging.
- Benchmarks run if this release is performance-sensitive (see `BENCHMARKS.md`).
- Distribution compatibility matrix (above) passes for all required paths.

## Post-release checklist
- GitHub Release exists and includes expected artifacts for each platform.
- `SHA256SUMS` matches downloaded artifacts.
- Crates.io publish succeeded (if configured) and the version matches the tag.
- Smoke test install paths (download binary + run `pi --version`).
