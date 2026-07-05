use super::candidate::{RawCandidate, ScoredCandidate};
use super::request::PredictionRequest;
use std::collections::HashMap;
use typeforge_protocol::{CandidateTrace, FeatureTrace, PipelineTrace, TimingMetrics};

pub trait PipelineObserver {
    fn on_start(&mut self, request: &PredictionRequest);
    fn on_generators_done(&mut self, candidates: &[RawCandidate], elapsed_us: u64);
    fn on_expanders_done(&mut self, candidates: &[RawCandidate], elapsed_us: u64);
    fn on_features_done(&mut self, candidates: &[ScoredCandidate], elapsed_us: u64);
    fn on_ranking_done(&mut self, candidates: &[ScoredCandidate], elapsed_us: u64);
    fn on_postprocess_done(&mut self, candidates: &[ScoredCandidate], elapsed_us: u64);
    fn on_complete(&mut self, elapsed_us: u64);
}

pub struct NoopObserver;

impl PipelineObserver for NoopObserver {
    #[inline(always)]
    fn on_start(&mut self, _request: &PredictionRequest) {}
    #[inline(always)]
    fn on_generators_done(&mut self, _candidates: &[RawCandidate], _elapsed_us: u64) {}
    #[inline(always)]
    fn on_expanders_done(&mut self, _candidates: &[RawCandidate], _elapsed_us: u64) {}
    #[inline(always)]
    fn on_features_done(&mut self, _candidates: &[ScoredCandidate], _elapsed_us: u64) {}
    #[inline(always)]
    fn on_ranking_done(&mut self, _candidates: &[ScoredCandidate], _elapsed_us: u64) {}
    #[inline(always)]
    fn on_postprocess_done(&mut self, _candidates: &[ScoredCandidate], _elapsed_us: u64) {}
    #[inline(always)]
    fn on_complete(&mut self, _elapsed_us: u64) {}
}

pub struct TraceObserver {
    pub trace: PipelineTrace,
    // Temporary storage during execution to map text to its sources
    generator_sources: HashMap<String, Vec<String>>,
    expander_sources: HashMap<String, Vec<String>>,
}

impl TraceObserver {
    pub fn new(pipeline_version: u32) -> Self {
        Self {
            trace: PipelineTrace {
                pipeline_version,
                timings: TimingMetrics {
                    generators_us: 0,
                    expanders_us: 0,
                    features_us: 0,
                    ranking_us: 0,
                    post_processing_us: 0,
                    total_us: 0,
                },
                candidates: Vec::new(),
            },
            generator_sources: HashMap::new(),
            expander_sources: HashMap::new(),
        }
    }
}

impl PipelineObserver for TraceObserver {
    fn on_start(&mut self, _request: &PredictionRequest) {}

    fn on_generators_done(&mut self, candidates: &[RawCandidate], elapsed_us: u64) {
        self.trace.timings.generators_us = elapsed_us;
        for c in candidates {
            self.generator_sources
                .entry(c.text.clone())
                .or_default()
                .push(format!("{:?}", c.metadata.source));
        }
    }

    fn on_expanders_done(&mut self, candidates: &[RawCandidate], elapsed_us: u64) {
        self.trace.timings.expanders_us = elapsed_us;
        for c in candidates {
            // Only add to expanders if it wasn't already in generators
            if !self.generator_sources.contains_key(&c.text) {
                self.expander_sources
                    .entry(c.text.clone())
                    .or_default()
                    .push(format!("{:?}", c.metadata.source));
            }
        }
    }

    fn on_features_done(&mut self, _candidates: &[ScoredCandidate], elapsed_us: u64) {
        self.trace.timings.features_us = elapsed_us;
    }

    fn on_ranking_done(&mut self, _candidates: &[ScoredCandidate], elapsed_us: u64) {
        self.trace.timings.ranking_us = elapsed_us;
    }

    fn on_postprocess_done(&mut self, candidates: &[ScoredCandidate], elapsed_us: u64) {
        self.trace.timings.post_processing_us = elapsed_us;

        // Finalize trace using the fully scored/post-processed candidates
        for (i, c) in candidates.iter().enumerate() {
            let mut gens = self
                .generator_sources
                .get(&c.candidate.text)
                .cloned()
                .unwrap_or_default();
            gens.sort();
            gens.dedup();

            let mut exps = self
                .expander_sources
                .get(&c.candidate.text)
                .cloned()
                .unwrap_or_default();
            exps.sort();
            exps.dedup();

            self.trace.candidates.push(CandidateTrace {
                text: c.candidate.text.clone(),
                generators: gens,
                expanders: exps,
                features: FeatureTrace {
                    base_frequency: c.features.base_frequency,
                    user_frequency: c.features.user_frequency,
                    context_match: c.features.context_score,
                    session_match: c.features.session_frequency,
                    edit_distance: c.features.edit_distance as f32,
                },
                score: c.ranking.score,
                confidence: c.ranking.confidence,
                rank: i + 1,
            });
        }
    }

    fn on_complete(&mut self, elapsed_us: u64) {
        self.trace.timings.total_us = elapsed_us;
    }
}
