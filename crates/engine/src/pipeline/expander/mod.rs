use super::candidate::{CandidateMetadata, CandidateSource, RawCandidate};
use super::request::PredictionRequest;
use super::traits::CandidateExpander;
use crate::dictionary::immutable::ImmutableDictionary;
use crate::dictionary::symspell_impl::SymSpellChecker;
use crate::traits::SpellChecker;
use arc_swap::ArcSwap;
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

pub struct SegmentationExpander {
    dictionary: Arc<ArcSwap<ImmutableDictionary>>,
}

impl SegmentationExpander {
    pub fn new(dictionary: Arc<ArcSwap<ImmutableDictionary>>) -> Self {
        Self { dictionary }
    }
}

impl CandidateExpander for SegmentationExpander {
    fn expand(
        &self,
        request: &PredictionRequest,
        _seed_pool: &[RawCandidate],
    ) -> Vec<RawCandidate> {
        let prefix = request
            .text_before_cursor
            .split_whitespace()
            .last()
            .unwrap_or("")
            .to_lowercase();
        if prefix.len() < 5 {
            return vec![]; // Too short to reasonably segment
        }

        let dict = self.dictionary.load();

        // Dynamic programming for word segmentation
        let n = prefix.len();
        let mut dp = vec![false; n + 1];
        let mut split = vec![0; n + 1];
        dp[0] = true;

        for i in 1..=n {
            for j in (0..i).rev() {
                if dp[j] {
                    let word = &prefix[j..i];
                    if dict.contains(word) {
                        dp[i] = true;
                        split[i] = j;
                        break;
                    }
                }
            }
        }

        if dp[n] && split[n] > 0 {
            let mut words = Vec::new();
            let mut curr = n;
            while curr > 0 {
                let prev = split[curr];
                words.push(&prefix[prev..curr]);
                curr = prev;
            }
            words.reverse();
            let segmented = words.join(" ");

            // Only return if it actually split into >1 word
            if words.len() > 1 {
                return vec![RawCandidate {
                    text: segmented,
                    metadata: CandidateMetadata {
                        source: CandidateSource::SpellCorrection,
                        matched_prefix: prefix.clone(),
                        edit_distance: 1,
                        context_match: false,
                    },
                }];
            }
        }

        vec![]
    }
}
