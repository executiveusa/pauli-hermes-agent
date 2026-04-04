use pi::extension_scoring::{
    OpeEvaluatorConfig, OpeGateReason, OpeTraceSample, evaluate_off_policy,
};

fn sample(
    action: &str,
    behavior_propensity: f64,
    target_propensity: f64,
    outcome: f64,
    baseline_outcome: f64,
    direct_method_prediction: f64,
) -> OpeTraceSample {
    OpeTraceSample {
        action: action.to_string(),
        behavior_propensity,
        target_propensity,
        outcome,
        baseline_outcome: Some(baseline_outcome),
        direct_method_prediction: Some(direct_method_prediction),
        context_lineage: Some(format!("ctx:{action}")),
    }
}

const fn permissive_thresholds() -> OpeEvaluatorConfig {
    OpeEvaluatorConfig {
        max_importance_weight: 100.0,
        min_effective_sample_size: 1.0,
        max_standard_error: 10.0,
        confidence_z: 1.96,
        max_regret_delta: 10.0,
    }
}

#[test]
fn ope_gate_no_valid_samples_integration() {
    let config = OpeEvaluatorConfig::default();
    let samples = vec![
        sample("invalid-a", 0.0, 0.5, 1.0, 1.0, 1.0),
        sample("invalid-b", -1.0, 0.5, 1.0, 1.0, 1.0),
    ];

    let report = evaluate_off_policy(&samples, &config);
    assert_eq!(report.gate.reason, OpeGateReason::NoValidSamples);
    assert!(!report.gate.passed);
    assert_eq!(report.diagnostics.valid_samples, 0);
}

#[test]
fn ope_gate_insufficient_support_integration() {
    let config = OpeEvaluatorConfig {
        min_effective_sample_size: 4.0,
        ..permissive_thresholds()
    };

    let mut samples = vec![sample("candidate", 0.02, 1.0, 0.0, 0.0, 0.0)];
    for _ in 0..9 {
        samples.push(sample("candidate", 1.0, 0.02, 1.0, 1.0, 1.0));
    }

    let report = evaluate_off_policy(&samples, &config);
    assert_eq!(report.gate.reason, OpeGateReason::InsufficientSupport);
    assert!(!report.gate.passed);
    assert!(report.diagnostics.effective_sample_size < 2.0);
}

#[test]
fn ope_gate_high_uncertainty_integration() {
    let config = OpeEvaluatorConfig {
        max_standard_error: 0.05,
        ..permissive_thresholds()
    };
    let samples = (0..20)
        .map(|idx| {
            let outcome = if idx % 2 == 0 { 0.0 } else { 1.0 };
            sample("uncertain", 0.5, 0.5, outcome, outcome, outcome)
        })
        .collect::<Vec<_>>();

    let report = evaluate_off_policy(&samples, &config);
    assert_eq!(report.gate.reason, OpeGateReason::HighUncertainty);
    assert!(!report.gate.passed);
    assert!(report.doubly_robust.standard_error > config.max_standard_error);
}

#[test]
fn ope_gate_excessive_regret_integration() {
    let config = OpeEvaluatorConfig {
        max_regret_delta: 0.1,
        ..permissive_thresholds()
    };
    let samples = (0..16)
        .map(|_| sample("regretful", 0.5, 0.5, 0.0, 1.0, 0.0))
        .collect::<Vec<_>>();

    let report = evaluate_off_policy(&samples, &config);
    assert_eq!(report.gate.reason, OpeGateReason::ExcessiveRegret);
    assert!(!report.gate.passed);
    assert!(report.estimated_regret_delta > config.max_regret_delta);
}

#[test]
fn ope_gate_approved_integration() {
    let config = OpeEvaluatorConfig {
        max_regret_delta: 0.25,
        ..permissive_thresholds()
    };
    let samples = vec![
        sample("approved", 0.5, 0.5, 0.9, 0.85, 0.9),
        sample("approved", 0.5, 0.5, 0.8, 0.75, 0.8),
        sample("approved", 0.5, 0.5, 0.7, 0.65, 0.7),
        sample("approved", 0.5, 0.5, 0.95, 0.9, 0.95),
        sample("approved", 0.5, 0.5, 0.85, 0.8, 0.85),
        sample("approved", 0.5, 0.5, 0.75, 0.7, 0.75),
    ];

    let report = evaluate_off_policy(&samples, &config);
    assert_eq!(report.gate.reason, OpeGateReason::Approved);
    assert!(report.gate.passed);
    assert_eq!(report.diagnostics.valid_samples, samples.len());
}
