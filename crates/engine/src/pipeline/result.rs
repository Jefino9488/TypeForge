use super::candidate::ScoredCandidate;

#[derive(Default)]
pub struct PredictionTelemetry {
    pub candidates_generated: usize,
    pub candidates_ranked: usize,
    pub generator_latency_us: u64,
    pub extraction_latency_us: u64,
    pub ranking_latency_us: u64,
    pub total_latency_us: u64,
}

pub struct PredictionResult {
    pub candidates: Vec<ScoredCandidate>,
    pub telemetry: PredictionTelemetry,
}
