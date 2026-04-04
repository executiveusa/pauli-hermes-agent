//! Comprehensive environment health checker for `pi doctor`.
//!
//! When invoked without a path, checks config, directories, auth, shell tools,
//! and sessions. When invoked with a path, runs extension preflight analysis.
//! With `--fix`, automatically repairs safe issues (missing dirs, permissions).

use crate::auth::{AuthStorage, CredentialStatus};
use crate::config::Config;
use crate::error::Result;
use crate::provider_metadata::provider_auth_env_keys;
use crate::session::SessionHeader;
use crate::session_index::walk_sessions;
use serde::Serialize;
use std::collections::HashSet;
use std::fmt;
use std::fmt::Write as _;
use std::io::{BufRead as _, BufReader, Write as _};
use std::path::{Path, PathBuf};
use std::process::Command;

// ── Core Types ──────────────────────────────────────────────────────

/// How severe a finding is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Pass,
    Info,
    Warn,
    Fail,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pass => write!(f, "PASS"),
            Self::Info => write!(f, "INFO"),
            Self::Warn => write!(f, "WARN"),
            Self::Fail => write!(f, "FAIL"),
        }
    }
}

/// Whether a finding can be auto-fixed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Fixability {
    /// Cannot be auto-fixed.
    NotFixable,
    /// Can be auto-fixed with `--fix`.
    AutoFixable,
    /// Was auto-fixed in this run.
    Fixed,
}

/// Which subsystem a check belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckCategory {
    Config,
    Dirs,
    Auth,
    Shell,
    Sessions,
    Extensions,
}

impl CheckCategory {
    const fn label(self) -> &'static str {
        match self {
            Self::Config => "Configuration",
            Self::Dirs => "Directories",
            Self::Auth => "Authentication",
            Self::Shell => "Shell & Tools",
            Self::Sessions => "Sessions",
            Self::Extensions => "Extensions",
        }
    }
}

impl fmt::Display for CheckCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl std::str::FromStr for CheckCategory {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "config" => Ok(Self::Config),
            "dirs" | "directories" => Ok(Self::Dirs),
            "auth" | "authentication" => Ok(Self::Auth),
            "shell" => Ok(Self::Shell),
            "sessions" => Ok(Self::Sessions),
            "extensions" | "ext" => Ok(Self::Extensions),
            other => Err(format!("unknown category: {other}")),
        }
    }
}

/// A single diagnostic finding.
#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub category: CheckCategory,
    pub severity: Severity,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
    pub fixability: Fixability,
}

impl Finding {
    fn pass(category: CheckCategory, title: impl Into<String>) -> Self {
        Self {
            category,
            severity: Severity::Pass,
            title: title.into(),
            detail: None,
            remediation: None,
            fixability: Fixability::NotFixable,
        }
    }

    fn info(category: CheckCategory, title: impl Into<String>) -> Self {
        Self {
            category,
            severity: Severity::Info,
            title: title.into(),
            detail: None,
            remediation: None,
            fixability: Fixability::NotFixable,
        }
    }

    fn warn(category: CheckCategory, title: impl Into<String>) -> Self {
        Self {
            category,
            severity: Severity::Warn,
            title: title.into(),
            detail: None,
            remediation: None,
            fixability: Fixability::NotFixable,
        }
    }

    fn fail(category: CheckCategory, title: impl Into<String>) -> Self {
        Self {
            category,
            severity: Severity::Fail,
            title: title.into(),
            detail: None,
            remediation: None,
            fixability: Fixability::NotFixable,
        }
    }

    fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    fn with_remediation(mut self, remediation: impl Into<String>) -> Self {
        self.remediation = Some(remediation.into());
        self
    }

    const fn auto_fixable(mut self) -> Self {
        self.fixability = Fixability::AutoFixable;
        self
    }

    const fn fixed(mut self) -> Self {
        self.fixability = Fixability::Fixed;
        self.severity = Severity::Pass;
        self
    }
}

/// Summary counters.
#[derive(Debug, Clone, Default, Serialize)]
pub struct DoctorSummary {
    pub pass: usize,
    pub info: usize,
    pub warn: usize,
    pub fail: usize,
}

/// Full diagnostic report.
#[derive(Debug, Clone, Serialize)]
pub struct DoctorReport {
    pub findings: Vec<Finding>,
    pub summary: DoctorSummary,
    pub overall: Severity,
}

impl DoctorReport {
    fn from_findings(findings: Vec<Finding>) -> Self {
        let mut summary = DoctorSummary::default();
        let mut overall = Severity::Pass;
        for f in &findings {
            match f.severity {
                Severity::Pass => summary.pass += 1,
                Severity::Info => summary.info += 1,
                Severity::Warn => {
                    summary.warn += 1;
                    if overall < Severity::Warn {
                        overall = Severity::Warn;
                    }
                }
                Severity::Fail => {
                    summary.fail += 1;
                    overall = Severity::Fail;
                }
            }
        }
        Self {
            findings,
            summary,
            overall,
        }
    }

    /// Render human-friendly text output.
    pub fn render_text(&self) -> String {
        let mut out = String::with_capacity(2048);
        out.push_str("Pi Doctor\n=========\n");

        // Group findings by category, preserving insertion order
        let mut seen_categories: Vec<CheckCategory> = Vec::new();
        for f in &self.findings {
            if !seen_categories.contains(&f.category) {
                seen_categories.push(f.category);
            }
        }

        for cat in &seen_categories {
            let cat_findings: Vec<&Finding> = self
                .findings
                .iter()
                .filter(|f| f.category == *cat)
                .collect();
            let cat_worst = cat_findings
                .iter()
                .map(|f| f.severity)
                .max()
                .unwrap_or(Severity::Pass);
            let _ = writeln!(out, "\n[{cat_worst}] {cat}");
            for f in &cat_findings {
                let _ = writeln!(out, "  [{}] {}", f.severity, f.title);
                if let Some(detail) = &f.detail {
                    let _ = writeln!(out, "       {detail}");
                }
                if let Some(rem) = &f.remediation {
                    let _ = writeln!(out, "       Fix: {rem}");
                }
                if f.fixability == Fixability::AutoFixable {
                    out.push_str("       (fixable with --fix)\n");
                }
            }
        }

        let _ = writeln!(
            out,
            "\nOverall: {} ({} pass, {} info, {} warn, {} fail)",
            self.overall,
            self.summary.pass,
            self.summary.info,
            self.summary.warn,
            self.summary.fail
        );
        out
    }

    /// Render as JSON.
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Render as markdown.
    pub fn render_markdown(&self) -> String {
        let mut out = String::with_capacity(2048);
        out.push_str("# Pi Doctor Report\n\n");

        let mut seen_categories: Vec<CheckCategory> = Vec::new();
        for f in &self.findings {
            if !seen_categories.contains(&f.category) {
                seen_categories.push(f.category);
            }
        }

        for cat in &seen_categories {
            let _ = writeln!(out, "## {cat}\n");
            for f in self.findings.iter().filter(|f| f.category == *cat) {
                let icon = match f.severity {
                    Severity::Pass => "✅",
                    Severity::Info => "ℹ️",
                    Severity::Warn => "⚠️",
                    Severity::Fail => "❌",
                };
                let _ = write!(out, "- {icon} **{}**", f.title);
                if let Some(detail) = &f.detail {
                    let _ = write!(out, " — {detail}");
                }
                out.push('\n');
                if let Some(rem) = &f.remediation {
                    let _ = writeln!(out, "  - Fix: {rem}");
                }
            }
            out.push('\n');
        }

        let _ = writeln!(
            out,
            "**Overall: {}** ({} pass, {} info, {} warn, {} fail)",
            self.overall,
            self.summary.pass,
            self.summary.info,
            self.summary.warn,
            self.summary.fail
        );
        out
    }
}

// ── Options ─────────────────────────────────────────────────────────

/// Options for `run_doctor`.
pub struct DoctorOptions<'a> {
    pub cwd: &'a Path,
    pub extension_path: Option<&'a str>,
    pub policy_override: Option<&'a str>,
    pub fix: bool,
    pub only: Option<HashSet<CheckCategory>>,
}

// ── Entry Point ─────────────────────────────────────────────────────

/// Run all applicable doctor checks and return a report.
#[allow(clippy::too_many_lines)]
pub fn run_doctor(opts: &DoctorOptions<'_>) -> Result<DoctorReport> {
    let mut findings = Vec::new();
    let extension_only_default = opts.extension_path.is_some() && opts.only.is_none();

    let should_run = |cat: CheckCategory| -> bool {
        if extension_only_default {
            return false;
        }
        opts.only.as_ref().is_none_or(|set| set.contains(&cat))
    };

    if let Some(ext_path) = opts.extension_path {
        if opts
            .only
            .as_ref()
            .is_none_or(|set| set.contains(&CheckCategory::Extensions))
        {
            check_extension(opts.cwd, ext_path, opts.policy_override, &mut findings);
        }
    } else if opts
        .only
        .as_ref()
        .is_some_and(|set| set.contains(&CheckCategory::Extensions))
    {
        findings.push(
            Finding::fail(
                CheckCategory::Extensions,
                "Extensions check requires an extension path",
            )
            .with_remediation(
                "Run `pi doctor <path-to-extension>` to evaluate extension compatibility",
            ),
        );
    }

    if should_run(CheckCategory::Config) {
        check_config(opts.cwd, &mut findings);
    }
    if should_run(CheckCategory::Dirs) {
        check_dirs(opts.fix, &mut findings);
    }
    if should_run(CheckCategory::Auth) {
        check_auth(opts.fix, &mut findings);
    }
    if should_run(CheckCategory::Shell) {
        check_shell(&mut findings);
    }
    if should_run(CheckCategory::Sessions) {
        check_sessions(&mut findings);
    }

    Ok(DoctorReport::from_findings(findings))
}

// ── Check: Config ───────────────────────────────────────────────────

fn check_config(cwd: &Path, findings: &mut Vec<Finding>) {
    let cat = CheckCategory::Config;

    // Global settings
    let global_path = Config::global_dir().join("settings.json");
    check_settings_file(cat, &global_path, "Global settings", findings);

    // Project settings
    let project_path = cwd.join(Config::project_dir()).join("settings.json");
    if project_path.exists() {
        check_settings_file(
            cat,
            &project_path,
            "Project settings (.pi/settings.json)",
            findings,
        );
    } else {
        findings.push(Finding::pass(cat, "No project settings (OK)"));
    }
}

fn check_settings_file(cat: CheckCategory, path: &Path, label: &str, findings: &mut Vec<Finding>) {
    if !path.exists() {
        findings.push(Finding::pass(cat, format!("{label}: not present (OK)")));
        return;
    }
    match std::fs::read_to_string(path) {
        Ok(content) => {
            let value: serde_json::Value = match serde_json::from_str(&content) {
                Ok(value) => value,
                Err(e) => {
                    findings.push(
                        Finding::fail(cat, format!("{label}: JSON parse error"))
                            .with_detail(e.to_string())
                            .with_remediation(format!("Fix the JSON syntax in {}", path.display())),
                    );
                    return;
                }
            };

            let serde_json::Value::Object(map) = value else {
                findings.push(
                    Finding::fail(
                        cat,
                        format!("{label}: top-level value must be a JSON object"),
                    )
                    .with_detail(format!("Found non-object JSON in {}", path.display()))
                    .with_remediation(format!("Wrap settings in {{ ... }} in {}", path.display())),
                );
                return;
            };

            let unknown: Vec<&String> = map.keys().filter(|k| !is_known_config_key(k)).collect();
            if unknown.is_empty() {
                findings.push(Finding::pass(cat, label.to_string()));
            } else {
                findings.push(
                    Finding::warn(cat, format!("{label}: unknown keys"))
                        .with_detail(format!(
                            "Unknown keys: {}",
                            unknown
                                .iter()
                                .map(|k| k.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ))
                        .with_remediation("Check for typos in settings key names"),
                );
            }
        }
        Err(e) => {
            findings.push(
                Finding::fail(cat, format!("{label}: read error"))
                    .with_detail(e.to_string())
                    .with_remediation(format!("Check file permissions on {}", path.display())),
            );
        }
    }
}

/// Known top-level config keys (from `Config` struct fields + their camelCase aliases).
fn is_known_config_key(key: &str) -> bool {
    matches!(
        key,
        "theme"
            | "hideThinkingBlock"
            | "hide_thinking_block"
            | "showHardwareCursor"
            | "show_hardware_cursor"
            | "defaultProvider"
            | "default_provider"
            | "defaultModel"
            | "default_model"
            | "defaultThinkingLevel"
            | "default_thinking_level"
            | "enabledModels"
            | "enabled_models"
            | "steeringMode"
            | "steering_mode"
            | "followUpMode"
            | "follow_up_mode"
            | "quietStartup"
            | "quiet_startup"
            | "collapseChangelog"
            | "collapse_changelog"
            | "lastChangelogVersion"
            | "last_changelog_version"
            | "doubleEscapeAction"
            | "double_escape_action"
            | "editorPaddingX"
            | "editor_padding_x"
            | "autocompleteMaxVisible"
            | "autocomplete_max_visible"
            | "sessionPickerInput"
            | "session_picker_input"
            | "sessionStore"
            | "sessionBackend"
            | "session_store"
            | "compaction"
            | "branchSummary"
            | "branch_summary"
            | "retry"
            | "shellPath"
            | "shell_path"
            | "shellCommandPrefix"
            | "shell_command_prefix"
            | "ghPath"
            | "gh_path"
            | "images"
            | "terminal"
            | "thinkingBudgets"
            | "thinking_budgets"
            | "packages"
            | "extensions"
            | "skills"
            | "prompts"
            | "themes"
            | "enableSkillCommands"
            | "enable_skill_commands"
            | "extensionPolicy"
            | "extension_policy"
            | "repairPolicy"
            | "repair_policy"
            | "extensionRisk"
            | "extension_risk"
            | "checkForUpdates"
            | "check_for_updates"
            | "sessionDurability"
            | "session_durability"
            | "markdown"
            | "queueMode"
    )
}

// ── Check: Dirs ─────────────────────────────────────────────────────

fn check_dirs(fix: bool, findings: &mut Vec<Finding>) {
    let cat = CheckCategory::Dirs;
    let dirs = [
        ("Agent directory", Config::global_dir()),
        ("Sessions directory", Config::sessions_dir()),
        ("Packages directory", Config::package_dir()),
    ];

    for (label, dir) in &dirs {
        check_dir(cat, label, dir, fix, findings);
    }
}

fn check_dir(cat: CheckCategory, label: &str, dir: &Path, fix: bool, findings: &mut Vec<Finding>) {
    if dir.is_dir() {
        // Check write permission
        match tempfile::NamedTempFile::new_in(dir) {
            Ok(mut probe_file) => match probe_file.write_all(b"probe") {
                Ok(()) => {
                    findings.push(Finding::pass(cat, format!("{label} ({})", dir.display())));
                }
                Err(e) => {
                    findings.push(
                        Finding::fail(cat, format!("{label}: not writable"))
                            .with_detail(format!("{}: {e}", dir.display()))
                            .with_remediation(format!("chmod u+w {}", dir.display())),
                    );
                }
            },
            Err(e) => {
                findings.push(
                    Finding::fail(cat, format!("{label}: not writable"))
                        .with_detail(format!("{}: {e}", dir.display()))
                        .with_remediation(format!("chmod u+w {}", dir.display())),
                );
            }
        }
    } else if fix {
        match std::fs::create_dir_all(dir) {
            Ok(()) => {
                findings.push(
                    Finding::pass(cat, format!("{label}: created ({})", dir.display())).fixed(),
                );
            }
            Err(e) => {
                findings.push(
                    Finding::fail(cat, format!("{label}: could not create"))
                        .with_detail(format!("{}: {e}", dir.display()))
                        .with_remediation(format!("mkdir -p {}", dir.display())),
                );
            }
        }
    } else {
        findings.push(
            Finding::warn(cat, format!("{label}: missing"))
                .with_detail(format!("{} does not exist", dir.display()))
                .with_remediation(format!("mkdir -p {}", dir.display()))
                .auto_fixable(),
        );
    }
}

// ── Check: Auth ─────────────────────────────────────────────────────

#[allow(clippy::too_many_lines)]
fn check_auth(fix: bool, findings: &mut Vec<Finding>) {
    let cat = CheckCategory::Auth;
    let auth_path = Config::auth_path();

    if !auth_path.exists() {
        findings.push(
            Finding::info(cat, "auth.json: not present")
                .with_detail("No credentials stored yet")
                .with_remediation("Run `pi` and follow the login prompt, or set ANTHROPIC_API_KEY"),
        );
        // Still check env vars
        check_auth_env_vars(cat, findings);
        return;
    }

    // Check if auth.json parses
    let auth = match AuthStorage::load(auth_path.clone()) {
        Ok(auth) => {
            findings.push(Finding::pass(cat, "auth.json parses correctly"));
            Some(auth)
        }
        Err(e) => {
            findings.push(
                Finding::fail(cat, "auth.json: parse error")
                    .with_detail(e.to_string())
                    .with_remediation("Check auth.json syntax or delete and re-authenticate"),
            );
            None
        }
    };

    // Check file permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(&auth_path) {
            let mode = meta.permissions().mode() & 0o777;
            if mode == 0o600 {
                findings.push(Finding::pass(cat, "auth.json permissions (600)"));
            } else if fix {
                match std::fs::set_permissions(&auth_path, std::fs::Permissions::from_mode(0o600)) {
                    Ok(()) => {
                        findings.push(
                            Finding::pass(
                                cat,
                                format!("auth.json permissions fixed (was {mode:o}, now 600)"),
                            )
                            .fixed(),
                        );
                    }
                    Err(e) => {
                        findings.push(
                            Finding::fail(cat, "auth.json: could not fix permissions")
                                .with_detail(e.to_string()),
                        );
                    }
                }
            } else {
                findings.push(
                    Finding::warn(
                        cat,
                        format!("auth.json permissions are {mode:o}, should be 600"),
                    )
                    .with_remediation(format!("chmod 600 {}", auth_path.display()))
                    .auto_fixable(),
                );
            }
        }
    }

    // Check stored credentials
    if let Some(auth) = &auth {
        let providers = auth.provider_names();
        if providers.is_empty() {
            findings.push(
                Finding::info(cat, "No stored credentials")
                    .with_remediation("Run `pi` to authenticate or set an API key env var"),
            );
        } else {
            for provider in &providers {
                let status = auth.credential_status(provider);
                match status {
                    CredentialStatus::ApiKey => {
                        findings.push(Finding::pass(
                            cat,
                            format!("{provider}: API key configured"),
                        ));
                    }
                    CredentialStatus::OAuthValid { .. } => {
                        findings.push(Finding::pass(cat, format!("{provider}: OAuth token valid")));
                    }
                    CredentialStatus::OAuthExpired { .. } => {
                        findings.push(
                            Finding::warn(cat, format!("{provider}: OAuth token expired"))
                                .with_remediation(format!("Run `pi /login {provider}` to refresh")),
                        );
                    }
                    CredentialStatus::BearerToken => {
                        findings.push(Finding::pass(
                            cat,
                            format!("{provider}: bearer token configured"),
                        ));
                    }
                    CredentialStatus::AwsCredentials => {
                        findings.push(Finding::pass(
                            cat,
                            format!("{provider}: AWS credentials configured"),
                        ));
                    }
                    CredentialStatus::ServiceKey => {
                        findings.push(Finding::pass(
                            cat,
                            format!("{provider}: service key configured"),
                        ));
                    }
                    CredentialStatus::Missing => {
                        // Shouldn't happen since we're iterating stored providers
                        findings.push(Finding::info(cat, format!("{provider}: no credentials")));
                    }
                }
            }
        }
    }

    check_auth_env_vars(cat, findings);
}

/// Check common auth-related environment variables.
fn check_auth_env_vars(cat: CheckCategory, findings: &mut Vec<Finding>) {
    let key_providers = [
        ("anthropic", "ANTHROPIC_API_KEY"),
        ("openai", "OPENAI_API_KEY"),
        ("google", "GOOGLE_API_KEY"),
    ];

    for (provider, env_key) in &key_providers {
        let env_keys = provider_auth_env_keys(provider);
        let has_env = env_keys.iter().any(|k| std::env::var(k).is_ok());
        if has_env {
            findings.push(Finding::pass(
                cat,
                format!("{provider}: env var set ({env_key})"),
            ));
        } else {
            findings.push(
                Finding::info(cat, format!("{provider}: no env var"))
                    .with_detail(format!("Set {env_key} or run `pi /login {provider}`")),
            );
        }
    }
}

// ── Check: Shell ────────────────────────────────────────────────────

fn check_shell(findings: &mut Vec<Finding>) {
    let cat = CheckCategory::Shell;

    // Required tools (Fail if missing)
    check_tool(
        cat,
        "bash",
        &["--version"],
        Severity::Fail,
        ToolCheckMode::PresenceOnly,
        findings,
    );
    check_tool(
        cat,
        "sh",
        &["--version"],
        Severity::Fail,
        ToolCheckMode::PresenceOnly,
        findings,
    );

    // Important tools (Warn if missing)
    check_tool(
        cat,
        "git",
        &["--version"],
        Severity::Warn,
        ToolCheckMode::PresenceOnly,
        findings,
    );
    check_tool(
        cat,
        "rg",
        &["--version"],
        Severity::Warn,
        ToolCheckMode::PresenceOnly,
        findings,
    );

    let fd_bin = if which_tool("fd").is_some() {
        "fd"
    } else {
        "fdfind"
    };
    check_tool(
        cat,
        fd_bin,
        &["--version"],
        Severity::Warn,
        ToolCheckMode::PresenceOnly,
        findings,
    );

    // Optional tools (Info if missing)
    check_tool(
        cat,
        "gh",
        &["--version"],
        Severity::Info,
        ToolCheckMode::PresenceOnly,
        findings,
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToolCheckMode {
    PresenceOnly,
    ProbeExecution,
}

fn check_tool(
    cat: CheckCategory,
    tool: &str,
    args: &[&str],
    missing_severity: Severity,
    mode: ToolCheckMode,
    findings: &mut Vec<Finding>,
) {
    let discovered_path = which_tool(tool);
    if mode == ToolCheckMode::PresenceOnly {
        if let Some(path) = discovered_path {
            findings.push(Finding::pass(cat, format!("{tool} ({path})")));
            return;
        }
        report_missing_tool(cat, tool, missing_severity, findings);
        return;
    }

    let command_target = discovered_path.as_deref().unwrap_or(tool);

    match Command::new(command_target)
        .args(args)
        .stdin(std::process::Stdio::null())
        .output()
    {
        Ok(output) if output.status.success() => {
            // Extract version from first line of stdout
            let version = String::from_utf8_lossy(&output.stdout);
            let first_line = version.lines().next().unwrap_or("").trim();
            let label = discovered_path.as_ref().map_or_else(
                || {
                    if first_line.is_empty() {
                        tool.to_string()
                    } else {
                        format!("{tool}: {first_line}")
                    }
                },
                |path| format!("{tool} ({path})"),
            );
            findings.push(Finding::pass(cat, label));
        }
        Ok(output)
            if discovered_path.is_some()
                && probe_failure_is_known_nonfatal(tool, args, &output) =>
        {
            // Some shells (e.g. dash as /bin/sh) do not support --version.
            // If this is the known non-fatal probe case, treat tool as present.
            let path = discovered_path.unwrap_or_default();
            findings.push(Finding::pass(cat, format!("{tool} ({path})")));
        }
        Ok(output) => {
            let suffix = if missing_severity == Severity::Info {
                " (optional)"
            } else {
                ""
            };
            let detail = {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                if stderr.is_empty() {
                    format!("Exit status: {:?}", output.status.code())
                } else {
                    stderr
                }
            };
            findings.push(Finding {
                category: cat,
                severity: missing_severity,
                title: format!("{tool}: invocation failed{suffix}"),
                detail: Some(detail),
                remediation: discovered_path
                    .as_ref()
                    .map(|path| format!("Verify this executable is healthy: {path}")),
                fixability: Fixability::NotFixable,
            });
        }
        Err(err) => {
            if discovered_path.is_some() || err.kind() != std::io::ErrorKind::NotFound {
                let suffix = if missing_severity == Severity::Info {
                    " (optional)"
                } else {
                    ""
                };
                findings.push(Finding {
                    category: cat,
                    severity: missing_severity,
                    title: format!("{tool}: invocation failed{suffix}"),
                    detail: Some(err.to_string()),
                    remediation: discovered_path
                        .as_ref()
                        .map(|path| format!("Verify this executable is healthy: {path}")),
                    fixability: Fixability::NotFixable,
                });
            } else {
                report_missing_tool(cat, tool, missing_severity, findings);
            }
        }
    }
}

fn report_missing_tool(
    cat: CheckCategory,
    tool: &str,
    missing_severity: Severity,
    findings: &mut Vec<Finding>,
) {
    let suffix = if missing_severity == Severity::Info {
        " (optional)"
    } else {
        ""
    };
    let mut f = Finding {
        category: cat,
        severity: missing_severity,
        title: format!("{tool}: not found{suffix}"),
        detail: None,
        remediation: None,
        fixability: Fixability::NotFixable,
    };
    if tool == "gh" {
        f.remediation = Some("Install: https://cli.github.com/".to_string());
    }
    findings.push(f);
}

fn probe_failure_is_known_nonfatal(
    tool: &str,
    args: &[&str],
    output: &std::process::Output,
) -> bool {
    if tool != "sh" || args != ["--version"] {
        return false;
    }
    let stderr = String::from_utf8_lossy(&output.stderr).to_ascii_lowercase();
    stderr.contains("illegal option")
        || stderr.contains("unknown option")
        || stderr.contains("invalid option")
}

fn which_tool(tool: &str) -> Option<String> {
    let tool_path = Path::new(tool);
    if tool_path.components().count() > 1 {
        return is_executable(tool_path).then(|| tool_path.display().to_string());
    }

    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        if let Some(path) = resolve_executable_in_dir(&dir, tool) {
            return Some(path.display().to_string());
        }
    }
    None
}

fn resolve_executable_in_dir(dir: &Path, tool: &str) -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let candidate = dir.join(tool);
        if is_executable(&candidate) {
            return Some(candidate);
        }
        let pathext = std::env::var_os("PATHEXT").unwrap_or_else(|| ".COM;.EXE;.BAT;.CMD".into());
        for ext in std::env::split_paths(&pathext) {
            let ext = ext.to_string_lossy();
            let suffix = ext.trim_matches('.');
            if suffix.is_empty() {
                continue;
            }
            let candidate = dir.join(format!("{tool}.{suffix}"));
            if is_executable(&candidate) {
                return Some(candidate);
            }
        }
        None
    }

    #[cfg(not(windows))]
    {
        let candidate = dir.join(tool);
        is_executable(&candidate).then_some(candidate)
    }
}

fn is_executable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::metadata(path)
            .ok()
            .is_some_and(|metadata| metadata.permissions().mode() & 0o111 != 0)
    }

    #[cfg(not(unix))]
    {
        true
    }
}

// ── Check: Sessions ─────────────────────────────────────────────────

fn check_sessions(findings: &mut Vec<Finding>) {
    let cat = CheckCategory::Sessions;
    let sessions_dir = Config::sessions_dir();

    if !sessions_dir.is_dir() {
        findings.push(Finding::info(
            cat,
            "Sessions directory does not exist (no sessions yet)",
        ));
        return;
    }

    let entries = walk_sessions(&sessions_dir);
    let total = entries.len().min(500); // Cap scan
    let mut corrupt = 0u32;

    for entry in entries.into_iter().take(500) {
        let Ok(path) = entry else {
            corrupt += 1;
            continue;
        };
        if !is_session_healthy(&path) {
            corrupt += 1;
        }
    }

    if corrupt == 0 {
        findings.push(Finding::pass(cat, format!("{total} sessions, 0 corrupt")));
    } else {
        findings.push(
            Finding::warn(cat, format!("{total} sessions, {corrupt} corrupt"))
                .with_detail("Some session files are empty or have invalid headers")
                .with_remediation("Corrupt sessions can be safely deleted"),
        );
    }
}

/// Quick health check: non-empty and first line parses as a valid session header.
fn is_session_healthy(path: &Path) -> bool {
    #[cfg(feature = "sqlite-sessions")]
    if path.extension().and_then(|ext| ext.to_str()) == Some("sqlite") {
        return futures::executor::block_on(async {
            crate::session_sqlite::load_session_meta(path)
                .await
                .is_ok_and(|meta| meta.header.is_valid())
        });
    }

    let Ok(file) = std::fs::File::open(path) else {
        return false;
    };
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    match reader.read_line(&mut line) {
        Ok(0) | Err(_) => false, // empty or unreadable
        Ok(_) => serde_json::from_str::<SessionHeader>(&line).is_ok_and(|header| header.is_valid()),
    }
}

// ── Check: Extension ────────────────────────────────────────────────

fn check_extension(
    cwd: &Path,
    path: &str,
    policy_override: Option<&str>,
    findings: &mut Vec<Finding>,
) {
    use crate::extension_preflight::{FindingSeverity, PreflightAnalyzer, PreflightVerdict};

    let cat = CheckCategory::Extensions;
    let ext_path = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        cwd.join(path)
    };

    if !ext_path.exists() {
        findings.push(
            Finding::fail(
                cat,
                format!("Extension path not found: {}", ext_path.display()),
            )
            .with_remediation("Check the path and try again"),
        );
        return;
    }

    let config_path = Config::config_path_override_from_env(cwd);
    let resolved = match Config::load_with_roots(config_path.as_deref(), &Config::global_dir(), cwd)
    {
        Ok(config) => config.resolve_extension_policy_with_metadata(policy_override),
        Err(err) => {
            findings.push(
                Finding::fail(
                    cat,
                    "Failed to load configuration for extension policy resolution",
                )
                .with_detail(err.to_string())
                .with_remediation(
                    "Fix the malformed settings.json, point PI_CONFIG_PATH at a valid file, or rerun with `--policy <safe|balanced|permissive>` to inspect extension compatibility independently",
                ),
            );
            let has_explicit_policy =
                policy_override.is_some() || std::env::var_os("PI_EXTENSION_POLICY").is_some();
            if has_explicit_policy {
                Config::default().resolve_extension_policy_with_metadata(policy_override)
            } else {
                // If project config is unreadable, fail closed instead of silently
                // analyzing under the default permissive profile.
                Config::default().resolve_extension_policy_with_metadata(Some("safe"))
            }
        }
    };
    let ext_id = ext_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let analyzer = PreflightAnalyzer::new(&resolved.policy, Some(ext_id));
    let report = analyzer.analyze(&ext_path);

    // Convert preflight verdict to a top-level finding
    match report.verdict {
        PreflightVerdict::Pass => {
            findings.push(Finding::pass(
                cat,
                format!("Extension {ext_id}: compatible"),
            ));
        }
        PreflightVerdict::Warn => {
            findings.push(
                Finding::warn(cat, format!("Extension {ext_id}: partial compatibility"))
                    .with_detail(format!(
                        "{} warning(s), {} info",
                        report.summary.warnings, report.summary.info
                    )),
            );
        }
        PreflightVerdict::Fail => {
            findings.push(
                Finding::fail(cat, format!("Extension {ext_id}: incompatible"))
                    .with_detail(format!(
                        "{} error(s), {} warning(s)",
                        report.summary.errors, report.summary.warnings
                    ))
                    .with_remediation(format!("Try: pi doctor {path} --policy permissive")),
            );
        }
    }

    // Convert individual preflight findings
    for pf in &report.findings {
        let severity = match pf.severity {
            FindingSeverity::Error => Severity::Fail,
            FindingSeverity::Warning => Severity::Warn,
            FindingSeverity::Info => Severity::Info,
        };
        let mut f = Finding {
            category: cat,
            severity,
            title: pf.message.clone(),
            detail: pf.file.as_ref().map(|file| {
                pf.line
                    .map_or_else(|| format!("at {file}"), |line| format!("at {file}:{line}"))
            }),
            remediation: pf.remediation.clone(),
            fixability: Fixability::NotFixable,
        };
        // Ensure we don't lose location info
        if f.detail.is_none() && pf.file.is_some() {
            f.detail.clone_from(&pf.file);
        }
        findings.push(f);
    }
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn write_extension_fixture(cwd: &Path, source: &str) -> PathBuf {
        let extension_dir = cwd.join("ext");
        std::fs::create_dir_all(&extension_dir).expect("create extension dir");
        std::fs::write(extension_dir.join("index.js"), source).expect("write extension source");
        extension_dir
    }

    #[test]
    fn severity_ordering() {
        assert!(Severity::Pass < Severity::Info);
        assert!(Severity::Info < Severity::Warn);
        assert!(Severity::Warn < Severity::Fail);
    }

    #[test]
    fn severity_display() {
        assert_eq!(Severity::Pass.to_string(), "PASS");
        assert_eq!(Severity::Fail.to_string(), "FAIL");
    }

    #[test]
    fn check_category_parse() {
        assert_eq!(
            "config".parse::<CheckCategory>().unwrap(),
            CheckCategory::Config
        );
        assert_eq!(
            "dirs".parse::<CheckCategory>().unwrap(),
            CheckCategory::Dirs
        );
        assert_eq!(
            "directories".parse::<CheckCategory>().unwrap(),
            CheckCategory::Dirs
        );
        assert_eq!(
            "auth".parse::<CheckCategory>().unwrap(),
            CheckCategory::Auth
        );
        assert_eq!(
            "shell".parse::<CheckCategory>().unwrap(),
            CheckCategory::Shell
        );
        assert_eq!(
            "sessions".parse::<CheckCategory>().unwrap(),
            CheckCategory::Sessions
        );
        assert_eq!(
            "extensions".parse::<CheckCategory>().unwrap(),
            CheckCategory::Extensions
        );
        assert_eq!(
            "ext".parse::<CheckCategory>().unwrap(),
            CheckCategory::Extensions
        );
        assert!("unknown".parse::<CheckCategory>().is_err());
    }

    #[test]
    fn finding_builders() {
        let f = Finding::pass(CheckCategory::Config, "test")
            .with_detail("detail")
            .with_remediation("fix it");
        assert_eq!(f.severity, Severity::Pass);
        assert_eq!(f.detail.as_deref(), Some("detail"));
        assert_eq!(f.remediation.as_deref(), Some("fix it"));

        let f = Finding::warn(CheckCategory::Auth, "warn test").auto_fixable();
        assert_eq!(f.fixability, Fixability::AutoFixable);

        let f = Finding::fail(CheckCategory::Dirs, "fail test").fixed();
        assert_eq!(f.severity, Severity::Pass); // fixed downgrades to pass
        assert_eq!(f.fixability, Fixability::Fixed);
    }

    #[test]
    fn report_summary() {
        let findings = vec![
            Finding::pass(CheckCategory::Config, "ok"),
            Finding::info(CheckCategory::Auth, "info"),
            Finding::warn(CheckCategory::Shell, "warn"),
            Finding::fail(CheckCategory::Dirs, "fail"),
        ];
        let report = DoctorReport::from_findings(findings);
        assert_eq!(report.summary.pass, 1);
        assert_eq!(report.summary.info, 1);
        assert_eq!(report.summary.warn, 1);
        assert_eq!(report.summary.fail, 1);
        assert_eq!(report.overall, Severity::Fail);
    }

    #[test]
    fn report_all_pass() {
        let findings = vec![
            Finding::pass(CheckCategory::Config, "a"),
            Finding::pass(CheckCategory::Dirs, "b"),
        ];
        let report = DoctorReport::from_findings(findings);
        assert_eq!(report.overall, Severity::Pass);
    }

    #[test]
    fn render_text_includes_header() {
        let report =
            DoctorReport::from_findings(vec![Finding::pass(CheckCategory::Config, "all good")]);
        let text = report.render_text();
        assert!(text.contains("Pi Doctor"));
        assert!(text.contains("[PASS] Configuration"));
        assert!(text.contains("[PASS] all good"));
    }

    #[test]
    fn render_json_valid() {
        let report = DoctorReport::from_findings(vec![Finding::pass(CheckCategory::Config, "ok")]);
        let json = report.to_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("findings").is_some());
        assert!(parsed.get("summary").is_some());
        assert!(parsed.get("overall").is_some());
    }

    #[test]
    fn render_markdown_includes_header() {
        let report =
            DoctorReport::from_findings(vec![Finding::warn(CheckCategory::Auth, "expired")]);
        let md = report.render_markdown();
        assert!(md.contains("# Pi Doctor Report"));
        assert!(md.contains("## Authentication"));
    }

    #[test]
    fn known_config_keys_includes_common() {
        assert!(is_known_config_key("theme"));
        assert!(is_known_config_key("defaultModel"));
        assert!(is_known_config_key("extensionPolicy"));
        assert!(!is_known_config_key("nonexistent_key_xyz"));
    }

    #[test]
    fn session_healthy_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.jsonl");
        std::fs::write(&path, "").unwrap();
        assert!(!is_session_healthy(&path));
    }

    #[test]
    fn session_healthy_valid_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("valid.jsonl");
        std::fs::write(
            &path,
            r#"{"type":"session","version":3,"id":"doctor-jsonl","timestamp":"2026-01-01T00:00:00.000Z","cwd":"/tmp"}"#,
        )
        .unwrap();
        assert!(is_session_healthy(&path));
    }

    #[test]
    fn session_healthy_invalid_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("invalid.jsonl");
        std::fs::write(&path, "not json at all\n").unwrap();
        assert!(!is_session_healthy(&path));
    }

    #[test]
    fn session_healthy_rejects_non_header_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("array.jsonl");
        std::fs::write(&path, "[1,2,3]\n").unwrap();
        assert!(!is_session_healthy(&path));
    }

    #[cfg(feature = "sqlite-sessions")]
    #[test]
    fn session_healthy_valid_sqlite() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("valid.sqlite");
        let header = SessionHeader {
            id: "doctor-sqlite".to_string(),
            ..SessionHeader::default()
        };
        futures::executor::block_on(async {
            crate::session_sqlite::save_session(&path, &header, &[])
                .await
                .expect("save sqlite session");
        });
        assert!(is_session_healthy(&path));
    }

    #[cfg(feature = "sqlite-sessions")]
    #[test]
    fn session_healthy_rejects_invalid_sqlite_header() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("invalid.sqlite");
        let header = SessionHeader {
            id: "doctor-sqlite".to_string(),
            ..SessionHeader::default()
        };
        futures::executor::block_on(async {
            crate::session_sqlite::save_session(&path, &header, &[])
                .await
                .expect("save sqlite session");
        });
        let invalid_header = SessionHeader {
            r#type: "not-session".to_string(),
            ..header
        };
        let invalid_json =
            serde_json::to_string(&invalid_header).expect("serialize invalid session header");
        let config = sqlmodel_sqlite::SqliteConfig::file(path.to_string_lossy())
            .flags(sqlmodel_sqlite::OpenFlags::create_read_write());
        let conn = sqlmodel_sqlite::SqliteConnection::open(&config).expect("open sqlite db");
        conn.execute_sync(
            "UPDATE pi_session_header SET json = ?1",
            &[sqlmodel_core::Value::Text(invalid_json)],
        )
        .expect("corrupt sqlite header row");
        assert!(!is_session_healthy(&path));
    }

    #[test]
    fn check_dir_creates_missing_with_fix() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("sub/nested");
        let mut findings = Vec::new();
        check_dir(CheckCategory::Dirs, "test", &missing, true, &mut findings);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Pass);
        assert_eq!(findings[0].fixability, Fixability::Fixed);
        assert!(missing.is_dir());
    }

    #[test]
    fn check_dir_warns_missing_without_fix() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("sub/nested");
        let mut findings = Vec::new();
        check_dir(CheckCategory::Dirs, "test", &missing, false, &mut findings);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Warn);
        assert_eq!(findings[0].fixability, Fixability::AutoFixable);
        assert!(!missing.exists());
    }

    #[test]
    fn check_shell_finds_bash() {
        let mut findings = Vec::new();
        check_tool(
            CheckCategory::Shell,
            "bash",
            &["--version"],
            Severity::Fail,
            ToolCheckMode::ProbeExecution,
            &mut findings,
        );
        // bash should be available in CI/dev environments
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Pass);
    }

    #[cfg(unix)]
    #[test]
    fn check_tool_falls_back_when_probe_args_are_unsupported() {
        let mut findings = Vec::new();
        check_tool(
            CheckCategory::Shell,
            "sh",
            &["--version"],
            Severity::Fail,
            ToolCheckMode::ProbeExecution,
            &mut findings,
        );
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Pass);
    }

    #[cfg(unix)]
    #[test]
    fn check_tool_reports_invocation_failure_for_broken_executable() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("broken_tool.sh");
        // Mark a non-binary, non-script blob as executable so spawn fails with
        // "exec format error" rather than "not found".
        std::fs::write(&script, "not an executable format").unwrap();
        let mut perms = std::fs::metadata(&script).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script, perms).unwrap();

        let mut findings = Vec::new();
        check_tool(
            CheckCategory::Shell,
            script.to_str().unwrap(),
            &["--version"],
            Severity::Fail,
            ToolCheckMode::ProbeExecution,
            &mut findings,
        );

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Fail);
        assert!(findings[0].title.contains("invocation failed"));
    }

    #[test]
    fn check_settings_file_rejects_non_object_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, "[1,2,3]").unwrap();
        let mut findings = Vec::new();
        check_settings_file(CheckCategory::Config, &path, "Settings", &mut findings);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Fail);
        assert!(
            findings[0]
                .title
                .contains("top-level value must be a JSON object")
        );
    }

    #[test]
    fn fixability_display() {
        // Ensure serialization works
        let json = serde_json::to_string(&Fixability::AutoFixable).unwrap();
        assert!(json.contains("autoFixable") || json.contains("auto"));
    }

    #[test]
    fn run_doctor_path_mode_defaults_to_extension_checks_only() {
        let dir = tempfile::tempdir().unwrap();
        let opts = DoctorOptions {
            cwd: dir.path(),
            extension_path: Some("missing-ext"),
            policy_override: None,
            fix: false,
            only: None,
        };
        let report = run_doctor(&opts).unwrap();
        assert!(
            !report.findings.is_empty(),
            "missing extension path should produce at least one finding"
        );
        assert!(
            report
                .findings
                .iter()
                .all(|f| f.category == CheckCategory::Extensions),
            "path mode should not run unrelated environment categories by default"
        );
    }

    #[test]
    fn run_doctor_only_extensions_without_path_reports_error_finding() {
        let mut only = HashSet::new();
        only.insert(CheckCategory::Extensions);
        let dir = tempfile::tempdir().unwrap();
        let opts = DoctorOptions {
            cwd: dir.path(),
            extension_path: None,
            policy_override: None,
            fix: false,
            only: Some(only),
        };
        let report = run_doctor(&opts).unwrap();
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.category == CheckCategory::Extensions && f.severity == Severity::Fail),
            "extensions-only mode without a path should emit a clear failure finding"
        );
    }

    #[test]
    fn run_doctor_extension_path_uses_supplied_cwd_for_policy_resolution() {
        let project = tempfile::tempdir().expect("project dir");
        let config_dir = project.path().join(".pi");
        std::fs::create_dir_all(&config_dir).expect("create project config dir");
        std::fs::write(
            config_dir.join("settings.json"),
            r#"{ "extensionPolicy": { "profile": "safe" } }"#,
        )
        .expect("write project settings");
        write_extension_fixture(
            project.path(),
            r#"
const { exec } = require("child_process");
export default function(pi) {
    pi.exec("ls");
}
"#,
        );

        let opts = DoctorOptions {
            cwd: project.path(),
            extension_path: Some("ext"),
            policy_override: None,
            fix: false,
            only: None,
        };
        let report = run_doctor(&opts).expect("doctor report");

        assert!(
            report.findings.iter().any(|f| f.title.contains("exec")),
            "doctor should honor the supplied cwd's safe policy and flag exec use"
        );
    }

    #[test]
    fn run_doctor_extension_path_reports_config_load_failure_without_aborting() {
        let project = tempfile::tempdir().expect("project dir");
        let config_dir = project.path().join(".pi");
        std::fs::create_dir_all(&config_dir).expect("create project config dir");
        std::fs::write(config_dir.join("settings.json"), r#"{ "extensionPolicy": "#)
            .expect("write malformed project settings");
        write_extension_fixture(
            project.path(),
            r#"
import net from "node:net";
"#,
        );

        let opts = DoctorOptions {
            cwd: project.path(),
            extension_path: Some("ext"),
            policy_override: None,
            fix: false,
            only: None,
        };
        let report = run_doctor(&opts).expect("doctor report");

        assert!(
            report
                .findings
                .iter()
                .all(|f| f.category == CheckCategory::Extensions),
            "extension path mode should keep findings scoped to extensions"
        );
        assert!(
            report.findings.iter().any(|f| {
                f.title == "Failed to load configuration for extension policy resolution"
            }),
            "doctor should surface config load failures as findings instead of returning Err"
        );
        assert!(
            report.findings.iter().any(|f| f.title.contains("node:net")),
            "doctor should continue extension analysis after a config load failure"
        );
    }

    #[test]
    fn run_doctor_extension_path_config_load_failure_falls_back_to_safe_policy() {
        let project = tempfile::tempdir().expect("project dir");
        let config_dir = project.path().join(".pi");
        std::fs::create_dir_all(&config_dir).expect("create project config dir");
        std::fs::write(config_dir.join("settings.json"), r#"{ "extensionPolicy": "#)
            .expect("write malformed project settings");
        write_extension_fixture(
            project.path(),
            r#"
export default function(pi) {
    pi.exec("ls");
}
"#,
        );

        let opts = DoctorOptions {
            cwd: project.path(),
            extension_path: Some("ext"),
            policy_override: None,
            fix: false,
            only: None,
        };
        let report = run_doctor(&opts).expect("doctor report");

        assert!(
            report
                .findings
                .iter()
                .any(|f| f.title == "Extension ext: incompatible"),
            "doctor should fail closed under a safe fallback when config loading fails"
        );
        assert!(
            report.findings.iter().any(|f| f.title.contains("exec")),
            "safe fallback should still flag denied exec usage"
        );
    }

    #[test]
    fn run_doctor_extension_path_config_load_failure_honors_cli_policy_override() {
        let project = tempfile::tempdir().expect("project dir");
        let config_dir = project.path().join(".pi");
        std::fs::create_dir_all(&config_dir).expect("create project config dir");
        std::fs::write(config_dir.join("settings.json"), r#"{ "extensionPolicy": "#)
            .expect("write malformed project settings");
        write_extension_fixture(
            project.path(),
            r#"
export default function(pi) {
    pi.exec("ls");
}
"#,
        );

        let opts = DoctorOptions {
            cwd: project.path(),
            extension_path: Some("ext"),
            policy_override: Some("permissive"),
            fix: false,
            only: None,
        };
        let report = run_doctor(&opts).expect("doctor report");

        assert!(
            report
                .findings
                .iter()
                .any(|f| f.title == "Extension ext: compatible"),
            "explicit CLI overrides should still control fallback analysis"
        );
        assert!(
            !report.findings.iter().any(|f| f.title.contains("exec")),
            "permissive override should suppress safe-only exec denial findings"
        );
    }

    mod proptest_doctor {
        use super::*;
        use proptest::prelude::*;

        const ALL_SEVERITIES: &[Severity] = &[
            Severity::Pass,
            Severity::Info,
            Severity::Warn,
            Severity::Fail,
        ];

        const CATEGORY_ALIASES: &[&str] = &[
            "config",
            "dirs",
            "directories",
            "auth",
            "authentication",
            "shell",
            "sessions",
            "extensions",
            "ext",
        ];

        proptest! {
            /// Severity ordering is total: Pass < Info < Warn < Fail.
            #[test]
            fn severity_ordering_total(a in 0..4usize, b in 0..4usize) {
                let sa = ALL_SEVERITIES[a];
                let sb = ALL_SEVERITIES[b];
                match a.cmp(&b) {
                    std::cmp::Ordering::Less => assert!(sa < sb),
                    std::cmp::Ordering::Equal => assert!(sa == sb),
                    std::cmp::Ordering::Greater => assert!(sa > sb),
                }
            }

            /// Severity display produces uppercase 4-char labels.
            #[test]
            fn severity_display_uppercase(idx in 0..4usize) {
                let s = ALL_SEVERITIES[idx];
                let display = s.to_string();
                assert_eq!(display.len(), 4);
                assert!(display.chars().all(|c| c.is_ascii_uppercase()));
            }

            /// `CheckCategory::from_str` accepts all known aliases.
            #[test]
            fn check_category_known_aliases(idx in 0..CATEGORY_ALIASES.len()) {
                let alias = CATEGORY_ALIASES[idx];
                assert!(alias.parse::<CheckCategory>().is_ok());
            }

            /// `CheckCategory::from_str` is case-insensitive.
            #[test]
            fn check_category_case_insensitive(idx in 0..CATEGORY_ALIASES.len()) {
                let alias = CATEGORY_ALIASES[idx];
                let upper = alias.to_uppercase();
                let lower_result = alias.parse::<CheckCategory>();
                let upper_result = upper.parse::<CheckCategory>();
                assert_eq!(lower_result, upper_result);
            }

            /// Unknown category names are rejected.
            #[test]
            fn check_category_unknown_rejected(s in "[a-z]{10,20}") {
                assert!(s.parse::<CheckCategory>().is_err());
            }

            /// `CheckCategory::label` returns non-empty strings.
            #[test]
            fn check_category_label_non_empty(idx in 0..6usize) {
                let cats = [
                    CheckCategory::Config,
                    CheckCategory::Dirs,
                    CheckCategory::Auth,
                    CheckCategory::Shell,
                    CheckCategory::Sessions,
                    CheckCategory::Extensions,
                ];
                let label = cats[idx].label();
                assert!(!label.is_empty());
                // Label starts with uppercase
                assert!(label.starts_with(|c: char| c.is_uppercase()));
            }

            /// `DoctorReport::from_findings` summary counts match input.
            #[test]
            fn from_findings_counts_match(
                pass in 0..5usize,
                info in 0..5usize,
                warn in 0..5usize,
                fail in 0..5usize
            ) {
                let mut findings = Vec::new();
                for _ in 0..pass {
                    findings.push(Finding::pass(CheckCategory::Config, "test"));
                }
                for _ in 0..info {
                    findings.push(Finding::info(CheckCategory::Config, "test"));
                }
                for _ in 0..warn {
                    findings.push(Finding::warn(CheckCategory::Config, "test"));
                }
                for _ in 0..fail {
                    findings.push(Finding::fail(CheckCategory::Config, "test"));
                }

                let report = DoctorReport::from_findings(findings);
                assert_eq!(report.summary.pass, pass);
                assert_eq!(report.summary.info, info);
                assert_eq!(report.summary.warn, warn);
                assert_eq!(report.summary.fail, fail);
            }

            /// `DoctorReport::from_findings` overall severity is max of inputs.
            #[test]
            fn from_findings_overall_severity(
                pass in 0..3usize,
                info in 0..3usize,
                warn in 0..3usize,
                fail in 0..3usize
            ) {
                let mut findings = Vec::new();
                for _ in 0..pass {
                    findings.push(Finding::pass(CheckCategory::Config, "test"));
                }
                for _ in 0..info {
                    findings.push(Finding::info(CheckCategory::Config, "test"));
                }
                for _ in 0..warn {
                    findings.push(Finding::warn(CheckCategory::Config, "test"));
                }
                for _ in 0..fail {
                    findings.push(Finding::fail(CheckCategory::Config, "test"));
                }

                let report = DoctorReport::from_findings(findings);

                if fail > 0 {
                    assert_eq!(report.overall, Severity::Fail);
                } else if warn > 0 {
                    assert_eq!(report.overall, Severity::Warn);
                } else {
                    assert_eq!(report.overall, Severity::Pass);
                }
            }

            /// `is_known_config_key` accepts both camelCase and snake_case forms.
            #[test]
            fn config_key_pairs(idx in 0..10usize) {
                let pairs = [
                    ("hideThinkingBlock", "hide_thinking_block"),
                    ("showHardwareCursor", "show_hardware_cursor"),
                    ("defaultProvider", "default_provider"),
                    ("defaultModel", "default_model"),
                    ("defaultThinkingLevel", "default_thinking_level"),
                    ("enabledModels", "enabled_models"),
                    ("steeringMode", "steering_mode"),
                    ("followUpMode", "follow_up_mode"),
                    ("quietStartup", "quiet_startup"),
                    ("collapseChangelog", "collapse_changelog"),
                ];
                let (camel, snake) = pairs[idx];
                assert!(is_known_config_key(camel), "camelCase key {camel} should be known");
                assert!(is_known_config_key(snake), "snake_case key {snake} should be known");
            }

            /// `is_known_config_key` rejects garbage keys.
            #[test]
            fn config_key_rejects_garbage(s in "[A-Z]{20,30}") {
                assert!(!is_known_config_key(&s));
            }

            /// Severity serde roundtrip is lowercase.
            #[test]
            fn severity_serde_lowercase(idx in 0..4usize) {
                let s = ALL_SEVERITIES[idx];
                let json = serde_json::to_string(&s).unwrap();
                let expected = format!("\"{}\"", s.to_string().to_lowercase());
                assert_eq!(json, expected);
            }

            /// Finding builder chain preserves fields.
            #[test]
            fn finding_builder_chain(title in "[a-z ]{1,20}", detail in "[a-z ]{1,20}") {
                let f = Finding::warn(CheckCategory::Shell, title.clone())
                    .with_detail(detail.clone())
                    .with_remediation("fix it")
                    .auto_fixable();
                assert_eq!(f.title, title);
                assert_eq!(f.detail.as_deref(), Some(detail.as_str()));
                assert_eq!(f.remediation.as_deref(), Some("fix it"));
                assert_eq!(f.fixability, Fixability::AutoFixable);
                assert_eq!(f.severity, Severity::Warn);
            }

            /// `fixed()` resets severity to Pass.
            #[test]
            fn finding_fixed_resets_severity(idx in 0..4usize) {
                let builders = [
                    Finding::pass(CheckCategory::Config, "t"),
                    Finding::info(CheckCategory::Config, "t"),
                    Finding::warn(CheckCategory::Config, "t"),
                    Finding::fail(CheckCategory::Config, "t"),
                ];
                let fixed = builders[idx].clone().fixed();
                assert_eq!(fixed.severity, Severity::Pass);
                assert_eq!(fixed.fixability, Fixability::Fixed);
            }
        }
    }
}
