use clap::{Parser, Subcommand};
use colored::*;
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use typeforge_common::config::{AppConfig, get_socket_path};
use typeforge_protocol::{LearnRequest, PredictRequest, ProtocolMessage, Request, Response};
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "typeforge")]
#[command(version = "0.3.0-alpha1", about = "CLI for TypeForge daemon", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Predict {
        #[arg(short, long)]
        prefix: String,
    },
    Learn {
        #[arg(short, long)]
        word: String,

        #[arg(short, long, default_value_t = 1)]
        freq: i64,
    },
    ToggleLearning {
        #[arg(short, long)]
        enabled: bool,
    },
    Doctor,
    Info,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    let socket_path = get_socket_path();

    if let Some(cmd) = cli.command {
        match cmd {
            Commands::Doctor => run_doctor(&socket_path).await,
            Commands::Info => run_info(&socket_path).await,
            Commands::Predict {
                prefix: text_before,
            } => {
                send_request(
                    &socket_path,
                    Request::Predict(PredictRequest {
                        prefix: text_before.clone(),
                        text_before_cursor: text_before,
                        text_after_cursor: String::new(),
                        cursor_position: 0,
                        application: None,
                        language: None,
                    }),
                )
                .await?
            }
            Commands::Learn { word, freq } => {
                send_request(
                    &socket_path,
                    Request::Learn(LearnRequest {
                        word,
                        frequency_delta: freq,
                    }),
                )
                .await?
            }
            Commands::ToggleLearning { enabled } => {
                send_request(&socket_path, Request::SetLearningEnabled(enabled)).await?
            }
        }
    } else {
        println!("{}", "TypeForge CLI".bold().cyan());
        println!("Run `typeforge --help` for available commands.");
    }

    Ok(())
}

async fn send_request(
    socket_path: &str,
    payload: Request,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = UnixStream::connect(socket_path).await?;
    let request_id = Uuid::new_v4();

    let envelope = ProtocolMessage {
        version: 1,
        request_id,
        payload,
    };

    let req_str = serde_json::to_string(&envelope)?;
    stream.write_all(req_str.as_bytes()).await?;

    let mut buf = vec![0; 4096];
    let n = stream.read(&mut buf).await?;
    let resp: ProtocolMessage<Response> = serde_json::from_slice(&buf[0..n])?;

    println!("{:#?}", resp);
    Ok(())
}

async fn run_doctor(socket_path: &str) {
    println!("{}", "TypeForge Doctor".bold().blue());
    println!("Checking system health...\n");

    // Check Daemon / Socket
    if Path::new(socket_path).exists() {
        if UnixStream::connect(socket_path).await.is_ok() {
            println!("{} Daemon running", "✓".green());
            println!("{} Socket found ({})", "✓".green(), socket_path);
        } else {
            println!(
                "{} Daemon not responding (Socket exists but connection refused)",
                "✗".red()
            );
        }
    } else {
        println!("{} Daemon running (Socket not found)", "✗".red());
    }

    // Check Fcitx5 Plugin
    let local_plugin = dirs::data_local_dir().map(|d| d.join("fcitx5/addon/typeforge.conf"));
    let sys_plugin = Path::new("/usr/share/fcitx5/addon/typeforge.conf");
    let mut fcitx_installed = false;

    if sys_plugin.exists() {
        fcitx_installed = true;
    } else if let Some(local_path) = local_plugin
        && local_path.exists()
    {
        fcitx_installed = true;
    }

    if fcitx_installed {
        println!("{} Fcitx plugin installed", "✓".green());
    } else {
        println!("{} Fcitx plugin installed", "✗".red());
    }

    // Check Config
    let config = AppConfig::load();
    println!("{} Config valid", "✓".green());

    // Check Dictionary
    if Path::new(&config.dictionary.path).exists() {
        println!(
            "{} Dictionary loaded ({})",
            "✓".green(),
            config.dictionary.path
        );
    } else {
        println!(
            "{} Dictionary loaded (Missing: {})",
            "✗".red(),
            config.dictionary.path
        );
    }

    if config.general.learning {
        println!("{} Learning enabled", "✓".green());
    } else {
        println!("{} Learning disabled", "-".yellow());
    }
}

async fn run_info(socket_path: &str) {
    let config = AppConfig::load();
    let daemon_status =
        if Path::new(socket_path).exists() && UnixStream::connect(socket_path).await.is_ok() {
            "Running".green()
        } else {
            "Stopped".red()
        };

    let fcitx_status = if Path::new("/usr/share/fcitx5/addon/typeforge.conf").exists()
        || dirs::data_local_dir()
            .map(|d| d.join("fcitx5/addon/typeforge.conf"))
            .unwrap_or_default()
            .exists()
    {
        "Installed".green()
    } else {
        "Not Installed".red()
    };

    println!("{}", "TypeForge v0.3.0-alpha1".bold().cyan());
    println!();

    println!("{}", "Dictionary:".bold());
    println!("{}", config.dictionary.language);
    println!();

    println!("{}", "Learning:".bold());
    println!(
        "{}",
        if config.general.learning {
            "Enabled"
        } else {
            "Disabled"
        }
    );
    println!();

    println!("{}", "Daemon:".bold());
    println!("{}", daemon_status);
    println!();

    println!("{}", "Socket:".bold());
    println!("{}", socket_path);
    println!();

    println!("{}", "Fcitx:".bold());
    println!("{}", fcitx_status);
}
