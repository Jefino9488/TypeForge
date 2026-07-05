use super::super::dictionary::immutable::ImmutableDictionary;
use super::super::learning::Learner;
use super::candidate::{CandidateMetadata, CandidateSource, RawCandidate};
use super::request::PredictionRequest;
use super::traits::CandidateGenerator;
use std::sync::Arc;
use arc_swap::ArcSwap;
use crate::traits::Predictor;

pub struct PrefixGenerator {
    dictionary: Arc<ArcSwap<ImmutableDictionary>>,
    limit: usize,
}

impl PrefixGenerator {
    pub fn new(dictionary: Arc<ArcSwap<ImmutableDictionary>>, limit: usize) -> Self {
        Self { dictionary, limit }
    }
}

impl CandidateGenerator for PrefixGenerator {
    fn generate(&self, request: &PredictionRequest) -> Vec<RawCandidate> {
        // Find the last word in text_before_cursor
        let prefix = request
            .text_before_cursor
            .split_whitespace()
            .last()
            .unwrap_or("")
            .to_lowercase();
        if prefix.is_empty() {
            return vec![];
        }

        let dict = self.dictionary.load();
        
        let dummy_req = typeforge_protocol::PredictRequest {
            prefix: prefix.clone(),
            text_before_cursor: request.text_before_cursor.clone(),
            text_after_cursor: request.text_after_cursor.clone(),
            cursor_position: request.cursor_position,
            application: Some(request.application.clone()),
            language: request.language.clone(),
        };

        let words = dict.predict(&prefix, &dummy_req, self.limit);
        words
            .into_iter()
            .map(|p| RawCandidate {
                text: p.text,
                metadata: CandidateMetadata {
                    source: CandidateSource::Dictionary,
                    matched_prefix: prefix.clone(),
                    edit_distance: 0,
                    context_match: false,
                },
            })
            .collect()
    }
}

pub struct SessionGenerator {
    learner: Arc<Learner>,
    limit: usize,
}

impl SessionGenerator {
    pub fn new(learner: Arc<Learner>, limit: usize) -> Self {
        Self { learner, limit }
    }
}

impl CandidateGenerator for SessionGenerator {
    fn generate(&self, request: &PredictionRequest) -> Vec<RawCandidate> {
        let prefix = request
            .text_before_cursor
            .split_whitespace()
            .last()
            .unwrap_or("")
            .to_lowercase();
        if prefix.is_empty() {
            return vec![];
        }
        
        let learned_words = self.learner.get_candidates_by_prefix(&prefix, self.limit).unwrap_or_default();
        learned_words.into_iter().map(|word| RawCandidate {
            text: word,
            metadata: CandidateMetadata {
                source: CandidateSource::SessionMemory,
                matched_prefix: prefix.clone(),
                edit_distance: 0,
                context_match: false, // Could check request.application here
            }
        }).collect()
    }
}
