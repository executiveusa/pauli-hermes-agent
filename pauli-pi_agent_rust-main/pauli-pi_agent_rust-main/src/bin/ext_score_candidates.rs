#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use clap::Parser;
use pi::extension_popularity::{CandidateItem as PopularityCandidateItem, CandidatePool};
use pi::extension_scoring::{
    CandidateInput, CompatStatus, Compatibility, Gates, LicenseInfo, MarketplaceSignals, Recency,
    Redistribution, RiskInfo, Signals, Tags, score_candidates,
};
use serde::Deserialize;

#[derive(Debug, Parser)]
#[command(name = "ext_score_candidates")]
#[command(about = "Score extension candidates and emit a ranked list")]
struct Args {
    #[arg(long)]
    input: PathBuf,
    #[arg(long)]
    out: PathBuf,
    #[arg(long)]
    summary_out: Option<PathBuf>,
    #[arg(long)]
    as_of: Option<String>,
    #[arg(long)]
    generated_at: Option<String>,
    #[arg(long, default_value_t = 10)]
    top_n: usize,
    #[arg(long, default_value_t = false)]
    check: bool,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum InputFile {
    List(Vec<CandidateInput>),
    Document(InputDocument),
    CandidatePool(CandidatePool),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InputDocument {
    generated_at: Option<String>,
    candidates: Vec<CandidateInput>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let bytes = fs::read(&args.input).with_context(|| format!("read {}", args.input.display()))?;
    let input: InputFile =
        serde_json::from_slice(&bytes).context("parse candidate scoring input")?;

    let (candidates, embedded_generated_at) = match input {
        InputFile::List(list) => (list, None),
        InputFile::Document(doc) => (doc.candidates, doc.generated_at),
        InputFile::CandidatePool(pool) => (
            pool.items
                .iter()
                .map(candidate_from_popularity_item)
                .collect(),
            Some(pool.generated_at),
        ),
    };

    let as_of = parse_timestamp(args.as_of, "as_of")?.unwrap_or_else(Utc::now);
    let generated_at = parse_timestamp(args.generated_at, "generated_at")?
        .or_else(|| {
            parse_timestamp(embedded_generated_at, "generated_at")
                .ok()
                .flatten()
        })
        .unwrap_or(as_of);

    let report = score_candidates(&candidates, as_of, generated_at, args.top_n);
    let json = serde_json::to_string_pretty(&report).context("serialize report")?;
    let json = format!("{json}\n");

    if args.check {
        match fs::read_to_string(&args.out) {
            Ok(existing) => {
                if existing != json {
                    bail!("Generated report differs from {}", args.out.display());
                }
            }
            Err(_) => bail!("Missing output file: {}", args.out.display()),
        }
    } else {
        fs::write(&args.out, json).with_context(|| format!("write {}", args.out.display()))?;
    }

    if let Some(summary_path) = args.summary_out {
        let summary_json =
            serde_json::to_string_pretty(&report.summary).context("serialize summary")?;
        fs::write(&summary_path, format!("{summary_json}\n"))
            .with_context(|| format!("write {}", summary_path.display()))?;
    }

    Ok(())
}

fn candidate_from_popularity_item(item: &PopularityCandidateItem) -> CandidateInput {
    let signals = Signals {
        official_listing: Some(item.source_tier == "official-pi-mono"),
        pi_mono_example: Some(item.source_tier == "official-pi-mono"),
        badlogic_gist: item
            .repository_url
            .as_deref()
            .map(|url| url.contains("gist.github.com") && url.contains("badlogic"))
            .or(Some(false)),
        github_stars: item.popularity.github_stars,
        github_forks: item.popularity.github_forks,
        npm_downloads_month: item.popularity.npm_downloads_monthly,
        references: item.popularity.mentions_sources.clone().unwrap_or_default(),
        marketplace: Some(MarketplaceSignals {
            rank: item.popularity.marketplace_rank,
            installs_month: item.popularity.marketplace_installs_monthly,
            featured: item.popularity.marketplace_featured,
        }),
    };

    let runtime = if item.source_tier == "npm-registry" {
        Some("pkg-with-deps".to_string())
    } else {
        Some("legacy-js".to_string())
    };
    let tags = Tags {
        runtime,
        ..Tags::default()
    };

    let recency = Recency {
        updated_at: item
            .popularity
            .github_last_commit
            .clone()
            .or_else(|| item.popularity.npm_last_publish.clone())
            .or_else(|| item.retrieved.clone()),
    };

    let compat = Compatibility {
        status: match item.status.as_str() {
            "unvendored" | "excluded" => Some(CompatStatus::Blocked),
            _ => Some(CompatStatus::RequiresShims),
        },
        ..Compatibility::default()
    };

    let license = LicenseInfo {
        spdx: Some(item.license.clone()),
        redistribution: Some(infer_redistribution(&item.license)),
        notes: None,
    };

    let gates = Gates {
        provenance_pinned: Some(item.checksum.is_some()),
        deterministic: Some(item.status != "unvendored"),
    };

    CandidateInput {
        id: item.id.clone(),
        name: Some(item.name.clone()),
        source_tier: Some(item.source_tier.clone()),
        signals,
        tags,
        recency,
        compat,
        license,
        gates,
        risk: RiskInfo::default(),
        manual_override: None,
    }
}

fn infer_redistribution(license: &str) -> Redistribution {
    let normalized = license.trim().to_ascii_uppercase();
    if normalized.is_empty() || matches!(normalized.as_str(), "UNKNOWN" | "UNLICENSED") {
        return Redistribution::Unknown;
    }
    if normalized.contains("GPL") || normalized.contains("AGPL") {
        return Redistribution::Restricted;
    }
    Redistribution::Ok
}

fn parse_timestamp(value: Option<String>, label: &str) -> Result<Option<DateTime<Utc>>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let parsed =
        DateTime::parse_from_rfc3339(&value).with_context(|| format!("parse {label} timestamp"))?;
    Ok(Some(parsed.with_timezone(&Utc)))
}
