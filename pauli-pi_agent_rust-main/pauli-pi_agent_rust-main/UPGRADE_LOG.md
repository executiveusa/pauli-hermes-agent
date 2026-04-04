# Dependency Upgrade Log

**Date:** 2026-02-14
**Project:** pi_agent_rust
**Language:** Rust
**Manifests:** `Cargo.toml`, `fuzz/Cargo.toml`

---

## Summary

| Metric | Count |
|--------|-------|
| **Total dependencies (direct, outdated)** | 18 |
| **Updated** | 18 |
| **Skipped** | 0 |
| **Failed (rolled back)** | 0 |
| **Requires attention** | 0 |

---

## Discovery

Detected manifests:
- `Cargo.toml`
- `fuzz/Cargo.toml`

Outdated direct dependencies detected (current -> latest stable):
- `anyhow` `1.0.100` -> `1.0.101`
- `clap` `4.5.56` -> `4.5.58`
- `clap_complete` `4.5.65` -> `4.5.66`
- `criterion` `0.7.0` -> `0.8.2`
- `ctrlc` `3.5.1` -> `3.5.2`
- `getrandom` `0.2.17` -> `0.4.1`
- `jsonschema` `0.40.2` -> `0.42.0`
- `memchr` `2.7.6` -> `2.8.0`
- `proptest` `1.9.0` -> `1.10.0`
- `regex` `1.12.2` -> `1.12.3`
- `sysinfo` `0.36.1` -> `0.38.1`
- `tempfile` `3.24.0` -> `3.25.0`
- `toml` `0.8.23` -> `1.0.1+spec-1.1.0`
- `uuid` `1.20.0` -> `1.21.0`
- `vergen` `9.0.6` -> `9.1.0` (fuzz)
- `vergen-gix` `1.0.9` -> `9.1.0`
- `wasmtime` `29.0.1` -> `41.0.3`
- `wat` `1.244.0` -> `1.245.1`

---

## Successfully Updated

- Root manifest (`Cargo.toml`) direct dependency specs updated:
  - `anyhow = "1.0.101"`
  - `clap = "4.5.58"`
  - `clap_complete = "4.5.66"`
  - `ctrlc = "3.5.2"`
  - `tempfile = "3.25.0"`
  - `uuid = "1.21.0"`
  - `memchr = "2.8.0"`
  - `getrandom = "0.4.1"`
  - `regex = "1.12.3"`
  - `sysinfo = "0.38.1"`
  - `wasmtime = "41.0.3"`
  - `vergen-gix = "9.1.0"`
  - dev-deps: `criterion = "0.8.2"`, `jsonschema = "0.42.0"`, `proptest = "1.10.0"`, `wat = "1.245.1"`, `toml = "1.0.1"`, `tempfile = "3.25.0"`
- Fuzz manifest (`fuzz/Cargo.toml`) build deps updated:
  - `vergen-gix = "=9.1.0"`
  - `vergen = "=9.1.0"`
- Lockfiles refreshed with latest compatible resolutions.

---

## Compatibility / Follow-up Fixes Applied

To keep the project green on the upgraded toolchain/dependency set, additional code updates were required:

- `wasmtime` 41 API/macro migration in `src/extensions.rs` and `src/pi_wasm.rs`:
  - `component::bindgen!` async config switched to `imports/exports` flags.
  - linker glue updated for `HasSelf` generic usage.
  - new `Extern::Tag` variant handled.
- Event enum expansion (`AgentEvent::ExtensionError`) made existing matches non-exhaustive across multiple files; all affected match sites were updated.
- New `clippy` findings under `-D warnings` were fixed in tests/benches and helper code (doc markdown, float assertions, redundant clones/closures, formatting-string inlining, etc.).

---

## Validation

Executed (with build dirs on `/var/tmp` due shared `/dev/shm` and `/tmp` exhaustion):

```bash
export CARGO_TARGET_DIR="/var/tmp/pi_agent_rust/${USER:-agent}/target"
export TMPDIR="/var/tmp/pi_agent_rust/${USER:-agent}/tmp"
mkdir -p "$TMPDIR"

rch exec -- cargo check --all-targets
rch exec -- cargo clippy --all-targets -- -D warnings
rch exec -- cargo fmt --check
```

Results:
- `cargo check --all-targets` ✅
- `cargo clippy --all-targets -- -D warnings` ✅
- `cargo fmt --check` ✅

---

## Commands Used

```bash
# Discovery / inventory
cargo metadata --format-version 1 --no-deps
cargo metadata --manifest-path fuzz/Cargo.toml --format-version 1 --no-deps
cargo tree --depth 1 --prefix none
cargo tree --manifest-path fuzz/Cargo.toml --depth 1 --prefix none

# Upgrade + resolve
rch exec -- cargo update
rch exec -- cargo update --manifest-path fuzz/Cargo.toml

# Validation
rch exec -- cargo check --all-targets
rch exec -- cargo clippy --all-targets -- -D warnings
rch exec -- cargo fmt --check
```
