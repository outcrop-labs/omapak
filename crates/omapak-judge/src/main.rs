mod build_stage;
mod digest;
mod dynamic_stage;
mod judge_stage;
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

    /// Output directory (default: omapak-out)
    #[arg(long, default_value = "omapak-out")]
    out: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let app_dir = &cli.app_dir;
    if !app_dir.is_dir() {
        bail!("{} is not a directory", app_dir.display());
    }

    let (static_report, build_ok) = match &cli.partial {
        Some(path) => {
            let text = std::fs::read_to_string(path)
                .with_context(|| format!("read partial report {}", path.display()))?;
            let prior: Report = serde_json::from_str(&text).context("parse partial report")?;
            (prior.static_report, prior.build.ok)
        }
        None => {
            eprintln!("→ static stage");
            let static_report = static_stage::run(app_dir, cli.source_dir.as_deref())?;
            let build_ok = if cli.skip_build {
                eprintln!("→ build stage SKIPPED (--skip-build)");
                true
            } else {
                eprintln!("→ build stage");
                let manifest = omapak_core::find_manifest(app_dir)
                    .context("no flatpak manifest found in app dir")?;
                let build = build_stage::run(
                    &manifest,
                    &cli.out.join("build"),
                    &cli.out.join("repo"),
                )?;
                for line in &build.log_tail {
                    eprintln!("  {line}");
                }
                build.ok
            };
            (static_report, build_ok)
        }
    };

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
    let (rubric, judge_info, build_report) =
        finish_judging(&cli, &static_report, build_ok, config.as_ref())?;

    // Deterministic gates: build passed, appstream valid, submitter metadata
    // present. Store-presence matters; blank tiles in software stores don't
    // ship from here.
    let gates_ok = build_ok
        && omapak_core::appstream_clean(&static_report)
        && static_report.metadata_present;

    // Closed source ships only through owner-assisted review: until a
    // maintainer runs the judge against the real source, the verdict can
    // not exceed needs-human, no matter what the rubric says.
    let assisted_pending = metadata_source_access(&cli.app_dir)
        == Some(omapak_core::SourceAccess::PrivateAssisted)
        && cli.source_dir.is_none();

    let mut verdict = match &rubric {
        Some(r) => compute_verdict(r, gates_ok),
        None => gates_only_verdict(&static_report, gates_ok),
    };
    if assisted_pending && verdict == Verdict::AcceptRecommended {
        verdict = Verdict::NeedsHuman;
    }

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
        build: build_report.unwrap_or(omapak_core::BuildReport {
            ok: build_ok,
            duration_secs: 0,
            log_tail: vec!["skipped: --skip-build or resumed from partial".into()],
        }),
        dynamic,
        rubric: rubric.clone(),
        verdict,
        judge: judge_info,
    };

    std::fs::create_dir_all(&cli.out)?;
    let json_path = cli.out.join("report.json");
    let md_path = cli.out.join("report.md");
    std::fs::write(&json_path, serde_json::to_vec_pretty(&report)?)?;
    std::fs::write(&md_path, render_markdown(&report))?;

    eprintln!(
        "✦ {} → {} (report: {})",
        report.app_id,
        match verdict {
            Verdict::AcceptRecommended => "ACCEPT recommended",
            Verdict::NeedsHuman => "NEEDS HUMAN",
            Verdict::RejectRecommended => "REJECT recommended",
        },
        json_path.display()
    );
    Ok(())
}

/// Judge (and, when resuming from a partial, reconstruct the build report).
fn finish_judging(
    cli: &Cli,
    static_report: &StaticReport,
    build_ok: bool,
    config: Option<&judge_stage::JudgeConfig>,
) -> Result<(Option<Rubric>, Option<omapak_core::JudgeInfo>, Option<omapak_core::BuildReport>)> {
    let build_report = if cli.partial.is_some() || cli.skip_build {
        Some(omapak_core::BuildReport {
            ok: build_ok,
            duration_secs: 0,
            log_tail: vec!["recorded in an earlier phase".into()],
        })
    } else {
        None
    };

    let Some(config) = config else {
        eprintln!("→ judge stage SKIPPED (no OMAPAK_LLM_* env) — gates only");
        return Ok((None, None, build_report));
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
    let static_findings = serde_json::to_string_pretty(&static_report.linters).ok();

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
    Ok((Some(rubric), Some(judge_info), build_report))
}

fn metadata_source_access(app_dir: &std::path::Path) -> Option<omapak_core::SourceAccess> {
    omapak_core::load_metadata(app_dir).map(|m| m.source_access)
}

/// Without an agent rubric; only the deterministic gates speak.
fn gates_only_verdict(static_report: &StaticReport, gates_ok: bool) -> Verdict {
    if !gates_ok || static_report.manifest.is_none() {
        return Verdict::RejectRecommended;
    }
    let hard_lint_fail = static_report.linters.iter().any(|l| {
        l.status == omapak_core::LinterStatus::Failed
            && l.tool == "flatpak-builder-lint"
            && l.findings.iter().any(|f| f.contains("error"))
    });
    if hard_lint_fail {
        return Verdict::RejectRecommended;
    }
    Verdict::NeedsHuman
}
