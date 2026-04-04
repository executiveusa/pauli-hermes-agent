#![forbid(unsafe_code)]

use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, SecondsFormat, Utc};
use clap::Parser;
use pi::extension_popularity::{
    CandidateItem, CandidatePool, CandidateSource, GitHubRepoCandidate, GitHubRepoMetrics,
    GitHubRepoRef, NpmDownloads, NpmRegistryMeta, github_repo_candidate_from_url,
    github_repo_guesses_from_slug, parse_github_repo_response, parse_npm_downloads_response,
    parse_npm_registry_response,
};
use pi::http::client::Client;
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(name = "ext_popularity_snapshot")]
#[command(about = "Snapshot popularity evidence onto extension-candidate-pool.json")]
struct Args {
    /// Candidate pool JSON input path.
    #[arg(long, default_value = "docs/extension-candidate-pool.json")]
    input: PathBuf,
    /// Output path (defaults to in-place update of --input).
    #[arg(long)]
    out: Option<PathBuf>,
    /// Optional JSONL audit log output path.
    #[arg(long)]
    log_jsonl: Option<PathBuf>,
    /// RFC3339 timestamp to stamp into `popularity.snapshot_at` (defaults to now UTC).
    #[arg(long)]
    snapshot_at: Option<String>,
    /// Environment variable name that contains a GitHub token.
    #[arg(long, default_value = "GITHUB_TOKEN")]
    github_token_env: String,
    /// Process only a specific candidate ID (repeatable).
    #[arg(long = "id")]
    ids: Vec<String>,
    /// Limit processed candidates after filtering (0 = no limit).
    #[arg(long, default_value_t = 0)]
    max_candidates: usize,
    /// Do not write output, only print summary.
    #[arg(long, default_value_t = false)]
    dry_run: bool,
}

#[derive(Debug, Default)]
struct SnapshotStats {
    total_candidates: usize,
    processed_candidates: usize,
    candidates_with_any_signal_before: usize,
    candidates_with_any_signal_after: usize,
    github_signal_updates: usize,
    npm_signal_updates: usize,
    npm_candidates_seen: usize,
    npm_fetch_errors: usize,
    github_repos_seen: usize,
    github_lookups_attempted: usize,
    github_fetch_errors: usize,
}

#[derive(Debug, Clone)]
struct NpmSnapshot {
    downloads: Option<NpmDownloads>,
    registry: Option<NpmRegistryMeta>,
}

#[derive(Debug, Clone)]
struct GitHubLookup {
    metrics: Option<GitHubRepoMetrics>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
struct AuditEvent<'a> {
    ts: String,
    event: &'a str,
    subject: String,
    elapsed_ms: u128,
    ok: bool,
    details: serde_json::Value,
}

struct JsonlLogger {
    file: File,
}

impl JsonlLogger {
    fn open(path: &Path) -> Result<Self> {
        let file = File::create(path).with_context(|| format!("create {}", path.display()))?;
        Ok(Self { file })
    }

    fn write<T: Serialize>(&mut self, value: &T) -> Result<()> {
        serde_json::to_writer(&mut self.file, value)?;
        self.file.write_all(b"\n")?;
        self.file.flush()?;
        Ok(())
    }
}

fn main() {
    if let Err(err) = main_impl() {
        eprintln!("{err:?}");
        std::process::exit(1);
    }
}

fn main_impl() -> Result<()> {
    let args = Args::parse();
    run(args)
}

fn run(args: Args) -> Result<()> {
    let input_text = fs::read_to_string(&args.input)
        .with_context(|| format!("read {}", args.input.display()))?;
    let mut pool: CandidatePool =
        serde_json::from_str(&input_text).context("parse candidate pool json")?;

    let snapshot_at = parse_snapshot_at(args.snapshot_at)?;
    let snapshot_at_string = snapshot_at.to_rfc3339_opts(SecondsFormat::Secs, true);

    let mut logger = args
        .log_jsonl
        .as_deref()
        .map(JsonlLogger::open)
        .transpose()?;
    let client = Client::new();

    let selected_ids = build_selected_id_set(&args.ids);
    let process_all = selected_ids.is_empty();

    let mut stats = SnapshotStats {
        total_candidates: pool.items.len(),
        ..Default::default()
    };

    let npm_packages = collect_npm_packages(
        &pool.items,
        &selected_ids,
        process_all,
        args.max_candidates,
        &mut stats,
    );
    let npm_map = snapshot_npm_packages(&client, &npm_packages, &mut logger, &mut stats);

    let github_token = resolve_github_token(&args.github_token_env);

    let github_refs = collect_github_refs(
        &pool.items,
        &npm_map,
        &selected_ids,
        process_all,
        args.max_candidates,
        &mut stats,
    );
    let github_map = snapshot_github_repos(
        &client,
        github_token.as_deref(),
        &github_refs,
        &mut logger,
        &mut stats,
    );

    let mut processed = 0usize;
    for item in &mut pool.items {
        if !should_process(
            item,
            &selected_ids,
            process_all,
            args.max_candidates,
            processed,
        ) {
            continue;
        }
        processed = processed.saturating_add(1);
        stats.processed_candidates = stats.processed_candidates.saturating_add(1);

        if has_any_signal(&item.popularity) {
            stats.candidates_with_any_signal_before =
                stats.candidates_with_any_signal_before.saturating_add(1);
        }

        let mut updated = item.popularity.clone();
        updated.snapshot_at = Some(snapshot_at_string.clone());

        if apply_npm_signals(&mut updated, item, &npm_map) {
            stats.npm_signal_updates = stats.npm_signal_updates.saturating_add(1);
        }

        if apply_github_signals(&mut updated, item, &npm_map, &github_map) {
            stats.github_signal_updates = stats.github_signal_updates.saturating_add(1);
        }

        if has_any_signal(&updated) {
            stats.candidates_with_any_signal_after =
                stats.candidates_with_any_signal_after.saturating_add(1);
        }

        item.popularity = updated;
    }

    let out_path = args.out.as_ref().unwrap_or(&args.input);
    let output_json = serde_json::to_string_pretty(&pool)?;
    if !args.dry_run {
        fs::write(out_path, format!("{output_json}\n"))
            .with_context(|| format!("write {}", out_path.display()))?;
    }

    print_summary(
        &stats,
        !args.dry_run,
        out_path,
        github_token.is_some(),
        &args.github_token_env,
    );
    Ok(())
}

fn parse_snapshot_at(raw: Option<String>) -> Result<DateTime<Utc>> {
    let Some(raw) = raw else {
        return Ok(Utc::now());
    };
    let parsed = DateTime::parse_from_rfc3339(raw.trim())
        .with_context(|| format!("parse --snapshot-at value '{raw}'"))?;
    Ok(parsed.with_timezone(&Utc))
}

fn resolve_github_token(env_name: &str) -> Option<String> {
    if let Ok(token) = std::env::var(env_name) {
        let trimmed = token.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    let output = Command::new("gh")
        .args(["auth", "token"])
        .stdin(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

fn build_selected_id_set(ids: &[String]) -> HashSet<String> {
    ids.iter()
        .map(|id| id.trim().to_string())
        .filter(|id| !id.is_empty())
        .collect()
}

fn should_process(
    item: &CandidateItem,
    selected_ids: &HashSet<String>,
    process_all: bool,
    max_candidates: usize,
    processed_so_far: usize,
) -> bool {
    if max_candidates > 0 && processed_so_far >= max_candidates {
        return false;
    }
    process_all || selected_ids.contains(&item.id)
}

fn collect_npm_packages(
    items: &[CandidateItem],
    selected_ids: &HashSet<String>,
    process_all: bool,
    max_candidates: usize,
    stats: &mut SnapshotStats,
) -> Vec<String> {
    let mut seen = HashSet::<String>::new();
    let mut out = Vec::new();
    let mut processed = 0usize;

    for item in items {
        if !should_process(item, selected_ids, process_all, max_candidates, processed) {
            continue;
        }
        processed = processed.saturating_add(1);

        if let CandidateSource::Npm { package, .. } = &item.source {
            stats.npm_candidates_seen = stats.npm_candidates_seen.saturating_add(1);
            if seen.insert(package.clone()) {
                out.push(package.clone());
            }
        }
    }

    out
}

fn collect_github_refs(
    items: &[CandidateItem],
    npm_map: &HashMap<String, NpmSnapshot>,
    selected_ids: &HashSet<String>,
    process_all: bool,
    max_candidates: usize,
    stats: &mut SnapshotStats,
) -> Vec<GitHubRepoRef> {
    let mut seen = HashSet::<GitHubRepoRef>::new();
    let mut out = Vec::new();
    let mut processed = 0usize;

    for item in items {
        if !should_process(item, selected_ids, process_all, max_candidates, processed) {
            continue;
        }
        processed = processed.saturating_add(1);

        for repo in github_refs_for_item(item, npm_map) {
            if seen.insert(repo.clone()) {
                out.push(repo);
            }
        }
    }

    stats.github_repos_seen = out.len();
    out
}

fn github_refs_for_item(
    item: &CandidateItem,
    npm_map: &HashMap<String, NpmSnapshot>,
) -> Vec<GitHubRepoRef> {
    let mut out = Vec::new();
    let mut seen = HashSet::<GitHubRepoRef>::new();
    let mut push_candidate = |candidate: GitHubRepoCandidate| match candidate {
        GitHubRepoCandidate::Repo(repo) => {
            if seen.insert(repo.clone()) {
                out.push(repo);
            }
        }
        GitHubRepoCandidate::Slug(slug) => {
            for guess in github_repo_guesses_from_slug(&slug) {
                if seen.insert(guess.clone()) {
                    out.push(guess);
                }
            }
        }
    };

    match &item.source {
        CandidateSource::Git { repo, .. } => {
            if let Some(candidate) = github_repo_candidate_from_url(repo) {
                push_candidate(candidate);
            }
        }
        CandidateSource::Npm { package, .. } => {
            if let Some(snapshot) = npm_map.get(package)
                && let Some(meta) = &snapshot.registry
                && let Some(repo_url) = &meta.repository_url
                && let Some(candidate) = github_repo_candidate_from_url(repo_url)
            {
                push_candidate(candidate);
            }
        }
        CandidateSource::Url { url } => {
            if let Some(candidate) = github_repo_candidate_from_url(url) {
                push_candidate(candidate);
            }
        }
    }

    if let Some(repo_url) = &item.repository_url
        && let Some(candidate) = github_repo_candidate_from_url(repo_url)
    {
        push_candidate(candidate);
    }

    out
}

fn snapshot_npm_packages(
    _client: &Client,
    packages: &[String],
    logger: &mut Option<JsonlLogger>,
    stats: &mut SnapshotStats,
) -> HashMap<String, NpmSnapshot> {
    let mut out = HashMap::<String, NpmSnapshot>::new();

    for package in packages {
        let mut snapshot = NpmSnapshot {
            downloads: None,
            registry: None,
        };

        let downloads_start = Instant::now();
        match fetch_npm_downloads_via_curl(package) {
            Ok(downloads) => {
                snapshot.downloads = Some(downloads.clone());
                let _ = log_event(
                    logger,
                    "npm_downloads",
                    package,
                    downloads_start.elapsed().as_millis(),
                    true,
                    serde_json::json!({
                        "weekly": downloads.weekly,
                        "monthly": downloads.monthly,
                    }),
                );
            }
            Err(err) => {
                stats.npm_fetch_errors = stats.npm_fetch_errors.saturating_add(1);
                let _ = log_event(
                    logger,
                    "npm_downloads",
                    package,
                    downloads_start.elapsed().as_millis(),
                    false,
                    serde_json::json!({ "error": err.to_string() }),
                );
            }
        }

        let registry_start = Instant::now();
        match fetch_npm_registry_meta_via_curl(package) {
            Ok(meta) => {
                let found = meta.is_some();
                snapshot.registry = meta;
                let _ = log_event(
                    logger,
                    "npm_registry",
                    package,
                    registry_start.elapsed().as_millis(),
                    true,
                    serde_json::json!({
                        "found": found,
                    }),
                );
            }
            Err(err) => {
                stats.npm_fetch_errors = stats.npm_fetch_errors.saturating_add(1);
                let _ = log_event(
                    logger,
                    "npm_registry",
                    package,
                    registry_start.elapsed().as_millis(),
                    false,
                    serde_json::json!({ "error": err.to_string() }),
                );
            }
        }

        out.insert(package.clone(), snapshot);
    }

    out
}

fn fetch_npm_downloads_via_curl(package: &str) -> Result<NpmDownloads> {
    fn fetch_range(package: &str, range: &str) -> Result<Option<u64>> {
        let encoded = url::form_urlencoded::byte_serialize(package.as_bytes()).collect::<String>();
        let url = format!("https://api.npmjs.org/downloads/point/{range}/{encoded}");
        let (_status, body) = fetch_url_via_curl(&url)?;
        parse_npm_downloads_response(&body).map_err(|err| anyhow!(err.to_string()))
    }

    let weekly = fetch_range(package, "last-week")?;
    let monthly = fetch_range(package, "last-month")?;
    Ok(NpmDownloads { weekly, monthly })
}

fn fetch_npm_registry_meta_via_curl(package: &str) -> Result<Option<NpmRegistryMeta>> {
    let encoded = url::form_urlencoded::byte_serialize(package.as_bytes()).collect::<String>();
    let url = format!("https://registry.npmjs.org/{encoded}");
    let (status, body) = fetch_url_via_curl(&url)?;

    match status {
        200 => parse_npm_registry_response(&body)
            .map(Some)
            .map_err(|err| anyhow!(err.to_string())),
        404 => Ok(None),
        other => Err(anyhow!("npm registry error {other}: {body}")),
    }
}

fn fetch_url_via_curl(url: &str) -> Result<(u16, String)> {
    let output = Command::new("curl")
        .stdin(Stdio::null())
        .arg("--silent")
        .arg("--show-error")
        .arg("--location")
        .arg("--max-time")
        .arg("20")
        .arg("--write-out")
        .arg("\n%{http_code}")
        .arg(url)
        .output()
        .with_context(|| format!("run curl for {url}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("curl failed for {url}: {}", stderr.trim()));
    }

    let stdout = String::from_utf8(output.stdout)
        .with_context(|| format!("decode curl output for {url}"))?;
    let Some((body, status_raw)) = stdout.rsplit_once('\n') else {
        return Err(anyhow!("curl output missing status line for {url}"));
    };
    let status = status_raw
        .trim()
        .parse::<u16>()
        .with_context(|| format!("parse HTTP status '{status_raw}' for {url}"))?;
    Ok((status, body.to_string()))
}

fn snapshot_github_repos(
    _client: &Client,
    github_token: Option<&str>,
    repos: &[GitHubRepoRef],
    logger: &mut Option<JsonlLogger>,
    stats: &mut SnapshotStats,
) -> HashMap<String, GitHubLookup> {
    let mut out = HashMap::<String, GitHubLookup>::new();

    let Some(token) = github_token else {
        return out;
    };

    for repo in repos {
        stats.github_lookups_attempted = stats.github_lookups_attempted.saturating_add(1);
        let start = Instant::now();
        let key = repo.full_name();
        match fetch_github_repo_metrics_via_gh(token, repo) {
            Ok(metrics) => {
                let found = metrics.is_some();
                out.insert(key.clone(), GitHubLookup { metrics });
                let _ = log_event(
                    logger,
                    "github_repo",
                    &key,
                    start.elapsed().as_millis(),
                    true,
                    serde_json::json!({ "found": found }),
                );
            }
            Err(err) => {
                stats.github_fetch_errors = stats.github_fetch_errors.saturating_add(1);
                let _ = log_event(
                    logger,
                    "github_repo",
                    &key,
                    start.elapsed().as_millis(),
                    false,
                    serde_json::json!({ "error": err.to_string() }),
                );
            }
        }
    }

    out
}

fn fetch_github_repo_metrics_via_gh(
    token: &str,
    repo: &GitHubRepoRef,
) -> Result<Option<GitHubRepoMetrics>> {
    let endpoint = format!("repos/{}/{}", repo.owner, repo.repo);
    let output = Command::new("gh")
        .stdin(Stdio::null())
        .arg("api")
        .arg("--method")
        .arg("GET")
        .arg("-H")
        .arg("Accept: application/vnd.github+json")
        .arg("-H")
        .arg("X-GitHub-Api-Version: 2022-11-28")
        .arg("-H")
        .arg(format!("Authorization: Bearer {token}"))
        .arg(&endpoint)
        .output()
        .with_context(|| format!("run `gh api` for {endpoint}"))?;

    if output.status.success() {
        let body = String::from_utf8(output.stdout)
            .with_context(|| format!("decode `gh api` output for {endpoint}"))?;
        return Ok(Some(
            parse_github_repo_response(&body).map_err(|err| anyhow!(err.to_string()))?,
        ));
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stderr}\n{stdout}");
    if combined.contains("HTTP 404") || combined.contains("\"message\": \"Not Found\"") {
        return Ok(None);
    }

    Err(anyhow!("`gh api {endpoint}` failed: {}", combined.trim()))
}

fn apply_npm_signals(
    target: &mut pi::extension_popularity::PopularityEvidence,
    item: &CandidateItem,
    npm_map: &HashMap<String, NpmSnapshot>,
) -> bool {
    let CandidateSource::Npm { package, .. } = &item.source else {
        return false;
    };
    let Some(snapshot) = npm_map.get(package) else {
        return false;
    };

    let updated_downloads = if let Some(downloads) = &snapshot.downloads {
        target.npm_downloads_weekly = downloads.weekly;
        target.npm_downloads_monthly = downloads.monthly;
        true
    } else {
        false
    };
    let updated_registry = if let Some(meta) = &snapshot.registry {
        target.npm_last_publish.clone_from(&meta.last_publish);
        true
    } else {
        false
    };

    updated_downloads || updated_registry
}

fn apply_github_signals(
    target: &mut pi::extension_popularity::PopularityEvidence,
    item: &CandidateItem,
    npm_map: &HashMap<String, NpmSnapshot>,
    github_map: &HashMap<String, GitHubLookup>,
) -> bool {
    for repo in github_refs_for_item(item, npm_map) {
        let key = repo.full_name();
        let Some(lookup) = github_map.get(&key) else {
            continue;
        };

        target.github_repo = Some(key);
        if let Some(metrics) = &lookup.metrics {
            target.github_stars = Some(metrics.stars);
            target.github_forks = Some(metrics.forks);
            target.github_watchers = metrics.watchers;
            target.github_open_issues = Some(metrics.open_issues);
            target.github_last_commit.clone_from(&metrics.pushed_at);
        } else {
            target.github_stars = None;
            target.github_forks = None;
            target.github_watchers = None;
            target.github_open_issues = None;
            target.github_last_commit = None;
        }
        return true;
    }
    false
}

fn has_any_signal(evidence: &pi::extension_popularity::PopularityEvidence) -> bool {
    evidence.github_repo.is_some()
        || evidence.github_stars.is_some()
        || evidence.github_forks.is_some()
        || evidence.github_watchers.is_some()
        || evidence.github_open_issues.is_some()
        || evidence.github_last_commit.is_some()
        || evidence.npm_downloads_weekly.is_some()
        || evidence.npm_downloads_monthly.is_some()
        || evidence.npm_last_publish.is_some()
        || evidence.npm_dependents.is_some()
        || evidence.marketplace_rank.is_some()
        || evidence.marketplace_installs_monthly.is_some()
        || evidence.marketplace_featured.is_some()
        || evidence.mentions_count.is_some()
        || evidence
            .mentions_sources
            .as_ref()
            .is_some_and(|sources| !sources.is_empty())
}

fn log_event(
    logger: &mut Option<JsonlLogger>,
    event: &'static str,
    subject: &str,
    elapsed_ms: u128,
    ok: bool,
    details: serde_json::Value,
) -> Result<()> {
    let Some(logger) = logger.as_mut() else {
        return Ok(());
    };
    logger.write(&AuditEvent {
        ts: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        event,
        subject: subject.to_string(),
        elapsed_ms,
        ok,
        details,
    })
}

fn print_summary(
    stats: &SnapshotStats,
    wrote_output: bool,
    output_path: &Path,
    github_enabled: bool,
    github_env_name: &str,
) {
    let before_pct = ratio_pct(
        stats.candidates_with_any_signal_before,
        stats.processed_candidates,
    );
    let after_pct = ratio_pct(
        stats.candidates_with_any_signal_after,
        stats.processed_candidates,
    );

    println!("Popularity snapshot summary");
    println!("  total candidates: {}", stats.total_candidates);
    println!("  processed candidates: {}", stats.processed_candidates);
    println!(
        "  candidates w/ >=1 signal: {} ({before_pct:.1}%) -> {} ({after_pct:.1}%)",
        stats.candidates_with_any_signal_before, stats.candidates_with_any_signal_after
    );
    println!(
        "  npm updates: {} (npm candidates: {}, npm fetch errors: {})",
        stats.npm_signal_updates, stats.npm_candidates_seen, stats.npm_fetch_errors
    );
    println!(
        "  github updates: {} (repos seen: {}, lookups attempted: {}, errors: {})",
        stats.github_signal_updates,
        stats.github_repos_seen,
        stats.github_lookups_attempted,
        stats.github_fetch_errors
    );
    if wrote_output {
        println!("  wrote: {}", output_path.display());
    } else {
        println!("  dry-run: no file written");
    }
    if !github_enabled {
        println!(
            "  github lookup skipped (set env var {github_env_name} to enable full GitHub snapshots)"
        );
    }
}

#[allow(clippy::cast_precision_loss)]
fn ratio_pct(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        return 0.0;
    }
    (numerator as f64 / denominator as f64) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use pi::extension_popularity::{PopularityEvidence, Sha256Checksum};

    fn mk_item(
        id: &str,
        source: CandidateSource,
        repository_url: Option<&str>,
        popularity: PopularityEvidence,
    ) -> CandidateItem {
        CandidateItem {
            id: id.to_string(),
            name: id.to_string(),
            source_tier: "test".to_string(),
            status: "active".to_string(),
            license: "MIT".to_string(),
            retrieved: Some("2026-02-06T00:00:00Z".to_string()),
            artifact_path: Some("artifacts/test".to_string()),
            checksum: Some(Sha256Checksum {
                sha256: "abc".to_string(),
            }),
            source,
            repository_url: repository_url.map(ToString::to_string),
            popularity,
            aliases: Vec::new(),
            notes: None,
        }
    }

    #[test]
    fn github_refs_include_source_and_repository_url() {
        let item = mk_item(
            "x",
            CandidateSource::Git {
                repo: "https://github.com/owner/repo.git".to_string(),
                path: None,
            },
            Some("https://github.com/owner/backup"),
            PopularityEvidence::default(),
        );
        let refs = github_refs_for_item(&item, &HashMap::new());
        let names = refs
            .iter()
            .map(GitHubRepoRef::full_name)
            .collect::<HashSet<_>>();
        assert!(names.contains("owner/repo"));
        assert!(names.contains("owner/backup"));
    }

    #[test]
    fn github_refs_include_slug_guesses_from_repo_url() {
        let item = mk_item(
            "x",
            CandidateSource::Url {
                url: "https://example.com/pkg.tgz".to_string(),
            },
            Some("https://github.com/owner-pi-foo"),
            PopularityEvidence::default(),
        );
        let refs = github_refs_for_item(&item, &HashMap::new());
        let names = refs
            .iter()
            .map(GitHubRepoRef::full_name)
            .collect::<HashSet<_>>();
        assert!(names.contains("owner/pi-foo"));
    }

    #[test]
    fn has_any_signal_detects_empty_and_non_empty_mentions() {
        let mut evidence = PopularityEvidence::default();
        assert!(!has_any_signal(&evidence));
        evidence.mentions_sources = Some(Vec::new());
        assert!(!has_any_signal(&evidence));
        evidence.mentions_sources = Some(vec!["https://example.com".to_string()]);
        assert!(has_any_signal(&evidence));
    }

    #[test]
    fn apply_npm_signals_updates_fields() {
        let item = mk_item(
            "pkg",
            CandidateSource::Npm {
                package: "@scope/pkg".to_string(),
                version: "1.0.0".to_string(),
                url: "https://registry.npmjs.org/@scope/pkg/-/pkg-1.0.0.tgz".to_string(),
            },
            None,
            PopularityEvidence::default(),
        );
        let mut evidence = PopularityEvidence::default();
        let mut map = HashMap::new();
        map.insert(
            "@scope/pkg".to_string(),
            NpmSnapshot {
                downloads: Some(NpmDownloads {
                    weekly: Some(123),
                    monthly: Some(456),
                }),
                registry: Some(NpmRegistryMeta {
                    latest_version: Some("1.0.0".to_string()),
                    last_publish: Some("2026-01-01T00:00:00Z".to_string()),
                    repository_url: Some("https://github.com/owner/repo".to_string()),
                }),
            },
        );
        let changed = apply_npm_signals(&mut evidence, &item, &map);
        assert!(changed);
        assert_eq!(evidence.npm_downloads_weekly, Some(123));
        assert_eq!(evidence.npm_downloads_monthly, Some(456));
        assert_eq!(
            evidence.npm_last_publish,
            Some("2026-01-01T00:00:00Z".to_string())
        );
    }

    #[test]
    fn apply_github_signals_prefers_resolved_repo() {
        let item = mk_item(
            "x",
            CandidateSource::Git {
                repo: "https://github.com/owner/repo".to_string(),
                path: None,
            },
            None,
            PopularityEvidence::default(),
        );
        let mut evidence = PopularityEvidence::default();
        let mut map = HashMap::new();
        map.insert(
            "owner/repo".to_string(),
            GitHubLookup {
                metrics: Some(GitHubRepoMetrics {
                    full_name: "owner/repo".to_string(),
                    stars: 10,
                    forks: 2,
                    watchers: Some(1),
                    open_issues: 3,
                    pushed_at: Some("2026-01-31T00:00:00Z".to_string()),
                }),
            },
        );
        let changed = apply_github_signals(&mut evidence, &item, &HashMap::new(), &map);
        assert!(changed);
        assert_eq!(evidence.github_repo, Some("owner/repo".to_string()));
        assert_eq!(evidence.github_stars, Some(10));
        assert_eq!(evidence.github_forks, Some(2));
        assert_eq!(evidence.github_watchers, Some(1));
        assert_eq!(evidence.github_open_issues, Some(3));
    }
}
