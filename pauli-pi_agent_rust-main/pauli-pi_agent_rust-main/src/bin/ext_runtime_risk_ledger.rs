#![forbid(unsafe_code)]
#![allow(clippy::needless_pass_by_value)]

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};
use pi::extensions::{
    RuntimeRiskCalibrationConfig, RuntimeRiskCalibrationObjective, RuntimeRiskLedgerArtifact,
    calibrate_runtime_risk_from_ledger, replay_runtime_risk_ledger_artifact,
    verify_runtime_risk_ledger_artifact,
};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(name = "ext_runtime_risk_ledger")]
#[command(about = "Verify, replay, and calibrate runtime risk ledger artifacts")]
struct Cli {
    #[command(subcommand)]
    command: CommandMode,
}

#[derive(Debug, Subcommand)]
enum CommandMode {
    /// Verify a runtime risk ledger hash-chain artifact.
    Verify(VerifyArgs),
    /// Reconstruct deterministic decision path from a verified ledger artifact.
    Replay(ReplayArgs),
    /// Run deterministic threshold calibration against a verified ledger artifact.
    Calibrate(CalibrateArgs),
}

#[derive(Debug, Args)]
struct VerifyArgs {
    /// Input ledger artifact JSON file.
    #[arg(long)]
    input: PathBuf,
    /// Optional output path for JSON report (stdout when omitted).
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct ReplayArgs {
    /// Input ledger artifact JSON file.
    #[arg(long)]
    input: PathBuf,
    /// Optional output path for JSON replay artifact (stdout when omitted).
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "snake_case")]
enum ObjectiveArg {
    MinExpectedLoss,
    MinFalsePositives,
    BalancedAccuracy,
}

impl From<ObjectiveArg> for RuntimeRiskCalibrationObjective {
    fn from(value: ObjectiveArg) -> Self {
        match value {
            ObjectiveArg::MinExpectedLoss => Self::MinExpectedLoss,
            ObjectiveArg::MinFalsePositives => Self::MinFalsePositives,
            ObjectiveArg::BalancedAccuracy => Self::BalancedAccuracy,
        }
    }
}

#[derive(Debug, Args)]
struct CalibrateArgs {
    /// Input ledger artifact JSON file.
    #[arg(long)]
    input: PathBuf,
    /// Optional output path for JSON calibration report (stdout when omitted).
    #[arg(long)]
    output: Option<PathBuf>,
    /// Calibration objective.
    #[arg(long, value_enum, default_value_t = ObjectiveArg::BalancedAccuracy)]
    objective: ObjectiveArg,
    /// Baseline threshold used to compute recommended delta.
    #[arg(long, default_value_t = 0.65)]
    baseline_threshold: f64,
    /// Minimum threshold candidate to evaluate.
    #[arg(long, default_value_t = 0.05)]
    min_threshold: f64,
    /// Maximum threshold candidate to evaluate.
    #[arg(long, default_value_t = 0.95)]
    max_threshold: f64,
    /// Threshold step size for candidate grid.
    #[arg(long, default_value_t = 0.05)]
    step: f64,
    /// False-positive penalty multiplier.
    #[arg(long, default_value_t = 1.0)]
    false_positive_weight: f64,
    /// False-negative penalty multiplier.
    #[arg(long, default_value_t = 1.0)]
    false_negative_weight: f64,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        CommandMode::Verify(args) => run_verify(args),
        CommandMode::Replay(args) => run_replay(args),
        CommandMode::Calibrate(args) => run_calibrate(args),
    }
}

fn run_verify(args: VerifyArgs) -> Result<()> {
    let artifact = read_ledger_artifact(&args.input)?;
    let report = verify_runtime_risk_ledger_artifact(&artifact);
    write_json(args.output.as_deref(), &report)?;
    if !report.valid {
        bail!(
            "runtime risk ledger integrity verification failed ({} errors)",
            report.errors.len()
        );
    }
    Ok(())
}

fn run_replay(args: ReplayArgs) -> Result<()> {
    let artifact = read_ledger_artifact(&args.input)?;
    let replay = replay_runtime_risk_ledger_artifact(&artifact)
        .context("runtime risk ledger replay failed integrity checks")?;
    write_json(args.output.as_deref(), &replay)
}

fn run_calibrate(args: CalibrateArgs) -> Result<()> {
    let artifact = read_ledger_artifact(&args.input)?;
    let config = RuntimeRiskCalibrationConfig {
        objective: RuntimeRiskCalibrationObjective::from(args.objective),
        baseline_threshold: args.baseline_threshold,
        threshold_grid: calibration_threshold_grid(
            args.min_threshold,
            args.max_threshold,
            args.step,
        )?,
        false_positive_weight: args.false_positive_weight,
        false_negative_weight: args.false_negative_weight,
    };
    let report = calibrate_runtime_risk_from_ledger(&artifact, &config)?;
    write_json(args.output.as_deref(), &report)
}

fn read_ledger_artifact(path: &Path) -> Result<RuntimeRiskLedgerArtifact> {
    let raw = fs::read_to_string(path).with_context(|| {
        format!(
            "failed to read runtime risk ledger artifact {}",
            path.display()
        )
    })?;
    serde_json::from_str(&raw).with_context(|| {
        format!(
            "failed to parse runtime risk ledger artifact {}",
            path.display()
        )
    })
}

fn write_json(path: Option<&Path>, value: &impl Serialize) -> Result<()> {
    let payload = serde_json::to_string_pretty(value)
        .context("failed to serialize runtime risk JSON payload")?;
    if let Some(path) = path {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create parent directories for output {}",
                    path.display()
                )
            })?;
        }
        fs::write(path, payload).with_context(|| format!("failed to write {}", path.display()))?;
    } else {
        println!("{payload}");
    }
    Ok(())
}

fn calibration_threshold_grid(
    min_threshold: f64,
    max_threshold: f64,
    step: f64,
) -> Result<Vec<f64>> {
    if !min_threshold.is_finite() || !max_threshold.is_finite() || !step.is_finite() {
        bail!("threshold bounds and step must be finite");
    }
    if step <= 0.0 {
        bail!("step must be > 0");
    }
    let min = min_threshold.clamp(0.0, 1.0);
    let max = max_threshold.clamp(0.0, 1.0);
    if min > max {
        bail!("min_threshold must be <= max_threshold");
    }

    let mut thresholds = Vec::new();
    let mut current = min;
    let max_with_margin = step.mul_add(0.25, max);
    loop {
        if current > max_with_margin {
            break;
        }
        thresholds.push(current.clamp(0.0, 1.0));
        current += step;
        if thresholds.len() > 10_000 {
            bail!("threshold grid is too large (>10,000 points)");
        }
    }

    if thresholds.is_empty() {
        thresholds.push(min);
    }
    thresholds.sort_by(f64::total_cmp);
    thresholds.dedup_by(|left, right| left.total_cmp(right).is_eq());
    Ok(thresholds)
}
