use super::super::candidate::ScoredCandidate;
use super::super::request::PredictionRequest;
use super::super::traits::PostProcessor;

pub struct AdaptiveCountProcessor;

impl PostProcessor for AdaptiveCountProcessor {
    fn process(&self, _request: &PredictionRequest, candidates: &mut Vec<ScoredCandidate>) {
        if candidates.is_empty() {
            return;
        }

        let highest_confidence = candidates[0].ranking.confidence;

        let limit = if highest_confidence > 0.90 {
            1
        } else if highest_confidence > 0.75 {
            3
        } else {
            5
        };

        if candidates.len() > limit {
            candidates.truncate(limit);
        }
    }
}
