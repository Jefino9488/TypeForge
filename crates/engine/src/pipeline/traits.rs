use super::candidate::{FeatureSet, RankingResult, RawCandidate, ScoredCandidate};
use super::request::PredictionRequest;

pub trait CandidateGenerator: Send + Sync {
    fn generate(&self, request: &PredictionRequest) -> Vec<RawCandidate>;
}

pub trait CandidateExpander: Send + Sync {
    fn expand(&self, request: &PredictionRequest, seed_pool: &[RawCandidate]) -> Vec<RawCandidate>;
}

pub trait FeatureExtractor: Send + Sync {
    fn extract(&self, request: &PredictionRequest, candidate: &RawCandidate, features: &mut FeatureSet);
}

pub trait RankingStrategy: Send + Sync {
    fn score(&self, request: &PredictionRequest, candidate: &RawCandidate, features: &FeatureSet) -> RankingResult;
}

pub trait PostProcessor: Send + Sync {
    fn process(&self, request: &PredictionRequest, candidates: &mut Vec<ScoredCandidate>);
}
