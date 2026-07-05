use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
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
                                let predictions =
                                    engine.predict(&r.prefix, &r, engine.get_candidate_limit());
                                Response::Predict { predictions }
                            }
                            Request::Explain(r) => {
                                let trace = engine.explain(&r);
                                Response::Explain { trace }
                            }
                            Request::Learn(r) => {
                                let accepted = r.frequency_delta > 0;
                                // In the future, Fcitx should send `application` in LearnRequest as well
                                match engine.learn(&r.word, None, accepted) {
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
                            Request::SetLearningEnabled(enabled) => {
                                engine.set_learning_enabled(enabled);
                                Response::Success
                            }
                        };

                        let resp_envelope = ProtocolMessage {
                            version: env.version,
                            request_id: env.request_id,
                            payload: response_payload,
                        };

                        let resp_str =
                            serde_json::to_string(&resp_envelope).unwrap_or_else(|_| "{}".into());
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
                        let err_str =
                            serde_json::to_string(&err_resp).unwrap_or_else(|_| "{}".into());
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::UnixStream;
    use typeforge_protocol::{PredictRequest, ProtocolMessage, Request, Response};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_ipc_predict() {
        let (mut client, server) = UnixStream::pair().unwrap();

        let test_dir = std::env::temp_dir().join(Uuid::new_v4().to_string());
        std::fs::create_dir_all(&test_dir).unwrap();
        let dict_path = test_dir.join("dict.bin").to_string_lossy().to_string();
        let l_db_path = test_dir.join("learning.db").to_string_lossy().to_string();
        let t_db_path = test_dir.join("telemetry.db").to_string_lossy().to_string();

        let mut file = std::fs::File::create(&dict_path).unwrap();
        // Just write 48 bytes, starting with "TYPEDICT"
        let mut header_bytes = [0u8; 48];
        header_bytes[0..8].copy_from_slice(b"TYPEDICT");
        std::io::Write::write_all(&mut file, &header_bytes).unwrap();

        let config = typeforge_common::config::RankingConfig {
            candidate_limit: 5,
            ..Default::default()
        };
        let engine =
            Arc::new(TypeForgeEngine::new(dict_path, &l_db_path, &t_db_path, config).unwrap());

        tokio::spawn(async move {
            handle_client(server, engine).await;
        });

        let req_id = Uuid::new_v4();
        let msg = ProtocolMessage {
            version: 1,
            request_id: req_id,
            payload: Request::Predict(PredictRequest {
                prefix: "app".to_string(),
                text_before_cursor: "app".to_string(),
                text_after_cursor: "".to_string(),
                cursor_position: 3,
                application: None,
                language: None,
            }),
        };

        let json = serde_json::to_string(&msg).unwrap();
        client.write_all(json.as_bytes()).await.unwrap();

        let mut buf = vec![0; 4096];
        let n = client.read(&mut buf).await.unwrap();
        let resp: ProtocolMessage<Response> = serde_json::from_slice(&buf[0..n]).unwrap();

        assert_eq!(resp.request_id, req_id);
        match resp.payload {
            Response::Predict { predictions } => {
                assert_eq!(predictions.len(), 0);
            }
            _ => panic!("Expected Predict response"),
        }
    }
}
