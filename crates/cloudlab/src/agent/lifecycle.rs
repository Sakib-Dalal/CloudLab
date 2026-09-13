use super::Credentials;
use crate::model::{private_dir, token, valid_id, write_private};
use anyhow::{bail, Context};
use std::{
    fs::{File, OpenOptions, TryLockError},
    path::{Path, PathBuf},
    time::Duration,
};

const LOCK: &str = "agent.lock";
const RUN: &str = "agent-run";
const STOP: &str = "agent-stop";

fn is_agent_start(args: &[std::ffi::OsString]) -> bool {
    if args.get(1).is_none_or(|arg| arg != "agent") {
        return false;
    }
    let mut args = args.iter().skip(2);
    while let Some(arg) = args.next() {
        if arg == "--data-dir" || arg == "--coordinator" || arg == "--enrollment" {
            args.next();
        } else if arg == "--help"
            || arg == "-h"
            || arg == "--version"
            || arg == "-V"
            || (!arg.to_string_lossy().starts_with('-') && arg != "start")
        {
            return false;
        }
    }
    true
}

// Older agents do not hold our lock. Detect a matching legacy process, but do
// not signal an unverified PID or erase credentials while it may still use them.
fn reject_legacy_agent(dir: &Path) -> anyhow::Result<()> {
    use sysinfo::{ProcessRefreshKind, RefreshKind, System, UpdateKind};
    let system = System::new_with_specifics(
        RefreshKind::nothing().with_processes(
            ProcessRefreshKind::nothing()
                // Linux lists threads as processes by default. Their IDs differ
                // from our PID, but their arguments and data folder are ours.
                .without_tasks()
                .with_cmd(UpdateKind::Always)
                .with_cwd(UpdateKind::Always)
                .with_environ(UpdateKind::Always),
        ),
    );
    let target = dir.canonicalize()?;
    for (pid, process) in system.processes() {
        if pid.as_u32() == std::process::id() {
            continue;
        }
        let args = process.cmd();
        if args
            .first()
            .and_then(|arg| Path::new(arg).file_stem())
            .is_none_or(|name| name != "cloudlab")
            || !is_agent_start(args)
        {
            continue;
        }
        let explicit = args
            .windows(2)
            .find(|pair| pair[0] == "--data-dir")
            .map(|pair| PathBuf::from(&pair[1]))
            .or_else(|| {
                args.iter()
                    .filter_map(|a| a.to_str())
                    .find_map(|a| a.strip_prefix("--data-dir=").map(PathBuf::from))
            })
            .or_else(|| {
                process
                    .environ()
                    .iter()
                    .filter_map(|a| a.to_str())
                    .find_map(|a| a.strip_prefix("CLOUDLAB_AGENT_DIR=").map(PathBuf::from))
            })
            .unwrap_or_else(|| PathBuf::from(".cloudlab/agent"));
        let candidate = if explicit.is_absolute() {
            Some(explicit)
        } else {
            process.cwd().map(|cwd| cwd.join(explicit))
        };
        let same_directory = match candidate.map(|p| p.canonicalize()) {
            Some(Ok(path)) => path == target,
            Some(Err(error)) if error.kind() == std::io::ErrorKind::NotFound => false,
            _ => true,
        };
        if same_directory {
            bail!("An older agent may still be using {}. Stop it once with Ctrl+C in its terminal or through its service manager, then retry. No process was killed and enrollment was kept.", dir.display());
        }
    }
    Ok(())
}

fn lock_file(dir: &Path) -> anyhow::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    Ok(options.open(dir.join(LOCK))?)
}

fn remove_if_present(path: &Path) -> anyhow::Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub(super) struct RunningAgent {
    // Keep this inode on disk: unlinking a lock file permits simultaneous owners.
    _lock: File,
    dir: PathBuf,
    generation: String,
}

impl RunningAgent {
    pub fn start(dir: &Path) -> anyhow::Result<Self> {
        private_dir(dir)?;
        let lock = lock_file(dir)?;
        lock.try_lock().with_context(|| format!(
            "An agent or enrollment cleanup is already using {}. Use cloudlab agent stop --data-dir {} to stop the agent",
            dir.display(), dir.display()
        ))?;
        reject_legacy_agent(dir)?;
        remove_if_present(&dir.join(STOP))?;
        let generation = token();
        write_private(&dir.join(RUN), generation.as_bytes())?;
        Ok(Self {
            _lock: lock,
            dir: dir.into(),
            generation,
        })
    }

    pub async fn wait_for_stop(&self) -> anyhow::Result<()> {
        loop {
            if std::fs::read_to_string(self.dir.join(STOP)).is_ok_and(|v| v == self.generation) {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

impl Drop for RunningAgent {
    fn drop(&mut self) {
        let _ = remove_if_present(&self.dir.join(RUN));
        let _ = remove_if_present(&self.dir.join(STOP));
    }
}

// Return the acquired lock so delete and a concurrent startup cannot race.
async fn stop_and_lock(dir: &Path) -> anyhow::Result<Option<File>> {
    if !dir.exists() {
        return Ok(None);
    }
    let lock = lock_file(dir)?;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    let mut requested_stop = false;
    loop {
        match lock.try_lock() {
            Ok(()) => {
                if !requested_stop {
                    reject_legacy_agent(dir)?;
                }
                remove_if_present(&dir.join(RUN))?;
                remove_if_present(&dir.join(STOP))?;
                return Ok(Some(lock));
            }
            Err(TryLockError::WouldBlock) => {}
            Err(e) => return Err(e.into()),
        }
        if let Ok(generation) = std::fs::read_to_string(dir.join(RUN)) {
            anyhow::ensure!(
                generation.len() == 64 && generation.bytes().all(|b| b.is_ascii_hexdigit()),
                "Invalid agent control file"
            );
            write_private(&dir.join(STOP), generation.as_bytes())?;
            requested_stop = true;
        }
        if tokio::time::Instant::now() >= deadline {
            bail!("The agent did not stop. Stop it in its terminal or through its service manager, then retry with the same --data-dir.");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

pub async fn stop(dir: PathBuf) -> anyhow::Result<()> {
    let _lock = stop_and_lock(&dir).await?;
    println!("Agent stopped (or already stopped).\nData folder: {}\nEnrollment, containers, and volumes are retained.\nResume with: cloudlab agent start --data-dir {}", dir.display(), dir.display());
    Ok(())
}

pub fn coordinator(explicit: Option<String>, dir: &Path) -> anyhow::Result<String> {
    if let Some(value) = explicit {
        return Ok(value);
    }
    let contents = std::fs::read(dir.join("credentials.json"))
        .context("No saved pairing. Generate a pairing command in Compute nodes > Connect a node, then run cloudlab agent --coordinator URL --enrollment KEY")?;
    let credentials: Credentials = serde_json::from_slice(&contents).context("Cannot read the saved pairing. Use cloudlab agent delete --local-only with the same --data-dir to reset it")?;
    Ok(credentials.coordinator)
}

#[derive(serde::Serialize)]
struct LocalStatus {
    scope: &'static str,
    data_dir: PathBuf,
    process: &'static str,
    enrollment: &'static str,
    node_id: Option<String>,
    coordinator: Option<String>,
}

pub fn status(dir: PathBuf, json: bool) -> anyhow::Result<()> {
    let mut status = LocalStatus {
        scope: "local",
        data_dir: std::path::absolute(&dir)?,
        process: "stopped",
        enrollment: "not_paired",
        node_id: None,
        coordinator: None,
    };
    let lock = if dir.exists() {
        Some(lock_file(&dir)?)
    } else {
        None
    };
    if let Some(lock) = &lock {
        match lock.try_lock() {
            Ok(()) => {
                reject_legacy_agent(&dir)?;
            }
            Err(TryLockError::WouldBlock) => {
                status.process = if dir.join(RUN).exists() {
                    "running"
                } else {
                    "busy"
                }
            }
            Err(e) => return Err(e.into()),
        }
    }
    match std::fs::read(dir.join("credentials.json")) {
        Ok(contents) => {
            let credentials: Credentials = serde_json::from_slice(&contents).context(
                "Saved enrollment is unreadable; use agent delete --local-only to reset it",
            )?;
            super::validate_coordinator(&credentials.coordinator)?;
            status.enrollment = "saved";
            status.node_id = Some(credentials.id);
            status.coordinator = Some(credentials.coordinator);
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&status)?);
    } else {
        println!(
            "Data folder: {}\nAgent process: {}\nEnrollment: {}",
            status.data_dir.display(),
            if status.process == "busy" {
                "enrollment cleanup in progress"
            } else {
                status.process
            },
            if status.enrollment == "not_paired" {
                "not paired"
            } else {
                status.enrollment
            }
        );
        if let (Some(node), Some(coordinator)) = (status.node_id, status.coordinator) {
            println!("Node: {node}\nCoordinator: {coordinator}\nThis is local status; node access may have been revoked in the dashboard.");
        }
    }
    Ok(())
}

pub async fn delete(dir: PathBuf, local_only: bool) -> anyhow::Result<()> {
    let _lock = stop_and_lock(&dir).await?;
    let path = dir.join("credentials.json");
    let contents = match std::fs::read(&path) {
        Ok(contents) => contents,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("No saved agent enrollment in {}.", dir.display());
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };
    if !local_only {
        let credentials: Credentials = serde_json::from_slice(&contents).context(
            "Cannot read enrollment. Use --local-only to clear only local pairing files",
        )?;
        super::validate_coordinator(&credentials.coordinator)?;
        anyhow::ensure!(
            valid_id(&credentials.id) && credentials.token.len() == 64,
            "Invalid agent credential file; use --local-only to clear local enrollment"
        );
        let response = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(10)).build()?
            .post(format!("{}/api/agent/unenroll", credentials.coordinator.trim_end_matches('/')))
            .bearer_auth(&credentials.token).send().await
            .context("Cannot contact the coordinator. Enrollment was kept. Retry when it is online, or use --local-only and revoke the old node in the dashboard later")?;
        if response.status() != reqwest::StatusCode::UNAUTHORIZED {
            let result: serde_json::Value = response.error_for_status()
                .context("Coordinator could not revoke this node. Update the coordinator, or use --local-only and revoke it in the dashboard")?
                .json().await?;
            anyhow::ensure!(
                result["revoked"] == true,
                "Coordinator did not confirm node revocation; local enrollment was kept"
            );
        }
    }
    // Remove only this agent's known pairing/receipt files, never its whole directory.
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name
            .strip_prefix("receipt-")
            .and_then(|v| v.strip_suffix(".json"))
            .is_some_and(valid_id)
        {
            remove_if_present(&entry.path())?;
        }
    }
    remove_if_present(&path)?;
    println!("Agent enrollment deleted from {}. Containers and volumes are retained. Generate a fresh pairing command to enroll again.", dir.display());
    if local_only {
        println!("The coordinator was not contacted. Revoke the old node in the dashboard if it is still listed.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_detection_distinguishes_commands_from_option_values() {
        let running = |args: &[&str]| {
            is_agent_start(
                &args
                    .iter()
                    .map(std::ffi::OsString::from)
                    .collect::<Vec<_>>(),
            )
        };
        assert!(running(&["cloudlab", "agent", "--data-dir", "stop"]));
        assert!(running(&[
            "cloudlab",
            "agent",
            "start",
            "--data-dir=status"
        ]));
        assert!(!running(&["cloudlab", "agent", "status", "--json"]));
        assert!(!running(&[
            "cloudlab",
            "agent",
            "--data-dir",
            "start",
            "stop"
        ]));
        assert!(!running(&["cloudlab", "agent", "start", "--help"]));
        assert!(!running(&["cloudlab", "agent", "help", "start"]));
        assert!(!running(&["cloudlab", "serve"]));
    }
}
