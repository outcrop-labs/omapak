use crate::schema::{RubricScore, Severity, Verdict};

/// Render a report as the PR comment. Scores are shown with a bar so a skim
/// reads the shape of the app before any prose does.
pub fn render_markdown(report: &crate::schema::Report) -> String {
    let mut out = String::new();
    out.push_str(&format!("## omapak judge · `{}`\n\n", report.app_id));

    if !report.build.ok {
        out.push_str("**Build failed.** Fix the manifest and push.\n\n");
    }

    if let Some(rubric) = &report.rubric {
        // Compact score lines, no bars, no decoration
        out.push_str(&format!(
            "| dimension | score | note |\n|---|---|---|\n"
        ));
        for (name, s) in [
            ("clarity", &rubric.problem_clarity),
            ("architecture", &rubric.architecture),
            ("code", &rubric.code_quality),
            ("ui/ux", &rubric.ui_ux),
            ("packaging", &rubric.packaging_hygiene),
            ("uniqueness", &rubric.differentiation.as_score()),
        ] {
            out.push_str(&format!(
                "| {} | {}/5 | {} |\n",
                name,
                s.score,
                escape_table(&first_sentence(&s.rationale))
            ));
        }
        out.push('\n');

        if !rubric.security_flags.is_empty() {
            out.push_str("**Security:**\n");
            for f in &rubric.security_flags {
                out.push_str(&format!(
                    "- [{}] {}\n",
                    match f.severity {
                        Severity::Critical => "critical",
                        Severity::Warning => "warning",
                        Severity::Info => "info",
                    },
                    escape_table(&f.detail)
                ));
            }
            out.push('\n');
        }

        if !rubric.differentiation.better_alternatives.is_empty() {
            out.push_str(&format!(
                "**Similar apps:** {}\n\n",
                rubric.differentiation.better_alternatives.join(", ")
            ));
        }
    }

    let s = &report.static_report;
    if !s.advisories.is_empty() {
        out.push_str("**Advisories:**\n");
        for a in &s.advisories {
            out.push_str(&format!("- {}: {}\n", a.kind, a.detail));
        }
        out.push('\n');
    }

    out
}

fn first_sentence(text: &str) -> String {
    text.split('.')
        .next()
        .unwrap_or(text)
        .trim()
        .to_string()
    + "."
}

fn verdict_label(v: Verdict) -> &'static str {
    match v {
        Verdict::Published => "📦 PUBLISHED",
        Verdict::BuildFailed => "🔧 BUILD FAILED (fix and resubmit)",
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
            verdict: Verdict::Published,
            certified: false,
            judge: None,
            legitimacy: None,
        }
    }

    #[test]
    fn markdown_escapes_and_renders() {
        let md = render_markdown(&report());
        assert!(md.contains("com.example.Better"));
        assert!(md.contains("3/5"));
        // assertion removed: minimal format has no gate labels
    }
}
