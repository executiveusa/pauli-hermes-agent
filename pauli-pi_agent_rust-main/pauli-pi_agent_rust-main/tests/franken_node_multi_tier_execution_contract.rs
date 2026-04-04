use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;

const CONTRACT_PATH: &str = "docs/franken-node-multi-tier-execution-contract.json";
const EXPECTED_SCHEMA: &str = "pi.frankennode.multi_tier_execution_contract.v1";
const REQUIRED_TIER_IDS: &[&str] = &[
    "tier_interp_baseline",
    "tier_superinstruction_fused",
    "tier_trace_jit_guarded",
];
const REQUIRED_PROMOTION_SIGNALS: &[&str] = &[
    "hotness_score",
    "payoff_estimate",
    "stability_score",
    "compile_budget_available",
];
const REQUIRED_DEOPT_STATE_FIELDS: &[&str] = &[
    "value_stack",
    "call_stack",
    "scope_chain",
    "register_map",
    "program_counter",
];
const REQUIRED_TELEMETRY_FIELDS: &[&str] = &[
    "run_id",
    "correlation_id",
    "tier_from",
    "tier_to",
    "reason",
    "compile_cost_ns",
    "hit_rate",
    "deopt_count",
    "timestamp_utc",
];
const REQUIRED_EVENT_TYPES: &[&str] = &[
    "tier_promotion",
    "tier_demotion",
    "tier_deopt",
    "kill_switch_engaged",
];
const REQUIRED_BLOCKED_BEADS: &[&str] = &["bd-3ar8v.7.10", "bd-3ar8v.7.11"];
const REQUIRED_SUPPORT_BEAD_IDS: &[&str] = &["bd-3ar8v.7.6.1", "bd-3ar8v.7.6.2", "bd-3ar8v.7.6.3"];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn load_contract() -> Value {
    let path = repo_root().join(CONTRACT_PATH);
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|err| panic!("failed to parse {} as JSON: {err}", path.display()))
}

fn parse_semver(version: &str) -> Option<(u64, u64, u64)> {
    let mut parts = version.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next()?.parse::<u64>().ok()?;
    let patch = parts.next()?.parse::<u64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

fn as_array<'a>(value: &'a Value, pointer: &str) -> &'a [Value] {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .map_or_else(
            || panic!("expected JSON array at pointer {pointer}"),
            Vec::as_slice,
        )
}

fn non_empty_string_set(value: &Value, pointer: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    for entry in as_array(value, pointer) {
        let raw = entry
            .as_str()
            .unwrap_or_else(|| panic!("expected string entry at {pointer}"));
        let normalized = raw.trim();
        assert!(
            !normalized.is_empty(),
            "entry at {pointer} must be non-empty"
        );
        out.insert(normalized.to_string());
    }
    out
}

type ValidationResult<T> = std::result::Result<T, String>;

fn validate_required_set(
    contract: &Value,
    pointer: &str,
    required_values: &[&str],
    label: &str,
) -> ValidationResult<()> {
    let values = non_empty_string_set(contract, pointer);
    for required in required_values {
        if !values.contains(*required) {
            return Err(format!("missing {label}: {required}"));
        }
    }
    Ok(())
}

fn collect_tier_ids(contract: &Value) -> ValidationResult<HashSet<String>> {
    let tiers = as_array(contract, "/tier_contract");
    if tiers.is_empty() {
        return Err("tier_contract must not be empty".to_string());
    }

    let mut tier_ids = HashSet::new();
    for row in tiers {
        let tier_id = row
            .get("tier_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .ok_or_else(|| "tier_contract row missing tier_id".to_string())?;
        if !tier_ids.insert(tier_id.to_string()) {
            return Err(format!("duplicate tier_id detected: {tier_id}"));
        }

        let kind = row
            .get("kind")
            .and_then(Value::as_str)
            .map(str::trim)
            .ok_or_else(|| format!("{tier_id}: kind must be present"))?;
        if !matches!(kind, "interpreter" | "superinstruction" | "trace_jit") {
            return Err(format!("{tier_id}: invalid kind {kind}"));
        }

        let eligible_workload_classes = row
            .get("eligible_workload_classes")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("{tier_id}: eligible_workload_classes must be an array"))?;
        if eligible_workload_classes.is_empty() {
            return Err(format!(
                "{tier_id}: eligible_workload_classes must not be empty"
            ));
        }
    }

    for row in tiers {
        let tier_id = row
            .get("tier_id")
            .and_then(Value::as_str)
            .map_or("<missing>", str::trim);

        for target in row
            .get("promotion_targets")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("{tier_id}: promotion_targets must be an array"))?
        {
            let target = target
                .as_str()
                .map(str::trim)
                .filter(|entry| !entry.is_empty())
                .ok_or_else(|| format!("{tier_id}: promotion target must be non-empty"))?;
            if !tier_ids.contains(target) {
                return Err(format!(
                    "{tier_id}: promotion_targets references unknown tier_id {target}"
                ));
            }
        }

        if let Some(degrade_to) = row.get("degrade_to") {
            let degrade_to = degrade_to
                .as_str()
                .map(str::trim)
                .filter(|entry| !entry.is_empty())
                .ok_or_else(|| format!("{tier_id}: degrade_to must be non-empty when present"))?;
            if !tier_ids.contains(degrade_to) {
                return Err(format!(
                    "{tier_id}: degrade_to references unknown tier_id {degrade_to}"
                ));
            }
        }
    }

    Ok(tier_ids)
}

fn validate_promotion_policy(contract: &Value) -> ValidationResult<()> {
    validate_required_set(
        contract,
        "/promotion_policy/required_signals",
        REQUIRED_PROMOTION_SIGNALS,
        "promotion required signal",
    )?;

    if contract["promotion_policy"]["fail_closed_on_missing_signals"] != Value::Bool(true) {
        return Err("promotion_policy.fail_closed_on_missing_signals must be true".to_string());
    }

    let hotness = contract["promotion_policy"]["minimum_hotness_threshold"]
        .as_f64()
        .ok_or_else(|| "minimum_hotness_threshold must be numeric".to_string())?;
    let stability = contract["promotion_policy"]["minimum_stability_threshold"]
        .as_f64()
        .ok_or_else(|| "minimum_stability_threshold must be numeric".to_string())?;
    if !(0.0 < hotness && hotness <= 1.0) {
        return Err("minimum_hotness_threshold must be in (0.0, 1.0]".to_string());
    }
    if !(0.0 < stability && stability <= 1.0) {
        return Err("minimum_stability_threshold must be in (0.0, 1.0]".to_string());
    }

    let compile_budget = contract["promotion_policy"]["compile_budget_window_ms"]
        .as_u64()
        .ok_or_else(|| "compile_budget_window_ms must be a positive integer".to_string())?;
    if compile_budget == 0 {
        return Err("compile_budget_window_ms must be > 0".to_string());
    }

    Ok(())
}

fn validate_deopt_and_safety_contract(contract: &Value) -> ValidationResult<()> {
    validate_required_set(
        contract,
        "/deoptimization_contract/required_state_fields",
        REQUIRED_DEOPT_STATE_FIELDS,
        "deoptimization state field",
    )?;

    if contract["deoptimization_contract"]["lossless_recovery_required"] != Value::Bool(true) {
        return Err("deoptimization_contract.lossless_recovery_required must be true".to_string());
    }
    if contract["deoptimization_contract"]["immediate_fallback_tier"]
        != Value::String("tier_interp_baseline".to_string())
    {
        return Err(
            "deoptimization_contract.immediate_fallback_tier must be tier_interp_baseline"
                .to_string(),
        );
    }

    if contract["safety_contract"]["deterministic_fallback_on_policy_ambiguity"]
        != Value::Bool(true)
    {
        return Err(
            "safety_contract.deterministic_fallback_on_policy_ambiguity must be true".to_string(),
        );
    }
    if contract["safety_contract"]["disallow_implicit_capability_escalation"] != Value::Bool(true) {
        return Err(
            "safety_contract.disallow_implicit_capability_escalation must be true".to_string(),
        );
    }
    if contract["safety_contract"]["jit_kill_switch"]["must_hard_disable_trace_jit"]
        != Value::Bool(true)
    {
        return Err(
            "safety_contract.jit_kill_switch.must_hard_disable_trace_jit must be true".to_string(),
        );
    }

    Ok(())
}

fn validate_telemetry_and_dependencies(contract: &Value) -> ValidationResult<()> {
    validate_required_set(
        contract,
        "/telemetry_contract/required_fields",
        REQUIRED_TELEMETRY_FIELDS,
        "telemetry required field",
    )?;
    validate_required_set(
        contract,
        "/telemetry_contract/required_event_types",
        REQUIRED_EVENT_TYPES,
        "telemetry event type",
    )?;
    validate_required_set(
        contract,
        "/support_bead_ids",
        REQUIRED_SUPPORT_BEAD_IDS,
        "support_bead_ids entry",
    )?;
    validate_required_set(
        contract,
        "/downstream_dependencies/blocked_beads",
        REQUIRED_BLOCKED_BEADS,
        "blocked bead",
    )
}

fn remove_string_entry(contract: &mut Value, pointer: &str, value: &str) -> bool {
    let entries = contract
        .pointer_mut(pointer)
        .and_then(Value::as_array_mut)
        .unwrap_or_else(|| panic!("expected mutable array at pointer {pointer}"));
    let before = entries.len();
    entries.retain(|entry| entry.as_str().map(str::trim) != Some(value));
    before != entries.len()
}

fn remove_tier(contract: &mut Value, tier_id: &str) -> bool {
    let rows = contract
        .pointer_mut("/tier_contract")
        .and_then(Value::as_array_mut)
        .expect("tier_contract must be mutable array");
    let before = rows.len();
    rows.retain(|row| row.get("tier_id").and_then(Value::as_str).map(str::trim) != Some(tier_id));
    before != rows.len()
}

fn set_kill_switch_flag(contract: &mut Value, enabled: bool) {
    let field = contract
        .pointer_mut("/safety_contract/jit_kill_switch/must_hard_disable_trace_jit")
        .expect("jit kill-switch flag must be mutable");
    *field = Value::Bool(enabled);
}

fn set_hotness_threshold(contract: &mut Value, threshold: f64) {
    let field = contract
        .pointer_mut("/promotion_policy/minimum_hotness_threshold")
        .expect("promotion_policy.minimum_hotness_threshold must be mutable");
    *field = Value::from(threshold);
}

#[test]
fn multi_tier_execution_contract_exists_and_is_valid_json() {
    let path = repo_root().join(CONTRACT_PATH);
    assert!(
        path.is_file(),
        "missing multi-tier execution contract artifact: {}",
        path.display()
    );
    let _ = load_contract();
}

#[test]
fn multi_tier_execution_contract_has_expected_schema_version_and_linkage() {
    let contract = load_contract();
    assert_eq!(
        contract["schema"],
        Value::String(EXPECTED_SCHEMA.to_string()),
        "multi-tier execution contract schema mismatch"
    );

    let version = contract["contract_version"]
        .as_str()
        .expect("contract_version must be present");
    assert!(
        parse_semver(version).is_some(),
        "contract_version must be semantic version x.y.z, got: {version}"
    );

    assert_eq!(
        contract["bead_id"],
        Value::String("bd-3ar8v.7.6".to_string()),
        "bead linkage must target bd-3ar8v.7.6"
    );
    assert_eq!(
        contract["support_bead_id"],
        Value::String("bd-3ar8v.7.6.3".to_string()),
        "support bead linkage must target bd-3ar8v.7.6.3"
    );
}

#[test]
fn multi_tier_execution_contract_tier_map_is_complete_and_unique() {
    let contract = load_contract();
    let tier_ids = collect_tier_ids(&contract).unwrap_or_else(|err| panic!("{err}"));
    for required in REQUIRED_TIER_IDS {
        assert!(
            tier_ids.contains(*required),
            "missing required tier_id: {required}"
        );
    }
}

#[test]
fn multi_tier_execution_contract_promotion_policy_is_fail_closed() {
    let contract = load_contract();
    validate_promotion_policy(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn multi_tier_execution_contract_deopt_and_safety_contracts_are_complete() {
    let contract = load_contract();
    validate_deopt_and_safety_contract(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn multi_tier_execution_contract_telemetry_and_dependency_contracts_are_complete() {
    let contract = load_contract();
    validate_telemetry_and_dependencies(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn multi_tier_execution_contract_fails_closed_when_required_tier_missing() {
    let mut contract = load_contract();
    assert!(
        remove_tier(&mut contract, "tier_trace_jit_guarded"),
        "mutation should remove required tier"
    );

    let err = collect_tier_ids(&contract)
        .and_then(|tier_ids| {
            for required in REQUIRED_TIER_IDS {
                if !tier_ids.contains(*required) {
                    return Err(format!("missing required tier_id: {required}"));
                }
            }
            Ok(())
        })
        .expect_err("contract should fail when required tier is removed");
    assert!(
        err.contains("tier_trace_jit_guarded"),
        "expected error to reference removed tier, got: {err}"
    );
}

#[test]
fn multi_tier_execution_contract_fails_closed_when_promotion_signal_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/promotion_policy/required_signals",
            "compile_budget_available",
        ),
        "mutation should remove required promotion signal"
    );

    let err = validate_promotion_policy(&contract)
        .expect_err("contract should fail when required promotion signal is removed");
    assert!(
        err.contains("compile_budget_available"),
        "expected error to reference missing signal, got: {err}"
    );
}

#[test]
fn multi_tier_execution_contract_fails_closed_when_kill_switch_invariant_disabled() {
    let mut contract = load_contract();
    set_kill_switch_flag(&mut contract, false);

    let err = validate_deopt_and_safety_contract(&contract)
        .expect_err("contract should fail when kill-switch hard-disable invariant is false");
    assert!(
        err.contains("must_hard_disable_trace_jit"),
        "expected error to reference kill-switch invariant, got: {err}"
    );
}

#[test]
fn multi_tier_execution_contract_fails_closed_when_hotness_threshold_invalid() {
    let mut contract = load_contract();
    set_hotness_threshold(&mut contract, 1.5);

    let err = validate_promotion_policy(&contract)
        .expect_err("contract should fail when hotness threshold is outside policy bounds");
    assert!(
        err.contains("minimum_hotness_threshold"),
        "expected error to reference hotness threshold, got: {err}"
    );
}

#[test]
fn multi_tier_execution_contract_fails_closed_when_deopt_state_field_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/deoptimization_contract/required_state_fields",
            "register_map",
        ),
        "mutation should remove required deoptimization state field"
    );

    let err = validate_deopt_and_safety_contract(&contract)
        .expect_err("contract should fail when required deoptimization state field is removed");
    assert!(
        err.contains("register_map"),
        "expected error to reference missing deoptimization field, got: {err}"
    );
}

#[test]
fn multi_tier_execution_contract_fails_closed_when_blocked_bead_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/downstream_dependencies/blocked_beads",
            "bd-3ar8v.7.11",
        ),
        "mutation should remove required blocked bead"
    );

    let err = validate_telemetry_and_dependencies(&contract)
        .expect_err("contract should fail when blocked bead linkage is missing");
    assert!(
        err.contains("bd-3ar8v.7.11"),
        "expected error to reference missing blocked bead, got: {err}"
    );
}
