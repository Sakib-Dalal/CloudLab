use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
pub fn id() -> String {
    Uuid::new_v4().to_string()
}
pub fn token() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}
pub fn hash(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}
pub fn valid_id(value: &str) -> bool {
    Uuid::parse_str(value).is_ok()
}
pub fn valid_name(value: &str) -> bool {
    (2..=60).contains(&value.trim().chars().count()) && !value.chars().any(char::is_control)
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct Gpu {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub utilization: Option<f32>,
    pub memory_used_mb: Option<u64>,
    pub memory_total_mb: Option<u64>,
    pub temperature_c: Option<f32>,
    pub power_watts: Option<f32>,
    #[serde(default)]
    pub shared_memory: bool,
    #[serde(default)]
    pub access: String,
    #[serde(default)]
    pub access_note: String,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct Metrics {
    pub at: u64,
    pub cpu_usage: f32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub disk_read_bytes: Option<u64>,
    pub disk_write_bytes: Option<u64>,
    pub pids: Option<u64>,
    #[serde(default)]
    pub gpus: Vec<Gpu>,
}

/// One hour of samples, held in memory rather than rewriting telemetry to disk.
pub fn record_metrics(history: &mut Vec<Metrics>, sample: &Metrics) {
    history.retain(|m| m.at + 3600 > sample.at);
    if history.last().is_none_or(|m| sample.at >= m.at + 10) {
        history.push(sample.clone());
    }
    if history.len() > 360 {
        history.drain(..history.len() - 360);
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Lab {
    pub id: String,
    pub name: String,
    pub description: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub lab_id: String,
    pub name: String,
    pub platform: String,
    pub arch: String,
    pub cpus: u32,
    pub memory_mb: u64,
    pub cpu_usage: f32,
    pub memory_used_mb: u64,
    pub last_seen: u64,
    pub docker: bool,
    pub revoked: bool,
    #[serde(default)]
    pub credential_hash: String,
    #[serde(default)]
    pub gpus: Vec<Gpu>,
    #[serde(default)]
    pub metrics: Option<Metrics>,
    #[serde(skip)]
    pub history: Vec<Metrics>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub lab_id: String,
    pub node_id: String,
    pub name: String,
    pub template: String,
    pub cpus: u32,
    pub memory_mb: u64,
    pub status: String,
    pub created_at: u64,
    pub last_used: u64,
    pub error: String,
    pub network: bool,
    #[serde(default)]
    pub gpu_ids: Vec<String>,
    #[serde(default)]
    pub metrics: Option<Metrics>,
    #[serde(skip)]
    pub history: Vec<Metrics>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub lab_id: String,
    pub message: String,
    pub at: u64,
    pub kind: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Settings {
    pub name: String,
    pub session_hours: u64,
    pub idle_minutes: u64,
    pub max_workspaces: usize,
    pub default_cpus: u32,
    pub default_memory_mb: u64,
    pub allow_network: bool,
    pub public_url: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            name: "My CloudLab".into(),
            session_hours: 12,
            idle_minutes: 60,
            max_workspaces: 12,
            default_cpus: 2,
            default_memory_mb: 2048,
            allow_network: false,
            public_url: String::new(),
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Access {
    pub id: String,
    pub lab_id: String,
    pub name: String,
    pub role: String,
    pub expires_at: u64,
    pub token_hash: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Enrollment {
    pub token_hash: String,
    pub lab_id: String,
    pub name: String,
    pub expires_at: u64,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Session {
    pub token_hash: String,
    pub access_id: Option<String>,
    pub expires_at: u64,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub node_id: String,
    pub workspace: Workspace,
    pub action: String,
    pub command: String,
    pub status: String,
    pub output: String,
    pub error: String,
    pub lease_until: u64,
    pub created_at: u64,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct AppSession {
    pub token_hash: String,
    pub workspace_id: String,
    pub session_hash: String,
    pub expires_at: u64,
    pub ticket: bool,
}
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Database {
    pub labs: Vec<Lab>,
    pub nodes: Vec<Node>,
    pub workspaces: Vec<Workspace>,
    pub events: Vec<Event>,
    pub settings: Settings,
    pub access: Vec<Access>,
    pub enrollments: Vec<Enrollment>,
    pub sessions: Vec<Session>,
    pub jobs: Vec<Job>,
    pub app_sessions: Vec<AppSession>,
}
impl Database {
    pub fn event(&mut self, lab: &str, message: String, kind: &str) {
        self.events.push(Event {
            id: id(),
            lab_id: lab.into(),
            message,
            at: now(),
            kind: kind.into(),
        });
        if self.events.len() > 1000 {
            self.events.remove(0);
        }
    }
    pub fn enqueue(&mut self, workspace: Workspace, action: &str, command: &str) -> Job {
        let job = Job {
            id: id(),
            node_id: workspace.node_id.clone(),
            workspace,
            action: action.into(),
            command: command.into(),
            status: "queued".into(),
            output: String::new(),
            error: String::new(),
            lease_until: 0,
            created_at: now(),
        };
        self.jobs.push(job.clone());
        job
    }
    pub fn prune(&mut self) {
        let t = now();
        self.sessions.retain(|s| s.expires_at > t);
        self.enrollments.retain(|s| s.expires_at > t);
        self.app_sessions.retain(|s| s.expires_at > t);
        self.jobs
            .retain(|j| j.created_at + 86400 > t || j.status == "queued" || j.status == "leased");
    }
}
pub struct Store {
    pub db: Database,
    path: PathBuf,
}
impl Store {
    pub fn open(dir: &Path) -> anyhow::Result<Self> {
        private_dir(dir)?;
        let path = dir.join("state.json");
        let db = if path.exists() {
            serde_json::from_slice(&std::fs::read(&path)?)?
        } else {
            let mut db = Database::default();
            db.labs.push(Lab {
                id: id(),
                name: "Home lab".into(),
                description: "A little space for big ideas.".into(),
            });
            db
        };
        let s = Self { db, path };
        s.save()?;
        Ok(s)
    }
    pub fn save(&self) -> anyhow::Result<()> {
        write_private(&self.path, &serde_json::to_vec_pretty(&self.db)?)
    }
}
pub fn private_dir(path: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}
pub fn write_private(path: &Path, contents: &[u8]) -> anyhow::Result<()> {
    use std::io::Write;
    let temp = path.with_extension(format!("{}.tmp", id()));
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut file = opts.open(&temp)?;
    file.write_all(contents)?;
    file.sync_all()?;
    drop(file);
    std::fs::rename(temp, path)?;
    #[cfg(unix)]
    {
        if let Some(parent) = path.parent() {
            std::fs::File::open(parent)?.sync_all()?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod telemetry_tests {
    use super::*;
    #[test]
    fn history_is_bounded_deduplicated_and_not_persisted() {
        let mut history = Vec::new();
        for at in 1..=4000 {
            record_metrics(
                &mut history,
                &Metrics {
                    at,
                    ..Default::default()
                },
            );
        }
        assert!(history.len() <= 360);
        assert!(history.windows(2).all(|v| v[1].at - v[0].at >= 10));
        assert!(history.first().unwrap().at + 3600 > 4000);
        let old = serde_json::json!({"id":id(),"lab_id":id(),"node_id":id(),"name":"Old workspace","template":"terminal","cpus":2,"memory_mb":1024,"status":"running","created_at":0,"last_used":0,"error":"","network":false});
        let mut workspace: Workspace = serde_json::from_value(old).unwrap();
        assert!(workspace.gpu_ids.is_empty());
        assert!(workspace.metrics.is_none());
        workspace.history = history;
        assert!(serde_json::to_value(workspace)
            .unwrap()
            .get("history")
            .is_none());
    }
}
