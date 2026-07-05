mod ipc;

use std::fs;
use std::sync::{Arc, Mutex};
use typeforge_engine::pipeline::request::CancellationToken;
use tokio::net::UnixListener;
use tracing::{Level, error, info};
use tracing_subscriber::FmtSubscriber;
use typeforge_common::config::{
    AppConfig, get_learning_db_path, get_socket_path, get_telemetry_db_path,
};
use typeforge_engine::engine::TypeForgeEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = AppConfig::load();

    // Setup logging
    let log_dir = dirs::data_local_dir()
        .map(|d| d.join("typeforge").join("logs"))
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp/typeforge/logs"));
    fs::create_dir_all(&log_dir)?;

    let file_appender = tracing_appender::rolling::daily(log_dir, "daemon.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let level = match config.logging.level.to_lowercase().as_str() {
        "debug" => Level::DEBUG,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        "trace" => Level::TRACE,
        _ => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_writer(non_blocking)
        .with_ansi(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("Starting TypeForge daemon...");
    info!("Loaded configuration: {:?}", config);

    let socket_path = get_socket_path();

    if fs::metadata(&socket_path).is_ok() {
        fs::remove_file(&socket_path)?;
    }

    let l_db_path = get_learning_db_path();
    let t_db_path = get_telemetry_db_path();

    let engine = Arc::new(TypeForgeEngine::new(
        config.dictionary.path.clone(),
        &l_db_path,
        &t_db_path,
        config.ranking.clone(),
    )?);
    engine.set_learning_enabled(config.general.learning);

    let listener = UnixListener::bind(&socket_path)?;
    info!("Listening on {}", socket_path);

    let global_token: Arc<Mutex<Option<CancellationToken>>> = Arc::new(Mutex::new(None));

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let engine_clone = Arc::clone(&engine);
                let token_clone = Arc::clone(&global_token);
                tokio::spawn(async move {
                    ipc::handle_client(stream, engine_clone, token_clone).await;
                });
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}
