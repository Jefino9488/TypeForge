use super::candidate::{FeatureSet, RawCandidate};
use super::request::PredictionRequest;
use super::traits::FeatureExtractor;

pub struct BasicFeatureExtractor;

impl FeatureExtractor for BasicFeatureExtractor {
    fn extract(&self, _request: &PredictionRequest, candidate: &RawCandidate, features: &mut FeatureSet) {
        features.word_length = candidate.text.len() as u8;
        features.prefix_length = candidate.metadata.matched_prefix.len() as u8;
        features.exact_prefix = candidate.text.to_lowercase().starts_with(&candidate.metadata.matched_prefix.to_lowercase());
        
        // Dummy frequency for MVP
        features.base_frequency = match candidate.metadata.source {
            super::candidate::CandidateSource::Dictionary => 0.5,
            _ => 0.1,
        };
    }
}
