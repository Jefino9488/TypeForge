#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CandidateSource {
    Dictionary,
    UserDictionary,
    SessionMemory,
    Phrase,
    Context,
    SpellCorrection,
    FuzzySearch,
    AI,
}

#[derive(Debug, Clone)]
pub struct CandidateMetadata {
    pub source: CandidateSource,
    pub matched_prefix: String,
    pub edit_distance: u8,
    pub context_match: bool,
}

#[derive(Debug, Clone)]
pub struct RawCandidate {
    pub text: String,
    pub metadata: CandidateMetadata,
}

#[derive(Debug, Clone, Default)]
pub struct FeatureSet {
    pub base_frequency: f32,
    pub user_frequency: f32,
    pub session_score: f32,
    pub context_score: f32,
    pub ngram_score: f32,
    pub edit_distance: u8,
    pub exact_prefix: bool,
    pub prefix_length: u8,
    pub is_custom: bool,
    pub word_length: u8,
}

#[derive(Debug, Clone)]
pub struct RankingResult {
    pub score: f32,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub struct ScoredCandidate {
    pub candidate: RawCandidate,
    pub features: FeatureSet,
    pub ranking: RankingResult,
}
