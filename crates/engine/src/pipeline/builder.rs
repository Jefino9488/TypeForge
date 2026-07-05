use super::candidate::{FeatureSet, RawCandidate, ScoredCandidate};
use super::request::PredictionRequest;
use super::result::{PredictionResult, PredictionTelemetry};
use super::traits::{
    CandidateExpander, CandidateGenerator, FeatureExtractor, PostProcessor, RankingStrategy,
};

use std::collections::HashSet;
use std::time::Instant;

pub struct Pipeline {
    generators: Vec<Box<dyn CandidateGenerator>>,
    expanders: Vec<Box<dyn CandidateExpander>>,
    extractors: Vec<Box<dyn FeatureExtractor>>,
    ranker: Option<Box<dyn RankingStrategy>>,
    postprocessors: Vec<Box<dyn PostProcessor>>,
}

impl Pipeline {
    pub fn execute(&self, request: &PredictionRequest) -> PredictionResult {
        let total_start = Instant::now();
        let mut telemetry = PredictionTelemetry::default();

        // Stage 1: Retrieval
        let gen_start = Instant::now();
        let mut raw_candidates = Vec::new();
        for generator in &self.generators {
            raw_candidates.extend(generator.generate(request));
        }
        telemetry.generator_latency_us = gen_start.elapsed().as_micros() as u64;

        if request.cancelled() {
            return PredictionResult {
                candidates: vec![],
                telemetry,
            };
        }

        // Stage 2: Expansion
        let mut expanded_candidates = Vec::new();
        for exp in &self.expanders {
            expanded_candidates.extend(exp.expand(request, &raw_candidates));
        }
        raw_candidates.extend(expanded_candidates);

        // Stage 3: Deduplicator
        // For now, basic deduplication by exact text to prevent duplicate feature extraction
        let mut seen = HashSet::new();
        raw_candidates.retain(|c| seen.insert(c.text.clone()));
        telemetry.candidates_generated = raw_candidates.len();

        if request.cancelled() {
            return PredictionResult {
                candidates: vec![],
                telemetry,
            };
        }

        // Stage 4: Feature Extraction (Lazy)
        let ext_start = Instant::now();
        let mut scored_candidates = Vec::with_capacity(raw_candidates.len());

        let ranker = self.ranker.as_ref().expect("Pipeline must have a ranker");

        for raw in raw_candidates {
            let mut features = FeatureSet::default();
            for ext in &self.extractors {
                ext.extract(request, &raw, &mut features);
            }

            // Stage 5: Ranking
            let ranking = ranker.score(request, &raw, &features);

            scored_candidates.push(ScoredCandidate {
                candidate: raw,
                features,
                ranking,
            });
        }
        telemetry.extraction_latency_us = ext_start.elapsed().as_micros() as u64; // Approximates both extraction and ranking for now
        telemetry.ranking_latency_us = 0; // Handled sequentially inside the loop above
        telemetry.candidates_ranked = scored_candidates.len();

        if request.cancelled() {
            return PredictionResult {
                candidates: vec![],
                telemetry,
            };
        }

        // Stage 6: Post-processing
        for post in &self.postprocessors {
            post.process(request, &mut scored_candidates);
        }

        telemetry.total_latency_us = total_start.elapsed().as_micros() as u64;

        PredictionResult {
            candidates: scored_candidates,
            telemetry,
        }
    }
}

pub struct PipelineBuilder {
    pipeline: Pipeline,
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineBuilder {
    pub fn new() -> Self {
        Self {
            pipeline: Pipeline {
                generators: Vec::new(),
                expanders: Vec::new(),
                extractors: Vec::new(),
                ranker: None,
                postprocessors: Vec::new(),
            },
        }
    }

    pub fn generator(mut self, generator: Box<dyn CandidateGenerator>) -> Self {
        self.pipeline.generators.push(generator);
        self
    }

    pub fn expander(mut self, expander: Box<dyn CandidateExpander>) -> Self {
        self.pipeline.expanders.push(expander);
        self
    }

    pub fn feature(mut self, extractor: Box<dyn FeatureExtractor>) -> Self {
        self.pipeline.extractors.push(extractor);
        self
    }

    pub fn ranker(mut self, ranker: Box<dyn RankingStrategy>) -> Self {
        self.pipeline.ranker = Some(ranker);
        self
    }

    pub fn postprocessor(mut self, processor: Box<dyn PostProcessor>) -> Self {
        self.pipeline.postprocessors.push(processor);
        self
    }

    pub fn build(self) -> Pipeline {
        assert!(
            self.pipeline.ranker.is_some(),
            "A RankingStrategy is required"
        );
        self.pipeline
    }
}
