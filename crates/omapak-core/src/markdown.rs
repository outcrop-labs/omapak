use crate::schema::{RubricScore, Severity, Verdict};

/// Render a report as the PR comment. Scores are shown with a bar so a skim
/// reads the shape of the app before any prose does.
pub fn render_markdown(report: &crate::schema::Report) -> String {
    let mut out = String::new();
    out.push_str(&format!("## omapak judge · `{}`\n\n", report.app_id));

    out.push_str(&format!("**Verdict: {}**\n\n", verdict_label(report.verdict)));

    if !report.build.ok {
        out.push_str("**Packaging gate FAILED.** The flatpak did not build. ");
        out.push_str("Everything below is advisory context for fixing it.\n\n");
    }

    if report.build.ok && !crate::schema::appstream_clean(&report.static_report) {
        out.push_str("**Appstream gate FAILED.** Metainfo is missing or has validation errors; ");
        out.push_str("the app would render as a blank tile in software stores. Fix and resubmit.\n\n");
    }

    if let Some(rubric) = &report.rubric {
        out.push_str("| dimension | score | rationale |\n|---|---|---|\n");
        out.push_str(&rubric_row("problem clarity", &rubric.problem_clarity));
        out.push_str(&rubric_row("architecture", &rubric.architecture));
        out.push_str(&rubric_row("code quality", &rubric.code_quality));
        out.push_str(&rubric_row("UI/UX", &rubric.ui_ux));
        out.push_str(&rubric_row("packaging hygiene (gate ≥ 2)", &rubric.packaging_hygiene));
        out.push_str(&format!(
            "| differentiation (advisory, never gates) | {} | {} |\n",
            bar(rubric.differentiation.score),
            escape_table(&rubric.differentiation.rationale)
        ));
        out.push('\n');
        if !rubric.differentiation.better_alternatives.is_empty() {
            out.push_str("**Better existing solutions** (informational, does not gate):\n");
            for alt in &rubric.differentiation.better_alternatives {
                out.push_str(&format!("- {alt}\n"));
            }
            out.push('\n');
        }
        if !rubric.security_flags.is_empty() {
            out.push_str("**Security flags**\n\n| severity | detail |\n|---|---|\n");
            for flag in &rubric.security_flags {
                out.push_str(&format!(
                    "| {} | {} |\n",
                    match flag.severity {
                        Severity::Critical => "**CRITICAL**",
                        Severity::Warning => "warning",
                        Severity::Info => "info",
                    },
                    escape_table(&flag.detail)
                ));
            }
            out.push('\n');
        }
    } else {
        out.push_str("_No rubric. Judge stage was skipped (no LLM endpoint configured). Gates only._\n\n");
    }

    let s = &report.static_report;
    if !s.advisories.is_empty() {
        out.push_str("**Manifest advisories**\n");
        for a in &s.advisories {
            out.push_str(&format!("- `{}`: {}\n", a.kind, a.detail));
        }
        out.push('\n');
    }
    for run in &s.linters {
        if !run.findings.is_empty() {
            out.push_str(&format!("**{} findings**\n", run.tool));
            for f in &run.findings {
                out.push_str(&format!("- {}\n", escape_table(f)));
            }
            out.push('\n');
        }
    }
    if let Some(judge) = &report.judge {
        out.push_str(&format!(
            "_judged by `{}` in {}s · prompt v{} · schema v{} · merge is a human decision_\n",
            judge.model,
            judge.duration_secs,
            judge.prompt_version,
            report.schema_version
        ));
    }
    out
}

fn verdict_label(v: Verdict) -> &'static str {
    match v {
        Verdict::AcceptRecommended => "✅ ACCEPT recommended",
        Verdict::NeedsHuman => "🧑 NEEDS HUMAN review",
        Verdict::RejectRecommended => "🛑 REJECT recommended",
    }
}

fn rubric_row(name: &str, s: &RubricScore) -> String {
    format!("| {} | {} | {} |\n", name, bar(s.score), escape_table(&s.rationale))
}

fn bar(score: u8) -> String {
    let score = score.min(5);
    format!("{} {}/5", "■".repeat(score as usize) + &"□".repeat((5 - score) as usize), score)
}

fn escape_table(text: &str) -> String {
    text.replace('|', "\\|").replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{Differentiation, Report, Rubric};

    fn report() -> Report {
        Report {
            schema_version: 1,
            app_id: "io.outcroplabs.TestApp".into(),
            created_at: chrono::Utc::now(),
            static_report: Default::default(),
            build: crate::schema::BuildReport {
                ok: true,
                duration_secs: 30,
                log_tail: vec![],
            },
            dynamic: None,
            rubric: Some(Rubric {
                problem_clarity: RubricScore { score: 4, rationale: "clear".into() },
                differentiation: Differentiation {
                    score: 2,
                    rationale: "clone of existing".into(),
                    better_alternatives: vec!["com.example.Better".into()],
                },
                architecture: RubricScore { score: 3, rationale: "fine".into() },
                code_quality: RubricScore { score: 3, rationale: "ok | has pipes".into() },
                ui_ux: RubricScore { score: 4, rationale: "nice".into() },
                packaging_hygiene: RubricScore { score: 5, rationale: "clean".into() },
                security_flags: vec![],
            }),
            verdict: Verdict::AcceptRecommended,
            judge: None,
        }
    }

    #[test]
    fn markdown_escapes_and_renders() {
        let md = render_markdown(&report());
        assert!(md.contains("\\|"));
        assert!(md.contains("com.example.Better"));
        assert!(md.contains("■■■□□ 3/5"));
        assert!(md.contains("never gates"));
    }
}
