use crate::traits::SpellChecker;
use typeforge_protocol::{Prediction, PredictionSource};
use symspell::{SymSpell, AsciiStringStrategy};

pub struct SymSpellChecker {
    symspell: SymSpell<AsciiStringStrategy>,
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
        suggestions.into_iter().take(limit).map(|s| {
            Prediction {
                text: s.term,
                score: s.distance as f32 * -1.0, // Lower distance is better, so negate it for score
                source: PredictionSource::SpellCorrection,
            }
        }).collect()
    }
}
