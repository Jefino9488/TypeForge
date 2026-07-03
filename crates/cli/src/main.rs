use clap::{Parser, Subcommand};
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use typeforge_protocol::{ProtocolMessage, Request, Response, PredictRequest, LearnRequest};
use typeforge_common::config::get_socket_path;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "typeforge-cli")]
#[command(about = "CLI for TypeForge daemon", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();
    
    let socket_path = get_socket_path();
    let mut stream = UnixStream::connect(&socket_path).await?;
    
    let request_id = Uuid::new_v4();
    
    let payload = match cli.command {
        Commands::Predict { prefix } => {
            Request::Predict(PredictRequest {
                text_before_cursor: prefix,
                text_after_cursor: "".to_string(),
                cursor_position: 0,
                application: None,
                language: None,
            })
        }
        Commands::Learn { word, freq } => {
            Request::Learn(LearnRequest {
                word,
                frequency_delta: freq,
            })
        }
    };
    
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
