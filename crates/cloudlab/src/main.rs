use clap::{Parser, Subcommand};
use std::{net::SocketAddr, path::PathBuf};
#[derive(Parser)]
#[command(
    name = "cloudlab",
    version,
    about = "A self-hosted, container-isolated compute lab"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    Serve {
        #[arg(long, default_value = "127.0.0.1:8088", env = "CLOUDLAB_BIND")]
        bind: SocketAddr,
        #[arg(long, default_value = "127.0.0.1:8089", env = "CLOUDLAB_APP_BIND")]
        app_bind: SocketAddr,
        #[arg(
            long,
            default_value = "http://{workspace}.localhost:8089",
            env = "CLOUDLAB_APP_URL"
        )]
        app_url: String,
        /// Managed HTTPS dashboard origin (also configures node pairing).
        #[arg(long, env = "CLOUDLAB_PUBLIC_URL")]
        public_url: Option<String>,
        #[arg(long, default_value = ".cloudlab", env = "CLOUDLAB_DATA_DIR")]
        data_dir: PathBuf,
        #[arg(long, default_value = "dist", env = "CLOUDLAB_WEB_DIR")]
        web_dir: PathBuf,
    },
    /// Stop a running standalone coordinator and its workspace gateway.
    Stop {
        /// Coordinator address used by `serve`.
        #[arg(long, default_value = "127.0.0.1:8088", env = "CLOUDLAB_BIND")]
        bind: SocketAddr,
        /// Data directory containing the running coordinator's owner key.
        #[arg(long, default_value = ".cloudlab", env = "CLOUDLAB_DATA_DIR")]
        data_dir: PathBuf,
    },
    Agent {
        #[arg(long, env = "CLOUDLAB_COORDINATOR")]
        coordinator: String,
        #[arg(long, env = "CLOUDLAB_ENROLLMENT")]
        enrollment: Option<String>,
        #[arg(long, default_value = ".cloudlab/agent", env = "CLOUDLAB_AGENT_DIR")]
        data_dir: PathBuf,
        #[arg(long, default_value_t = false)]
        allow_network: bool,
    },
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Commands::Serve {
            bind,
            app_bind,
            app_url,
            public_url,
            data_dir,
            web_dir,
        } => cloudlab::server::serve(bind, app_bind, app_url, public_url, data_dir, web_dir).await,
        Commands::Stop { bind, data_dir } => cloudlab::server::stop(bind, data_dir).await,
        Commands::Agent {
            coordinator,
            enrollment,
            data_dir,
            allow_network,
        } => cloudlab::agent::run(coordinator, enrollment, data_dir, allow_network).await,
    }
}
