//! System-level benchmarks: startup time, memory usage, binary size.
//!
// Allow some clippy lints that are acceptable in benchmarks
#![allow(clippy::cast_precision_loss)] // u64 -> f64 for size calculations is fine
#![allow(clippy::cmp_owned)] // PathBuf comparison with "pi" requires owned
//!
//! Run with:
//! - `cargo bench --bench system`
//! - `cargo bench startup`
//! - `cargo bench memory`
//!
//! These benchmarks measure real-world performance by spawning the actual binary.
//! They complement the micro-benchmarks in tools.rs and extensions.rs.
//!
//! Performance budgets:
//! - Startup time (--version): <100ms (p95), 11.2ms typical
//! - Startup time (cold, full agent): <200ms (p95)
//! - Idle memory: <50MB RSS
//! - Binary size (release): <20MB

#[path = "bench_env.rs"]
mod bench_env;

use std::env;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use sysinfo::{ProcessRefreshKind, RefreshKind, System};

fn criterion_config() -> Criterion {
    bench_env::criterion_config_system()
}

// ============================================================================
// Binary Path Resolution
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BinaryKind {
    Release,
    Debug,
    Unknown,
}

#[derive(Debug, Clone)]
struct ResolvedBinary {
    path: PathBuf,
    kind: BinaryKind,
}

fn infer_binary_kind(path: &Path) -> BinaryKind {
    let mut saw_release = false;
    let mut saw_debug = false;
    for component in path.components() {
        let part = component.as_os_str().to_string_lossy();
        if part == "release" {
            saw_release = true;
        } else if part == "debug" {
            saw_debug = true;
        }
    }

    if saw_release {
        BinaryKind::Release
    } else if saw_debug {
        BinaryKind::Debug
    } else {
        BinaryKind::Unknown
    }
}

fn target_roots(manifest_dir: &Path) -> Vec<PathBuf> {
    let cargo_target_dir = env::var_os("CARGO_TARGET_DIR").map(PathBuf::from);
    target_roots_with(manifest_dir, cargo_target_dir.as_deref())
}

fn target_roots_with(manifest_dir: &Path, cargo_target_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Some(candidate) = cargo_target_dir {
        let resolved = if candidate.is_absolute() {
            candidate.to_path_buf()
        } else {
            manifest_dir.join(candidate)
        };
        roots.push(resolved);
    }

    let default_target = manifest_dir.join("target");
    if !roots.contains(&default_target) {
        roots.push(default_target);
    }

    roots
}

const fn binary_file_names() -> &'static [&'static str] {
    #[cfg(windows)]
    {
        &["pi.exe", "pi"]
    }
    #[cfg(not(windows))]
    {
        &["pi"]
    }
}

fn find_profile_binary(root: &Path, profile: &str) -> Option<PathBuf> {
    let profile_dir = root.join(profile);
    for binary_name in binary_file_names() {
        let candidate = profile_dir.join(binary_name);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
#[allow(dead_code)]
fn run_resolution_regression_checks() {
    use std::path::{Path, PathBuf};

    // Binary kind inference
    let path = Path::new("/tmp/target/release/pi");
    assert_eq!(infer_binary_kind(path), BinaryKind::Release);
    let path = Path::new("/tmp/target/debug/pi");
    assert_eq!(infer_binary_kind(path), BinaryKind::Debug);
    let path = Path::new("/tmp/target/debug/release/pi");
    assert_eq!(infer_binary_kind(path), BinaryKind::Release);
    let path = Path::new("/tmp/pi");
    assert_eq!(infer_binary_kind(path), BinaryKind::Unknown);

    // Relative CARGO_TARGET_DIR is resolved from manifest dir.
    let manifest_dir = Path::new("/workspace/pi_agent_rust");
    let roots = target_roots_with(manifest_dir, Some(Path::new("target/agents/blackglen")));
    assert_eq!(roots.len(), 2);
    assert_eq!(roots[0], manifest_dir.join("target/agents/blackglen"));
    assert_eq!(roots[1], manifest_dir.join("target"));

    // Absolute CARGO_TARGET_DIR is preserved.
    let roots = target_roots_with(manifest_dir, Some(Path::new("/tmp/custom-target")));
    assert_eq!(roots.len(), 2);
    assert_eq!(roots[0], PathBuf::from("/tmp/custom-target"));
    assert_eq!(roots[1], manifest_dir.join("target"));

    // Default target root should not be duplicated.
    let roots = target_roots_with(manifest_dir, Some(Path::new("target")));
    assert_eq!(roots, vec![manifest_dir.join("target")]);

    // Platform-aware binary lookup candidate order.
    let names = binary_file_names();
    #[cfg(windows)]
    {
        assert_eq!(names.first().copied(), Some("pi.exe"));
        assert!(names.contains(&"pi"));
    }
    #[cfg(not(windows))]
    {
        assert_eq!(names, &["pi"]);
    }
}

fn resolve_pi_binary() -> ResolvedBinary {
    // Check for explicit override
    if let Ok(path) = env::var("PI_BENCH_BINARY") {
        let path = PathBuf::from(path);
        return ResolvedBinary {
            kind: infer_binary_kind(&path),
            path,
        };
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let target_roots = target_roots(&manifest_dir);

    // Look for release binary first (more realistic)
    for root in &target_roots {
        if let Some(release_path) = find_profile_binary(root, "release") {
            return ResolvedBinary {
                path: release_path,
                kind: BinaryKind::Release,
            };
        }
    }

    // Fall back to debug binary
    for root in &target_roots {
        if let Some(debug_path) = find_profile_binary(root, "debug") {
            return ResolvedBinary {
                path: debug_path,
                kind: BinaryKind::Debug,
            };
        }
    }

    // Last resort: hope it's in PATH
    ResolvedBinary {
        path: PathBuf::from("pi"),
        kind: BinaryKind::Unknown,
    }
}

fn binary_size_bytes(path: &Path) -> Option<u64> {
    std::fs::metadata(path).ok().map(|m| m.len())
}

// ============================================================================
// Startup Time Benchmarks
// ============================================================================

/// Measure startup time for `pi --version` (minimal startup path)
fn bench_startup_version(c: &mut Criterion) {
    let binary = resolve_pi_binary();
    // Pre-flight check: verify the binary is actually runnable (handles
    // both missing file AND "pi" not in PATH).
    if Command::new(&binary.path)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_err()
    {
        eprintln!(
            "[skip] bench_startup_version: binary not runnable at {}",
            binary.path.display()
        );
        return;
    }

    {
        let mut group = c.benchmark_group("startup");

        // Warm the filesystem cache
        for _ in 0..3 {
            let _ = Command::new(&binary.path)
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }

        group.bench_function(BenchmarkId::new("version", "warm"), |b| {
            b.iter(|| {
                let start = Instant::now();
                let status = Command::new(&binary.path)
                    .arg("--version")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .expect("failed to execute pi");
                let elapsed = start.elapsed();
                assert!(status.success(), "pi --version failed");
                black_box(elapsed)
            });
        });

        group.finish();
    }

    // Log binary size for reference
    if let Some(size) = binary_size_bytes(&binary.path) {
        let size_mb = size as f64 / 1024.0 / 1024.0;
        eprintln!(
            "[info] binary_size={size_mb:.2}MB path={}",
            binary.path.display()
        );
    }
}

/// Measure startup time for `pi --help` (loads more code paths)
fn bench_startup_help(c: &mut Criterion) {
    let binary = resolve_pi_binary();
    if Command::new(&binary.path)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_err()
    {
        eprintln!(
            "[skip] bench_startup_help: binary not runnable at {}",
            binary.path.display()
        );
        return;
    }

    {
        let mut group = c.benchmark_group("startup");

        // Warm the filesystem cache
        for _ in 0..3 {
            let _ = Command::new(&binary.path)
                .arg("--help")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }

        group.bench_function(BenchmarkId::new("help", "warm"), |b| {
            b.iter(|| {
                let start = Instant::now();
                let status = Command::new(&binary.path)
                    .arg("--help")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .expect("failed to execute pi");
                let elapsed = start.elapsed();
                assert!(status.success(), "pi --help failed");
                black_box(elapsed)
            });
        });

        group.finish();
    }
}

/// Measure startup time for `pi --list-models` (exercises provider listing)
fn bench_startup_list_models(c: &mut Criterion) {
    let binary = resolve_pi_binary();
    if Command::new(&binary.path)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_err()
    {
        eprintln!(
            "[skip] bench_startup_list_models: binary not runnable at {}",
            binary.path.display()
        );
        return;
    }

    {
        let mut group = c.benchmark_group("startup");

        // Warm the filesystem cache
        for _ in 0..3 {
            let _ = Command::new(&binary.path)
                .arg("--list-models")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }

        group.bench_function(BenchmarkId::new("list_models", "warm"), |b| {
            b.iter(|| {
                let start = Instant::now();
                let status = Command::new(&binary.path)
                    .arg("--list-models")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .expect("failed to execute pi");
                let elapsed = start.elapsed();
                // list-models may fail without API key, just measure time
                black_box((elapsed, status))
            });
        });

        group.finish();
    }
}

// ============================================================================
// Memory Benchmarks
// ============================================================================

/// Measure RSS memory for `pi --version` (process exits immediately)
fn bench_memory_version(c: &mut Criterion) {
    let binary = resolve_pi_binary();
    if Command::new(&binary.path)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_err()
    {
        eprintln!(
            "[skip] bench_memory_version: binary not runnable at {}",
            binary.path.display()
        );
        return;
    }

    let mut group = c.benchmark_group("memory");

    group.bench_function(BenchmarkId::new("version_peak", "spawn"), |b| {
        b.iter(|| {
            // Spawn process and immediately query its memory
            let mut child = Command::new(&binary.path)
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .expect("failed to spawn pi");

            let pid = sysinfo::Pid::from_u32(child.id());
            let mut system = System::new_with_specifics(
                RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing().with_memory()),
            );
            system.refresh_processes_specifics(
                sysinfo::ProcessesToUpdate::Some(&[pid]),
                true,
                ProcessRefreshKind::nothing().with_memory(),
            );

            let memory_kb = system.process(pid).map_or(0, |p| p.memory() / 1024);

            // Wait for completion
            let _ = child.wait();

            black_box(memory_kb)
        });
    });

    group.finish();
}

// ============================================================================
// Binary Size Benchmark
// ============================================================================

/// Report binary size (not a timing benchmark, just records the value)
fn bench_binary_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("binary");
    let binary = resolve_pi_binary();

    if let Some(size) = binary_size_bytes(&binary.path) {
        let size_mb = size as f64 / 1024.0 / 1024.0;
        eprintln!(
            "[metric] binary_size_mb={size_mb:.2} path={} kind={:?}",
            binary.path.display(),
            binary.kind
        );

        if binary.kind == BinaryKind::Release {
            // Check release binary against budget.
            let budget_mb = 20.0;
            if size_mb > budget_mb {
                eprintln!("[WARN] binary size {size_mb:.2}MB exceeds budget {budget_mb:.2}MB");
            } else {
                eprintln!("[OK] binary size {size_mb:.2}MB within budget {budget_mb:.2}MB");
            }
        } else {
            eprintln!(
                "[info] skipping release-size budget check for non-release binary ({:?})",
                binary.kind
            );
        }

        // "Benchmark" that just records the size for criterion tracking
        group.bench_function("size_mb", |b| {
            b.iter(|| black_box(size_mb));
        });
    } else {
        eprintln!("[skip] bench_binary_size: could not read binary");
    }

    group.finish();
}

// ============================================================================
// Criterion Groups
// ============================================================================

criterion_group!(
    name = benches;
    config = criterion_config();
    targets =
        bench_startup_version,
        bench_startup_help,
        bench_startup_list_models,
        bench_memory_version,
        bench_binary_size
);
criterion_main!(benches);
