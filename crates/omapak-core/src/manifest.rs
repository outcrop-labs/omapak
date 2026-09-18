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
    pub runtime_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sdk: Option<String>,
    /// Base app and its branch (`base:`/`base-version:`). Carried because the
    /// summary is what the model sees: without them a manifest that declares
    /// them reads as one that forgot to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_version: Option<String>,
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
    /// The module's own build commands. Part of the summary for the same
    /// reason as `base`: otherwise a module that installs its payload with
    /// `build-commands` reads as a module with no way to install anything.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub build_commands: Vec<String>,
    /// True when the manifest entry was a bare string — a *path to another
    /// module file* (`- python3-requirements.json`), which flatpak-builder
    /// expands in place. Such an entry has no fields, so requiring a `name`
    /// dropped it, and the summary then read as if nothing consumed that
    /// file. Its contents are not resolved here; the entry itself is listed.
    #[serde(default, skip_serializing_if = "is_false")]
    pub include: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
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
                    // A module entry can be a bare string: a path to another
                    // module file (`- python3-requirements.json`), which
                    // flatpak-builder expands in place. It has no `name`, so
                    // the old `m.get("name")?` dropped it and the summary read
                    // as though nothing consumed that file.
                    if let Some(included) = m.as_str() {
                        return Some(ModuleInfo {
                            name: included.to_string(),
                            sources: vec![],
                            buildsystem: None,
                            build_commands: vec![],
                            include: true,
                        });
                    }
                    let name = m.get("name")?.as_str()?.to_string();
                    let sources = m
                        .get("sources")
                        .and_then(|v| v.as_array())
                        .map(|ss| {
                            ss.iter()
                                .filter_map(|s| {
                                    let pinned = ["sha256", "tag", "commit"]
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
                        build_commands: m
                            .get("build-commands")
                            .and_then(|v| v.as_array())
                            .map(|a| {
                                a.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default(),
                        include: false,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(ManifestInfo {
        app_id,
        runtime: value.get("runtime").and_then(|v| v.as_str()).map(String::from),
        runtime_version: value
            .get("runtime-version")
            .and_then(|v| v.as_str())
            .map(String::from),
        sdk: value.get("sdk").and_then(|v| v.as_str()).map(String::from),
        base: value.get("base").and_then(|v| v.as_str()).map(String::from),
        base_version: value
            .get("base-version")
            .and_then(|v| v.as_str())
            .map(String::from),
        finish_args: str_vec("finish-args"),
        modules,
        command: value.get("command").and_then(|v| v.as_str()).map(String::from),
    })
}

/// Find the manifest file in an app dir: the JSON/YAML named after the app id
/// (flathub convention — the app dir itself is named after the app id), else
/// any single *.json/*.yml/*.yaml at the top level. metadata.yml and vendored
/// sources files (cargo-sources.json, python3-dependencies.json, …) are
/// excluded outright, not just deprioritized — they aren't manifests, and the
/// extension tie-break could still hand them to flatpak-builder.
pub fn find_manifest(dir: &Path) -> Option<std::path::PathBuf> {
    let dir_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let entries = std::fs::read_dir(dir).ok()?;
    let mut candidates: Vec<std::path::PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            if !p.is_file() {
                return false;
            }
            if !p
                .extension()
                .map(|e| e == "json" || e == "yml" || e == "yaml")
                .unwrap_or(false)
            {
                return false;
            }
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let stem = name
                .trim_end_matches(".json")
                .trim_end_matches(".yml")
                .trim_end_matches(".yaml");
            stem != "metadata"
                && !name.ends_with("-sources.json")
                && !name.ends_with("-dependencies.json")
        })
        .collect();
    candidates.sort_by_key(|p| {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let stem = name.trim_end_matches(".json").trim_end_matches(".yml").trim_end_matches(".yaml");
        let tier = if stem == dir_name && !stem.is_empty() { 0 } else { 1 };
        (tier, !name.ends_with(".json"))
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

    #[test]
    fn find_manifest_prefers_app_id_named_file_over_sources_json() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("io.github.example.app");
        std::fs::create_dir_all(&base).unwrap();
        std::fs::write(base.join("cargo-sources.json"), "[]").unwrap();
        std::fs::write(base.join("metadata.yml"), "submitter: jon").unwrap();
        std::fs::write(base.join("io.github.example.app.yml"), "app-id: io.github.example.app").unwrap();
        assert_eq!(
            find_manifest(&base).unwrap().file_name().unwrap().to_str().unwrap(),
            "io.github.example.app.yml"
        );
    }

    #[test]
    fn find_manifest_prefers_app_id_named_json_over_yml() {
        let dir = tempfile::tempdir().unwrap();
        let app_id = "io.github.example.app";
        let base = dir.path().join(app_id);
        std::fs::create_dir_all(&base).unwrap();
        std::fs::write(base.join(format!("{app_id}.yml")), "app-id: x").unwrap();
        std::fs::write(base.join(format!("{app_id}.json")), "{}").unwrap();
        assert_eq!(
            find_manifest(&base).unwrap().file_name().unwrap().to_str().unwrap(),
            format!("{app_id}.json")
        );
    }

    #[test]
    fn find_manifest_never_returns_metadata_yml() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("me.example.app");
        std::fs::create_dir_all(&base).unwrap();
        std::fs::write(base.join("metadata.yml"), "submitter: jon\nlicense: MIT\nhomepage: https://x").unwrap();
        std::fs::write(base.join("me.example.app.yml"), "app-id: me.example.app").unwrap();
        assert_eq!(
            find_manifest(&base).unwrap().file_name().unwrap().to_str().unwrap(),
            "me.example.app.yml"
        );
    }

    #[test]
    fn find_manifest_deprioritizes_sources_files_without_app_id_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("com.example.App");
        std::fs::create_dir_all(&base).unwrap();
        std::fs::write(base.join("python3-dependencies.json"), "{}").unwrap();
        std::fs::write(base.join("manifest.yml"), "app-id: com.example.App").unwrap();
        assert_eq!(
            find_manifest(&base).unwrap().file_name().unwrap().to_str().unwrap(),
            "manifest.yml"
        );
    }

    /// The shape of a real submission (io.github.alexwest1981.OmaAmp, judged
    /// 2026-09-18): the summary carried none of `runtime-version`, `base`,
    /// `base-version` or a module's `build-commands`, and it dropped the
    /// `- python3-requirements.json` include entirely — so a build that failed
    /// on a transient HTTP 504 was read by the model as "no runtime-version,
    /// no build-commands, nothing consuming the requirements file" and the
    /// app was scored as if its manifest were broken.
    #[test]
    fn carries_the_fields_the_model_needs_to_verify_a_manifest() {
        const PYQT_YAML: &str = r#"
app-id: io.example.PyQtApp
runtime: org.kde.Platform
runtime-version: "6.11"
sdk: org.kde.Sdk
base: com.riverbankcomputing.PyQt.BaseApp
base-version: "6.11"
command: app
finish-args:
  - --socket=pulseaudio
modules:
  - python3-requirements.json
  - name: app
    buildsystem: simple
    build-commands:
      - mkdir -p ${FLATPAK_DEST}/app
      - install -Dm755 app.sh ${FLATPAK_DEST}/bin/app
    sources:
      - type: git
        url: https://github.com/example/app
        commit: 7d630f8543e8226cb6e291fdd548fca5f08056c1
"#;
        let m = parse_manifest(PYQT_YAML).unwrap();
        assert_eq!(m.runtime_version.as_deref(), Some("6.11"));
        assert_eq!(m.base.as_deref(), Some("com.riverbankcomputing.PyQt.BaseApp"));
        assert_eq!(m.base_version.as_deref(), Some("6.11"));

        assert_eq!(m.modules.len(), 2, "the bare-string include must survive");
        assert!(m.modules[0].include);
        assert_eq!(m.modules[0].name, "python3-requirements.json");

        assert!(!m.modules[1].include);
        assert_eq!(m.modules[1].build_commands.len(), 2);
        assert!(m.modules[1].build_commands[1].contains("${FLATPAK_DEST}/bin/app"));
    }
}
