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
