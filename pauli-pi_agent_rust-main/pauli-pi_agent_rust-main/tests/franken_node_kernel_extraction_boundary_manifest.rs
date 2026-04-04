use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

const MANIFEST_PATH: &str = "docs/franken-node-kernel-extraction-boundary-manifest.json";
const EXPECTED_SCHEMA: &str = "pi.frankennode.kernel_extraction_boundary_manifest.v1";
const REQUIRED_CORE_MODULES: &[&str] = &[
    "src/agent_cx.rs",
    "src/scheduler.rs",
    "src/hostcall_queue.rs",
    "src/hostcall_amac.rs",
    "src/extensions.rs",
    "src/extensions_js.rs",
    "src/session.rs",
];
const REQUIRED_BANNED_CROSS_BOUNDARY_PAIRS: &[(&str, &str)] = &[
    ("hostcall_execution", "session_orchestration"),
    ("extension_runtime_js", "provider_runtime"),
];
const REQUIRED_REINTEGRATION_MAPPING_MODULES: &[&str] = &[
    "src/agent_cx.rs",
    "src/hostcall_queue.rs",
    "src/hostcall_amac.rs",
    "src/scheduler.rs",
    "src/extensions.rs",
    "src/extensions_js.rs",
    "src/session.rs",
];

#[derive(Debug)]
struct DomainBoundary {
    target_crate: String,
    current_modules: HashSet<String>,
    target_modules: HashSet<String>,
}

type ValidationResult<T> = std::result::Result<T, String>;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn load_manifest() -> Value {
    let path = repo_root().join(MANIFEST_PATH);
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

fn collect_module_ownership_index(manifest: &Value) -> ValidationResult<HashMap<String, String>> {
    let mut ownership = HashMap::new();
    for domain in as_array(manifest, "/boundary_domains") {
        let domain_id = domain
            .get("domain_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .ok_or_else(|| "every boundary domain must include non-empty domain_id".to_string())?;
        for module in as_array(domain, "/current_modules") {
            let module_path = module
                .as_str()
                .map(str::trim)
                .filter(|entry| !entry.is_empty())
                .ok_or_else(|| {
                    format!("every current_modules entry must be non-empty string in {domain_id}")
                })?;
            let prior = ownership.insert(module_path.to_string(), domain_id.to_string());
            if let Some(previous_owner) = prior {
                return Err(format!(
                    "module {module_path} appears in multiple ownership domains: {previous_owner} and {domain_id}"
                ));
            }
        }
    }
    Ok(ownership)
}

fn deferred_module_set(manifest: &Value) -> HashSet<String> {
    as_array(manifest, "/deferred_modules")
        .iter()
        .filter_map(|entry| {
            entry
                .get("module_path")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|path| !path.is_empty())
                .map(ToOwned::to_owned)
        })
        .collect()
}

fn validate_required_core_module_coverage(manifest: &Value) -> ValidationResult<()> {
    let ownership = collect_module_ownership_index(manifest)?;
    let deferred = deferred_module_set(manifest);

    for required_module in REQUIRED_CORE_MODULES {
        let required_module = *required_module;
        if !ownership.contains_key(required_module) && !deferred.contains(required_module) {
            return Err(format!(
                "required runtime module missing from boundary ownership/deferred maps: {required_module}"
            ));
        }
    }

    Ok(())
}

fn collect_banned_cross_boundary_pairs(
    manifest: &Value,
) -> ValidationResult<HashSet<(String, String)>> {
    let mut pairs = HashSet::new();
    for pair in as_array(manifest, "/ownership_rules/banned_cross_boundary_pairs") {
        let from_domain = pair
            .get("from_domain")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .ok_or_else(|| {
                "banned_cross_boundary_pairs entries must include non-empty from_domain".to_string()
            })?;
        let to_domain = pair
            .get("to_domain")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .ok_or_else(|| {
                "banned_cross_boundary_pairs entries must include non-empty to_domain".to_string()
            })?;

        let inserted = pairs.insert((from_domain.to_string(), to_domain.to_string()));
        if !inserted {
            return Err(format!(
                "duplicate banned cross-boundary pair detected: {from_domain}->{to_domain}"
            ));
        }
    }

    Ok(pairs)
}

fn validate_required_banned_cross_boundary_pairs(manifest: &Value) -> ValidationResult<()> {
    let pairs = collect_banned_cross_boundary_pairs(manifest)?;
    for (from_domain, to_domain) in REQUIRED_BANNED_CROSS_BOUNDARY_PAIRS {
        let required_pair = ((*from_domain).to_string(), (*to_domain).to_string());
        if !pairs.contains(&required_pair) {
            return Err(format!(
                "missing required banned cross-boundary pair: {from_domain}->{to_domain}"
            ));
        }
    }
    Ok(())
}

fn collect_domain_boundaries(
    manifest: &Value,
) -> ValidationResult<HashMap<String, DomainBoundary>> {
    let mut out = HashMap::new();
    for domain in as_array(manifest, "/boundary_domains") {
        let domain_id = domain
            .get("domain_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .ok_or_else(|| "every boundary domain must include non-empty domain_id".to_string())?;
        let target_crate = domain
            .get("target_crate")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .ok_or_else(|| {
                format!("boundary domain {domain_id} must define non-empty target_crate")
            })?
            .to_string();
        let current_modules = as_array(domain, "/current_modules")
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .map(ToOwned::to_owned)
            .collect::<HashSet<_>>();
        let target_modules = as_array(domain, "/target_modules")
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .map(ToOwned::to_owned)
            .collect::<HashSet<_>>();
        if current_modules.is_empty() {
            return Err(format!(
                "boundary domain {domain_id} must define at least one current_modules entry"
            ));
        }
        if target_modules.is_empty() {
            return Err(format!(
                "boundary domain {domain_id} must define at least one target_modules entry"
            ));
        }

        let inserted = out.insert(
            domain_id.to_string(),
            DomainBoundary {
                target_crate,
                current_modules,
                target_modules,
            },
        );
        if inserted.is_some() {
            return Err(format!("duplicate boundary domain detected: {domain_id}"));
        }
    }
    Ok(out)
}

#[allow(clippy::too_many_lines)]
fn validate_reintegration_module_mappings(manifest: &Value) -> ValidationResult<()> {
    let domain_boundaries = collect_domain_boundaries(manifest)?;
    let replacement_targets =
        non_empty_string_set(manifest, "/reintegration_linkage/replacement_targets");
    let mappings = as_array(manifest, "/reintegration_linkage/module_mappings");
    if mappings.is_empty() {
        return Err("reintegration_linkage.module_mappings must not be empty".to_string());
    }

    let mut seen_sources = HashSet::new();
    let mut seen_replacements = HashSet::new();
    for mapping in mappings {
        let source_module = mapping
            .get("source_module")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .ok_or_else(|| "module mapping must define non-empty source_module".to_string())?;
        let source_domain = mapping
            .get("source_domain")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .ok_or_else(|| {
                format!("module mapping for {source_module} must define source_domain")
            })?;
        let target_crate = mapping
            .get("target_crate")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .ok_or_else(|| {
                format!("module mapping for {source_module} must define target_crate")
            })?;
        let target_module = mapping
            .get("target_module")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .ok_or_else(|| {
                format!("module mapping for {source_module} must define target_module")
            })?;
        let replacement_target = mapping
            .get("replacement_target")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .ok_or_else(|| {
                format!("module mapping for {source_module} must define replacement_target")
            })?;

        if !seen_sources.insert(source_module.to_string()) {
            return Err(format!(
                "duplicate module mapping source_module detected: {source_module}"
            ));
        }
        if !seen_replacements.insert(replacement_target.to_string()) {
            return Err(format!(
                "duplicate module mapping replacement_target detected: {replacement_target}"
            ));
        }
        if source_module != replacement_target {
            return Err(format!(
                "module mapping must preserve source->replacement identity for deterministic reintegration: source={source_module}, replacement={replacement_target}"
            ));
        }
        if !replacement_targets.contains(replacement_target) {
            return Err(format!(
                "module mapping replacement_target not declared in reintegration replacement_targets: {replacement_target}"
            ));
        }

        let boundary = domain_boundaries.get(source_domain).ok_or_else(|| {
            format!("module mapping references unknown source_domain: {source_domain}")
        })?;
        if boundary.target_crate != target_crate {
            return Err(format!(
                "module mapping target_crate mismatch for {source_module}: expected {}, got {target_crate}",
                boundary.target_crate
            ));
        }
        if !boundary.current_modules.contains(source_module) {
            return Err(format!(
                "module mapping source_module not present in boundary domain {source_domain}: {source_module}"
            ));
        }
        if !boundary.target_modules.contains(target_module) {
            return Err(format!(
                "module mapping target_module not declared in boundary domain {source_domain}: {target_module}"
            ));
        }
    }

    for required in REQUIRED_REINTEGRATION_MAPPING_MODULES {
        if !seen_sources.contains(*required) {
            return Err(format!(
                "missing required reintegration module mapping source: {required}"
            ));
        }
    }
    for target in replacement_targets {
        if !seen_replacements.contains(target.as_str()) {
            return Err(format!(
                "replacement target missing explicit module mapping: {target}"
            ));
        }
    }
    Ok(())
}

fn domain_entry_mut<'a>(manifest: &'a mut Value, domain_id: &str) -> &'a mut Value {
    let domains = manifest
        .get_mut("boundary_domains")
        .and_then(Value::as_array_mut)
        .expect("manifest boundary_domains must be mutable array");
    domains
        .iter_mut()
        .find(|domain| {
            domain
                .get("domain_id")
                .and_then(Value::as_str)
                .map(str::trim)
                == Some(domain_id)
        })
        .unwrap_or_else(|| panic!("missing boundary domain for mutation: {domain_id}"))
}

fn remove_string_entry(entries: &mut Vec<Value>, needle: &str) -> bool {
    let before = entries.len();
    entries.retain(|entry| entry.as_str().map(str::trim) != Some(needle));
    before != entries.len()
}

fn remove_banned_pair(manifest: &mut Value, from_domain: &str, to_domain: &str) -> bool {
    let pairs = manifest
        .pointer_mut("/ownership_rules/banned_cross_boundary_pairs")
        .and_then(Value::as_array_mut)
        .expect("ownership_rules.banned_cross_boundary_pairs must be mutable array");
    let before = pairs.len();
    pairs.retain(|pair| {
        let from = pair
            .get("from_domain")
            .and_then(Value::as_str)
            .map(str::trim);
        let to = pair.get("to_domain").and_then(Value::as_str).map(str::trim);
        !(from == Some(from_domain) && to == Some(to_domain))
    });
    before != pairs.len()
}

fn mapping_entry_mut<'a>(manifest: &'a mut Value, source_module: &str) -> &'a mut Value {
    let mappings = manifest
        .pointer_mut("/reintegration_linkage/module_mappings")
        .and_then(Value::as_array_mut)
        .expect("reintegration_linkage.module_mappings must be mutable array");
    mappings
        .iter_mut()
        .find(|entry| {
            entry
                .get("source_module")
                .and_then(Value::as_str)
                .map(str::trim)
                == Some(source_module)
        })
        .unwrap_or_else(|| panic!("missing module mapping for mutation: {source_module}"))
}

fn remove_module_mapping(manifest: &mut Value, source_module: &str) -> bool {
    let mappings = manifest
        .pointer_mut("/reintegration_linkage/module_mappings")
        .and_then(Value::as_array_mut)
        .expect("reintegration_linkage.module_mappings must be mutable array");
    let before = mappings.len();
    mappings.retain(|entry| {
        entry
            .get("source_module")
            .and_then(Value::as_str)
            .map(str::trim)
            != Some(source_module)
    });
    before != mappings.len()
}

#[test]
fn kernel_boundary_manifest_exists_and_is_valid_json() {
    let path = repo_root().join(MANIFEST_PATH);
    assert!(
        path.is_file(),
        "missing kernel extraction boundary manifest artifact: {}",
        path.display()
    );
    let _ = load_manifest();
}

#[test]
fn kernel_boundary_manifest_has_expected_schema_and_linkage() {
    let manifest = load_manifest();
    assert_eq!(
        manifest["schema"],
        Value::String(EXPECTED_SCHEMA.to_string())
    );

    let version = manifest["contract_version"]
        .as_str()
        .expect("contract_version must be present");
    assert!(
        parse_semver(version).is_some(),
        "contract_version must be semantic version x.y.z, got: {version}"
    );

    assert_eq!(
        manifest["bead_id"],
        Value::String("bd-3ar8v.7.2".to_string())
    );
    assert_eq!(
        manifest["support_bead_id"],
        Value::String("bd-3ar8v.7.2.1".to_string())
    );
    let support_bead_ids = non_empty_string_set(&manifest, "/support_bead_ids");
    for required in ["bd-3ar8v.7.2.1", "bd-3ar8v.7.2.3"] {
        assert!(
            support_bead_ids.contains(required),
            "support_bead_ids missing required linkage {required}"
        );
    }
    assert_eq!(
        manifest["target_project_root"],
        Value::String("/dp/franken_node".to_string())
    );
}

#[test]
fn kernel_boundary_manifest_covers_required_core_modules() {
    let manifest = load_manifest();
    validate_required_core_module_coverage(&manifest).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn kernel_boundary_manifest_domain_entries_are_complete() {
    let manifest = load_manifest();
    let domains = as_array(&manifest, "/boundary_domains");
    assert!(
        domains.len() >= 6,
        "boundary_domains should define at least six extraction ownership domains"
    );

    for domain in domains {
        let domain_id = domain
            .get("domain_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .expect("domain_id must be present");
        assert!(!domain_id.is_empty(), "domain_id must be non-empty");

        let target_crate = domain
            .get("target_crate")
            .and_then(Value::as_str)
            .map(str::trim)
            .expect("target_crate must be present");
        assert!(
            !target_crate.is_empty(),
            "target_crate must be non-empty for domain {domain_id}"
        );

        let current_modules = as_array(domain, "/current_modules");
        assert!(
            !current_modules.is_empty(),
            "current_modules must not be empty for domain {domain_id}"
        );

        let target_modules = as_array(domain, "/target_modules");
        assert!(
            !target_modules.is_empty(),
            "target_modules must not be empty for domain {domain_id}"
        );

        let invariants = as_array(domain, "/invariants");
        assert!(
            !invariants.is_empty(),
            "invariants must not be empty for domain {domain_id}"
        );

        let forbidden = as_array(domain, "/forbidden_cross_boundary_refs");
        assert!(
            !forbidden.is_empty(),
            "forbidden_cross_boundary_refs must not be empty for domain {domain_id}"
        );
    }
}

#[test]
fn kernel_boundary_manifest_enforces_fail_closed_ownership_rules() {
    let manifest = load_manifest();
    let ownership_rules = &manifest["ownership_rules"];
    assert_eq!(
        ownership_rules["require_full_module_coverage"].as_bool(),
        Some(true),
        "ownership_rules.require_full_module_coverage must be true"
    );
    assert_eq!(
        ownership_rules["disallow_duplicate_module_ownership"].as_bool(),
        Some(true),
        "ownership_rules.disallow_duplicate_module_ownership must be true"
    );
    assert_eq!(
        ownership_rules["require_explicit_deferred_modules"].as_bool(),
        Some(true),
        "ownership_rules.require_explicit_deferred_modules must be true"
    );

    let banned_pairs = as_array(ownership_rules, "/banned_cross_boundary_pairs");
    assert!(
        !banned_pairs.is_empty(),
        "ownership_rules.banned_cross_boundary_pairs must not be empty"
    );
    for pair in banned_pairs {
        for required in ["from_domain", "to_domain", "reason"] {
            let value = pair
                .get(required)
                .and_then(Value::as_str)
                .map_or("", str::trim);
            assert!(
                !value.is_empty(),
                "banned_cross_boundary_pairs entries must include non-empty {required}"
            );
        }
    }
    validate_required_banned_cross_boundary_pairs(&manifest).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn kernel_boundary_manifest_deferred_modules_are_explicit_and_actionable() {
    let manifest = load_manifest();
    let deferred = as_array(&manifest, "/deferred_modules");
    assert!(
        !deferred.is_empty(),
        "deferred_modules must be non-empty when require_explicit_deferred_modules is true"
    );
    for entry in deferred {
        let module_path = entry
            .get("module_path")
            .and_then(Value::as_str)
            .map_or("", str::trim);
        assert!(
            module_path.starts_with("src/")
                && std::path::Path::new(module_path)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("rs")),
            "deferred module_path must be a src/*.rs path: {module_path}"
        );

        let reason = entry
            .get("reason")
            .and_then(Value::as_str)
            .map_or("", str::trim);
        assert!(
            !reason.is_empty(),
            "deferred entry reason must be non-empty"
        );

        let follow_up_bead = entry
            .get("follow_up_bead")
            .and_then(Value::as_str)
            .map_or("", str::trim);
        assert!(
            follow_up_bead.starts_with("bd-"),
            "deferred entry follow_up_bead must look like bead id, got: {follow_up_bead}"
        );
    }
}

#[test]
fn kernel_boundary_manifest_declares_drift_checks_reintegration_and_logging_fields() {
    let manifest = load_manifest();

    let checks = non_empty_string_set(&manifest, "/drift_detection_contract/required_checks");
    for required in [
        "kernel_boundary.all_modules_mapped_or_deferred",
        "kernel_boundary.no_duplicate_domain_ownership",
        "kernel_boundary.banned_cross_boundary_pairs_absent",
        "kernel_boundary.reintegration_target_list_complete",
        "kernel_boundary.reintegration_module_mappings_complete",
    ] {
        assert!(
            checks.contains(required),
            "drift_detection_contract.required_checks missing {required}"
        );
    }

    assert_eq!(
        manifest["drift_detection_contract"]["failure_policy"],
        Value::String("hard_fail".to_string()),
        "drift_detection_contract.failure_policy must be hard_fail"
    );

    assert_eq!(
        manifest["reintegration_linkage"]["required_bead"],
        Value::String("bd-3ar8v.7.13".to_string()),
        "reintegration linkage must point to bd-3ar8v.7.13"
    );

    let replacement_targets =
        non_empty_string_set(&manifest, "/reintegration_linkage/replacement_targets");
    for required in ["src/agent_cx.rs", "src/hostcall_queue.rs", "src/session.rs"] {
        assert!(
            replacement_targets.contains(required),
            "reintegration replacement_targets missing {required}"
        );
    }
    assert_eq!(
        manifest["reintegration_linkage"]["require_explicit_module_mappings"].as_bool(),
        Some(true),
        "reintegration_linkage.require_explicit_module_mappings must be true"
    );
    assert_eq!(
        manifest["reintegration_linkage"]["failure_policy"],
        Value::String("hard_fail".to_string()),
        "reintegration_linkage.failure_policy must be hard_fail"
    );
    validate_reintegration_module_mappings(&manifest).unwrap_or_else(|err| panic!("{err}"));

    let logging_fields =
        non_empty_string_set(&manifest, "/structured_logging_contract/required_fields");
    for required in [
        "run_id",
        "domain_id",
        "module_path",
        "decision",
        "reason",
        "timestamp_utc",
    ] {
        assert!(
            logging_fields.contains(required),
            "structured_logging_contract.required_fields missing {required}"
        );
    }
}

#[test]
fn kernel_boundary_manifest_duplicate_ownership_mutation_fails_closed() {
    let mut manifest = load_manifest();
    let target_domain = domain_entry_mut(&mut manifest, "hostcall_queueing");
    let target_modules = target_domain
        .get_mut("current_modules")
        .and_then(Value::as_array_mut)
        .expect("hostcall_queueing.current_modules must be mutable array");
    target_modules.push(Value::String("src/scheduler.rs".to_string()));

    let err = collect_module_ownership_index(&manifest)
        .expect_err("duplicate module ownership mutation must fail validation");
    assert!(
        err.contains("src/scheduler.rs"),
        "error should name duplicated module, got: {err}"
    );
}

#[test]
fn kernel_boundary_manifest_missing_required_mapping_mutation_fails_closed() {
    let mut manifest = load_manifest();
    let domain = domain_entry_mut(&mut manifest, "context_and_diagnostics");
    let modules = domain
        .get_mut("current_modules")
        .and_then(Value::as_array_mut)
        .expect("context_and_diagnostics.current_modules must be mutable array");
    assert!(
        remove_string_entry(modules, "src/agent_cx.rs"),
        "mutation setup should remove src/agent_cx.rs from current_modules"
    );

    let err = validate_required_core_module_coverage(&manifest)
        .expect_err("missing required module mapping must fail validation");
    assert!(
        err.contains("src/agent_cx.rs"),
        "error should reference missing required module, got: {err}"
    );
}

#[test]
fn kernel_boundary_manifest_banned_pair_drift_mutation_fails_closed() {
    let mut manifest = load_manifest();
    assert!(
        remove_banned_pair(&mut manifest, "extension_runtime_js", "provider_runtime"),
        "mutation setup should remove required banned cross-boundary pair"
    );

    let err = validate_required_banned_cross_boundary_pairs(&manifest)
        .expect_err("missing required banned pair must fail validation");
    assert!(
        err.contains("extension_runtime_js->provider_runtime"),
        "error should reference missing banned pair, got: {err}"
    );
}

#[test]
fn kernel_boundary_manifest_banned_pair_typo_mutation_fails_closed() {
    let mut manifest = load_manifest();
    let pairs = manifest
        .pointer_mut("/ownership_rules/banned_cross_boundary_pairs")
        .and_then(Value::as_array_mut)
        .expect("ownership_rules.banned_cross_boundary_pairs must be mutable array");
    let first_pair = pairs
        .first_mut()
        .expect("ownership_rules.banned_cross_boundary_pairs must have at least one pair");
    first_pair["to_domain"] = Value::String("session_orchestration_typo".to_string());

    let err = validate_required_banned_cross_boundary_pairs(&manifest)
        .expect_err("drifted banned pair domain typo must fail validation");
    assert!(
        err.contains("hostcall_execution->session_orchestration"),
        "error should reference required banned pair contract, got: {err}"
    );
}

#[test]
fn kernel_boundary_manifest_missing_module_mapping_mutation_fails_closed() {
    let mut manifest = load_manifest();
    assert!(
        remove_module_mapping(&mut manifest, "src/session.rs"),
        "mutation setup should remove module mapping for src/session.rs"
    );

    let err = validate_reintegration_module_mappings(&manifest)
        .expect_err("missing required module mapping must fail validation");
    assert!(
        err.contains("src/session.rs"),
        "error should reference missing required mapping, got: {err}"
    );
}

#[test]
fn kernel_boundary_manifest_mapping_target_crate_drift_mutation_fails_closed() {
    let mut manifest = load_manifest();
    let mapping = mapping_entry_mut(&mut manifest, "src/hostcall_queue.rs");
    mapping["target_crate"] = Value::String("franken-kernel-hostcall-queue-typo".to_string());

    let err = validate_reintegration_module_mappings(&manifest)
        .expect_err("drifted target crate in mapping must fail validation");
    assert!(
        err.contains("target_crate mismatch") && err.contains("src/hostcall_queue.rs"),
        "error should reference target_crate mismatch for src/hostcall_queue.rs, got: {err}"
    );
}
