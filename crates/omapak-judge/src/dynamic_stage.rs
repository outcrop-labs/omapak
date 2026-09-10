//! Headless launch + screenshots. Off by default (`--with-dynamic`) until it
//! proves stable in CI: run the built app under cage with a headless wlroots
//! backend, screenshot at t+2s and t+10s, and record whether it was still
//! alive at the end. Best-effort by design — every failure degrades to a
//! note, never a gate.

use omapak_core::DynamicReport;
use std::path::Path;
use std::process::Command;

pub fn run(app_id: &str, work_dir: &Path, manifest: &Path, out_dir: &Path) -> DynamicReport {
    for tool in ["cage", "grim"] {
        if which(tool).is_none() {
            return note_report(&format!(
                "skipped: {tool} not installed (pacman -S {tool} / apt install {tool})"
            ));
        }
    }

    let shim = out_dir.join("dynamic-shim.sh");
    let shot1 = out_dir.join("shot-2s.png");
    let shot2 = out_dir.join("shot-10s.png");
    let shim_body = format!(
        r#"#!/bin/sh
flatpak run --user {app_id} &
APP=$!
sleep 2
grim {}
sleep 8
grim {}
wait $APP
"#,
        shot1.display(),
        shot2.display()
    );
    if std::fs::write(&shim, shim_body).is_err() {
        return note_report("skipped: could not write dynamic shim");
    }
    make_executable(&shim);

    // The app must be installed from the local repo first.
    let install = Command::new("flatpak")
        .args(["--user", "install", "--noninteractive", "--reinstall", "omapak-local", app_id])
        .output();
    if let Ok(o) = &install {
        if !o.status.success() {
            return note_report(&format!(
                "skipped: flatpak install failed: {}",
                String::from_utf8_lossy(&o.stderr).lines().last().unwrap_or("?")
            ));
        }
    }

    let out = Command::new("cage")
        .env("WLR_BACKENDS", "headless")
        .env("WLR_LIBINPUT_NO_DEVICES", "1")
        .arg("--")
        .arg(&shim)
        .output();

    let mut screenshots = vec![];
    for shot in [&shot1, &shot2] {
        if shot.is_file() {
            screenshots.push(shot.to_string_lossy().into_owned());
        }
    }

    let launched = !screenshots.is_empty();
    let mut log_tail = vec![];
    match out {
        Ok(o) => {
            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&o.stdout),
                String::from_utf8_lossy(&o.stderr)
            );
            log_tail = combined.lines().map(String::from).collect();
            let tail = log_tail.len().saturating_sub(20);
            log_tail.drain(0..tail);
        }
        Err(e) => log_tail.push(format!("cage failed to launch: {e}")),
    }
    let _ = (work_dir, manifest);

    DynamicReport {
        launched,
        screenshots,
        log_tail,
    }
}

fn note_report(note: &str) -> DynamicReport {
    DynamicReport {
        launched: false,
        screenshots: vec![],
        log_tail: vec![note.into()],
    }
}

fn which(tool: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(tool))
        .find(|candidate| candidate.is_file())
}

fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = std::fs::metadata(path) {
        let mut perms = meta.permissions();
        perms.set_mode(0o755);
        let _ = std::fs::set_permissions(path, perms);
    }
}
