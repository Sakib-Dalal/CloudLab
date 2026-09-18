//! The only module that invokes host commands. Arguments are constructed here,
//! never by a caller. User commands appear only after `docker exec … sh -lc`.
use crate::model::{valid_id, Workspace};
use anyhow::{bail, Context};
use serde_json::Value;
use std::{collections::HashMap, time::Duration};
use tokio::{io::AsyncReadExt, process::Command};

pub const OUTPUT_LIMIT: usize = 64 * 1024;
pub fn container_name(id: &str) -> anyhow::Result<String> {
    anyhow::ensure!(valid_id(id), "Invalid workspace ID");
    Ok(format!("cloudlab-{id}"))
}
pub async fn docker(args: &[String], seconds: u64) -> anyhow::Result<String> {
    host_command("docker", args, seconds).await
}
// Only fixed executable names and structured arguments from trusted collectors.
pub(crate) async fn host_command(
    program: &str,
    args: &[String],
    seconds: u64,
) -> anyhow::Result<String> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .with_context(|| format!("{program} is not installed or is not on PATH"))?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    async fn drain<R: tokio::io::AsyncRead + Unpin>(mut reader: R) -> std::io::Result<Vec<u8>> {
        let mut buffer = [0; 8192];
        let mut output = Vec::new();
        loop {
            let n = reader.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            let take = n.min(OUTPUT_LIMIT.saturating_sub(output.len()));
            output.extend_from_slice(&buffer[..take]);
        }
        Ok(output)
    }
    let out = tokio::spawn(drain(stdout));
    let err = tokio::spawn(drain(stderr));
    let status = tokio::time::timeout(Duration::from_secs(seconds), child.wait()).await;
    match status {
        Ok(result) => {
            let status = result?;
            let output = out.await??;
            let error = err.await??;
            if !status.success() {
                bail!(
                    "{}",
                    String::from_utf8_lossy(&error)
                        .chars()
                        .take(3500)
                        .collect::<String>()
                );
            }
            let mut joined = String::from_utf8_lossy(&output).into_owned();
            let remaining = OUTPUT_LIMIT.saturating_sub(joined.len());
            joined.extend(String::from_utf8_lossy(&error).chars().take(remaining));
            while joined.len() > OUTPUT_LIMIT {
                joined.pop();
            }
            Ok(joined)
        }
        Err(_) => {
            let _ = child.kill().await;
            out.abort();
            err.abort();
            bail!("Docker operation timed out after {seconds} seconds")
        }
    }
}
fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|s| s.to_string()).collect()
}
pub async fn available() -> bool {
    docker(&strings(&["info", "--format", "{{.ServerVersion}}"]), 8)
        .await
        .is_ok()
}
pub fn run_args(
    w: &Workspace,
    node: &str,
    allow_network: bool,
    gpus: &[crate::model::Gpu],
) -> anyhow::Result<Vec<String>> {
    anyhow::ensure!(valid_id(node) && valid_id(&w.id), "Invalid resource ID");
    anyhow::ensure!(
        w.cpus > 0 && w.cpus <= 256 && (256..=1048576).contains(&w.memory_mb),
        "Invalid resource limits"
    );
    anyhow::ensure!(!w.network||allow_network,"This node does not allow outbound networking. Restart the agent with --allow-network only if you trust lab operators.");
    let image = match w.template.as_str() {
        "terminal" => "cloudlab/terminal:2",
        "jupyter" => "cloudlab/jupyter:2",
        "code" => "cloudlab/code:2",
        _ => bail!("Unknown workspace template"),
    };
    let name = container_name(&w.id)?;
    let mut args = strings(&[
        "run",
        "--detach",
        "--init",
        "--pull=never",
        "--name",
        &name,
        "--label",
        &format!("cloudlab.node={node}"),
        "--label",
        &format!("cloudlab.workspace={}", w.id),
        "--user",
        "1000:1000",
        "--read-only",
        "--cap-drop=ALL",
        "--security-opt=no-new-privileges:true",
        "--pids-limit=256",
        "--cpus",
        &w.cpus.to_string(),
        "--memory",
        &format!("{}m", w.memory_mb),
        "--memory-swap",
        &format!("{}m", w.memory_mb),
        "--network",
        &if w.network {
            format!("{name}-net")
        } else {
            "none".into()
        },
        "--mount",
        &format!("type=volume,src={name}-home,dst=/home/lab"),
        "--tmpfs",
        "/tmp:rw,noexec,nosuid,nodev,size=268435456,mode=1777",
        "--tmpfs",
        "/run:rw,noexec,nosuid,nodev,size=16777216,mode=755",
        "--workdir",
        "/home/lab",
        "--env",
        "HOME=/home/lab",
        "--log-driver=json-file",
        "--log-opt=max-size=10m",
        "--log-opt=max-file=2",
    ]);
    args.extend(gpu_args(&w.gpu_ids, gpus)?);
    args.push(image.into());
    match w.template.as_str() {
        "terminal" => args.extend(strings(&["sleep", "infinity"])),
        "jupyter" => args.extend(strings(&[
            "jupyter",
            "lab",
            "--ip=0.0.0.0",
            "--port=8080",
            "--no-browser",
            "--ServerApp.root_dir=/home/lab",
            "--IdentityProvider.token=",
            "--ServerApp.password=",
            "--ServerApp.allow_remote_access=True",
        ])),
        "code" => args.extend(strings(&[
            "--bind-addr",
            "0.0.0.0:8080",
            "--auth",
            "none",
            "--disable-telemetry",
            "--disable-update-check",
            "/home/lab",
        ])),
        _ => unreachable!(),
    };
    Ok(args)
}
/// Resolve only identifiers discovered on this host; client input cannot supply device paths.
fn gpu_args(ids: &[String], gpus: &[crate::model::Gpu]) -> anyhow::Result<Vec<String>> {
    anyhow::ensure!(
        ids.len() <= 8 && ids.iter().collect::<std::collections::HashSet<_>>().len() == ids.len(),
        "Choose up to eight distinct GPUs"
    );
    let mut args = Vec::new();
    let mut nvidia = Vec::new();
    let mut amd = false;
    for id in ids {
        let gpu = gpus
            .iter()
            .find(|g| &g.id == id)
            .context("Requested GPU is no longer detected on this node")?;
        match gpu.access.as_str() {
            "nvidia" => {
                anyhow::ensure!(
                    id.starts_with("GPU-")
                        && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'),
                    "Invalid NVIDIA GPU ID"
                );
                nvidia.push(id.as_str());
            }
            "dri" => {
                anyhow::ensure!(
                    id.strip_prefix("renderD")
                        .is_some_and(|v| !v.is_empty() && v.chars().all(|c| c.is_ascii_digit())),
                    "Invalid render device"
                );
                let path = format!("/dev/dri/{id}");
                args.push(format!("--device={path}"));
                #[cfg(target_os = "linux")]
                {
                    use std::os::unix::fs::{FileTypeExt, MetadataExt};
                    let metadata =
                        std::fs::metadata(&path).context("GPU render device is missing")?;
                    anyhow::ensure!(
                        metadata.file_type().is_char_device(),
                        "Invalid GPU render device"
                    );
                    args.push(format!("--group-add={}", metadata.gid()));
                }
                amd |= gpu.vendor == "AMD";
            }
            _ => bail!("This GPU cannot be passed to a workspace on this platform"),
        }
    }
    if !nvidia.is_empty() {
        // Docker parses --gpus as CSV; quote a list of device identifiers as one field.
        args.extend(strings(&[
            "--gpus",
            &format!("\"device={}\"", nvidia.join(",")),
        ]));
    }
    if amd {
        #[cfg(target_os = "linux")]
        if let Ok(metadata) = std::fs::metadata("/dev/kfd") {
            use std::os::unix::fs::{FileTypeExt, MetadataExt};
            anyhow::ensure!(
                metadata.file_type().is_char_device(),
                "Invalid AMD compute device"
            );
            args.push("--device=/dev/kfd".into());
            args.push(format!("--group-add={}", metadata.gid()));
        }
    }
    Ok(args)
}
async fn inspect(id: &str, node: &str) -> anyhow::Result<Value> {
    let name = container_name(id)?;
    let text = docker(&strings(&["inspect", &name]), 15).await?;
    let values: Vec<Value> = serde_json::from_str(&text)?;
    let v = values
        .into_iter()
        .next()
        .context("Container was not found")?;
    anyhow::ensure!(
        v["Config"]["Labels"]["cloudlab.node"].as_str() == Some(node)
            && v["Config"]["Labels"]["cloudlab.workspace"].as_str() == Some(id),
        "Container ownership verification failed"
    );
    Ok(v)
}
/// A byte stream to a fixed port *inside* the verified container. Nothing is
/// published on the host, so internal Docker networks work on every platform.
pub struct AppStream {
    reader: tokio::process::ChildStdout,
    writer: tokio::process::ChildStdin,
    _child: tokio::process::Child,
}
impl tokio::io::AsyncRead for AppStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.reader).poll_read(cx, buf)
    }
}
impl tokio::io::AsyncWrite for AppStream {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        std::pin::Pin::new(&mut self.writer).poll_write(cx, buf)
    }
    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.writer).poll_flush(cx)
    }
    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.writer).poll_shutdown(cx)
    }
}
pub async fn app_stream(id: &str, node: &str) -> anyhow::Result<AppStream> {
    let value = inspect(id, node).await?;
    anyhow::ensure!(
        value["State"]["Running"] == true,
        "Container is not running"
    );
    let mut child = Command::new("docker")
        .args([
            "exec",
            "--interactive",
            "--user",
            "1000:1000",
            &container_name(id)?,
            "/usr/bin/socat",
            "STDIO",
            "TCP:127.0.0.1:8080,connect-timeout=5",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()?;
    Ok(AppStream {
        reader: child.stdout.take().unwrap(),
        writer: child.stdin.take().unwrap(),
        _child: child,
    })
}
pub async fn containers(node: &str) -> anyhow::Result<HashMap<String, String>> {
    anyhow::ensure!(valid_id(node), "Invalid node ID");
    let text = docker(
        &strings(&[
            "ps",
            "--all",
            "--filter",
            &format!("label=cloudlab.node={node}"),
            "--format",
            "{{.Label \"cloudlab.workspace\"}} {{.State}}",
        ]),
        10,
    )
    .await?;
    Ok(text
        .lines()
        .filter_map(|line| {
            let (id, status) = line.split_once(' ')?;
            Some((
                id.into(),
                if status == "running" {
                    "running"
                } else {
                    "stopped"
                }
                .into(),
            ))
        })
        .collect())
}
pub async fn execute(
    w: &Workspace,
    node: &str,
    action: &str,
    command: &str,
    allow_network: bool,
) -> anyhow::Result<String> {
    let output = execute_inner(w, node, action, command, allow_network).await?;
    if matches!(action, "create" | "start") && w.template != "terminal" {
        let name = container_name(&w.id)?;
        for attempt in 0..30 {
            if docker(
                &strings(&[
                    "exec",
                    "--user",
                    "1000:1000",
                    &name,
                    "/usr/bin/socat",
                    "-u",
                    "OPEN:/dev/null",
                    "TCP:127.0.0.1:8080,connect-timeout=1",
                ]),
                5,
            )
            .await
            .is_ok()
            {
                return Ok(output);
            }
            if attempt < 29 {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
        bail!("The container started, but its application did not become ready. Check the container logs on the node.");
    }
    Ok(output)
}
async fn execute_inner(
    w: &Workspace,
    node: &str,
    action: &str,
    command: &str,
    allow_network: bool,
) -> anyhow::Result<String> {
    let name = container_name(&w.id)?;
    anyhow::ensure!(valid_id(node), "Invalid node ID");
    if matches!(action, "create" | "update") {
        let gpus = if w.gpu_ids.is_empty() {
            Vec::new()
        } else {
            crate::telemetry::gpus().await
        };
        let mut args = run_args(w, node, allow_network, &gpus)?;
        if action == "update" {
            // Validate the image and all resource/device arguments before
            // removing anything. Never stop a container behind the user's back.
            let image = match w.template.as_str() {
                "terminal" => "cloudlab/terminal:2",
                "jupyter" => "cloudlab/jupyter:2",
                "code" => "cloudlab/code:2",
                _ => unreachable!(),
            };
            docker(&strings(&["image", "inspect", image]), 15)
                .await
                .context("Workspace image is missing. Build the images on this node first.")?;
            if docker(&strings(&["inspect", &name]), 15).await.is_ok() {
                let existing = inspect(&w.id, node).await?;
                anyhow::ensure!(
                    existing["State"]["Running"] == false,
                    "Stop the workspace on its node before applying settings."
                );
                docker(&strings(&["rm", &name]), 40).await?;
            }
            // Docker create keeps the replacement stopped and reuses exactly
            // the same persistent home volume; no volume is removed.
            args[0] = "create".into();
            args.remove(1); // --detach belongs to docker run, not create.
            if !w.network {
                cleanup_network(&name, node).await?;
            }
        }
        if action == "create" {
            if let Ok(existing) = inspect(&w.id, node).await {
                if existing["State"]["Running"] == true {
                    return Ok("Container is already running".into());
                }
                return docker(&strings(&["start", &name]), 60).await;
            }
        }
        if w.network {
            let network = format!("{name}-net");
            let mut net = strings(&[
                "network",
                "create",
                "--label",
                &format!("cloudlab.node={node}"),
                "--label",
                &format!("cloudlab.workspace={}", w.id),
            ]);
            net.push(network.clone());
            if docker(&strings(&["network", "inspect", &network]), 10)
                .await
                .is_err()
            {
                docker(&net, 20).await?;
            } else {
                let text = docker(&strings(&["network", "inspect", &network]), 10).await?;
                let info: Vec<Value> = serde_json::from_str(&text)?;
                anyhow::ensure!(
                    info[0]["Labels"]["cloudlab.node"] == node && info[0]["Internal"] == false,
                    "Network policy does not match this workspace"
                );
            }
        }
        return docker(&args,120).await.context("Container creation failed. Build the workspace images on this node with ./scripts/build-images.sh");
    }
    if action == "delete" && inspect(&w.id, node).await.is_err() {
        // A missing container is an idempotent delete, but an existing foreign
        // container is never touched. Network cleanup also verifies ownership.
        if docker(&strings(&["inspect", &name]), 10).await.is_ok() {
            bail!("Container ownership verification failed");
        }
        cleanup_network(&name, node).await?;
        return Ok("Container already removed; volume retained".into());
    }
    inspect(&w.id, node).await?;
    match action {
        "start" => docker(&strings(&["start", &name]), 60).await,
        "stop" => docker(&strings(&["stop", "--time", "15", &name]), 40).await,
        "delete" => {
            docker(&strings(&["rm", "--force", &name]), 40).await?;
            cleanup_network(&name, node).await?;
            Ok("Container removed. Persistent volume retained.".into())
        }
        "exec" => {
            anyhow::ensure!(
                !command.trim().is_empty() && command.len() <= 4096,
                "Invalid command"
            );
            docker(
                &strings(&[
                    "exec",
                    "--user",
                    "1000:1000",
                    "--workdir",
                    "/home/lab",
                    &name,
                    "timeout",
                    "--signal=KILL",
                    "30",
                    "sh",
                    "-lc",
                    command,
                ]),
                40,
            )
            .await
        }
        _ => bail!("Unsupported container operation"),
    }
}
async fn cleanup_network(name: &str, node: &str) -> anyhow::Result<()> {
    let network = format!("{name}-net");
    if let Ok(text) = docker(&strings(&["network", "inspect", &network]), 10).await {
        let info: Vec<Value> = serde_json::from_str(&text)?;
        anyhow::ensure!(
            info[0]["Labels"]["cloudlab.node"] == node,
            "Network ownership verification failed"
        );
        docker(&strings(&["network", "rm", &network]), 20).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn workspace() -> Workspace {
        Workspace {
            id: crate::model::id(),
            lab_id: crate::model::id(),
            node_id: crate::model::id(),
            name: "Test".into(),
            template: "terminal".into(),
            cpus: 2,
            memory_mb: 512,
            status: "creating".into(),
            created_at: 0,
            last_used: 0,
            error: "".into(),
            network: false,
            gpu_ids: Vec::new(),
            metrics: None,
            history: Vec::new(),
        }
    }
    #[test]
    fn sandbox_enforces_boundaries() {
        let w = workspace();
        let args = run_args(&w, &w.node_id, false, &[]).unwrap();
        for flag in [
            "--read-only",
            "--cap-drop=ALL",
            "--security-opt=no-new-privileges:true",
            "--pids-limit=256",
            "1000:1000",
            "--memory-swap",
        ] {
            assert!(args.iter().any(|v| v == flag));
        }
        assert!(!args
            .iter()
            .any(|v| v.contains("docker.sock") || v == "--privileged" || v.contains("type=bind")));
    }
    #[test]
    fn gpu_devices_require_discovery_and_safe_identifiers() {
        use crate::model::Gpu;
        let gpu = Gpu {
            id: "GPU-abc-123".into(),
            access: "nvidia".into(),
            ..Default::default()
        };
        let ids = vec![gpu.id.clone()];
        assert!(gpu_args(&ids, &[]).is_err());
        assert_eq!(
            gpu_args(&ids, std::slice::from_ref(&gpu)).unwrap(),
            vec!["--gpus", "\"device=GPU-abc-123\""]
        );
        let mut w = workspace();
        w.gpu_ids = ids.clone();
        let args = run_args(&w, &w.node_id, false, std::slice::from_ref(&gpu)).unwrap();
        assert!(
            args.iter().position(|v| v == "--gpus").unwrap()
                < args
                    .iter()
                    .position(|v| v == "cloudlab/terminal:2")
                    .unwrap()
        );
        assert!(gpu_args(
            &[gpu.id.clone(), gpu.id.clone()],
            std::slice::from_ref(&gpu)
        )
        .is_err());
        for (id, access) in [
            ("GPU-123,all", "nvidia"),
            ("../../mem", "dri"),
            ("renderD128/../../mem", "dri"),
            ("mac-Apple", "none"),
        ] {
            assert!(gpu_args(
                &[id.into()],
                &[Gpu {
                    id: id.into(),
                    access: access.into(),
                    ..Default::default()
                }]
            )
            .is_err());
        }
    }
    #[test]
    fn rejects_arbitrary_images_ids_and_network() {
        let mut w = workspace();
        w.template = "alpine; touch /tmp/bad".into();
        assert!(run_args(&w, &w.node_id, false, &[]).is_err());
        w.template = "terminal".into();
        w.network = true;
        assert!(run_args(&w, &w.node_id, false, &[]).is_err());
        w.network = false;
        w.id = "../../etc".into();
        assert!(run_args(&w, &w.node_id, false, &[]).is_err());
    }
}
