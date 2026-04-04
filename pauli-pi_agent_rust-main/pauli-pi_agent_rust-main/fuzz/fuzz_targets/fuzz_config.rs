#![no_main]

//! Fuzz harness for config JSON deserialization surfaces.
//!
//! Exercises `Config` and key config sub-structs directly from arbitrary
//! UTF-8-ish payloads, including line-oriented and BOM-prefixed variants.

use libfuzzer_sys::fuzz_target;
use pi::config::{
    CompactionSettings, Config, ExtensionPolicyConfig, ExtensionRiskConfig, RepairPolicyConfig,
    ThinkingBudgets,
};

const MAX_INPUT_BYTES: usize = 64 * 1024;

fn fuzz_json(input: &str) {
    if input.is_empty() {
        return;
    }

    let config = serde_json::from_str::<Config>(input);
    let _ = serde_json::from_str::<ExtensionRiskConfig>(input);
    let _ = serde_json::from_str::<CompactionSettings>(input);
    let _ = serde_json::from_str::<ExtensionPolicyConfig>(input);
    let _ = serde_json::from_str::<RepairPolicyConfig>(input);
    let _ = serde_json::from_str::<ThinkingBudgets>(input);

    // Exercise accessor logic when full config parses.
    if let Ok(config) = config {
        let _ = config.compaction_enabled();
        let _ = config.compaction_reserve_tokens();
        let _ = config.compaction_keep_recent_tokens();
        let _ = config.retry_enabled();
        let _ = config.retry_max_retries();
        let _ = config.retry_base_delay_ms();
        let _ = config.retry_max_delay_ms();
        let _ = config.terminal_show_images();
        let _ = config.terminal_clear_on_shrink();
        let _ = config.image_auto_resize();
        let _ = config.thinking_budget("minimal");
        let _ = config.thinking_budget("high");
    }
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() || data.len() > MAX_INPUT_BYTES {
        return;
    }

    let lossy = String::from_utf8_lossy(data);

    // Whole payload.
    fuzz_json(&lossy);

    // Line-oriented parsing catches JSONL-style corruption and truncation.
    for line in lossy.lines().take(256) {
        fuzz_json(line.trim_end_matches('\r'));
    }

    // BOM-prefixed variant.
    let mut bom_prefixed = String::from("\u{feff}");
    bom_prefixed.push_str(&lossy);
    fuzz_json(&bom_prefixed);
});
