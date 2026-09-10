use serde::{Deserialize, Serialize};
use std::path::Path;

/// The subset of a flatpak-builder manifest the judge cares about. Parsed
/// permissively — manifests are JSON or YAML, and every field except app-id
/// is optional in the wild.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManifestInfo {
    pub app_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sdk: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub finish_args: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modules: Vec<ModuleInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<SourceRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buildsystem: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceRef {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// tag, commit, or sha256 the source is pinned to, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pinned: Option<String>,
}

/// Finish-args that punch holes in the sandbox. Advisory only — some apps
/// legitimately need these, but the human reviewer should see them called out.
const RISKY_FINISH_ARGS: &[&str] = &[
    "--filesystem=host",
    "--filesystem=host-os",
    "--filesystem=host-etc",
    "--filesystem=/",
    "--device=all",
    "--socket=system-bus",
];

pub fn parse_manifest(text: &str) -> anyhow::Result<ManifestInfo> {
    let value: serde_json::Value = if text.trim_start().starts_with('{') {
        serde_json::from_str(text)?
    } else {
        let yaml: serde_yaml::Value = serde_yaml::from_str(text)?;
        serde_json::to_value(yaml)?
    };

    let app_id = value
        .get("app-id")
        .or_else(|| value.get("id"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("manifest has no app-id/id"))?
        .to_string();

    let str_vec = |key: &str| -> Vec<String> {
        value
            .get(key)
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default()
    };

    let modules = value
        .get("modules")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    let name = m.get("name")?.as_str()?.to_string();
                    let sources = m
                        .get("sources")
                        .and_then(|v| v.as_array())
                        .map(|ss| {
                            ss.iter()
                                .filter_map(|s| {
                                    let pinned = ["tag", "commit"]
                                        .iter()
                                        .find_map(|k| s.get(*k).and_then(|v| v.as_str()))
                                        .map(String::from);
                                    Some(SourceRef {
                                        url: s.get("url").and_then(|v| v.as_str()).map(String::from),
                                        pinned,
                                    })
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    Some(ModuleInfo {
                        name,
                        sources,
                        buildsystem: m.get("buildsystem").and_then(|v| v.as_str()).map(String::from),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(ManifestInfo {
        app_id,
        runtime: value.get("runtime").and_then(|v| v.as_str()).map(String::from),
        sdk: value.get("sdk").and_then(|v| v.as_str()).map(String::from),
        finish_args: str_vec("finish-args"),
        modules,
        command: value.get("command").and_then(|v| v.as_str()).map(String::from),
    })
}

/// Find the manifest file in an app dir: the JSON/YAML named after the app id
/// (flathub convention), else any single *.json/*.yml/*.yaml at the top level.
pub fn find_manifest(dir: &Path) -> Option<std::path::PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    let mut candidates: Vec<std::path::PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension().map(|e| e == "json" || e == "yml" || e == "yaml").unwrap_or(false)
        })
        .collect();
    candidates.sort_by_key(|p| {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        // Prefer <app-id>.json, then .yml/.yaml, then anything else.
        (!name.ends_with(".json"), !name.ends_with(".yml") && !name.ends_with(".yaml"))
    });
    candidates.into_iter().next()
}

/// Finish args worth an advisory line in the report.
pub fn risky_finish_args(manifest: &ManifestInfo) -> Vec<&'static str> {
    RISKY_FINISH_ARGS
        .iter()
        .filter(|arg| manifest.finish_args.iter().any(|fa| fa == *arg))
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_YAML: &str = r#"
app-id: io.outcroplabs.TestApp
runtime: org.gnome.Platform
runtime-version: "48"
sdk: org.gnome.Sdk
command: testapp
finish-args:
  - --share=network
  - --device=all
modules:
  - name: testapp
    buildsystem: meson
    sources:
      - type: git
        url: https://github.com/outcrop-labs/testapp
        tag: v1.2.0
"#;

    #[test]
    fn parses_yaml_manifest() {
        let m = parse_manifest(SAMPLE_YAML).unwrap();
        assert_eq!(m.app_id, "io.outcroplabs.TestApp");
        assert_eq!(m.runtime.as_deref(), Some("org.gnome.Platform"));
        assert_eq!(m.modules.len(), 1);
        assert_eq!(
            m.modules[0].sources[0].pinned.as_deref(),
            Some("v1.2.0")
        );
        assert_eq!(risky_finish_args(&m), vec!["--device=all"]);
    }

    #[test]
    fn parses_json_manifest() {
        let json = r#"{"id": "com.example.JsonApp", "command": "run"}"#;
        let m = parse_manifest(json).unwrap();
        assert_eq!(m.app_id, "com.example.JsonApp");
        assert_eq!(m.command.as_deref(), Some("run"));
    }

    #[test]
    fn rejects_manifest_without_id() {
        assert!(parse_manifest("runtime: org.gnome.Platform").is_err());
    }
}
