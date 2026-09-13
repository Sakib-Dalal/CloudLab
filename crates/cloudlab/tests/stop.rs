use std::{net::SocketAddr, process::Stdio, time::Duration};
use tempfile::TempDir;
use tokio::{io::AsyncWriteExt, net::TcpStream, process::Command, time::timeout};

struct Server {
    child: tokio::process::Child,
    directory: TempDir,
    bind: SocketAddr,
    gateway: SocketAddr,
}

fn command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cloudlab"));
    for variable in [
        "CLOUDLAB_BIND",
        "CLOUDLAB_APP_BIND",
        "CLOUDLAB_APP_URL",
        "CLOUDLAB_PUBLIC_URL",
        "CLOUDLAB_DATA_DIR",
        "CLOUDLAB_WEB_DIR",
    ] {
        command.env_remove(variable);
    }
    command.kill_on_drop(true);
    command
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap()
}

impl Server {
    async fn start() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let log_path = directory.path().join("server.log");
        let mut child = command()
            .current_dir(directory.path())
            .args([
                "serve",
                "--bind",
                "127.0.0.1:0",
                "--app-bind",
                "127.0.0.1:0",
            ])
            .stdout(Stdio::null())
            .stderr(std::fs::File::create(&log_path).unwrap())
            .spawn()
            .unwrap();
        let (bind, gateway) = timeout(Duration::from_secs(10), async {
            loop {
                let log = std::fs::read_to_string(&log_path).unwrap();
                assert!(child.try_wait().unwrap().is_none(), "{log}");
                let address = |prefix| {
                    log.lines()
                        .find_map(|line| line.strip_prefix(prefix))
                        .and_then(|value| value.parse::<SocketAddr>().ok())
                };
                if let (Some(bind), Some(gateway)) = (
                    address("CloudLab coordinator: http://"),
                    address("Workspace gateway: "),
                ) {
                    if client()
                        .get(format!("http://{bind}/api/health"))
                        .send()
                        .await
                        .is_ok()
                    {
                        break (bind, gateway);
                    }
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .expect("server did not start");
        Self {
            child,
            directory,
            bind,
            gateway,
        }
    }

    fn stop_command(&self) -> Command {
        let mut command = command();
        command
            .current_dir(self.directory.path())
            .args(["stop", "--bind", &self.bind.to_string()]);
        command
    }

    async fn assert_stopped(&mut self) {
        let status = timeout(Duration::from_secs(8), self.child.wait())
            .await
            .expect("server did not stop within the grace period")
            .unwrap();
        assert!(status.success(), "{status}");
        for address in [self.bind, self.gateway] {
            assert!(
                TcpStream::connect(address).await.is_err(),
                "{address} is still open"
            );
        }
    }
}

#[tokio::test]
async fn stop_requires_owner_key_and_closes_both_servers_with_open_connections() {
    let mut server = Server::start().await;
    let endpoint = format!("http://{}/api/shutdown", server.bind);
    let data = server.directory.path().join(".cloudlab");
    let owner_key = std::fs::read_to_string(data.join("admin-token")).unwrap();

    // Neither unauthenticated requests nor an arbitrary bearer key may stop it.
    for key in [None, Some("0".repeat(64))] {
        let mut request = client().post(&endpoint).header("x-cloudlab-client", "web");
        if let Some(key) = key {
            request = request.bearer_auth(key);
        }
        assert_eq!(request.send().await.unwrap().status(), 401);
    }
    // The browser mutation guard still applies, even with the correct owner key.
    assert_eq!(
        client()
            .post(&endpoint)
            .bearer_auth(owner_key.trim())
            .send()
            .await
            .unwrap()
            .status(),
        403
    );

    let wrong_data = server.directory.path().join("wrong-lab");
    std::fs::create_dir(&wrong_data).unwrap();
    std::fs::write(wrong_data.join("admin-token"), "0".repeat(64)).unwrap();
    let output = timeout(
        Duration::from_secs(12),
        server
            .stop_command()
            .arg("--data-dir")
            .arg(&wrong_data)
            .output(),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("owner key does not match"));
    assert!(server.child.try_wait().unwrap().is_none());

    let saved_state = std::fs::read(data.join("state.json")).unwrap();
    // An unfinished gateway request must not keep the process alive forever.
    let mut connection = TcpStream::connect(server.gateway).await.unwrap();
    connection
        .write_all(b"GET / HTTP/1.1\r\nHost: unfinished")
        .await
        .unwrap();
    let output = timeout(Duration::from_secs(12), server.stop_command().output())
        .await
        .unwrap()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("shutdown requested"));
    server.assert_stopped().await;
    assert_eq!(std::fs::read(data.join("state.json")).unwrap(), saved_state);

    let output = timeout(Duration::from_secs(12), server.stop_command().output())
        .await
        .unwrap()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Cannot reach CloudLab"));
}

#[tokio::test]
async fn stop_honors_environment_and_wildcard_bind_without_stopping_another_lab() {
    let mut target = Server::start().await;
    let mut other = Server::start().await;
    let output = timeout(
        Duration::from_secs(12),
        command()
            .arg("stop")
            .env("CLOUDLAB_BIND", format!("0.0.0.0:{}", target.bind.port()))
            .env(
                "CLOUDLAB_DATA_DIR",
                target.directory.path().join(".cloudlab"),
            )
            .output(),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    target.assert_stopped().await;
    assert!(client()
        .get(format!("http://{}/api/health", other.bind))
        .send()
        .await
        .unwrap()
        .status()
        .is_success());
    let output = timeout(Duration::from_secs(12), other.stop_command().output())
        .await
        .unwrap()
        .unwrap();
    assert!(output.status.success());
    other.assert_stopped().await;
}

#[tokio::test]
async fn stop_reports_missing_owner_key_without_creating_a_lab() {
    let directory = tempfile::tempdir().unwrap();
    let output = timeout(
        Duration::from_secs(12),
        command().current_dir(directory.path()).arg("stop").output(),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Cannot read owner key"));
    assert!(!directory.path().join(".cloudlab").exists());
}

#[tokio::test]
async fn stop_explains_how_to_upgrade_a_running_server_without_shutdown_support() {
    let directory = tempfile::tempdir().unwrap();
    let data = directory.path().join(".cloudlab");
    std::fs::create_dir(&data).unwrap();
    std::fs::write(data.join("admin-token"), "0".repeat(64)).unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let bind = listener.local_addr().unwrap();
    // An older server's static fallback accepts GET but returns 405 to POST.
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            axum::Router::new().fallback_service(axum::routing::get(|| async { "Dashboard" })),
        )
        .await
        .unwrap();
    });
    let output = timeout(
        Duration::from_secs(12),
        command()
            .current_dir(directory.path())
            .args(["stop", "--bind", &bind.to_string()])
            .output(),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(!output.status.success());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("405 Method Not Allowed"), "{error}");
    assert!(error.contains("Ctrl+C"), "{error}");
    assert!(
        error.contains("restart serve with the updated binary"),
        "{error}"
    );
    assert!(
        error.contains("Rebuilding alone does not update a running server"),
        "{error}"
    );
    server.abort();
}
