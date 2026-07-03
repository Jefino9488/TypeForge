mod ipc;

use tokio::net::UnixListener;
use std::sync::Arc;
use tracing::{info, error};
use typeforge_engine::engine::TypeForgeEngine;
use typeforge_common::config::{get_socket_path, get_db_path};
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::init();
    
    info!("Starting TypeForge daemon...");

    let socket_path = get_socket_path();
    
    if fs::metadata(&socket_path).is_ok() {
        fs::remove_file(&socket_path)?;
    }

    let immutable_path = "assets/dictionary-v1.csv.gz".to_string();
    let db_path = get_db_path();
    
    let engine = Arc::new(TypeForgeEngine::new(immutable_path, &db_path)?);
    
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
