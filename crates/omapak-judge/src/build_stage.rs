use anyhow::{Context, Result};
use omapak_core::BuildReport;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub fn run(manifest: &Path, work_dir: &Path, repo_dir: &Path) -> Result<BuildReport> {
    std::fs::create_dir_all(work_dir).context("create build dir")?;
    std::fs::create_dir_all(repo_dir).context("create repo dir")?;

    let timeout = Duration::from_secs(
        std::env::var("OMAPAK_BUILD_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1800),
    );
    let started = Instant::now();

    let mut child = Command::new("flatpak-builder")
        .arg("--force-clean")
        .arg("--disable-rofiles-fuse")
        .arg("--user")
        .arg("--repo")
        .arg(repo_dir)
        .arg(work_dir)
        .arg(manifest)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawn flatpak-builder — is it installed?")?;

    // Spawn readers so a chatty build can't deadlock on a full pipe.
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let out_handle = std::thread::spawn(move || {
        use std::io::Read;
        let mut s = String::new();
        let _ = stdout.read_to_string(&mut s);
        s
    });
    let err_handle = std::thread::spawn(move || {
        use std::io::Read;
        let mut s = String::new();
        let _ = stderr.read_to_string(&mut s);
        s
    });

    let status = loop {
        match child.try_wait()? {
            Some(status) => break status,
            None if started.elapsed() > timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Ok(BuildReport {
                    ok: false,
                    duration_secs: started.elapsed().as_secs(),
                    log_tail: vec![format!(
                        "build exceeded OMAPAK_BUILD_TIMEOUT_SECS ({}) and was killed",
                        timeout.as_secs()
                    )],
                });
            }
            None => std::thread::sleep(Duration::from_millis(500)),
        }
    };

    let stdout = out_handle.join().unwrap_or_default();
    let stderr = err_handle.join().unwrap_or_default();
    let mut log_tail: Vec<String> = stdout
        .lines()
        .chain(stderr.lines())
        .map(String::from)
        .collect();
    let tail = log_tail.len().saturating_sub(40);
    log_tail.drain(0..tail);

    Ok(BuildReport {
        ok: status.success(),
        duration_secs: started.elapsed().as_secs(),
        log_tail,
    })
}
