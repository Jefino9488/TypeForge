use super::candidate::ScoredCandidate;
use super::request::PredictionRequest;
use super::traits::PostProcessor;

pub struct LimitProcessor {
    pub max_candidates: usize,
}

impl LimitProcessor {
    pub fn new(max_candidates: usize) -> Self {
        Self { max_candidates }
    }
}

impl PostProcessor for LimitProcessor {
    fn process(&self, _request: &PredictionRequest, candidates: &mut Vec<ScoredCandidate>) {
        // Sort by confidence (descending)
        candidates.sort_by(|a, b| b.ranking.confidence.partial_cmp(&a.ranking.confidence).unwrap_or(std::cmp::Ordering::Equal));
        // Truncate to max
        candidates.truncate(self.max_candidates);
    }
}
