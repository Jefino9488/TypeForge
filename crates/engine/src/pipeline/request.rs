pub struct PredictionRequest {
    pub text_before_cursor: String,
    pub text_after_cursor: String,
    pub cursor_position: usize,
    pub application: String,
    pub language: Option<String>, // TODO: replace with LanguageId
    pub timestamp: u64,
}

impl PredictionRequest {
    pub fn cancelled(&self) -> bool {
        // Future: integrate with atomic cancellation tokens
        false
    }
}
