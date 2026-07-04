use typeforge_protocol::Prediction;

pub trait Predictor: Send + Sync {
    fn predict(&self, prefix: &str, req: &typeforge_protocol::PredictRequest, limit: usize) -> Vec<Prediction>;
}

pub trait SpellChecker: Send + Sync {
    fn correct(&self, word: &str, limit: usize) -> Vec<Prediction>;
}

pub trait Ranker: Send + Sync {
    fn rank(&self, candidates: Vec<Prediction>, context_before: &str) -> Vec<Prediction>;
}

pub trait Dictionary: Send + Sync {
    fn load(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn get_frequency(&self, word: &str) -> Option<i64>;
    fn add_word(
        &mut self,
        word: &str,
        freq: i64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
