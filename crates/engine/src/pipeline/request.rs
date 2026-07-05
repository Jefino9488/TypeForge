use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct CancellationToken {
    is_cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            is_cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.is_cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.is_cancelled.load(Ordering::SeqCst)
    }
}

pub struct PredictionRequest {
    pub text_before_cursor: String,
    pub text_after_cursor: String,
    pub cursor_position: usize,
    pub application: String,
    pub language: Option<String>, // TODO: replace with LanguageId
    pub timestamp: u64,
    pub cancellation_token: CancellationToken,
}

impl PredictionRequest {
    pub fn cancelled(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }
}
