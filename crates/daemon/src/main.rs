mod ipc;

use std::fs;
use std::sync::Arc;
use tokio::net::UnixListener;
use tracing::{error, info};
use typeforge_common::config::{get_learning_db_path, get_telemetry_db_path, get_socket_path};
use typeforge_engine::engine::TypeForgeEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::init();
    info!("Starting TypeForge daemon...");

    let socket_path = get_socket_path();

    if fs::metadata(&socket_path).is_ok() {
        fs::remove_file(&socket_path)?;
    }

    let immutable_path = "assets/dictionary-v1.csv.gz".to_string();
    let l_db_path = get_learning_db_path();
    let t_db_path = get_telemetry_db_path();

    let engine = Arc::new(TypeForgeEngine::new(immutable_path, &l_db_path, &t_db_path)?);

    let listener = UnixListener::bind(&socket_path)?;
    info!("Listening on {}", socket_path);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let engine_clone = Arc::clone(&engine);
                tokio::spawn(async move {
                    ipc::handle_client(stream, engine_clone).await;
                });
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}
