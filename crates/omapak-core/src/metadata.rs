use serde::{Deserialize, Serialize};
use std::path::Path;

/// `metadata.yml` — what the submitter attests about themselves and the app.
/// The judge reads it; the judge does not trust it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SourceAccess {
    /// Source is public and gets digested for the judge.
    #[default]
    Public,
    /// Closed source, reviewed with the owner's cooperation: the owner
    /// grants a maintainer scoped read access, the judge runs against the
    /// real source, only the report is public. Unaudited binaries do not
    /// ship from omapak, full stop.
    PrivateAssisted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    pub submitter: String,
    /// Upstream repository or release channel the artifacts come from.
    pub source_repo: String,
    pub summary: String,
    #[serde(default)]
    pub source_access: SourceAccess,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

pub fn parse_metadata(text: &str) -> anyhow::Result<Metadata> {
    Ok(serde_yaml::from_str(text)?)
}

pub fn load_from_dir(dir: &Path) -> Option<Metadata> {
    let text = std::fs::read_to_string(dir.join("metadata.yml")).ok()?;
    parse_metadata(&text).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_metadata() {
        let m = parse_metadata(
            "submitter: jon\nsource_repo: https://github.com/outcrop-labs/testapp\nsummary: A test\nlicense: MIT\ntags: [utility, demo]\n",
        )
        .unwrap();
        assert_eq!(m.submitter, "jon");
        assert_eq!(m.tags, vec!["utility", "demo"]);
        assert!(m.homepage.is_none());
    }

    #[test]
    fn rejects_missing_required_fields() {
        assert!(parse_metadata("submitter: jon").is_err());
    }
}
