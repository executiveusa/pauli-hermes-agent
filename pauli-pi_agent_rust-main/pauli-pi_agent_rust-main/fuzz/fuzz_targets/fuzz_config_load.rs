#![no_main]

//! Fuzz harness for file-based config loading and patching paths.
//!
//! Exercises `Config::load_with_roots` with fuzzed settings files and, when
//! possible, `Config::patch_settings_with_roots` for merge/serialization paths.

use libfuzzer_sys::fuzz_target;
use pi::config::{Config, SettingsScope};

const MAX_INPUT_BYTES: usize = 64 * 1024;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() || data.len() > MAX_INPUT_BYTES {
        return;
    }

    let Ok(tmp) = tempfile::tempdir() else {
        return;
    };

    let cwd = tmp.path().join("cwd");
    let global = tmp.path().join("global");
    let project_settings = cwd.join(".pi/settings.json");
    let global_settings = global.join("settings.json");
    let override_path = tmp.path().join("override.json");

    let _ = std::fs::create_dir_all(cwd.join(".pi"));
    let _ = std::fs::create_dir_all(&global);

    // Override-only load path.
    let _ = std::fs::write(&override_path, data);
    let _ = Config::load_with_roots(Some(&override_path), &global, &cwd);

    // Global + project merge path.
    let _ = std::fs::write(&global_settings, data);
    let _ = std::fs::write(&project_settings, data);
    let _ = Config::load_with_roots(None, &global, &cwd);

    // Patch paths when input is valid JSON.
    if let Ok(text) = std::str::from_utf8(data) {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
            let _ = Config::patch_settings_with_roots(
                SettingsScope::Project,
                &global,
                &cwd,
                value.clone(),
            );
            let _ = Config::patch_settings_with_roots(SettingsScope::Global, &global, &cwd, value);
        }
    }
});
