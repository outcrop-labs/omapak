//! omapak-core — shared schema for the omapak judge pipeline.
//!
//! Everything the judge produces and the website consumes is defined here so
//! the Rust pipeline and the SvelteKit catalog agree on one wire format.

mod manifest;
mod markdown;
mod metadata;
mod schema;

pub use manifest::{find_manifest, parse_manifest, risky_finish_args, ManifestInfo, ModuleInfo, SourceRef};
pub use markdown::render_markdown;
pub use metadata::{load_from_dir as load_metadata, Metadata};
pub use schema::{
    compute_verdict, BuildReport, Differentiation, DynamicReport, FileStat, JudgeInfo, LinterRun,
    LinterStatus, Report, Rubric, RubricScore, SecurityFlag, Severity, SourceStats, StaticAdvisory,
    StaticReport, Verdict, GATE_MIN_PACKAGING_HYGIENE, REPORT_SCHEMA_VERSION,
};
