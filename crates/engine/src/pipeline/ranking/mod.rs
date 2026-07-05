use super::candidate::{FeatureSet, RankingResult, RawCandidate};
use super::request::PredictionRequest;
use super::traits::RankingStrategy;

pub struct LinearRanker;

impl LinearRanker {
    pub fn new() -> Self {
        Self
    }
}

impl RankingStrategy for LinearRanker {
    fn score(&self, _request: &PredictionRequest, _candidate: &RawCandidate, features: &FeatureSet) -> RankingResult {
        // Normalize and weigh features
        let base_score = features.base_frequency * 0.4;
        let prefix_bonus = if features.exact_prefix { 0.3 } else { 0.0 };
        let length_bonus = (features.prefix_length as f32 / std::cmp::max(1, features.word_length) as f32) * 0.3;

        let score = base_score + prefix_bonus + length_bonus;
        
        RankingResult {
            score,
            confidence: score, // Same as score for linear ranker
        }
    }
}
