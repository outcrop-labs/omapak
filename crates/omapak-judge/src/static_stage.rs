use anyhow::Result;
use omapak_core::{
    FileStat, LinterRun, LinterStatus, ManifestInfo, SourceStats, StaticAdvisory, StaticReport,
};
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

pub fn run(app_dir: &Path, source_dir: Option<&Path>) -> Result<StaticReport> {
    let mut report = StaticReport::default();

    let manifest_path = omapak_core::find_manifest(app_dir);
    let manifest: Option<ManifestInfo> = manifest_path.as_ref().and_then(|p| {
        std::fs::read_to_string(p)
            .ok()
            .and_then(|t| omapak_core::parse_manifest(&t).ok())
    });

    if let Some(m) = &manifest {
        for arg in omapak_core::risky_finish_args(m) {
            report.advisories.push(StaticAdvisory {
                kind: "finish-args".into(),
                detail: format!("{arg} punches a hole in the sandbox — confirm it's justified"),
            });
        }
        if m.modules.is_empty() {
            report.advisories.push(StaticAdvisory {
                kind: "manifest".into(),
                detail: "no modules — nothing will be built".into(),
            });
        }
        let unpinned: Vec<String> = m
            .modules
            .iter()
            .flat_map(|modu| modu.sources.iter().filter_map(|s| {
                (s.url.is_some() && s.pinned.is_none())
                    .then(|| modu.name.clone())
            }))
            .collect();
        if !unpinned.is_empty() {
            report.advisories.push(StaticAdvisory {
                kind: "manifest".into(),
                detail: format!("unpinned upstream sources: {}", unpinned.join(", ")),
            });
        }
    }

    report.manifest = manifest;
    report.metadata_present = app_dir.join("metadata.yml").is_file();
    let appstream = find_appstream(app_dir);
    report.appstream_present = appstream.is_some();

    if let Some(mpath) = &manifest_path {
        report.linters.push(lint_manifest(mpath));
    }
    if let Some(apath) = &appstream {
        report.linters.push(lint_appstream(apath));
    }

    report.source_stats = Some(collect_stats(app_dir, source_dir));
    Ok(report)
}

fn find_appstream(app_dir: &Path) -> Option<std::path::PathBuf> {
    WalkDir::new(app_dir)
        .max_depth(2)
        .into_iter()
        .flatten()
        .find(|e| {
            let n = e.file_name().to_string_lossy();
            e.path().is_file()
                && (n.ends_with(".metainfo.xml") || n.ends_with(".appdata.xml"))
        })
        .map(|e| e.path().to_path_buf())
}

fn lint_manifest(path: &Path) -> LinterRun {
    run_linter(
        "flatpak-builder-lint",
        &["manifest".to_string(), path.to_string_lossy().into_owned()],
    )
}

fn lint_appstream(path: &Path) -> LinterRun {
    let metainfo = run_linter(
        "flatpak-builder-lint",
        &["flatpakmetainfo".to_string(), path.to_string_lossy().into_owned()],
    );
    if metainfo.status != LinterStatus::NotFound {
        return metainfo;
    }
    run_linter(
        "appstreamcli",
        &["validate".to_string(), "--no-net".to_string(), path.to_string_lossy().into_owned()],
    )
}

fn run_linter(tool: &str, args: &[String]) -> LinterRun {
    let Ok(out) = Command::new(tool).args(args).output() else {
        return LinterRun {
            tool: tool.into(),
            status: LinterStatus::NotFound,
            findings: vec![],
        };
    };
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    if out.status.success() {
        LinterRun {
            tool: tool.into(),
            status: LinterStatus::Pass,
            findings: vec![],
        }
    } else {
        let mut findings: Vec<String> = stdout
            .lines()
            .chain(stderr.lines())
            .filter(|l| !l.trim().is_empty())
            .map(String::from)
            .collect();
        findings.truncate(20);
        LinterRun {
            tool: tool.into(),
            status: LinterStatus::Failed,
            findings,
        }
    }
}

fn collect_stats(app_dir: &Path, source_dir: Option<&Path>) -> SourceStats {
    let mut files = 0u64;
    let mut bytes = 0u64;
    let mut all: Vec<FileStat> = Vec::new();
    for dir in [Some(app_dir), source_dir].into_iter().flatten() {
        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_entry(|e| {
                let n = e.file_name().to_string_lossy();
                n != ".git" && n != "node_modules" && n != "target"
            })
            .flatten()
        {
            if entry.file_type().is_file() {
                let Ok(meta) = entry.metadata() else { continue };
                files += 1;
                bytes += meta.len();
                all.push(FileStat {
                    path: entry
                        .path()
                        .strip_prefix(dir.parent().unwrap_or(dir))
                        .unwrap_or(entry.path())
                        .to_string_lossy()
                        .into_owned(),
                    bytes: meta.len(),
                });
            }
        }
    }
    if files == 0 {
        return SourceStats {
            files: 0,
            bytes: 0,
            largest: vec![],
            commit_count: None,
            last_commit_date: None,
        };
    }
    all.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    all.truncate(5);
    let (commit_count, last_commit_date) = git_signals(source_dir.unwrap_or(app_dir));
    SourceStats {
        files,
        bytes,
        largest: all,
        commit_count,
        last_commit_date,
    }
}

fn git_signals(dir: &Path) -> (Option<u64>, Option<String>) {
    let count = Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "rev-list", "--count", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok());
    let last = Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "log", "-1", "--format=%cI"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty());
    (count, last)
}
