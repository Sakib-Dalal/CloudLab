//! Best-effort hardware and container telemetry. Unsupported sensors stay null.
use crate::{
    model::{now, valid_id, Gpu, Metrics},
    sandbox,
};
use serde_json::Value;
use std::collections::HashMap;

async fn command(program: &str, args: &[&str]) -> Option<String> {
    sandbox::host_command(
        program,
        &args.iter().map(|v| (*v).into()).collect::<Vec<_>>(),
        4,
    )
    .await
    .ok()
}
fn number(v: &str) -> Option<f32> {
    v.trim()
        .parse::<f32>()
        .ok()
        .filter(|v| v.is_finite() && *v >= 0.)
}
fn nvidia(text: &str) -> Vec<Gpu> {
    text.lines()
        .filter_map(|line| {
            let p: Vec<_> = line.split(',').map(str::trim).collect();
            if p.len() != 8 || !p[0].starts_with("GPU-") {
                return None;
            }
            Some(Gpu {
                id: p[0].into(),
                name: p[1].into(),
                vendor: "NVIDIA".into(),
                utilization: number(p[2]).map(|v| v.min(100.)),
                memory_used_mb: number(p[3]).map(|v| v as u64),
                memory_total_mb: number(p[4]).map(|v| v as u64),
                temperature_c: number(p[5]),
                power_watts: number(p[6]),
                access: "none".into(),
                access_note: "Configure NVIDIA Container Toolkit on the Docker host.".into(),
                ..Default::default()
            })
        })
        .collect()
}

#[cfg(any(target_os = "macos", test))]
fn mac_gpus(text: &str) -> Vec<Gpu> {
    fn stat(text: &str, key: &str) -> Option<f32> {
        let tail = text.split_once(&format!("\"{key}\"="))?.1;
        number(
            tail.split(|c: char| !(c.is_ascii_digit() || c == '.'))
                .next()?,
        )
    }
    text.split("+-o ").filter_map(|entry| {
        let name = entry.lines().find_map(|line| {
            line.split_once("\"model\" = \"").map(|(_, v)| v.split('"').next().unwrap_or("Mac GPU"))
        })?;
        let stats = entry.lines().find(|l| l.contains("\"PerformanceStatistics\""));
        let apple = name.to_lowercase().contains("apple");
        Some(Gpu {
            id: format!("mac-{}", name.replace(' ', "-")), name: name.into(),
            vendor: if apple { "Apple" } else if name.to_lowercase().contains("amd") { "AMD" } else if name.to_lowercase().contains("intel") { "Intel" } else { "Other" }.into(),
            utilization: stats.and_then(|s| stat(s, "Device Utilization %")).map(|v| v.min(100.)),
            memory_used_mb: stats.and_then(|s| stat(s, "In use system memory")).map(|v| (v as u64) / 1048576),
            shared_memory: apple,
            access: "none".into(),
            access_note: "Mac GPU detected. Docker Desktop does not expose Metal GPUs to these Linux workspaces.".into(),
            ..Default::default()
        })
    }).collect()
}

#[cfg(target_os = "linux")]
fn linux_gpus() -> Vec<Gpu> {
    use std::path::Path;
    let read = |p: &Path| {
        std::fs::read_to_string(p)
            .ok()
            .map(|s| s.trim().to_string())
    };
    let mut devices = Vec::new();
    let Ok(entries) = std::fs::read_dir("/sys/class/drm") else {
        return devices;
    };
    for entry in entries.flatten() {
        let id = entry.file_name().to_string_lossy().into_owned();
        if !id
            .strip_prefix("renderD")
            .is_some_and(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
        {
            continue;
        }
        let path = entry.path().join("device");
        let vendor = match read(&path.join("vendor")).as_deref() {
            Some("0x1002") => "AMD",
            Some("0x8086") => "Intel",
            Some("0x10de") => "NVIDIA",
            _ => "Other",
        };
        let value = |name: &str| read(&path.join(name)).and_then(|s| s.parse::<u64>().ok());
        let name = read(&path.join("product_name"))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                format!(
                    "{vendor} GPU {}",
                    read(&path.join("device")).unwrap_or_else(|| id.clone())
                )
            });
        let hwmon = std::fs::read_dir(path.join("hwmon"))
            .ok()
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .next();
        let sensor = |name: &str| {
            hwmon
                .as_ref()
                .and_then(|p| read(&p.join(name)))
                .and_then(|s| number(&s))
        };
        devices.push(Gpu {
            temperature_c: sensor("temp1_input").map(|v| v / 1000.),
            power_watts: sensor("power1_average")
                .or_else(|| sensor("power1_input"))
                .map(|v| v / 1_000_000.),
            id,
            name,
            vendor: vendor.into(),
            utilization: value("gpu_busy_percent").map(|v| v.min(100) as f32),
            memory_used_mb: value("mem_info_vram_used").map(|v| v / 1048576),
            memory_total_mb: value("mem_info_vram_total").map(|v| v / 1048576),
            access: "none".into(),
            access_note: "GPU sensors and container access depend on this device's Linux driver."
                .into(),
            ..Default::default()
        });
    }
    devices.sort_by(|a, b| a.id.cmp(&b.id));
    devices
}

pub async fn gpus() -> Vec<Gpu> {
    let mut devices = command("nvidia-smi", &["--query-gpu=uuid,name,utilization.gpu,memory.used,memory.total,temperature.gpu,power.draw,index", "--format=csv,noheader,nounits"])
        .await.map(|s| nvidia(&s)).unwrap_or_default();
    #[cfg(target_os = "macos")]
    {
        if let Some(text) = command(
            "/usr/sbin/ioreg",
            &["-r", "-c", "IOAccelerator", "-d", "1", "-l"],
        )
        .await
        {
            devices.extend(mac_gpus(&text));
        }
        if devices.is_empty() {
            if let Some(text) = command(
                "/usr/sbin/system_profiler",
                &["SPDisplaysDataType", "-json"],
            )
            .await
            {
                if let Ok(v) = serde_json::from_str::<Value>(&text) {
                    for (index, item) in v["SPDisplaysDataType"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .enumerate()
                    {
                        let name = item["sppci_model"].as_str().unwrap_or("Mac GPU");
                        devices.push(Gpu { id: format!("mac-{index}"), name: name.into(), vendor: if name.contains("Apple") { "Apple" } else { "Other" }.into(), shared_memory: name.contains("Apple"), access: "none".into(), access_note: "GPU detected; sensors unavailable. Docker Desktop does not expose Mac GPUs to Linux workspaces.".into(), ..Default::default() });
                    }
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        let has_nvidia = devices.iter().any(|g| g.vendor == "NVIDIA");
        devices.extend(
            linux_gpus()
                .into_iter()
                .filter(|g| !has_nvidia || g.vendor != "NVIDIA"),
        );
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(text) = command("powershell.exe", &["-NoProfile", "-NonInteractive", "-Command", "Get-CimInstance Win32_VideoController | Select-Object Name,PNPDeviceID | ConvertTo-Json -Compress"]).await {
            if let Ok(v) = serde_json::from_str::<Value>(&text) {
                let rows = v.as_array().cloned().unwrap_or_else(|| vec![v]);
                for row in rows {
                    let name = row["Name"].as_str().unwrap_or("GPU");
                    if name.contains("NVIDIA") && devices.iter().any(|g| g.vendor == "NVIDIA") { continue; }
                    devices.push(Gpu { id: row["PNPDeviceID"].as_str().unwrap_or(name).into(), name: name.into(), vendor: if name.contains("Intel") { "Intel" } else if name.contains("AMD") { "AMD" } else { "Other" }.into(), access: "none".into(), access_note: "Run the agent inside a supported Linux or WSL2 GPU environment for container access. Sensors are unavailable from this driver.".into(), ..Default::default() });
                }
            }
        }
    }
    // Only advertise passthrough for a local Linux engine, never a remote Docker context.
    #[cfg(target_os = "linux")]
    if !devices.is_empty() && local_docker().await {
        let info = command("docker", &["info", "--format", "{{json .Runtimes}}"])
            .await
            .unwrap_or_default();
        let nvidia_runtime = serde_json::from_str::<Value>(&info)
            .ok()
            .is_some_and(|v| v.get("nvidia").is_some());
        let wsl = std::fs::read_to_string("/proc/sys/kernel/osrelease")
            .unwrap_or_default()
            .to_lowercase()
            .contains("microsoft");
        for gpu in &mut devices {
            if gpu.vendor == "NVIDIA" && gpu.id.starts_with("GPU-") && (nvidia_runtime || wsl) {
                gpu.access = "nvidia".into();
                gpu.access_note = "NVIDIA compute access. The workspace image also needs compatible CUDA libraries.".into();
            } else if matches!(gpu.vendor.as_str(), "AMD" | "Intel")
                && std::path::Path::new(&format!("/dev/dri/{}", gpu.id)).exists()
            {
                gpu.access = "dri".into();
                gpu.access_note = "Linux render-device access. Compatible ROCm, oneAPI, or graphics libraries must be installed in the workspace image.".into();
            }
        }
    }
    devices.truncate(32);
    devices
}

#[cfg(target_os = "linux")]
async fn local_docker() -> bool {
    if std::env::var("DOCKER_HOST").is_ok_and(|v| !v.starts_with("unix://")) {
        return false;
    }
    let Some(text) = command(
        "docker",
        &[
            "context",
            "inspect",
            "--format",
            "{{.Endpoints.docker.Host}}",
        ],
    )
    .await
    else {
        return false;
    };
    text.trim().starts_with("unix://")
        && command("docker", &["info", "--format", "{{.OperatingSystem}}"])
            .await
            .is_some_and(|v| {
                !v.to_lowercase().contains("docker desktop")
                    || std::fs::read_to_string("/proc/sys/kernel/osrelease")
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains("microsoft")
            })
}

/// Docker's display values use SI/IEC units and percentages per single CPU core.
fn bytes(text: &str) -> Option<u64> {
    let text = text.trim();
    let split = text
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(text.len());
    let n = number(&text[..split])? as f64;
    let unit = text[split..].trim().to_lowercase();
    let scale: f64 = match unit.as_str() {
        "b" | "" => 1.,
        "kb" => 1e3,
        "mb" => 1e6,
        "gb" => 1e9,
        "tb" => 1e12,
        "kib" => 1024.,
        "mib" => 1048576.,
        "gib" => 1073741824.,
        "tib" => 1099511627776.,
        _ => return None,
    };
    Some((n * scale) as u64)
}
fn pair(v: &Value) -> Option<(u64, u64)> {
    let (a, b) = v.as_str()?.split_once('/')?;
    Some((bytes(a)?, bytes(b)?))
}
fn container_metric(v: &Value) -> Option<(String, Metrics)> {
    let id = v["Name"].as_str()?.strip_prefix("cloudlab-")?;
    if !valid_id(id) {
        return None;
    }
    let (memory, total) = pair(&v["MemUsage"])?;
    let (rx, tx) = pair(&v["NetIO"])?;
    let disk = pair(&v["BlockIO"]);
    Some((
        id.into(),
        Metrics {
            at: now(),
            cpu_usage: number(v["CPUPerc"].as_str()?.trim_end_matches('%'))?,
            memory_used_mb: memory / 1048576,
            memory_total_mb: total / 1048576,
            network_rx_bytes: rx,
            network_tx_bytes: tx,
            disk_read_bytes: disk.map(|p| p.0),
            disk_write_bytes: disk.map(|p| p.1),
            pids: v["PIDs"].as_str().and_then(|s| s.parse().ok()),
            ..Default::default()
        },
    ))
}
pub async fn workspace_metrics(node: &str) -> HashMap<String, Metrics> {
    let Ok(containers) = sandbox::containers(node).await else {
        return HashMap::new();
    };
    let mut args = vec![
        "stats".into(),
        "--no-stream".into(),
        "--format".into(),
        "{{json .}}".into(),
    ];
    args.extend(
        containers
            .iter()
            .filter(|(_, state)| *state == "running")
            .filter_map(|(id, _)| sandbox::container_name(id).ok()),
    );
    if args.len() == 4 {
        return HashMap::new();
    }
    let Ok(text) = sandbox::docker(&args, 6).await else {
        return HashMap::new();
    };
    text.lines()
        .filter_map(|s| serde_json::from_str::<Value>(s).ok())
        .filter_map(|v| container_metric(&v))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unsupported_nvidia_sensors_are_not_zero() {
        let rows = nvidia("GPU-123, RTX 4090, 41, 1024, 24564, [N/A], [Not Supported], 0\n");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].utilization, Some(41.));
        assert_eq!(rows[0].temperature_c, None);
        assert_eq!(rows[0].power_watts, None);
    }
    #[test]
    fn reads_mac_shared_memory_and_utilization() {
        let rows = mac_gpus("+-o AGXAccelerator\n  | \"model\" = \"Apple M5 Pro\"\n  | \"PerformanceStatistics\" = {\"Device Utilization %\"=32,\"In use system memory\"=536870912}");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].utilization, Some(32.));
        assert_eq!(rows[0].memory_used_mb, Some(512));
        assert_eq!(rows[0].access, "none");
    }
    #[test]
    fn docker_units_and_multi_core_usage() {
        let id = crate::model::id();
        let sample = serde_json::json!({"Name":format!("cloudlab-{id}"), "CPUPerc":"245.7%", "MemUsage":"512MiB / 2GiB", "NetIO":"1.25MB / 8kB", "BlockIO":"0B / 4.5MiB", "PIDs":"12"});
        let (parsed, m) = container_metric(&sample).unwrap();
        assert_eq!(parsed, id);
        assert_eq!(m.cpu_usage, 245.7);
        assert_eq!(m.memory_used_mb, 512);
        assert_eq!(m.memory_total_mb, 2048);
        assert_eq!(m.network_rx_bytes, 1250000);
        assert_eq!(m.disk_write_bytes, Some(4718592));
        assert_eq!(m.pids, Some(12));
        assert_eq!(bytes("--"), None);
    }
}
