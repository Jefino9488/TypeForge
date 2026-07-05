use super::candidate::{FeatureSet, RankingResult, RawCandidate};
use super::request::PredictionRequest;
use super::traits::RankingStrategy;
use typeforge_common::config::RankingConfig;
use std::sync::Arc;

pub struct WeightedRanker {
    config: Arc<RankingConfig>,
}

impl WeightedRanker {
    pub fn new(config: Arc<RankingConfig>) -> Self {
        Self { config }
    }
}

impl RankingStrategy for WeightedRanker {
    fn score(&self, _request: &PredictionRequest, _candidate: &RawCandidate, features: &FeatureSet) -> RankingResult {
        let mut score = 0.0;
        
        score += features.base_frequency * self.config.base_frequency;
        score += features.user_frequency * self.config.user_frequency;
        
        if self.config.enable_context {
            score += features.context_score * self.config.context;
        }
        
        if self.config.enable_session {
            score += features.session_frequency * self.config.session;
            if features.recently_accepted {
                score += self.config.session * 0.5; // Additional boost for very recent words
            }
        }
        
        score += features.ngram_probability * self.config.ngram;

        let prefix_bonus = if features.exact_prefix { 0.2 } else { 0.0 };
        let length_bonus = if features.word_length > 0 {
            (features.prefix_length as f32 / features.word_length as f32) * 0.1
        } else {
            0.0
        };
        
        let penalty = if features.edit_distance > 0 {
            features.edit_distance as f32 * 0.15
        } else {
            0.0
        };

        score = score + prefix_bonus + length_bonus - penalty;
        
        let confidence = score.max(0.0);
        
        RankingResult {
            score: confidence,
            confidence,
        }
    }
}
