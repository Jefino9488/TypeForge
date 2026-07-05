pub mod adaptive_count;

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
        candidates.sort_by(|a, b| {
            b.ranking
                .confidence
                .partial_cmp(&a.ranking.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        // Truncate to max
        candidates.truncate(self.max_candidates);
    }
}

pub struct CapitalizationProcessor;

impl CapitalizationProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CapitalizationProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl PostProcessor for CapitalizationProcessor {
    fn process(&self, request: &PredictionRequest, candidates: &mut Vec<ScoredCandidate>) {
        let prefix = request
            .text_before_cursor
            .split_whitespace()
            .last()
            .unwrap_or("");
        if prefix.is_empty() {
            return;
        }

        let is_first_upper = prefix.chars().next().unwrap().is_uppercase();
        let is_all_upper = prefix.chars().all(|c| c.is_uppercase());

        for candidate in candidates.iter_mut() {
            if is_all_upper {
                candidate.candidate.text = candidate.candidate.text.to_uppercase();
            } else if is_first_upper {
                let mut c = candidate.candidate.text.chars();
                if let Some(f) = c.next() {
                    candidate.candidate.text = f.to_uppercase().chain(c).collect();
                }
            }
        }
    }
}
