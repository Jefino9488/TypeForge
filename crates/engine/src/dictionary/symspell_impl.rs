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
        use symspell::SymSpellBuilder;
        let symspell: SymSpell<AsciiStringStrategy> = SymSpellBuilder::default()
            .max_dictionary_edit_distance(4)
            .build()
            .unwrap();
        Self { symspell }
    }

    pub fn load_words(&mut self, words: impl Iterator<Item = (String, i64)>) {
        let mut all_words: Vec<(String, i64)> = words.collect();
        println!("Loaded {} words into memory for sorting", all_words.len());
        // Sort descending by frequency
        all_words.sort_unstable_by_key(|b| std::cmp::Reverse(b.1));

        let mut count = 0;
        for (word, freq) in all_words {
            let line = format!("{} {}", word, freq);
            self.symspell.load_dictionary_line(&line, 0, 1, " ");
            count += 1;
            // Cap at 100,000 to save memory and startup time
            if count >= 100000 {
                break;
            }
        }
    }
}

impl SpellChecker for SymSpellChecker {
    fn correct(&self, word: &str, limit: usize) -> Vec<Prediction> {
        let mut suggestions = self.symspell.lookup(word, symspell::Verbosity::All, 4);
        // Sort our own way: score based on frequency and distance. Heavily penalize distance!
        suggestions.sort_by(|a, b| {
            let score_a = a.count as f32 / 100.0_f32.powi(a.distance as i32);
            let score_b = b.count as f32 / 100.0_f32.powi(b.distance as i32);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        suggestions
            .into_iter()
            .take(limit)
            .map(|s| {
                Prediction {
                    text: s.term,
                    // Pass the calculated score forward
                    score: s.count as f32 / 100.0_f32.powi(s.distance as i32),
                    source: PredictionSource::SpellCorrection,
                }
            })
            .collect()
    }
}
