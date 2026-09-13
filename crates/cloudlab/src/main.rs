use clap::{Args, CommandFactory, Parser, Subcommand};
use std::{io::IsTerminal, net::SocketAddr, path::PathBuf};

const LOGO: &str = include_str!("../assets/cli-logo.txt");
const EXAMPLES: &str = "GET STARTED
  cloudlab serve                         Start your dashboard
  cloudlab agent --coordinator URL --enrollment KEY
                                         Pair a compute node

EVERYDAY COMMANDS
  cloudlab agent start                   Resume a saved pairing
  cloudlab agent status                  Show local agent and pairing status
  cloudlab agent stop                    Stop the agent; keep its pairing
  cloudlab agent delete                  Stop and remove its pairing
  cloudlab stop                          Stop the dashboard

Use --help after any command for its options.
Agent commands use .cloudlab/agent in the current folder unless --data-dir is set.
Guide: https://cloudlab-alpha.vercel.app/docs/cli/";

fn banner() {
    if std::io::stderr().is_terminal() {
        eprintln!(
            "{LOGO}\n\n  Your computers. One lab.  v{}\n",
            env!("CARGO_PKG_VERSION")
        );
    }
}
#[derive(Parser)]
#[command(
    name = "cloudlab",
    version,
    before_help = LOGO,
    after_help = EXAMPLES,
    about = "A self-hosted, container-isolated compute lab"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    /// Start the dashboard and workspace gateway.
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
    /// Run, stop, or delete this computer's agent enrollment.
    #[command(before_help = LOGO, after_help = "Pair:    cloudlab agent --coordinator URL --enrollment KEY\nResume:  cloudlab agent start\nReset:   cloudlab agent delete\n\nUse the same --data-dir for start, status, stop, and delete.\nDeleting enrollment preserves Docker containers and volumes.")]
    Agent {
        #[command(subcommand)]
        command: Option<AgentCommand>,
        #[command(flatten)]
        options: AgentStartOptions,
        /// Folder containing this agent's pairing; use the same folder for every command.
        #[arg(
            long,
            default_value = ".cloudlab/agent",
            env = "CLOUDLAB_AGENT_DIR",
            global = true
        )]
        data_dir: PathBuf,
    },
}

#[derive(Args, Default)]
struct AgentStartOptions {
    /// Coordinator origin; uses the saved address when resuming.
    #[arg(long, env = "CLOUDLAB_COORDINATOR")]
    coordinator: Option<String>,
    /// One-time pairing key, used only for the first connection.
    #[arg(long, env = "CLOUDLAB_ENROLLMENT", hide_env_values = true)]
    enrollment: Option<String>,
    /// Allow network-enabled workspaces when the coordinator also permits them.
    #[arg(long)]
    allow_network: bool,
}
#[derive(Subcommand)]
enum AgentCommand {
    /// Run the agent using a saved pairing, or pair with --enrollment.
    #[command(
        after_help = "Resume: cloudlab agent start\nPair:   cloudlab agent start --coordinator URL --enrollment KEY\n\nUse the original --data-dir to resume an existing pairing."
    )]
    Start(AgentStartOptions),
    /// Show local process and enrollment status without revealing credentials.
    Status {
        /// Print machine-readable local status without credentials.
        #[arg(long)]
        json: bool,
    },
    /// Stop the local agent, keeping its enrollment and Docker workspaces.
    Stop,
    /// Stop the agent, revoke its node access, and remove its saved enrollment.
    Delete {
        /// Clear local enrollment without contacting the coordinator.
        #[arg(long)]
        local_only: bool,
    },
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::args_os().len() == 1 {
        Cli::command().print_help()?;
        println!();
        return Ok(());
    }
    match Cli::parse().command {
        Commands::Serve {
            bind,
            app_bind,
            app_url,
            public_url,
            data_dir,
            web_dir,
        } => {
            banner();
            cloudlab::server::serve(bind, app_bind, app_url, public_url, data_dir, web_dir).await
        }
        Commands::Stop { bind, data_dir } => cloudlab::server::stop(bind, data_dir).await,
        Commands::Agent {
            command,
            options,
            data_dir,
        } => match command {
            Some(AgentCommand::Status { json }) => cloudlab::agent::status(data_dir, json),
            Some(AgentCommand::Stop) => cloudlab::agent::stop(data_dir).await,
            Some(AgentCommand::Delete { local_only }) => {
                cloudlab::agent::delete(data_dir, local_only).await
            }
            command => {
                let options = match command {
                    Some(AgentCommand::Start(start)) => AgentStartOptions {
                        coordinator: start.coordinator.or(options.coordinator),
                        enrollment: start.enrollment.or(options.enrollment),
                        allow_network: start.allow_network || options.allow_network,
                    },
                    None => options,
                    _ => unreachable!(),
                };
                let coordinator = cloudlab::agent::coordinator(options.coordinator, &data_dir)?;
                banner();
                cloudlab::agent::run(
                    coordinator,
                    options.enrollment,
                    data_dir,
                    options.allow_network,
                )
                .await
            }
        },
    }
}
