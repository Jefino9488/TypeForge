use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;
use typeforge_protocol::{PredictRequest, ProtocolMessage, Request, Response};
use uuid::Uuid;

// Re-export protocol structs so clients don't need to depend on protocol directly
pub use typeforge_protocol::{Prediction, PredictionSource};

pub struct TypeForgeClient {
    socket_path: String,
}

impl TypeForgeClient {
    pub fn new() -> Self {
        Self {
            // For now, hardcode MVP path. Later, get from config.
            socket_path: "/tmp/typeforge.sock".to_string(),
        }
    }

    pub fn predict(
        &self,
        prefix: &str,
        limit: usize,
        app: Option<String>,
    ) -> Result<Vec<Prediction>, String> {
        let mut stream = UnixStream::connect(&self.socket_path)
            .map_err(|e| format!("Failed to connect to daemon: {}", e))?;

        stream
            .set_read_timeout(Some(Duration::from_millis(50)))
            .map_err(|e| e.to_string())?;
        stream
            .set_write_timeout(Some(Duration::from_millis(50)))
            .map_err(|e| e.to_string())?;

        let req_id = Uuid::new_v4();
        let msg = ProtocolMessage {
            version: 1,
            request_id: req_id,
            payload: Request::Predict(PredictRequest {
                text_before_cursor: prefix.to_string(),
                text_after_cursor: String::new(),
                cursor_position: unicode_segmentation::UnicodeSegmentation::graphemes(prefix, true).count(),
                application: app,
                language: None,
            }),
        };

        let json_req = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
        stream
            .write_all(json_req.as_bytes())
            .map_err(|e| e.to_string())?;

        let mut buf = vec![0; 8192];
        let n = stream.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Empty response".to_string());
        }

        let resp_str = std::str::from_utf8(&buf[0..n]).map_err(|e| e.to_string())?;
        let resp: ProtocolMessage<Response> =
            serde_json::from_str(resp_str).map_err(|e| e.to_string())?;

        match resp.payload {
            Response::Predict { predictions } => Ok(predictions.into_iter().take(limit).collect()),
            Response::Error { message, .. } => Err(message),
            _ => Err("Unexpected response type".to_string()),
        }
    }

    pub fn learn(
        &self,
        word: &str,
        delta: i64,
        _app: Option<String>,
    ) -> Result<(), String> {
        let mut stream = UnixStream::connect(&self.socket_path)
            .map_err(|e| format!("Failed to connect to daemon: {}", e))?;

        stream
            .set_read_timeout(Some(Duration::from_millis(50)))
            .map_err(|e| e.to_string())?;
        stream
            .set_write_timeout(Some(Duration::from_millis(50)))
            .map_err(|e| e.to_string())?;

        let req_id = Uuid::new_v4();
        let msg = ProtocolMessage {
            version: 1,
            request_id: req_id,
            payload: Request::Learn(typeforge_protocol::LearnRequest {
                word: word.to_string(),
                frequency_delta: delta,
            }),
        };

        let json_req = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
        stream
            .write_all(json_req.as_bytes())
            .map_err(|e| e.to_string())?;

        let mut buf = vec![0; 4096];
        let n = stream.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Empty response".to_string());
        }

        let resp_str = std::str::from_utf8(&buf[0..n]).map_err(|e| e.to_string())?;
        let resp: ProtocolMessage<Response> =
            serde_json::from_str(resp_str).map_err(|e| e.to_string())?;

        match resp.payload {
            Response::Success => Ok(()),
            Response::Error { message, .. } => Err(message),
            _ => Err("Unexpected response type".to_string()),
        }
    }

    pub fn set_learning_enabled(&self, enabled: bool) -> Result<(), String> {
        let mut stream = UnixStream::connect(&self.socket_path)
            .map_err(|e| format!("Failed to connect to daemon: {}", e))?;

        stream
            .set_read_timeout(Some(Duration::from_millis(50)))
            .map_err(|e| e.to_string())?;
        stream
            .set_write_timeout(Some(Duration::from_millis(50)))
            .map_err(|e| e.to_string())?;

        let req_id = Uuid::new_v4();
        let msg = ProtocolMessage {
            version: 1,
            request_id: req_id,
            payload: Request::SetLearningEnabled(enabled),
        };

        let json_req = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
        stream
            .write_all(json_req.as_bytes())
            .map_err(|e| e.to_string())?;

        let mut buf = vec![0; 4096];
        let n = stream.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Empty response".to_string());
        }

        let resp_str = std::str::from_utf8(&buf[0..n]).map_err(|e| e.to_string())?;
        let resp: ProtocolMessage<Response> =
            serde_json::from_str(resp_str).map_err(|e| e.to_string())?;

        match resp.payload {
            Response::Success => Ok(()),
            Response::Error { message, .. } => Err(message),
            _ => Err("Unexpected response type".to_string()),
        }
    }
}

impl Default for TypeForgeClient {
    fn default() -> Self {
        Self::new()
    }
}
