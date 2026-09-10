mod build_stage;
mod digest;
mod dynamic_stage;
mod judge_stage;
mod legitimacy;
mod prompt;
mod static_stage;

use anyhow::{bail, Context, Result};
use clap::Parser;
use omapak_core::{
    compute_verdict, render_markdown, Report, Rubric, StaticReport, Verdict, REPORT_SCHEMA_VERSION,
};
use std::path::PathBuf;

/// omapak submission judge — grade the artifact, not the authorship.
#[derive(Parser)]
struct Cli {
    /// App submission directory (contains the manifest + metadata.yml)
    app_dir: PathBuf,

    /// Resume from a partial report produced by an earlier phase
    /// (reuses its static/build results, runs judge + report only)
    #[arg(long)]
    partial: Option<PathBuf>,

    /// Local checkout of the upstream source repo to digest for the judge
    #[arg(long)]
    source_dir: Option<PathBuf>,

    /// Run the headless launch/screenshot stage (best-effort, off by default)
    #[arg(long)]
    with_dynamic: bool,

    /// Skip the flatpak build (local iteration; CI never uses this)
    #[arg(long)]
    skip_build: bool,

    /// Output directory for reports (default: omapak-out)
    #[arg(long, default_value = "omapak-out")]
    out: PathBuf,

    /// Build scratch directory (default: omapak-work). Kept separate from
    /// --out because flatpak build trees contain root-owned files that
    /// artifact uploaders cannot scan.
    #[arg(long, default_value = "omapak-work")]
    work: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let app_dir = &cli.app_dir;
    if !app_dir.is_dir() {
        bail!("{} is not a directory", app_dir.display());
    }

    let (static_report, build_report) = match &cli.partial {
        Some(path) => {
            let text = std::fs::read_to_string(path)
                .with_context(|| format!("read partial report {}", path.display()))?;
            let prior: Report = serde_json::from_str(&text).context("parse partial report")?;
            (prior.static_report, prior.build)
        }
        None => {
            eprintln!("→ static stage");
            let static_report = static_stage::run(app_dir, cli.source_dir.as_deref())?;
            eprintln!("→ build stage");
            let build_report = if cli.skip_build {
                eprintln!("  SKIPPED (--skip-build)");
                omapak_core::BuildReport {
                    ok: true,
                    duration_secs: 0,
                    log_tail: vec!["skipped: --skip-build".into()],
                }
            } else {
                let manifest = omapak_core::find_manifest(app_dir)
                    .context("no flatpak manifest found in app dir")?;
                let build = build_stage::run(
                    &manifest,
                    &cli.work.join("build"),
                    &cli.work.join("repo"),
                )?;
                for line in &build.log_tail {
                    eprintln!("  {line}");
                }
                build
            };
            (static_report, build_report)
        }
    };
    let build_ok = build_report.ok;

    let dynamic = if cli.with_dynamic {
        let manifest = omapak_core::find_manifest(app_dir);
        match manifest {
            Some(m) => {
                eprintln!("→ dynamic stage (best-effort)");
                let app_id = static_report
                    .manifest
                    .as_ref()
                    .map(|m| m.app_id.clone())
                    .unwrap_or_default();
                Some(dynamic_stage::run(&app_id, &cli.out.join("build"), &m, &cli.out))
            }
            None => None,
        }
    } else {
        None
    };

    let config = judge_stage::config_from_env()?;
    let (rubric, judge_info) = finish_judging(&cli, &static_report, build_ok, config.as_ref())?;

    // Proprietary submissions get a web legitimacy pass: does this thing
    // exist, is the channel real, is anything known-bad. Published with
    // sources; documented malware is a hard gate.
    let legitimacy = match (
        config.as_ref(),
        omapak_core::load_metadata(&cli.app_dir)
            .map(|m| m.source_access == omapak_core::SourceAccess::Proprietary),
    ) {
        (Some(cfg), Some(true)) => {
            eprintln!("→ legitimacy stage (web check)");
            let app_id = static_report
                .manifest
                .as_ref()
                .map(|m| m.app_id.clone())
                .unwrap_or_else(|| "unknown".into());
            let meta = omapak_core::load_metadata(&cli.app_dir)
                .context("re-read metadata for legitimacy")?;
            Some(legitimacy::run(cfg, &meta, &app_id).unwrap_or_else(|e| {
                eprintln!("  legitimacy stage failed (continuing): {e:#}");
                omapak_core::LegitimacyReport {
                    model: format!("{}:online", cfg.model),
                    summary: format!("legitimacy check failed to run: {e:#}"),
                    confidence: 0,
                    findings: vec![],
                }
            }))
        }
        _ => None,
    };

    // ONE gate: the build passed. Everything else is tags and scores.
    // The omapak Certified badge is computed from the rubric but never
    // gates publication.
    let _ = metadata_source_access(&cli.app_dir);

    let verdict = compute_verdict(
        rubric.as_ref().unwrap_or(&empty_rubric()),
        build_ok,
    );
    let certified = rubric.as_ref().is_some_and(|r| {
        omapak_core::is_certified(r, omapak_core::appstream_clean(&static_report))
    });

    let app_id = static_report
        .manifest
        .as_ref()
        .map(|m| m.app_id.clone())
        .unwrap_or_else(|| "unknown".into());

    let report = Report {
        schema_version: REPORT_SCHEMA_VERSION,
        app_id,
        created_at: chrono::Utc::now(),
        static_report,
        build: build_report,
        dynamic,
        rubric: rubric.clone(),
        verdict,
        certified,
        judge: judge_info,
        legitimacy,
    };

    std::fs::create_dir_all(&cli.out)?;
    let json_path = cli.out.join("report.json");
    let md_path = cli.out.join("report.md");
    std::fs::write(&json_path, serde_json::to_vec_pretty(&report)?)?;
    std::fs::write(&md_path, render_markdown(&report))?;

    eprintln!(
        "✦ {} → {}{} (report: {})",
        report.app_id,
        match verdict {
            Verdict::Published => "PUBLISHED",
            Verdict::BuildFailed => "BUILD FAILED",
        },
        if certified { " [CERTIFIED]" } else { "" },
        json_path.display()
    );
    Ok(())
}

/// Judge stage; assumes static + build already ran.
fn finish_judging(
    cli: &Cli,
    static_report: &StaticReport,
    build_ok: bool,
    config: Option<&judge_stage::JudgeConfig>,
) -> Result<(Option<Rubric>, Option<omapak_core::JudgeInfo>)> {
    let Some(config) = config else {
        eprintln!("→ judge stage SKIPPED (no OMAPAK_LLM_* env) — gates only");
        return Ok((None, None));
    };
    eprintln!("→ judge stage ({} @ {})", config.model, config.base_url);

    let metadata = omapak_core::load_metadata(&cli.app_dir);
    let source_digest = cli
        .source_dir
        .as_ref()
        .filter(|d| d.is_dir())
        .map(|d| digest::build(d));

    let manifest_summary = static_report.manifest.as_ref().map(|m| {
        serde_json::to_string_pretty(m).unwrap_or_else(|_| format!("app-id: {}", m.app_id))
    });
    // The full static report, not just linters: the agent needs to see
    // appstream presence, advisories, and source stats to judge honestly.
    let static_findings = serde_json::to_string_pretty(static_report).ok();

    let inputs = prompt::JudgeInputs {
        app_id: static_report
            .manifest
            .as_ref()
            .map(|m| m.app_id.as_str())
            .unwrap_or("unknown"),
        metadata: metadata.as_ref(),
        manifest_summary: manifest_summary.as_deref(),
        static_findings: static_findings.as_deref(),
        source_digest: source_digest.as_deref(),
        has_screenshots: false,
        build_ok,
    };

    let (rubric, judge_info) = judge_stage::run(config, &inputs)?;
    Ok((Some(rubric), Some(judge_info)))
}

fn metadata_source_access(app_dir: &std::path::Path) -> Option<omapak_core::SourceAccess> {
    omapak_core::load_metadata(app_dir).map(|m| m.source_access)
}

/// Neutral rubric for when the agent couldn't run; certification is false.
fn empty_rubric() -> Rubric {
    let score = omapak_core::RubricScore {
        score: 0,
        rationale: "agent did not run".into(),
    };
    Rubric {
        problem_clarity: omapak_core::RubricScore { ..score.clone() },
        differentiation: omapak_core::Differentiation {
            score: 0,
            rationale: "agent did not run".into(),
            better_alternatives: vec![],
        },
        architecture: omapak_core::RubricScore { ..score.clone() },
        code_quality: omapak_core::RubricScore { ..score.clone() },
        ui_ux: omapak_core::RubricScore { ..score.clone() },
        packaging_hygiene: omapak_core::RubricScore { ..score },
        security_flags: vec![],
    }
}
