use super::candidate::{FeatureSet, RawCandidate};
use super::request::PredictionRequest;
use super::traits::FeatureExtractor;
use crate::dictionary::immutable::ImmutableDictionary;
use crate::learning::Learner;
use crate::traits::Dictionary;
use arc_swap::ArcSwap;
use std::sync::Arc;

pub struct BasicFeatureExtractor {
    learner: Arc<Learner>,
    dictionary: Arc<ArcSwap<ImmutableDictionary>>,
}

impl BasicFeatureExtractor {
    pub fn new(learner: Arc<Learner>, dictionary: Arc<ArcSwap<ImmutableDictionary>>) -> Self {
        Self {
            learner,
            dictionary,
        }
    }
}

impl FeatureExtractor for BasicFeatureExtractor {
    fn extract(
        &self,
        request: &PredictionRequest,
        candidate: &RawCandidate,
        features: &mut FeatureSet,
    ) {
        features.word_length = candidate.text.len() as u8;
        features.prefix_length = candidate.metadata.matched_prefix.len() as u8;

        let text_lower = candidate.text.to_lowercase();
        let prefix_lower = candidate.metadata.matched_prefix.to_lowercase();
        features.exact_prefix = text_lower.starts_with(&prefix_lower);

        if features.exact_prefix && features.word_length > 0 {
            features.prefix_confidence =
                (features.prefix_length as f32) / (features.word_length as f32);
        } else {
            features.prefix_confidence = 0.0;
        }

        features.edit_distance = candidate.metadata.edit_distance;

        // Base frequency from static dictionary
        let dict = self.dictionary.load();
        let base_freq = dict.get_frequency(&candidate.text).unwrap_or(0);
        // Normalize base frequency (assuming max freq is ~1,000,000 for top word)
        features.base_frequency = (base_freq as f32 / 1_000_000.0).min(1.0);

        // User frequency from learning db
        let user_freq = self
            .learner
            .learning_db
            .get_weight(&candidate.text, None)
            .unwrap_or(0);
        features.user_frequency = (user_freq as f32 / 1000.0).min(1.0); // Assuming 1000 uses is "max" weight

        // Context frequency
        if !request.application.is_empty() {
            let ctx_freq = self
                .learner
                .learning_db
                .get_weight(&candidate.text, Some(&request.application))
                .unwrap_or(0);
            features.context_score = (ctx_freq as f32 / 500.0).min(1.0);
        }

        // Session frequency
        features.session_frequency = if self.learner.session_memory.contains(&candidate.text) {
            1.0
        } else {
            0.0
        };

        // Custom word?
        features.is_custom_word = base_freq == 0 && user_freq > 0;

        // Recently accepted?
        let history = self.learner.session_memory.get_recent(5);
        features.recently_accepted = history.iter().any(|w| w == &candidate.text);
    }
}
