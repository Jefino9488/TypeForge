use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProtocolMessage<T> {
    pub version: u32,
    pub request_id: Uuid,
    pub payload: T,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Request {
    Predict(PredictRequest),
    Learn(LearnRequest),
    ReloadDictionary,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PredictRequest {
    pub text_before_cursor: String,
    pub text_after_cursor: String,
    pub cursor_position: usize,
    pub application: Option<String>,
    pub language: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LearnRequest {
    pub word: String,
    pub frequency_delta: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PredictionSource {
    Dictionary,
    User,
    SpellCorrection,
    AI,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Prediction {
    pub text: String,
    pub score: f32,
    pub source: PredictionSource,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Response {
    Predict { predictions: Vec<Prediction> },
    Success,
    Error { code: String, message: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_protocol_serialization() {
        let req_id = Uuid::new_v4();
        let msg = ProtocolMessage {
            version: 1,
            request_id: req_id,
            payload: Request::Predict(PredictRequest {
                text_before_cursor: "he".to_string(),
                text_after_cursor: "".to_string(),
                cursor_position: 2,
                application: None,
                language: None,
            }),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("Predict"));
        assert!(json.contains("text_before_cursor"));
        assert!(json.contains(&req_id.to_string()));

        let deserialized: ProtocolMessage<Request> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.version, 1);
        assert_eq!(deserialized.request_id, req_id);

        match deserialized.payload {
            Request::Predict(p) => assert_eq!(p.text_before_cursor, "he"),
            _ => panic!("Wrong payload type"),
        }
    }
}
