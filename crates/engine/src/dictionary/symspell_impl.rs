use crate::traits::SpellChecker;
use symspell::{AsciiStringStrategy, SymSpell};
use typeforge_protocol::{Prediction, PredictionSource};

pub struct SymSpellChecker {
    symspell: SymSpell<AsciiStringStrategy>,
}

impl Default for SymSpellChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl SymSpellChecker {
    pub fn new() -> Self {
        let symspell: SymSpell<AsciiStringStrategy> = SymSpell::default();
        Self { symspell }
    }
}

impl SpellChecker for SymSpellChecker {
    fn correct(&self, word: &str, limit: usize) -> Vec<Prediction> {
        let suggestions = self.symspell.lookup(word, symspell::Verbosity::Top, 2);
        suggestions
            .into_iter()
            .take(limit)
            .map(|s| {
                Prediction {
                    text: s.term,
                    score: -(s.distance as f32), // Lower distance is better, so negate it for score
                    source: PredictionSource::SpellCorrection,
                }
            })
            .collect()
    }
}
