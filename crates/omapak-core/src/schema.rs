use serde::{Deserialize, Serialize};

pub const REPORT_SCHEMA_VERSION: u32 = 1;

/// Below this, packaging_hygiene is a hard reject — an app that cannot install
/// cleanly has no business being distributed, whatever else it scores.
pub const GATE_MIN_PACKAGING_HYGIENE: u8 = 2;

/// Advisory scores at or above this average support an accept recommendation.
pub const ACCEPT_ADVISORY_AVERAGE: f32 = 3.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    AcceptRecommended,
    NeedsHuman,
    RejectRecommended,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RubricScore {
    /// 0–5 inclusive.
    pub score: u8,
    pub rationale: String,
}

/// The one dimension that must never gate: it names better existing solutions
/// purely as information for the human reviewer and the user.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Differentiation {
    /// 0–5 inclusive. Low score = "this is a clone" — still advisory.
    pub score: u8,
    pub rationale: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub better_alternatives: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityFlag {
    pub severity: Severity,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rubric {
    pub problem_clarity: RubricScore,
    pub differentiation: Differentiation,
    pub architecture: RubricScore,
    pub code_quality: RubricScore,
    pub ui_ux: RubricScore,
    pub packaging_hygiene: RubricScore,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub security_flags: Vec<SecurityFlag>,
}

impl Rubric {
    /// Mean of the five advisory dimensions. Packaging hygiene and security
    /// are gates, not averages.
    pub fn advisory_average(&self) -> f32 {
        let scores = [
            self.problem_clarity.score,
            self.differentiation.score,
            self.architecture.score,
            self.code_quality.score,
            self.ui_ux.score,
        ];
        scores.iter().sum::<u8>() as f32 / scores.len() as f32
    }

    /// Every score must be 0–5; the LLM's arithmetic is not trusted.
    pub fn validate(&self) -> Result<(), String> {
        let named: [(&str, u8); 6] = [
            ("problem_clarity", self.problem_clarity.score),
            ("differentiation", self.differentiation.score),
            ("architecture", self.architecture.score),
            ("code_quality", self.code_quality.score),
            ("ui_ux", self.ui_ux.score),
            ("packaging_hygiene", self.packaging_hygiene.score),
        ];
        for (name, score) in named {
            if score > 5 {
                return Err(format!("{name} score {score} out of range 0-5"));
            }
        }
        Ok(())
    }
}

/// The entire gate logic of omapak, in one pure function. The judge's rubric
/// informs; these rules decide.
pub fn compute_verdict(rubric: &Rubric, build_ok: bool) -> Verdict {
    if !build_ok {
        return Verdict::RejectRecommended;
    }
    if rubric
        .security_flags
        .iter()
        .any(|f| f.severity == Severity::Critical)
    {
        return Verdict::RejectRecommended;
    }
    if rubric.packaging_hygiene.score < GATE_MIN_PACKAGING_HYGIENE {
        return Verdict::RejectRecommended;
    }
    if rubric.security_flags.is_empty() && rubric.advisory_average() >= ACCEPT_ADVISORY_AVERAGE {
        return Verdict::AcceptRecommended;
    }
    Verdict::NeedsHuman
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinterStatus {
    Pass,
    Failed,
    NotFound,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinterRun {
    pub tool: String,
    pub status: LinterStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileStat {
    pub path: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceStats {
    pub files: u64,
    pub bytes: u64,
    pub largest: Vec<FileStat>,
    /// None when the app dir carries no git history (e.g. a sparse CI checkout).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit_count: Option<u64>,
    /// RFC 3339 date of the upstream's last commit, when a source checkout
    /// was available. Feeds the maintenance lifecycle policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_commit_date: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaticAdvisory {
    pub kind: String,
    pub detail: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StaticReport {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest: Option<ManifestInfo>,
    pub metadata_present: bool,
    pub appstream_present: bool,
    #[serde(default)]
    pub linters: Vec<LinterRun>,
    #[serde(default)]
    pub advisories: Vec<StaticAdvisory>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_stats: Option<SourceStats>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildReport {
    pub ok: bool,
    pub duration_secs: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub log_tail: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DynamicReport {
    pub launched: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub screenshots: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub log_tail: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JudgeInfo {
    pub model: String,
    pub base_url: String,
    /// Version tag of the in-repo judge prompt that produced the rubric.
    pub prompt_version: String,
    pub duration_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Report {
    pub schema_version: u32,
    pub app_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "static")]
    pub static_report: StaticReport,
    pub build: BuildReport,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dynamic: Option<DynamicReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rubric: Option<Rubric>,
    pub verdict: Verdict,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub judge: Option<JudgeInfo>,
}

use crate::manifest::ManifestInfo;

#[cfg(test)]
mod tests {
    use super::*;

    fn score(n: u8) -> RubricScore {
        RubricScore {
            score: n,
            rationale: "test".into(),
        }
    }

    fn rubric_with(avg_dim: u8, packaging: u8, security: Vec<SecurityFlag>) -> Rubric {
        Rubric {
            problem_clarity: score(avg_dim),
            differentiation: Differentiation {
                score: avg_dim,
                rationale: "test".into(),
                better_alternatives: vec![],
            },
            architecture: score(avg_dim),
            code_quality: score(avg_dim),
            ui_ux: score(avg_dim),
            packaging_hygiene: score(packaging),
            security_flags: security,
        }
    }

    #[test]
    fn verdict_gates() {
        let critical = SecurityFlag {
            severity: Severity::Critical,
            detail: "obfuscated payload".into(),
        };
        assert_eq!(compute_verdict(&rubric_with(5, 5, vec![]), true), Verdict::AcceptRecommended);
        assert_eq!(compute_verdict(&rubric_with(5, 5, vec![]), false), Verdict::RejectRecommended);
        assert_eq!(
            compute_verdict(&rubric_with(5, 5, vec![critical]), true),
            Verdict::RejectRecommended
        );
        assert_eq!(
            compute_verdict(&rubric_with(5, 1, vec![]), true),
            Verdict::RejectRecommended
        );
        assert_eq!(
            compute_verdict(&rubric_with(2, 5, vec![]), true),
            Verdict::NeedsHuman
        );
        let warning = SecurityFlag {
            severity: Severity::Warning,
            detail: "unexplained endpoint".into(),
        };
        assert_eq!(
            compute_verdict(&rubric_with(5, 5, vec![warning]), true),
            Verdict::NeedsHuman
        );
    }

    #[test]
    fn report_roundtrips() {
        let report = Report {
            schema_version: REPORT_SCHEMA_VERSION,
            app_id: "io.outcroplabs.TestApp".into(),
            created_at: chrono::Utc::now(),
            static_report: StaticReport::default(),
            build: BuildReport {
                ok: true,
                duration_secs: 42,
                log_tail: vec![],
            },
            dynamic: None,
            rubric: Some(rubric_with(4, 5, vec![])),
            verdict: Verdict::AcceptRecommended,
            judge: None,
        };
        let json = serde_json::to_string(&report).unwrap();
        let back: Report = serde_json::from_str(&json).unwrap();
        assert_eq!(back, report);
        assert!(json.contains("\"static\":"));
    }

    #[test]
    fn rubric_validation_rejects_out_of_range() {
        let mut r = rubric_with(4, 5, vec![]);
        r.code_quality.score = 6;
        assert!(r.validate().is_err());
        assert!(rubric_with(4, 5, vec![]).validate().is_ok());
    }
}
