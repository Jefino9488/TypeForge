use super::candidate::{FeatureSet, ScoredCandidate};
use super::observer::PipelineObserver;
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
    pub fn execute<O: PipelineObserver>(
        &self,
        request: &PredictionRequest,
        observer: &mut O,
    ) -> PredictionResult {
        let total_start = Instant::now();
        let mut telemetry = PredictionTelemetry::default();
        observer.on_start(request);

        // Stage 1: Retrieval
        let gen_start = Instant::now();
        let mut raw_candidates = Vec::new();
        for generator in &self.generators {
            raw_candidates.extend(generator.generate(request));
        }
        let gen_elapsed = gen_start.elapsed().as_micros() as u64;
        telemetry.generator_latency_us = gen_elapsed;
        observer.on_generators_done(&raw_candidates, gen_elapsed);

        if request.cancelled() {
            return PredictionResult {
                candidates: vec![],
                telemetry,
            };
        }

        // Stage 2: Expansion
        let exp_start = Instant::now();
        let mut expanded_candidates = Vec::new();
        for exp in &self.expanders {
            expanded_candidates.extend(exp.expand(request, &raw_candidates));
        }
        raw_candidates.extend(expanded_candidates.clone());
        let exp_elapsed = exp_start.elapsed().as_micros() as u64;
        observer.on_expanders_done(&expanded_candidates, exp_elapsed);

        // Stage 3: Deduplicator
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
        let _ext_start = std::time::Instant::now();
        let mut scored_candidates = Vec::with_capacity(raw_candidates.len());
        let ranker = self.ranker.as_ref().expect("Pipeline must have a ranker");

        // We run features and ranking together
        let mut extract_total = 0;
        let mut rank_total = 0;

        for raw in raw_candidates {
            let mut features = FeatureSet::default();

            let f_s = Instant::now();
            for ext in &self.extractors {
                ext.extract(request, &raw, &mut features);
            }
            extract_total += f_s.elapsed().as_micros() as u64;

            let r_s = Instant::now();
            let ranking = ranker.score(request, &raw, &features);
            rank_total += r_s.elapsed().as_micros() as u64;

            scored_candidates.push(ScoredCandidate {
                candidate: raw,
                features,
                ranking,
            });
        }

        telemetry.extraction_latency_us = extract_total;
        telemetry.ranking_latency_us = rank_total;
        telemetry.candidates_ranked = scored_candidates.len();

        observer.on_features_done(&scored_candidates, extract_total);
        observer.on_ranking_done(&scored_candidates, rank_total);

        if request.cancelled() {
            return PredictionResult {
                candidates: vec![],
                telemetry,
            };
        }

        // Stage 6: Post-processing
        let post_start = Instant::now();
        for post in &self.postprocessors {
            post.process(request, &mut scored_candidates);
        }
        let post_elapsed = post_start.elapsed().as_micros() as u64;
        observer.on_postprocess_done(&scored_candidates, post_elapsed);

        let total_elapsed = total_start.elapsed().as_micros() as u64;
        telemetry.total_latency_us = total_elapsed;
        observer.on_complete(total_elapsed);

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
