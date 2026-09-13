use crate::{model::*, relay::agent_connection, sandbox};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::Mutex;
#[derive(Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub id: String,
    pub token: String,
    pub coordinator: String,
}
pub fn validate_coordinator(value: &str) -> anyhow::Result<url::Url> {
    let u = url::Url::parse(value)?;
    let local = matches!(u.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if let Some(url::Host::Domain(host)) = u.host() {
        anyhow::ensure!(
            host.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.'),
            "Invalid coordinator hostname"
        );
    }
    anyhow::ensure!(
        u.scheme() == "https" || (u.scheme() == "http" && local),
        "Use HTTPS for remote coordinators; HTTP is only allowed on localhost"
    );
    anyhow::ensure!(
        u.username().is_empty()
            && u.password().is_none()
            && u.path() == "/"
            && u.query().is_none()
            && u.fragment().is_none(),
        "Coordinator URL must be an origin without credentials or a path"
    );
    Ok(u)
}
pub async fn run(
    coordinator: String,
    enrollment: Option<String>,
    dir: PathBuf,
    allow_network: bool,
) -> anyhow::Result<()> {
    let coordinator = validate_coordinator(&coordinator)?
        .as_str()
        .trim_end_matches('/')
        .to_string();
    private_dir(&dir)?;
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(30))
        .build()?;
    let path = dir.join("credentials.json");
    let mut system = sysinfo::System::new_all();
    system.refresh_all();
    let credentials = if path.exists() {
        anyhow::ensure!(
            enrollment.is_none(),
            "This agent is already enrolled. Use a different --data-dir to enroll another node."
        );
        let c: Credentials = serde_json::from_slice(&std::fs::read(&path)?)?;
        anyhow::ensure!(
            c.coordinator == coordinator,
            "Stored credentials belong to a different coordinator"
        );
        c
    } else {
        let token = enrollment
            .context("Pair this node first: generate an enrollment command in the dashboard")?;
        let response=client.post(format!("{coordinator}/api/agent/enroll")).json(&json!({"token":token,"platform":sysinfo::System::name().unwrap_or_else(||std::env::consts::OS.into()),"arch":std::env::consts::ARCH,"cpus":system.cpus().len(),"memory_mb":system.total_memory()/1024/1024})).send().await?;
        let value: Value = response.error_for_status()?.json().await?;
        let c = Credentials {
            id: value["id"].as_str().context("Missing node ID")?.into(),
            token: value["token"]
                .as_str()
                .context("Missing credential")?
                .into(),
            coordinator: coordinator.clone(),
        };
        write_private(&path, &serde_json::to_vec_pretty(&c)?)?;
        c
    };
    anyhow::ensure!(
        valid_id(&credentials.id) && credentials.token.len() == 64,
        "Invalid agent credential file"
    );
    eprintln!(
        "CloudLab node {} enrolled; host shell access is disabled",
        credentials.id
    );
    let c = credentials.clone();
    tokio::spawn(async move {
        loop {
            if let Err(e) = agent_connection(c.clone()).await {
                eprintln!("Workspace relay disconnected: {e}");
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
    // Sensor tools run separately so an unavailable driver cannot block heartbeats or jobs.
    let samples = Arc::new(Mutex::new((
        None::<Metrics>,
        std::collections::HashMap::<String, Metrics>::new(),
    )));
    let collected = samples.clone();
    let telemetry_node = credentials.id.clone();
    tokio::spawn(async move {
        let mut system = sysinfo::System::new_all();
        let mut networks = sysinfo::Networks::new_with_refreshed_list();
        loop {
            let (gpus, workspaces) = tokio::join!(
                crate::telemetry::gpus(),
                crate::telemetry::workspace_metrics(&telemetry_node)
            );
            system.refresh_cpu_usage();
            system.refresh_memory();
            networks.refresh(true);
            let metrics = Metrics {
                at: now(),
                cpu_usage: system.global_cpu_usage(),
                memory_used_mb: system.used_memory() / 1048576,
                memory_total_mb: system.total_memory() / 1048576,
                network_rx_bytes: networks.values().map(|n| n.total_received()).sum(),
                network_tx_bytes: networks.values().map(|n| n.total_transmitted()).sum(),
                gpus,
                ..Default::default()
            };
            *collected.lock().await = (Some(metrics), workspaces);
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
    let working = Arc::new(Mutex::new(false));
    loop {
        system.refresh_cpu_usage();
        system.refresh_memory();
        let docker = sandbox::available().await;
        let ready = !*working.lock().await;
        let containers = if docker {
            sandbox::containers(&credentials.id)
                .await
                .unwrap_or_default()
        } else {
            Default::default()
        };
        // Heartbeats continue while a Docker operation runs; poll may lease only
        // when this process is ready to execute a job.
        {
            let (metrics, workspace_metrics) = samples.lock().await.clone();
            let request=client.post(format!("{coordinator}/api/agent/poll")).bearer_auth(&credentials.token).json(&json!({"cpu_usage":system.global_cpu_usage(),"memory_used_mb":system.used_memory()/1024/1024,"docker":docker,"containers":containers,"ready":ready,"metrics":metrics,"workspace_metrics":workspace_metrics})).send().await;
            match request {
                Ok(response) if response.status().is_success()=>{
                    let value:Value=response.json().await?;
                    if !value["job"].is_null(){let job:Job=serde_json::from_value(value["job"].clone())?;*working.lock().await=true;let done=working.clone();let client=client.clone();let credentials=credentials.clone();let dir=dir.clone();
                        tokio::spawn(async move{
                            let receipt=dir.join(format!("receipt-{}.json",job.id));
                            let result=if receipt.exists(){std::fs::read(&receipt).ok().and_then(|v|serde_json::from_slice::<Value>(&v).ok())}else{None};
                            let result=if let Some(result)=result{result}else{
                                let outcome=if job.node_id!=credentials.id||job.workspace.node_id!=credentials.id {Err(anyhow::anyhow!("Job belongs to a different node"))}else{sandbox::execute(&job.workspace,&credentials.id,&job.action,&job.command,allow_network).await};
                                let v=match outcome{Ok(output)=>json!({"ok":true,"output":output}),Err(e)=>json!({"ok":false,"error":format!("{e:#}").chars().take(3500).collect::<String>()})};
                                if let Err(e)=write_private(&receipt,&serde_json::to_vec(&v).unwrap()){eprintln!("Could not persist job receipt: {e}");}v
                            };
                            for _ in 0..12 {match client.post(format!("{}/api/agent/jobs/{}",credentials.coordinator,job.id)).bearer_auth(&credentials.token).json(&result).send().await {Ok(r) if r.status().is_success()=>break,_=>tokio::time::sleep(Duration::from_secs(5)).await}}
                            *done.lock().await=false;
                        });
                    }
                },
                Ok(response) if response.status()==reqwest::StatusCode::UNAUTHORIZED=>anyhow::bail!("This node's credential was revoked. Stop its containers locally before enrolling again."),
                Ok(response)=>eprintln!("Coordinator returned {}",response.status()),
                Err(e)=>eprintln!("Coordinator unavailable: {e}"),
            }
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
