#![forbid(unsafe_code)]
#![cfg_attr(test, allow(unused_variables, clippy::uninlined_format_args))]

use std::fmt::Write as _;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use chrono::{SecondsFormat, Utc};
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
#[command(name = "ext_release_binary_e2e")]
#[command(about = "Run release-binary extension E2E checks against a live provider")]
struct Args {
    /// Validated extension manifest (contains `entry_path` for each extension).
    #[arg(long, default_value = "tests/ext_conformance/VALIDATED_MANIFEST.json")]
    manifest: PathBuf,

    /// Root directory that contains extension artifacts referenced by `entry_path`.
    #[arg(long, default_value = "tests/ext_conformance/artifacts")]
    artifacts_root: PathBuf,

    /// Release binary under test.
    #[arg(long, default_value = "target/release/pi")]
    pi_bin: PathBuf,

    /// Provider ID passed to pi.
    #[arg(long, default_value = "ollama")]
    provider: String,

    /// Model ID passed to pi.
    #[arg(long, default_value = "qwen2.5:0.5b")]
    model: String,

    /// Optional API key passed to pi.
    ///
    /// If omitted, the harness relies on provider-native auth (for example OAuth).
    /// Ollama still gets a synthetic key automatically for compatibility.
    #[arg(long)]
    api_key: Option<String>,

    /// Prompt used for each extension run.
    #[arg(long, default_value = "Respond with exactly: ok")]
    prompt: String,

    /// Per-extension timeout in seconds.
    #[arg(long, default_value_t = 90)]
    timeout_secs: u64,

    /// Number of extension cases to execute concurrently.
    #[arg(long, default_value_t = 1)]
    jobs: usize,

    /// Set `PI_EXTENSION_ALLOW_DANGEROUS=1` for each invoked pi process.
    #[arg(long, default_value_t = false)]
    allow_dangerous: bool,

    /// Optional extension policy override (safe, balanced, permissive).
    #[arg(long)]
    extension_policy: Option<String>,

    /// Shard index (0-based), used with --shard-total.
    #[arg(long)]
    shard_index: Option<usize>,

    /// Number of shards for partitioned runs.
    #[arg(long)]
    shard_total: Option<usize>,

    /// Optional cap on the number of selected extensions.
    #[arg(long)]
    max_cases: Option<usize>,

    /// Output JSON report path.
    #[arg(
        long,
        default_value = "tests/ext_conformance/reports/release_binary_e2e/ollama_release_e2e.json"
    )]
    out_json: PathBuf,

    /// Output Markdown report path.
    #[arg(
        long,
        default_value = "tests/ext_conformance/reports/release_binary_e2e/ollama_release_e2e.md"
    )]
    out_md: PathBuf,
}

#[derive(Debug, Deserialize)]
struct Manifest {
    extensions: Vec<ManifestEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct ManifestEntry {
    id: String,
    entry_path: String,
    source_tier: String,
    conformance_tier: u32,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum CaseStatus {
    Pass,
    MissingExtension,
    Timeout,
    ProcessError,
    EmptyOutput,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CaseResult {
    id: String,
    entry_path: String,
    source_tier: String,
    conformance_tier: u32,
    extension_path: String,
    status: CaseStatus,
    pass: bool,
    duration_ms: u64,
    exit_code: Option<i32>,
    stdout_path: String,
    stderr_path: String,
    stdout_preview: String,
    stderr_preview: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
struct Counts {
    total: usize,
    pass: usize,
    fail: usize,
    timeout: usize,
    missing_extension: usize,
    process_error: usize,
    empty_output: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Report {
    schema: String,
    generated_at: String,
    run_id: String,
    correlation_id: String,
    provider: String,
    model: String,
    prompt: String,
    timeout_secs: u64,
    jobs: usize,
    allow_dangerous: bool,
    pi_bin: String,
    manifest: String,
    artifacts_root: String,
    shard_index: Option<usize>,
    shard_total: Option<usize>,
    max_cases: Option<usize>,
    extension_policy: Option<String>,
    counts: Counts,
    results: Vec<CaseResult>,
}

#[derive(Debug)]
struct RunOutput {
    exit_code: Option<i32>,
    timed_out: bool,
    duration_ms: u64,
}

const WRITE_ZERO_RETRY_ATTEMPTS: usize = 1;

fn main() {
    if let Err(err) = run() {
        eprintln!("{err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();
    validate_args(&args)?;

    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest_path = absolutize(&project_root, &args.manifest);
    let artifacts_root = absolutize(&project_root, &args.artifacts_root);
    let pi_bin = absolutize(&project_root, &args.pi_bin);
    let out_json = absolutize(&project_root, &args.out_json);
    let out_md = absolutize(&project_root, &args.out_md);

    if !pi_bin.exists() {
        bail!(
            "pi release binary not found at {}. Build first: cargo build --release --bin pi",
            pi_bin.display()
        );
    }

    let manifest = load_manifest(&manifest_path)?;
    let selected = select_entries(&manifest.extensions, args.shard_index, args.shard_total);
    let mut selected = if let Some(max_cases) = args.max_cases {
        selected.into_iter().take(max_cases).collect::<Vec<_>>()
    } else {
        selected
    };

    selected.sort_by(|left, right| left.id.cmp(&right.id));

    if selected.is_empty() {
        bail!("no extensions selected after shard/max-case filters");
    }

    let generated_at = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let run_id = format!("release-e2e-{}", Utc::now().format("%Y%m%dT%H%M%SZ"));
    let correlation_id = format!("ext-release-binary-{run_id}");

    eprintln!(
        "[ext_release_binary_e2e] run_id={run_id} provider={} model={} selected={}",
        args.provider,
        args.model,
        selected.len()
    );

    let out_dir = out_json
        .parent()
        .map(Path::to_path_buf)
        .context("output JSON path must have a parent directory")?;
    fs::create_dir_all(&out_dir)
        .with_context(|| format!("creating output directory {}", out_dir.display()))?;
    let per_case_dir = out_dir.join("cases");
    fs::create_dir_all(&per_case_dir)
        .with_context(|| format!("creating per-case directory {}", per_case_dir.display()))?;

    let results = execute_cases(
        &selected,
        &args,
        &project_root,
        &artifacts_root,
        &pi_bin,
        &per_case_dir,
    )?;

    let counts = summarize_counts(&results);
    let report = Report {
        schema: "pi.ext.release_binary_e2e.v1".to_string(),
        generated_at,
        run_id,
        correlation_id,
        provider: args.provider,
        model: args.model,
        prompt: args.prompt,
        timeout_secs: args.timeout_secs,
        jobs: args.jobs.max(1),
        allow_dangerous: args.allow_dangerous,
        pi_bin: path_to_string(&pi_bin),
        manifest: path_to_string(&manifest_path),
        artifacts_root: path_to_string(&artifacts_root),
        shard_index: args.shard_index,
        shard_total: args.shard_total,
        max_cases: args.max_cases,
        extension_policy: args.extension_policy,
        counts,
        results,
    };

    write_report(&report, &out_json, &out_md)?;
    eprintln!(
        "[ext_release_binary_e2e] wrote JSON: {}",
        out_json.display()
    );
    eprintln!(
        "[ext_release_binary_e2e] wrote Markdown: {}",
        out_md.display()
    );
    eprintln!(
        "[ext_release_binary_e2e] summary: pass={} fail={} timeout={} missing={}",
        report.counts.pass,
        report.counts.fail,
        report.counts.timeout,
        report.counts.missing_extension
    );

    Ok(())
}

fn validate_args(args: &Args) -> Result<()> {
    if args.timeout_secs == 0 {
        bail!("--timeout-secs must be > 0");
    }
    if args.jobs == 0 {
        bail!("--jobs must be > 0");
    }
    match (args.shard_index, args.shard_total) {
        (Some(_), None) | (None, Some(_)) => {
            bail!("--shard-index and --shard-total must be provided together");
        }
        (Some(index), Some(total)) => {
            if total == 0 {
                bail!("--shard-total must be > 0");
            }
            if index >= total {
                bail!("--shard-index must be less than --shard-total");
            }
        }
        (None, None) => {}
    }
    Ok(())
}

fn execute_cases(
    selected: &[ManifestEntry],
    args: &Args,
    project_root: &Path,
    artifacts_root: &Path,
    pi_bin: &Path,
    per_case_dir: &Path,
) -> Result<Vec<CaseResult>> {
    let total = selected.len();
    let jobs = args.jobs.max(1).min(total);
    if jobs == 1 {
        let mut results = Vec::with_capacity(total);
        for (index, entry) in selected.iter().enumerate() {
            eprintln!("[{}/{}] {}", index + 1, total, entry.id);
            let result = run_one_case(
                entry,
                args,
                project_root,
                artifacts_root,
                pi_bin,
                per_case_dir,
            )
            .with_context(|| format!("running extension case '{}'", entry.id))?;
            eprintln!("  -> {}", case_summary(&result));
            results.push(result);
        }
        return Ok(results);
    }

    eprintln!("[ext_release_binary_e2e] parallel jobs={jobs}");

    let mut first_error: Option<anyhow::Error> = None;
    let mut completed = 0usize;
    let mut ordered = vec![None; total];
    let (tx, rx) = mpsc::channel::<Result<(usize, CaseResult)>>();
    let next_index = Arc::new(AtomicUsize::new(0));
    thread::scope(|scope| {
        for _ in 0..jobs {
            let tx = tx.clone();
            let next_index = Arc::clone(&next_index);
            scope.spawn(move || {
                loop {
                    let index = next_index.fetch_add(1, Ordering::Relaxed);
                    if index >= total {
                        break;
                    }
                    let entry = &selected[index];
                    let result = run_one_case(
                        entry,
                        args,
                        project_root,
                        artifacts_root,
                        pi_bin,
                        per_case_dir,
                    )
                    .with_context(|| format!("running extension case '{}'", entry.id))
                    .map(|case| (index, case));
                    if tx.send(result).is_err() {
                        break;
                    }
                }
            });
        }

        drop(tx);
        for message in rx {
            match message {
                Ok((index, result)) => {
                    completed += 1;
                    eprintln!(
                        "[{completed}/{total}] {} -> {}",
                        result.id,
                        case_summary(&result)
                    );
                    ordered[index] = Some(result);
                }
                Err(err) => {
                    if first_error.is_none() {
                        first_error = Some(err);
                    }
                }
            }
        }
    });

    if let Some(err) = first_error {
        return Err(err);
    }

    ordered
        .into_iter()
        .enumerate()
        .map(|(index, maybe)| maybe.with_context(|| format!("missing result for index {index}")))
        .collect()
}

fn load_manifest(path: &Path) -> Result<Manifest> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("reading manifest {}", path.display()))?;
    let manifest: Manifest =
        serde_json::from_str(&raw).with_context(|| format!("parsing {}", path.display()))?;
    Ok(manifest)
}

fn select_entries(
    entries: &[ManifestEntry],
    shard_index: Option<usize>,
    shard_total: Option<usize>,
) -> Vec<ManifestEntry> {
    match (shard_index, shard_total) {
        (Some(index), Some(total)) => entries
            .iter()
            .enumerate()
            .filter_map(|(i, entry)| {
                if i % total == index {
                    Some(entry.clone())
                } else {
                    None
                }
            })
            .collect(),
        _ => entries.to_vec(),
    }
}

fn run_one_case(
    entry: &ManifestEntry,
    args: &Args,
    project_root: &Path,
    artifacts_root: &Path,
    pi_bin: &Path,
    per_case_dir: &Path,
) -> Result<CaseResult> {
    let extension_path = artifacts_root.join(&entry.entry_path);
    let case_dir = per_case_dir.join(sanitize_id(&entry.id));
    fs::create_dir_all(&case_dir)
        .with_context(|| format!("creating case dir {}", case_dir.display()))?;

    let stdout_path = case_dir.join("stdout.txt");
    let stderr_path = case_dir.join("stderr.txt");

    if !extension_path.exists() {
        fs::write(&stdout_path, b"")?;
        fs::write(
            &stderr_path,
            format!("missing extension file: {}", extension_path.display()),
        )?;
        return Ok(CaseResult {
            id: entry.id.clone(),
            entry_path: entry.entry_path.clone(),
            source_tier: entry.source_tier.clone(),
            conformance_tier: entry.conformance_tier,
            extension_path: path_to_string(&extension_path),
            status: CaseStatus::MissingExtension,
            pass: false,
            duration_ms: 0,
            exit_code: None,
            stdout_path: path_to_string(&stdout_path),
            stderr_path: path_to_string(&stderr_path),
            stdout_preview: String::new(),
            stderr_preview: format!("missing extension file: {}", extension_path.display()),
        });
    }

    let env_root = case_dir.join("env");
    fs::create_dir_all(&env_root)?;

    let (run_output, stdout_text, stderr_text) = execute_case_command(
        pi_bin,
        project_root,
        &extension_path,
        args,
        &env_root,
        &stdout_path,
        &stderr_path,
    )?;

    let mut run_output = run_output;
    let mut stdout_text = stdout_text;
    let mut stderr_text = stderr_text;
    let mut attempt = 0usize;
    while attempt < WRITE_ZERO_RETRY_ATTEMPTS
        && !run_output.timed_out
        && run_output.exit_code != Some(0)
        && stderr_text.contains("write zero")
    {
        attempt += 1;
        thread::sleep(Duration::from_millis(150));
        let rerun = execute_case_command(
            pi_bin,
            project_root,
            &extension_path,
            args,
            &env_root,
            &stdout_path,
            &stderr_path,
        )?;
        run_output = rerun.0;
        stdout_text = rerun.1;
        stderr_text = rerun.2;
    }
    let status = if run_output.timed_out {
        CaseStatus::Timeout
    } else if run_output.exit_code != Some(0) {
        CaseStatus::ProcessError
    } else if stdout_text.trim().is_empty() {
        CaseStatus::EmptyOutput
    } else {
        CaseStatus::Pass
    };

    Ok(CaseResult {
        id: entry.id.clone(),
        entry_path: entry.entry_path.clone(),
        source_tier: entry.source_tier.clone(),
        conformance_tier: entry.conformance_tier,
        extension_path: path_to_string(&extension_path),
        status,
        pass: status == CaseStatus::Pass,
        duration_ms: run_output.duration_ms,
        exit_code: run_output.exit_code,
        stdout_path: path_to_string(&stdout_path),
        stderr_path: path_to_string(&stderr_path),
        stdout_preview: preview(&stdout_text),
        stderr_preview: preview(&stderr_text),
    })
}

fn execute_case_command(
    pi_bin: &Path,
    project_root: &Path,
    extension_path: &Path,
    args: &Args,
    env_root: &Path,
    stdout_path: &Path,
    stderr_path: &Path,
) -> Result<(RunOutput, String, String)> {
    let mut command = Command::new(pi_bin);
    let stdout_file = File::create(stdout_path)
        .with_context(|| format!("creating stdout capture file {}", stdout_path.display()))?;
    let stderr_file = File::create(stderr_path)
        .with_context(|| format!("creating stderr capture file {}", stderr_path.display()))?;
    command
        .current_dir(project_root)
        .arg("--print")
        .arg("--no-session")
        .arg("--provider")
        .arg(&args.provider)
        .arg("--model")
        .arg(&args.model)
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file));

    if let Some(api_key) = args.api_key.as_deref() {
        command.arg("--api-key").arg(api_key);
    } else if args.provider.eq_ignore_ascii_case("ollama") {
        command.arg("--api-key").arg("ollama-no-key-needed");
    }

    if let Some(policy) = &args.extension_policy {
        command.arg("--extension-policy").arg(policy);
    }
    command
        .arg("--extension")
        .arg(extension_path)
        .arg(&args.prompt);

    let home_dir = env_root.join("home");
    fs::create_dir_all(&home_dir)
        .with_context(|| format!("creating isolated HOME {}", home_dir.display()))?;
    if args.api_key.is_none() {
        seed_oauth_home(&args.provider, &home_dir)?;
    }
    command.env("HOME", &home_dir);
    command.env("PI_CODING_AGENT_DIR", env_root.join("agent"));
    command.env("PI_CONFIG_PATH", isolated_settings_path(env_root));
    command.env("PI_SESSIONS_DIR", env_root.join("sessions"));
    command.env("PI_PACKAGE_DIR", env_root.join("packages"));
    command.env("PI_TEST_MODE", "1");
    if args.allow_dangerous {
        command.env("PI_EXTENSION_ALLOW_DANGEROUS", "1");
    }

    let run_output = run_with_timeout(command, Duration::from_secs(args.timeout_secs))?;
    let stdout_bytes = fs::read(stdout_path)
        .with_context(|| format!("reading stdout capture file {}", stdout_path.display()))?;
    let stderr_bytes = fs::read(stderr_path)
        .with_context(|| format!("reading stderr capture file {}", stderr_path.display()))?;
    let stdout_text = String::from_utf8_lossy(&stdout_bytes).to_string();
    let stderr_text = String::from_utf8_lossy(&stderr_bytes).to_string();
    Ok((run_output, stdout_text, stderr_text))
}

fn run_with_timeout(mut command: Command, timeout: Duration) -> Result<RunOutput> {
    let started = Instant::now();
    let mut child = command.spawn().context("spawning pi process")?;

    loop {
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let status = child
                .wait()
                .context("waiting for timed-out process after kill")?;
            let duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
            return Ok(RunOutput {
                exit_code: status.code(),
                timed_out: true,
                duration_ms,
            });
        }

        match child.try_wait().context("polling process state")? {
            Some(status) => {
                let duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
                return Ok(RunOutput {
                    exit_code: status.code(),
                    timed_out: false,
                    duration_ms,
                });
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
}

fn seed_oauth_home(provider: &str, home_dir: &Path) -> Result<()> {
    if provider.eq_ignore_ascii_case("openai-codex") {
        let Some(host_home) = std::env::var_os("HOME") else {
            return Ok(());
        };
        let source = PathBuf::from(host_home).join(".codex/auth.json");
        if source.exists() {
            let target = home_dir.join(".codex/auth.json");
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("creating OAuth target directory {}", parent.display())
                })?;
            }
            fs::copy(&source, &target).with_context(|| {
                format!(
                    "copying openai-codex auth from {} to {}",
                    source.display(),
                    target.display()
                )
            })?;
        }
    }
    Ok(())
}

fn summarize_counts(results: &[CaseResult]) -> Counts {
    let total = results.len();
    let pass = results.iter().filter(|result| result.pass).count();
    let timeout = results
        .iter()
        .filter(|result| result.status == CaseStatus::Timeout)
        .count();
    let missing_extension = results
        .iter()
        .filter(|result| result.status == CaseStatus::MissingExtension)
        .count();
    let process_error = results
        .iter()
        .filter(|result| result.status == CaseStatus::ProcessError)
        .count();
    let empty_output = results
        .iter()
        .filter(|result| result.status == CaseStatus::EmptyOutput)
        .count();
    let fail = total.saturating_sub(pass);
    Counts {
        total,
        pass,
        fail,
        timeout,
        missing_extension,
        process_error,
        empty_output,
    }
}

fn write_report(report: &Report, out_json: &Path, out_md: &Path) -> Result<()> {
    if let Some(parent) = out_json.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    if let Some(parent) = out_md.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }

    let json =
        serde_json::to_string_pretty(report).context("serializing release binary E2E report")?;
    fs::write(out_json, format!("{json}\n"))
        .with_context(|| format!("writing {}", out_json.display()))?;

    let md = render_markdown(report);
    fs::write(out_md, md).with_context(|| format!("writing {}", out_md.display()))?;
    Ok(())
}

fn render_markdown(report: &Report) -> String {
    let mut out = String::new();
    out.push_str("# Release Binary Extension E2E (Live Provider)\n\n");
    let _ = writeln!(out, "- Generated: `{}`", report.generated_at);
    let _ = writeln!(out, "- Run ID: `{}`", report.run_id);
    let _ = writeln!(out, "- Correlation ID: `{}`", report.correlation_id);
    let _ = writeln!(
        out,
        "- Provider/model: `{}` / `{}`",
        report.provider, report.model
    );
    let _ = writeln!(out, "- Prompt: `{}`", report.prompt);
    let _ = writeln!(out, "- Timeout per extension: `{}`s", report.timeout_secs);
    let _ = writeln!(out, "- Parallel jobs: `{}`", report.jobs);
    let _ = writeln!(
        out,
        "- PI_EXTENSION_ALLOW_DANGEROUS: `{}`",
        report.allow_dangerous
    );
    let _ = writeln!(
        out,
        "- Counts: pass `{}` / total `{}` (fail `{}`)",
        report.counts.pass, report.counts.total, report.counts.fail
    );
    out.push('\n');

    out.push_str("| id | status | ms | exit | tier | source |\n");
    out.push_str("|---|---|---:|---:|---:|---|\n");
    for result in &report.results {
        let exit = result
            .exit_code
            .map_or_else(|| "-".to_string(), |code| code.to_string());
        let _ = writeln!(
            out,
            "| `{}` | `{}` | {} | {} | {} | `{}` |",
            result.id,
            status_label(result.status),
            result.duration_ms,
            exit,
            result.conformance_tier,
            result.source_tier
        );
    }

    if report.counts.fail > 0 {
        out.push_str("\n## Failures\n\n");
        for result in &report.results {
            if result.pass {
                continue;
            }
            let _ = writeln!(
                out,
                "- `{}`: `{}` (exit={:?}, stdout=`{}`, stderr=`{}`)",
                result.id,
                status_label(result.status),
                result.exit_code,
                result.stdout_preview.replace('\n', " "),
                result.stderr_preview.replace('\n', " ")
            );
        }
    }

    out
}

const fn status_label(status: CaseStatus) -> &'static str {
    match status {
        CaseStatus::Pass => "pass",
        CaseStatus::MissingExtension => "missing_extension",
        CaseStatus::Timeout => "timeout",
        CaseStatus::ProcessError => "process_error",
        CaseStatus::EmptyOutput => "empty_output",
    }
}

fn preview(text: &str) -> String {
    const MAX_CHARS: usize = 240;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let mut output = String::new();
    for (index, ch) in trimmed.chars().enumerate() {
        if index >= MAX_CHARS {
            output.push('…');
            break;
        }
        output.push(ch);
    }
    output
}

fn isolated_settings_path(env_root: &Path) -> PathBuf {
    env_root.join("settings.json")
}

fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn absolutize(project_root: &Path, value: &Path) -> PathBuf {
    if value.is_absolute() {
        value.to_path_buf()
    } else {
        project_root.join(value)
    }
}

fn path_to_string(path: &Path) -> String {
    path.display().to_string()
}

fn case_summary(result: &CaseResult) -> String {
    format!(
        "{:?} ({} ms, exit={:?})",
        result.status, result.duration_ms, result.exit_code
    )
}

#[allow(clippy::similar_names)]
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;

    #[test]
    fn select_entries_applies_shard_partition() {
        let entries = vec![
            ManifestEntry {
                id: "a".to_string(),
                entry_path: "a.ts".to_string(),
                source_tier: "x".to_string(),
                conformance_tier: 1,
            },
            ManifestEntry {
                id: "b".to_string(),
                entry_path: "b.ts".to_string(),
                source_tier: "x".to_string(),
                conformance_tier: 1,
            },
            ManifestEntry {
                id: "c".to_string(),
                entry_path: "c.ts".to_string(),
                source_tier: "x".to_string(),
                conformance_tier: 1,
            },
            ManifestEntry {
                id: "d".to_string(),
                entry_path: "d.ts".to_string(),
                source_tier: "x".to_string(),
                conformance_tier: 1,
            },
        ];

        let shard = select_entries(&entries, Some(1), Some(2));
        let ids: Vec<String> = shard.into_iter().map(|entry| entry.id).collect();
        assert_eq!(ids, vec!["b".to_string(), "d".to_string()]);
    }

    #[test]
    fn preview_truncates_long_output() {
        let input = "a".repeat(300);
        let output = preview(&input);
        assert!(output.ends_with('…'));
        assert!(output.len() < input.len());
    }

    #[test]
    fn isolated_settings_path_uses_settings_json_override() {
        let env_root = PathBuf::from("/tmp/ext-release-run");
        assert_eq!(
            isolated_settings_path(&env_root),
            env_root.join("settings.json")
        );
    }

    #[test]
    fn sanitize_id_replaces_path_separators() {
        assert_eq!(sanitize_id("npm/pi-tool"), "npm_pi-tool");
    }

    #[test]
    fn validate_args_rejects_invalid_shard_combo() {
        let args = Args {
            manifest: PathBuf::from("a"),
            artifacts_root: PathBuf::from("b"),
            pi_bin: PathBuf::from("c"),
            provider: "ollama".to_string(),
            model: "qwen2.5:0.5b".to_string(),
            api_key: Some("x".to_string()),
            prompt: "ok".to_string(),
            timeout_secs: 30,
            jobs: 1,
            allow_dangerous: false,
            extension_policy: None,
            shard_index: Some(2),
            shard_total: Some(2),
            max_cases: None,
            out_json: PathBuf::from("d"),
            out_md: PathBuf::from("e"),
        };
        let err = validate_args(&args).expect_err("invalid shard should fail");
        assert!(err.to_string().contains("--shard-index"));
    }

    #[test]
    fn validate_args_rejects_zero_jobs() {
        let args = Args {
            manifest: PathBuf::from("a"),
            artifacts_root: PathBuf::from("b"),
            pi_bin: PathBuf::from("c"),
            provider: "ollama".to_string(),
            model: "qwen2.5:0.5b".to_string(),
            api_key: Some("x".to_string()),
            prompt: "ok".to_string(),
            timeout_secs: 30,
            jobs: 0,
            allow_dangerous: false,
            extension_policy: None,
            shard_index: None,
            shard_total: None,
            max_cases: None,
            out_json: PathBuf::from("d"),
            out_md: PathBuf::from("e"),
        };
        let err = validate_args(&args).expect_err("jobs=0 should fail");
        assert!(err.to_string().contains("--jobs"));
    }

    #[test]
    fn write_report_outputs_files() {
        let dir = std::env::temp_dir().join(format!(
            "ext_release_binary_e2e_test_{}",
            std::process::id()
        ));
        match fs::remove_dir_all(&dir) {
            Ok(()) => {}
            Err(err) if err.kind() == ErrorKind::NotFound => {}
            Err(err) => panic!("Failed to remove dir {:?}: {}", dir, err),
        }
        fs::create_dir_all(&dir).expect("create temp report dir");

        let report = Report {
            schema: "pi.ext.release_binary_e2e.v1".to_string(),
            generated_at: "2026-02-18T00:00:00Z".to_string(),
            run_id: "run-1".to_string(),
            correlation_id: "corr-1".to_string(),
            provider: "ollama".to_string(),
            model: "qwen2.5:0.5b".to_string(),
            prompt: "ok".to_string(),
            timeout_secs: 30,
            jobs: 1,
            allow_dangerous: false,
            pi_bin: "target/release/pi".to_string(),
            manifest: "tests/ext_conformance/VALIDATED_MANIFEST.json".to_string(),
            artifacts_root: "tests/ext_conformance/artifacts".to_string(),
            shard_index: None,
            shard_total: None,
            max_cases: None,
            extension_policy: None,
            counts: Counts {
                total: 1,
                pass: 1,
                fail: 0,
                timeout: 0,
                missing_extension: 0,
                process_error: 0,
                empty_output: 0,
            },
            results: vec![CaseResult {
                id: "hello".to_string(),
                entry_path: "hello.ts".to_string(),
                source_tier: "official".to_string(),
                conformance_tier: 1,
                extension_path: "tests/ext_conformance/artifacts/hello.ts".to_string(),
                status: CaseStatus::Pass,
                pass: true,
                duration_ms: 1,
                exit_code: Some(0),
                stdout_path: "stdout.txt".to_string(),
                stderr_path: "stderr.txt".to_string(),
                stdout_preview: "ok".to_string(),
                stderr_preview: String::new(),
            }],
        };

        let json_path = dir.join("report.json");
        let md_path = dir.join("report.md");
        write_report(&report, &json_path, &md_path).expect("write report");
        assert!(json_path.exists());
        assert!(md_path.exists());
    }
}
