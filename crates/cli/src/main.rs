use clap::{Parser, Subcommand};
use colored::*;
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use typeforge_common::config::{AppConfig, get_socket_path};
use typeforge_protocol::{LearnRequest, PredictRequest, ProtocolMessage, Request, Response};
use uuid::Uuid;

mod theme;

#[derive(Parser)]
#[command(name = "typeforge")]
#[command(version = "0.3.0", about = "CLI for TypeForge daemon", long_about = None)]
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
    Explain {
        prefix: String,
        #[arg(long)]
        app: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Benchmark {
        prefix: String,
    },
    Replay {
        #[arg(short, long)]
        file: String,
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
    Theme {
        #[command(subcommand)]
        action: ThemeCommand,
    },
    Layout {
        #[arg(value_name = "LAYOUT")]
        mode: String,
    },
}

#[derive(Subcommand)]
pub enum ThemeCommand {
    List,
    Apply { theme_name: String },
    Current,
    Restore,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    let socket_path = get_socket_path();

    if let Some(cmd) = cli.command {
        match cmd {
            Commands::Doctor => {
                theme::doctor_info();
                run_doctor(&socket_path).await
            }
            Commands::Info => run_info(&socket_path).await,
            Commands::Theme { action } => match action {
                ThemeCommand::List => theme::list_themes(),
                ThemeCommand::Apply { theme_name } => theme::apply_theme(&theme_name),
                ThemeCommand::Current => theme::current_theme(),
                ThemeCommand::Restore => theme::restore_theme(),
            },
            Commands::Layout { mode } => {
                theme::set_layout(&mode);
            }
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
            Commands::Replay { file } => {
                send_replay_request(&socket_path, file).await?;
            }
            Commands::Explain { prefix, app, json } => {
                send_explain_request(&socket_path, prefix, app, json).await?
            }
            Commands::Benchmark { prefix } => send_benchmark_request(&socket_path, prefix).await?,
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

async fn send_explain_request(
    socket_path: &str,
    prefix: String,
    app: Option<String>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = UnixStream::connect(socket_path).await?;
    let req = Request::Explain(PredictRequest {
        prefix: prefix.clone(),
        text_before_cursor: prefix.clone(),
        text_after_cursor: String::new(),
        cursor_position: 0,
        application: app,
        language: None,
    });

    let env = ProtocolMessage {
        version: 1,
        request_id: Uuid::new_v4(),
        payload: req,
    };

    let req_str = serde_json::to_string(&env)?;
    stream.write_all(req_str.as_bytes()).await?;

    let mut buf = vec![0; 65536]; // Trace can be large
    let n = stream.read(&mut buf).await?;
    let resp: ProtocolMessage<Response> = serde_json::from_slice(&buf[0..n])?;

    if json {
        println!("{}", serde_json::to_string_pretty(&resp)?);
        return Ok(());
    }

    if let Response::Explain { trace } = resp.payload {
        println!(
            "{}",
            format!("Pipeline Version: {}", trace.pipeline_version).bright_black()
        );

        for candidate in trace.candidates {
            println!("\nCandidate\t\t{}", candidate.text.bold().green());

            println!("\nGenerators");
            for g in candidate.generators {
                println!("{} {}", "✓".green(), g);
            }

            println!("\nExpanders");
            for exp in candidate.expanders {
                println!("{} {}", "✓".green(), exp);
            }

            println!("\nFeatures");
            println!(
                "{:20} {:.2}",
                "Base Frequency", candidate.features.base_frequency
            );
            println!(
                "{:20} {:.2}",
                "User Frequency", candidate.features.user_frequency
            );
            println!("{:20} {:.2}", "Context", candidate.features.context_match);
            println!("{:20} {:.2}", "Session", candidate.features.session_match);
            println!(
                "{:20} {:.2}",
                "Edit Distance", candidate.features.edit_distance
            );

            println!("\nScore\t\t\t{:.2}", candidate.score);
            println!("Confidence\t\t{:.2}", candidate.confidence);
            println!("Rank\t\t\t{}", candidate.rank);
            println!("{}", "─".repeat(40).bright_black());
        }
    } else {
        println!("{:#?}", resp);
    }

    Ok(())
}

async fn send_benchmark_request(
    socket_path: &str,
    prefix: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = UnixStream::connect(socket_path).await?;
    let req = Request::Explain(PredictRequest {
        prefix: prefix.clone(),
        text_before_cursor: prefix.clone(),
        text_after_cursor: String::new(),
        cursor_position: 0,
        application: None,
        language: None,
    });

    let env = ProtocolMessage {
        version: 1,
        request_id: Uuid::new_v4(),
        payload: req,
    };

    let req_str = serde_json::to_string(&env)?;
    stream.write_all(req_str.as_bytes()).await?;

    let mut buf = vec![0; 65536];
    let n = stream.read(&mut buf).await?;
    let resp: ProtocolMessage<Response> = serde_json::from_slice(&buf[0..n])?;

    if let Response::Explain { trace } = resp.payload {
        let t = trace.timings;
        println!("{:20} {} μs", "Generator", t.generators_us);
        println!("{:20} {} μs", "Expansion", t.expanders_us);
        println!("{:20} {} μs", "Feature Extraction", t.features_us);
        println!("{:20} {} μs", "Ranking", t.ranking_us);
        println!("{:20} {} μs", "Post Processing", t.post_processing_us);
        println!();
        println!("{:20} {} μs", "Total", t.total_us);
    } else {
        println!("{:#?}", resp);
    }

    Ok(())
}

async fn send_replay_request(
    socket_path: &str,
    file: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let content = std::fs::read_to_string(file)?;
    let mut stream = UnixStream::connect(socket_path).await?;

    let words: Vec<&str> = content.split_whitespace().collect();
    let total = words.len();
    println!("Replaying {} words...", total);

    let start = std::time::Instant::now();
    let mut count = 0;

    for word in words {
        // Simulate typing the first 3 characters
        let prefix = if word.len() > 3 { &word[..3] } else { word };

        let req = Request::Predict(PredictRequest {
            prefix: prefix.to_string(),
            text_before_cursor: prefix.to_string(),
            text_after_cursor: String::new(),
            cursor_position: 0,
            application: None,
            language: None,
        });

        let env = ProtocolMessage {
            version: 1,
            request_id: Uuid::new_v4(),
            payload: req,
        };

        let req_str = serde_json::to_string(&env)?;
        stream.write_all(req_str.as_bytes()).await?;

        let mut buf = vec![0; 65536];
        let _n = stream.read(&mut buf).await?;
        count += 1;

        if count % 100 == 0 {
            print!(".");
            use std::io::Write;
            let _ = std::io::stdout().flush();
        }
    }

    let duration = start.elapsed();
    println!("\nReplay finished.");
    println!("Total words: {}", count);
    println!("Total time: {:?}", duration);
    if count > 0 {
        println!("Average latency: {:?}", duration / count as u32);
    }

    Ok(())
}
