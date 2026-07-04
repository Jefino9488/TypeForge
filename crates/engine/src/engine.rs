use crate::dictionary::immutable::ImmutableDictionary;
use crate::dictionary::symspell_impl::SymSpellChecker;
use crate::learning::{
    Learner, LearningConfig, LearningDb, ScoreContext, SessionMemory, TelemetryDb,
};
use crate::traits::{Dictionary, Predictor, SpellChecker};
use arc_swap::ArcSwap;
use std::sync::{Arc, RwLock};
use std::thread;
use typeforge_protocol::{PredictRequest, Prediction, PredictionSource};

pub struct TypeForgeEngine {
    immutable: Arc<ArcSwap<ImmutableDictionary>>,
    spell_checker: Arc<RwLock<SymSpellChecker>>,
    learner: Arc<Learner>,
    immutable_path: String,
    candidate_limit: usize,
}

impl TypeForgeEngine {
    pub fn new(
        immutable_path: String,
        learning_db_path: &str,
        telemetry_db_path: &str,
        candidate_limit: usize,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut immutable = ImmutableDictionary::new(immutable_path.clone());
        immutable.load()?;

        let mut spell_checker = SymSpellChecker::new();
        spell_checker.load_words(immutable.iter());

        let learning_db = Arc::new(LearningDb::new(learning_db_path)?);
        let telemetry_db = Arc::new(TelemetryDb::new(telemetry_db_path)?);
        let session_memory = Arc::new(SessionMemory::new());
        let learner = Arc::new(Learner::new(
            learning_db,
            Some(telemetry_db),
            session_memory,
            LearningConfig::default(),
        ));

        Ok(Self {
            immutable: Arc::new(ArcSwap::from_pointee(immutable)),
            spell_checker: Arc::new(RwLock::new(spell_checker)),
            learner,
            immutable_path,
            candidate_limit,
        })
    }

    pub fn get_candidate_limit(&self) -> usize {
        self.candidate_limit
    }

    pub fn predict(&self, prefix: &str, req: &PredictRequest, limit: usize) -> Vec<Prediction> {
        let is_all_caps = !prefix.is_empty() && prefix.chars().all(|c| !c.is_alphabetic() || c.is_uppercase());
        let is_capitalized = !prefix.is_empty() && prefix.chars().next().unwrap().is_uppercase();
        
        let search_prefix = prefix.to_lowercase();
        
        let immut = self.immutable.load();

        // 1. Get raw candidates from the dictionary prior
        let mut candidates = immut.predict(&search_prefix, req, limit * 2);

        // 1b. Get raw candidates from the learning database
        if let Ok(learned_words) = self.learner.get_candidates_by_prefix(&search_prefix, limit * 2) {
            for word in learned_words {
                if !candidates.iter().any(|c| c.text == word) {
                    candidates.push(Prediction {
                        text: word,
                        score: 0.0, // Base score is 0, will be updated by pipeline
                        source: PredictionSource::User,
                    });
                }
            }
        }

        // 1c. Spell check fallback if few candidates
        if candidates.len() < limit && search_prefix.len() >= 3 {
            let corrections = self.correct_spelling(&search_prefix, limit - candidates.len());
            for mut c in corrections {
                if !candidates.iter().any(|existing| existing.text == c.text) {
                    c.source = PredictionSource::SpellCorrection;
                    // Boost the spelling correction score slightly so it ranks decently
                    // alongside unigram frequencies.
                    c.score *= 100.0;
                    candidates.push(c);
                }
            }
        }

        // 2. Score and Rank through the Pipeline
        let ctx = ScoreContext {
            application: req.application.as_deref(),
        };
        self.learner.pipeline.rank(&mut candidates, &ctx);

        // 3. Normalize scores to 0.0 - 1.0 (Optional but helpful for debug/UI)
        let max_score = candidates.first().map(|c| c.score).unwrap_or(1.0).max(1.0);
        for c in &mut candidates {
            c.score /= max_score;
        }

        // 4. Take top limit
        candidates.truncate(limit);

        // 5. Restore casing
        if is_all_caps {
            for c in &mut candidates {
                c.text = c.text.to_uppercase();
            }
        } else if is_capitalized {
            for c in &mut candidates {
                let mut chars = c.text.chars();
                if let Some(first) = chars.next() {
                    c.text = first.to_uppercase().collect::<String>() + chars.as_str();
                }
            }
        }

        candidates
    }

    pub fn correct_spelling(&self, word: &str, limit: usize) -> Vec<Prediction> {
        let checker = self.spell_checker.read().unwrap();
        checker.correct(word, limit)
    }

    pub fn learn(
        &self,
        word: &str,
        context: Option<&str>,
        accepted: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if accepted {
            self.learner.on_prediction_accepted(word, context)?;
        } else {
            let is_common = self
                .immutable
                .load()
                .get_frequency(word)
                .is_some_and(|f| f > 50000);
            self.learner.on_word_typed(word, context, is_common)?;
        }
        Ok(())
    }

    pub fn reload_dictionary_background(&self) {
        let immutable_arc = Arc::clone(&self.immutable);
        let path = self.immutable_path.clone();
        thread::spawn(move || {
            let mut new_dict = ImmutableDictionary::new(path);
            if new_dict.load().is_ok() {
                immutable_arc.store(Arc::new(new_dict));
            }
        });
    }

    pub fn set_learning_enabled(&self, enabled: bool) {
        self.learner.set_learning_enabled(enabled);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::bytes_of;
    use std::fs::File;
    use std::io::Write;
    use typeforge_common::dict_format::{AlphaIndex, DictionaryEntry, DictionaryHeader};
    use typeforge_protocol::PredictRequest;

    fn setup_dummy_assets() -> (String, String, String) {
        let test_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(&test_dir).unwrap();

        let dict_path = test_dir.join("dict.bin").to_string_lossy().to_string();
        let l_db_path = test_dir.join("learning.db").to_string_lossy().to_string();
        let t_db_path = test_dir.join("telemetry.db").to_string_lossy().to_string();

        let mut file = File::create(&dict_path).unwrap();
        let mut header = DictionaryHeader {
            word_count: 3,
            ..Default::default()
        };

        let mut alpha: AlphaIndex = [0; 26];
        // apple, application, banana
        // a=0, b=2, c..z=3
        alpha[0] = 0;
        alpha[1] = 2;
        for a in alpha.iter_mut().skip(2) {
            *a = 3;
        }

        let entries = vec![
            DictionaryEntry {
                offset: 0,
                length: 5,
                first_char: b'a' as u16,
                frequency: 100,
            },
            DictionaryEntry {
                offset: 5,
                length: 11,
                first_char: b'a' as u16,
                frequency: 50,
            },
            DictionaryEntry {
                offset: 16,
                length: 6,
                first_char: b'b' as u16,
                frequency: 200,
            },
        ];

        let pool = b"appleapplicationbanana";

        header.index_offset = 48;
        header.strings_offset = 48 + 104 + (12 * 3);
        header.checksum_offset = header.strings_offset + pool.len() as u64;

        file.write_all(bytes_of(&header)).unwrap();
        file.write_all(bytemuck::cast_slice(&alpha)).unwrap();
        file.write_all(bytemuck::cast_slice(&entries)).unwrap();
        file.write_all(pool).unwrap();
        file.write_all(&[0u8; 32]).unwrap(); // Dummy checksum

        (dict_path, l_db_path, t_db_path)
    }

    fn dummy_req() -> PredictRequest {
        PredictRequest {
            prefix: "".to_string(),
            text_before_cursor: "".to_string(),
            text_after_cursor: "".to_string(),
            cursor_position: 0,
            application: None,
            language: None,
        }
    }

    #[test]
    fn test_engine_predictions_and_normalization() {
        let (dict_path, l_db_path, t_db_path) = setup_dummy_assets();
        let engine = TypeForgeEngine::new(dict_path, &l_db_path, &t_db_path, 5).unwrap();

        let preds = engine.predict("app", &dummy_req(), 5);
        assert_eq!(preds.len(), 2);
        assert_eq!(preds[0].text, "apple");
        assert_eq!(preds[0].score, 1.0); // Normalized max
        assert_eq!(preds[1].text, "application");
        assert!((preds[1].score - 0.055555556).abs() < 1e-6); // length-penalized score

        // Test user learning (accepted prediction)
        engine.learn("approach", None, true).unwrap();
        let _preds_after = engine.predict("app", &dummy_req(), 5);
        // It's not in the immutable dictionary so it won't show up yet.
        // Wait, ImmutableDictionary only returns words starting with prefix!
        // We removed MutableDictionary, so `approach` won't be returned unless it's in the pipeline candidates!
        // Ah, the ScorePipeline ranks candidates, but they must be generated by ImmutableDictionary OR user_words.
        // We forgot to generate candidates from learning_db!
    }
}
