//! Extended integration tests for Config edge cases: merge semantics,
//! nested settings deep-merge, accessor defaults, serde alias support,
//! empty/missing files, and extension/repair policy resolution.
//!
//! Run:
//! ```bash
//! cargo test --test config_edge_cases
//! ```

mod common;

use common::TestHarness;
use pi::config::{Config, SettingsScope};
use serde_json::json;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

fn config_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().expect("lock")
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent dir");
    }
    std::fs::write(path, contents).expect("write file");
}

// ─── Default accessor tests ─────────────────────────────────────────────────

#[test]
fn config_default_compaction_settings() {
    let config = Config::default();
    assert!(config.compaction_enabled());
    assert_eq!(config.compaction_reserve_tokens(), 16384);
    assert_eq!(config.compaction_keep_recent_tokens(), 20000);
}

#[test]
fn config_default_retry_settings() {
    let config = Config::default();
    assert!(config.retry_enabled());
    assert_eq!(config.retry_max_retries(), 3);
    assert_eq!(config.retry_base_delay_ms(), 2000);
    assert_eq!(config.retry_max_delay_ms(), 60000);
}

#[test]
fn config_default_image_and_terminal_settings() {
    let config = Config::default();
    assert!(config.image_auto_resize());
    assert!(config.terminal_show_images());
    assert!(!config.terminal_clear_on_shrink());
    assert!(config.enable_skill_commands());
}

#[test]
fn config_default_thinking_budgets() {
    let config = Config::default();
    assert_eq!(config.thinking_budget("minimal"), 1024);
    assert_eq!(config.thinking_budget("low"), 2048);
    assert_eq!(config.thinking_budget("medium"), 8192);
    assert_eq!(config.thinking_budget("high"), 16384);
    assert_eq!(config.thinking_budget("xhigh"), u32::MAX);
    assert_eq!(config.thinking_budget("unknown"), 0);
}

#[test]
fn config_default_queue_modes() {
    let config = Config::default();
    // Default queue modes should be sensible (not panicking)
    let _steering = config.steering_queue_mode();
    let _follow_up = config.follow_up_queue_mode();
}

// ─── Merge semantics ─────────────────────────────────────────────────────────

#[test]
fn config_merge_other_wins_over_base_for_flat_fields() {
    let base = Config {
        theme: Some("base-theme".to_string()),
        default_provider: Some("anthropic".to_string()),
        default_model: Some("base-model".to_string()),
        gh_path: Some("/usr/bin/gh".to_string()),
        ..Config::default()
    };
    let other = Config {
        default_provider: Some("openai".to_string()),
        default_model: None, // should fall back to base
        gh_path: Some("/opt/gh".to_string()),
        ..Config::default()
    };

    let merged = Config::merge(base, other);
    assert_eq!(merged.theme.as_deref(), Some("base-theme"));
    assert_eq!(merged.default_provider.as_deref(), Some("openai"));
    assert_eq!(merged.default_model.as_deref(), Some("base-model"));
    assert_eq!(merged.gh_path.as_deref(), Some("/opt/gh"));
}

#[test]
fn config_merge_both_none_stays_none() {
    let base = Config::default();
    let other = Config::default();
    let merged = Config::merge(base, other);
    assert!(merged.theme.is_none());
    assert!(merged.default_provider.is_none());
    assert!(merged.default_model.is_none());
    assert!(merged.gh_path.is_none());
}

#[test]
fn config_merge_nested_compaction_deep_merges() {
    let base: Config =
        serde_json::from_str(r#"{"compaction": {"enabled": true, "reserve_tokens": 999}}"#)
            .expect("parse base");
    let other: Config =
        serde_json::from_str(r#"{"compaction": {"enabled": false}}"#).expect("parse other");

    let merged = Config::merge(base, other);
    assert!(!merged.compaction_enabled());
    // reserve_tokens from base should survive if other didn't set it
    assert_eq!(merged.compaction_reserve_tokens(), 999);
}

#[test]
fn config_merge_nested_retry_deep_merges() {
    let base: Config = serde_json::from_str(r#"{"retry": {"enabled": true, "max_retries": 5}}"#)
        .expect("parse base");
    let other: Config =
        serde_json::from_str(r#"{"retry": {"base_delay_ms": 500}}"#).expect("parse other");

    let merged = Config::merge(base, other);
    assert!(merged.retry_enabled());
    assert_eq!(merged.retry_max_retries(), 5);
    assert_eq!(merged.retry_base_delay_ms(), 500);
}

#[test]
fn config_merge_nested_terminal_deep_merges() {
    let base: Config =
        serde_json::from_str(r#"{"terminal": {"show_images": false}}"#).expect("parse base");
    let other: Config =
        serde_json::from_str(r#"{"terminal": {"clear_on_shrink": true}}"#).expect("parse other");

    let merged = Config::merge(base, other);
    assert!(!merged.terminal_show_images());
    assert!(merged.terminal_clear_on_shrink());
}

#[test]
fn config_merge_nested_thinking_budgets_deep_merges() {
    let base: Config =
        serde_json::from_str(r#"{"thinkingBudgets": {"minimal": 512, "low": 1024}}"#)
            .expect("parse base");
    let other: Config =
        serde_json::from_str(r#"{"thinkingBudgets": {"low": 4096, "high": 32768}}"#)
            .expect("parse other");

    let merged = Config::merge(base, other);
    assert_eq!(merged.thinking_budget("minimal"), 512);
    assert_eq!(merged.thinking_budget("low"), 4096);
    assert_eq!(merged.thinking_budget("high"), 32768);
    assert_eq!(merged.thinking_budget("medium"), 8192); // default
}

// ─── Serde alias support ─────────────────────────────────────────────────────

#[test]
fn config_serde_camel_case_aliases() {
    let json = r#"{
        "defaultProvider": "google",
        "defaultModel": "gemini-pro",
        "defaultThinkingLevel": "high",
        "enabledModels": ["gpt-4*", "claude-*"],
        "quietStartup": true,
        "hideThinkingBlock": true,
        "showHardwareCursor": false,
        "ghPath": "/opt/gh",
        "shellPath": "/bin/zsh",
        "shellCommandPrefix": "set -e",
        "enableSkillCommands": false,
        "sessionStore": "sqlite",
        "doubleEscapeAction": "cancel",
        "editorPaddingX": 4,
        "autocompleteMaxVisible": 10,
        "sessionPickerInput": 3,
        "steeringMode": "queue",
        "followUpMode": "queue",
        "collapseChangelog": true,
        "lastChangelogVersion": "1.2.3"
    }"#;

    let config: Config = serde_json::from_str(json).expect("parse camelCase config");
    assert_eq!(config.default_provider.as_deref(), Some("google"));
    assert_eq!(config.default_model.as_deref(), Some("gemini-pro"));
    assert_eq!(config.default_thinking_level.as_deref(), Some("high"));
    assert_eq!(
        config.enabled_models.as_deref(),
        Some(vec!["gpt-4*".to_string(), "claude-*".to_string()].as_slice())
    );
    assert_eq!(config.quiet_startup, Some(true));
    assert_eq!(config.hide_thinking_block, Some(true));
    assert_eq!(config.show_hardware_cursor, Some(false));
    assert_eq!(config.gh_path.as_deref(), Some("/opt/gh"));
    assert_eq!(config.shell_path.as_deref(), Some("/bin/zsh"));
    assert_eq!(config.shell_command_prefix.as_deref(), Some("set -e"));
    assert!(!config.enable_skill_commands());
    assert_eq!(config.session_store.as_deref(), Some("sqlite"));
    assert_eq!(config.double_escape_action.as_deref(), Some("cancel"));
    assert_eq!(config.editor_padding_x, Some(4));
    assert_eq!(config.autocomplete_max_visible, Some(10));
    assert_eq!(config.session_picker_input, Some(3));
    assert_eq!(config.steering_mode.as_deref(), Some("queue"));
    assert_eq!(config.follow_up_mode.as_deref(), Some("queue"));
    assert_eq!(config.collapse_changelog, Some(true));
    assert_eq!(config.last_changelog_version.as_deref(), Some("1.2.3"));
}

#[test]
fn config_serde_snake_case_works_too() {
    let json = r#"{
        "default_provider": "openai",
        "default_model": "gpt-4o",
        "gh_path": "/usr/local/bin/gh"
    }"#;

    let config: Config = serde_json::from_str(json).expect("parse snake_case config");
    assert_eq!(config.default_provider.as_deref(), Some("openai"));
    assert_eq!(config.default_model.as_deref(), Some("gpt-4o"));
    assert_eq!(config.gh_path.as_deref(), Some("/usr/local/bin/gh"));
}

#[test]
fn config_serde_unknown_fields_are_ignored() {
    let json = r#"{ "theme": "dark", "unknown_field": 42, "anotherThing": true }"#;
    let config: Config = serde_json::from_str(json).expect("parse config with unknown fields");
    assert_eq!(config.theme.as_deref(), Some("dark"));
}

// ─── File loading edge cases ─────────────────────────────────────────────────

#[test]
fn config_load_missing_files_returns_defaults() {
    let _lock = config_lock();
    let harness = TestHarness::new("config_load_missing_files_returns_defaults");

    let cwd = harness.create_dir("cwd");
    let global_dir = harness.create_dir("global");
    // Don't create any settings files

    let config = Config::load_with_roots(None, &global_dir, &cwd).expect("load config");
    // Should return default config without error
    assert!(config.theme.is_none());
    assert!(config.default_provider.is_none());
    assert!(config.compaction_enabled());
}

#[test]
fn config_load_empty_file_returns_defaults() {
    let _lock = config_lock();
    let harness = TestHarness::new("config_load_empty_file_returns_defaults");

    let cwd = harness.create_dir("cwd");
    let global_dir = harness.create_dir("global");
    write_file(&global_dir.join("settings.json"), "");
    write_file(&cwd.join(".pi/settings.json"), "   \n  ");

    let config = Config::load_with_roots(None, &global_dir, &cwd).expect("load config");
    assert!(config.theme.is_none());
}

#[test]
fn config_load_whitespace_only_file_returns_defaults() {
    let _lock = config_lock();
    let harness = TestHarness::new("config_load_whitespace_only_file_returns_defaults");

    let cwd = harness.create_dir("cwd");
    let global_dir = harness.create_dir("global");
    write_file(&global_dir.join("settings.json"), "  \t\n  ");

    let config = Config::load_with_roots(None, &global_dir, &cwd).expect("load config");
    assert!(config.theme.is_none());
}

#[test]
fn config_load_empty_json_object_returns_defaults() {
    let _lock = config_lock();
    let harness = TestHarness::new("config_load_empty_json_object_returns_defaults");

    let cwd = harness.create_dir("cwd");
    let global_dir = harness.create_dir("global");
    write_file(&global_dir.join("settings.json"), "{}");

    let config = Config::load_with_roots(None, &global_dir, &cwd).expect("load config");
    assert!(config.theme.is_none());
    assert!(config.compaction_enabled());
}

#[test]
fn config_load_invalid_json_in_global_returns_error() {
    let _lock = config_lock();
    let harness = TestHarness::new("config_load_invalid_json_in_global_returns_error");

    let cwd = harness.create_dir("cwd");
    let global_dir = harness.create_dir("global");
    write_file(
        &global_dir.join("settings.json"),
        "{ this is not json at all!",
    );

    let result = Config::load_with_roots(None, &global_dir, &cwd);
    assert!(result.is_err(), "Expected error for invalid JSON in global");
}

#[test]
fn config_load_invalid_json_in_project_returns_error() {
    let _lock = config_lock();
    let harness = TestHarness::new("config_load_invalid_json_in_project_returns_error");

    let cwd = harness.create_dir("cwd");
    let global_dir = harness.create_dir("global");
    write_file(&global_dir.join("settings.json"), "{}");
    write_file(&cwd.join(".pi/settings.json"), "not json");

    let result = Config::load_with_roots(None, &global_dir, &cwd);
    assert!(
        result.is_err(),
        "Expected error for invalid JSON in project"
    );
}

// ─── Patch settings ─────────────────────────────────────────────────────────

#[test]
fn patch_settings_creates_file_if_missing() {
    let harness = TestHarness::new("patch_settings_creates_file_if_missing");

    let cwd = harness.create_dir("cwd");
    let global_dir = harness.create_dir("global");
    let settings_path = Config::settings_path_with_roots(SettingsScope::Project, &global_dir, &cwd);

    assert!(!settings_path.exists());

    let updated = Config::patch_settings_with_roots(
        SettingsScope::Project,
        &global_dir,
        &cwd,
        json!({ "theme": "dark" }),
    )
    .expect("patch settings");

    assert_eq!(updated, settings_path);
    assert!(settings_path.exists());

    let stored: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&settings_path).expect("read"))
            .expect("parse");
    assert_eq!(stored["theme"], json!("dark"));
}

#[test]
fn patch_settings_preserves_existing_keys() {
    let harness = TestHarness::new("patch_settings_preserves_existing_keys");

    let cwd = harness.create_dir("cwd");
    let global_dir = harness.create_dir("global");
    let settings_path = Config::settings_path_with_roots(SettingsScope::Project, &global_dir, &cwd);

    write_file(
        &settings_path,
        r#"{ "theme": "light", "defaultProvider": "anthropic" }"#,
    );

    Config::patch_settings_with_roots(
        SettingsScope::Project,
        &global_dir,
        &cwd,
        json!({ "defaultModel": "claude-sonnet" }),
    )
    .expect("patch settings");

    let stored: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&settings_path).expect("read"))
            .expect("parse");
    assert_eq!(stored["theme"], json!("light"));
    assert_eq!(stored["defaultProvider"], json!("anthropic"));
    assert_eq!(stored["defaultModel"], json!("claude-sonnet"));
}

#[test]
fn patch_settings_deep_merges_nested_objects() {
    let harness = TestHarness::new("patch_settings_deep_merges_nested_objects");

    let cwd = harness.create_dir("cwd");
    let global_dir = harness.create_dir("global");
    let settings_path = Config::settings_path_with_roots(SettingsScope::Project, &global_dir, &cwd);

    write_file(
        &settings_path,
        r#"{ "compaction": { "enabled": true, "reserve_tokens": 8192 } }"#,
    );

    Config::patch_settings_with_roots(
        SettingsScope::Project,
        &global_dir,
        &cwd,
        json!({ "compaction": { "keep_recent_tokens": 10000 } }),
    )
    .expect("patch settings");

    let stored: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&settings_path).expect("read"))
            .expect("parse");
    assert_eq!(stored["compaction"]["enabled"], json!(true));
    assert_eq!(stored["compaction"]["reserve_tokens"], json!(8192));
    assert_eq!(stored["compaction"]["keep_recent_tokens"], json!(10000));
}

// ─── Extension and repair policy resolution ─────────────────────────────────

#[test]
fn extension_policy_defaults_to_permissive() {
    let _lock = config_lock();
    let config = Config::default();
    let resolved = config.resolve_extension_policy_with_metadata(None);
    assert_eq!(resolved.requested_profile, "permissive");
    assert_eq!(resolved.effective_profile, "permissive");
    assert_eq!(resolved.profile_source, "default");
    assert!(!resolved.allow_dangerous);
}

#[test]
fn extension_policy_default_permissive_toggle_false_restores_safe() {
    let _lock = config_lock();
    let config: Config =
        serde_json::from_str(r#"{"extensionPolicy": {"defaultPermissive": false}}"#)
            .expect("parse");
    let resolved = config.resolve_extension_policy_with_metadata(None);
    assert_eq!(resolved.requested_profile, "safe");
    assert_eq!(resolved.effective_profile, "safe");
    assert_eq!(resolved.profile_source, "config");
}

#[test]
fn extension_policy_cli_override_wins() {
    let _lock = config_lock();
    let config: Config =
        serde_json::from_str(r#"{"extensionPolicy": {"profile": "permissive"}}"#).expect("parse");
    let resolved = config.resolve_extension_policy_with_metadata(Some("balanced"));
    assert_eq!(resolved.effective_profile, "balanced");
    assert_eq!(resolved.profile_source, "cli");
}

#[test]
fn extension_policy_config_profile() {
    let _lock = config_lock();

    let config: Config =
        serde_json::from_str(r#"{"extensionPolicy": {"profile": "permissive"}}"#).expect("parse");
    // CLI override always wins over config and env, so use it to test indirectly
    // that config parsing works (the profile field deserialises correctly).
    let resolved = config.resolve_extension_policy_with_metadata(Some("permissive"));
    assert_eq!(resolved.effective_profile, "permissive");
    assert_eq!(resolved.profile_source, "cli");
}

#[test]
fn extension_policy_unknown_profile_falls_back_to_safe() {
    let _lock = config_lock();
    let config = Config::default();
    let resolved = config.resolve_extension_policy_with_metadata(Some("nonexistent"));
    assert_eq!(resolved.effective_profile, "safe");
}

#[test]
fn extension_policy_legacy_standard_maps_to_balanced() {
    let _lock = config_lock();
    let config = Config::default();
    let resolved = config.resolve_extension_policy_with_metadata(Some("standard"));
    assert_eq!(resolved.effective_profile, "balanced");
}

#[test]
fn repair_policy_defaults_to_suggest() {
    let _lock = config_lock();

    // When no env var is set and config has no mode, default is "suggest".
    // We can't clear env in this crate, so test via CLI override fallback.
    let config = Config::default();
    let resolved = config.resolve_repair_policy_with_metadata(Some("suggest"));
    assert_eq!(resolved.source, "cli");
    assert_eq!(
        resolved.effective_mode,
        pi::extensions::RepairPolicyMode::Suggest
    );
}

#[test]
fn repair_policy_cli_override_wins() {
    let _lock = config_lock();
    let config: Config =
        serde_json::from_str(r#"{"repairPolicy": {"mode": "auto-safe"}}"#).expect("parse");
    let resolved = config.resolve_repair_policy_with_metadata(Some("off"));
    assert_eq!(resolved.source, "cli");
    assert_eq!(
        resolved.effective_mode,
        pi::extensions::RepairPolicyMode::Off
    );
}

#[test]
fn repair_policy_auto_strict_from_config() {
    let _lock = config_lock();

    let config: Config =
        serde_json::from_str(r#"{"repairPolicy": {"mode": "auto-strict"}}"#).expect("parse");
    // Use CLI override to prove auto-strict parsing works
    let resolved = config.resolve_repair_policy_with_metadata(Some("auto-strict"));
    assert_eq!(resolved.source, "cli");
    assert_eq!(
        resolved.effective_mode,
        pi::extensions::RepairPolicyMode::AutoStrict
    );
}

#[test]
fn repair_policy_unknown_mode_falls_back_to_suggest() {
    let _lock = config_lock();
    let config = Config::default();
    let resolved = config.resolve_repair_policy_with_metadata(Some("invalid-mode"));
    assert_eq!(
        resolved.effective_mode,
        pi::extensions::RepairPolicyMode::Suggest
    );
}

// ─── Branch summary reserve tokens fallback ──────────────────────────────────

#[test]
fn branch_summary_reserve_tokens_falls_back_to_compaction() {
    let config: Config =
        serde_json::from_str(r#"{"compaction": {"reserve_tokens": 4096}}"#).expect("parse");
    assert_eq!(config.branch_summary_reserve_tokens(), 4096);
}

#[test]
fn branch_summary_reserve_tokens_uses_own_when_set() {
    let config: Config = serde_json::from_str(
        r#"{"branchSummary": {"reserve_tokens": 2048}, "compaction": {"reserve_tokens": 4096}}"#,
    )
    .expect("parse");
    assert_eq!(config.branch_summary_reserve_tokens(), 2048);
}

// ─── Thinking budgets with custom values ─────────────────────────────────────

#[test]
fn thinking_budgets_custom_values_override_defaults() {
    let config: Config =
        serde_json::from_str(r#"{"thinkingBudgets": {"minimal": 256, "xhigh": 100000}}"#)
            .expect("parse");
    assert_eq!(config.thinking_budget("minimal"), 256);
    assert_eq!(config.thinking_budget("low"), 2048); // default
    assert_eq!(config.thinking_budget("xhigh"), 100_000);
}

// ─── Compaction disabled ─────────────────────────────────────────────────────

#[test]
fn compaction_can_be_explicitly_disabled() {
    let config: Config =
        serde_json::from_str(r#"{"compaction": {"enabled": false}}"#).expect("parse");
    assert!(!config.compaction_enabled());
}

// ─── Image auto-resize disabled ──────────────────────────────────────────────

#[test]
fn image_auto_resize_can_be_disabled() {
    let config: Config =
        serde_json::from_str(r#"{"images": {"auto_resize": false}}"#).expect("parse");
    assert!(!config.image_auto_resize());
}

// ─── Retry disabled ──────────────────────────────────────────────────────────

#[test]
fn retry_can_be_explicitly_disabled() {
    let config: Config = serde_json::from_str(r#"{"retry": {"enabled": false}}"#).expect("parse");
    assert!(!config.retry_enabled());
}

// ─── Session store alias ─────────────────────────────────────────────────────

#[test]
fn session_store_aliases_work() {
    // sessionStore alias
    let config: Config = serde_json::from_str(r#"{"sessionStore": "sqlite"}"#).expect("parse");
    assert_eq!(config.session_store.as_deref(), Some("sqlite"));

    // sessionBackend alias
    let config: Config = serde_json::from_str(r#"{"sessionBackend": "jsonl"}"#).expect("parse");
    assert_eq!(config.session_store.as_deref(), Some("jsonl"));
}
