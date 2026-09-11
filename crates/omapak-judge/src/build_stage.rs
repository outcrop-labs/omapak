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
    // Read bytes and convert lossily: build tools emit arbitrary bytes,
    // and read_to_string silently truncates at the first invalid UTF-8.
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let out_handle = std::thread::spawn(move || {
        use std::io::Read;
        let mut b = Vec::new();
        let _ = stdout.read_to_end(&mut b);
        b
    });
    let err_handle = std::thread::spawn(move || {
        use std::io::Read;
        let mut b = Vec::new();
        let _ = stderr.read_to_end(&mut b);
        b
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

    // Status's Display names signals ("signal: 9 (SIGKILL)") — the only
    // way to distinguish a real builder error from the runner killing it.
    if !status.success() {
        eprintln!("  flatpak-builder {status}");
    }
    let out_bytes = out_handle.join().unwrap_or_default();
    let err_bytes = err_handle.join().unwrap_or_default();
    let stdout = String::from_utf8_lossy(&out_bytes);
    let stderr = String::from_utf8_lossy(&err_bytes);
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
