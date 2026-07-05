use super::candidate::{CandidateMetadata, CandidateSource, RawCandidate};
use super::request::PredictionRequest;
use super::traits::CandidateExpander;
use crate::dictionary::symspell_impl::SymSpellChecker;
use crate::traits::SpellChecker;
use std::sync::{Arc, RwLock};

pub struct SpellExpander {
    spell_checker: Arc<RwLock<SymSpellChecker>>,
    limit: usize,
}

impl SpellExpander {
    pub fn new(spell_checker: Arc<RwLock<SymSpellChecker>>, limit: usize) -> Self {
        Self {
            spell_checker,
            limit,
        }
    }
}

impl CandidateExpander for SpellExpander {
    fn expand(&self, request: &PredictionRequest, seed_pool: &[RawCandidate]) -> Vec<RawCandidate> {
        let prefix = request
            .text_before_cursor
            .split_whitespace()
            .last()
            .unwrap_or("")
            .to_lowercase();
        if prefix.len() < 3 || seed_pool.len() >= self.limit {
            return vec![];
        }

        let checker = self.spell_checker.read().unwrap();
        let corrections = checker.correct(&prefix, self.limit);

        corrections
            .into_iter()
            .map(|p| RawCandidate {
                text: p.text,
                metadata: CandidateMetadata {
                    source: CandidateSource::SpellCorrection,
                    matched_prefix: prefix.clone(),
                    edit_distance: 1, // simplified for MVP
                    context_match: false,
                },
            })
            .collect()
    }
}

pub struct FuzzyExpander {
    spell_checker: Arc<RwLock<SymSpellChecker>>,
    limit: usize,
}

impl FuzzyExpander {
    pub fn new(spell_checker: Arc<RwLock<SymSpellChecker>>, limit: usize) -> Self {
        Self {
            spell_checker,
            limit,
        }
    }
}

impl CandidateExpander for FuzzyExpander {
    fn expand(&self, request: &PredictionRequest, seed_pool: &[RawCandidate]) -> Vec<RawCandidate> {
        let prefix = request
            .text_before_cursor
            .split_whitespace()
            .last()
            .unwrap_or("")
            .to_lowercase();
        if prefix.len() < 3 || seed_pool.len() >= self.limit {
            return vec![];
        }

        let checker = self.spell_checker.read().unwrap();
        // SymSpell's correction is inherently fuzzy matching edit distance.
        // For this phase, we reuse it, but tag it as fuzzy and assign edit_distance 2.
        let corrections = checker.correct(&prefix, self.limit);

        corrections
            .into_iter()
            .map(|p| RawCandidate {
                text: p.text,
                metadata: CandidateMetadata {
                    source: CandidateSource::FuzzySearch,
                    matched_prefix: prefix.clone(),
                    edit_distance: 2,
                    context_match: false,
                },
            })
            .collect()
    }
}
