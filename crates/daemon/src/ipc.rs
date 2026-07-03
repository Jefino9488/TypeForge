use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tracing::{error, warn};
use typeforge_engine::engine::TypeForgeEngine;
use typeforge_protocol::{ProtocolMessage, Request, Response};

pub async fn handle_client(mut stream: UnixStream, engine: Arc<TypeForgeEngine>) {
    let mut buf = vec![0; 4096];
    loop {
        match stream.read(&mut buf).await {
            Ok(0) => break, // Connection closed
            Ok(n) => {
                let msg = &buf[0..n];
                
                // Parse envelope
                let envelope: Result<ProtocolMessage<Request>, _> = serde_json::from_slice(msg);
                
                match envelope {
                    Ok(env) => {
                        // In a real high-throughput scenario, we could spawn a task per request
                        // and track it in a HashMap via request_id to support Cancellation.
                        // Since predictions are <5ms, we process sequentially per client connection.
                        
                        let response_payload = match env.payload {
                            Request::Predict(r) => {
                                let predictions = engine.predict(&r.text_before_cursor, 5);
                                Response::Predict { predictions }
                            }
                            Request::Learn(r) => {
                                match engine.learn(&r.word, r.frequency_delta) {
                                    Ok(_) => Response::Success,
                                    Err(e) => {
                                        error!("Failed to learn word: {}", e);
                                        Response::Error {
                                            code: "LEARN_ERROR".into(),
                                            message: e.to_string(),
                                        }
                                    }
                                }
                            }
                            Request::ReloadDictionary => {
                                engine.reload_dictionary_background();
                                Response::Success
                            }
                        };
                        
                        let resp_envelope = ProtocolMessage {
                            version: env.version,
                            request_id: env.request_id,
                            payload: response_payload,
                        };

                        let resp_str = serde_json::to_string(&resp_envelope).unwrap_or_else(|_| "{}".into());
                        if let Err(e) = stream.write_all(resp_str.as_bytes()).await {
                            error!("Failed to write to socket: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse request: {}", e);
                        // If we can't parse the envelope, we don't know the request_id
                        let err_resp = Response::Error {
                            code: "PARSE_ERROR".into(),
                            message: e.to_string(),
                        };
                        let err_str = serde_json::to_string(&err_resp).unwrap_or_else(|_| "{}".into());
                        let _ = stream.write_all(err_str.as_bytes()).await;
                    }
                }
            }
            Err(e) => {
                error!("Failed to read from socket: {}", e);
                break;
            }
        }
    }
}
