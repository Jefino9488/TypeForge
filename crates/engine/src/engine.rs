use crate::traits::{Dictionary, Predictor, SpellChecker};
use crate::dictionary::immutable::ImmutableDictionary;
use crate::dictionary::mutable::MutableDictionary;
use crate::dictionary::symspell_impl::SymSpellChecker;
use std::sync::{Arc, RwLock};
use typeforge_protocol::{Prediction, PredictionSource};
use arc_swap::ArcSwap;
use std::thread;

pub struct TypeForgeEngine {
    immutable: Arc<ArcSwap<ImmutableDictionary>>,
    mutable: Arc<RwLock<MutableDictionary>>,
    spell_checker: Arc<RwLock<SymSpellChecker>>,
    immutable_path: String,
}

impl TypeForgeEngine {
    pub fn new(immutable_path: String, db_path: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut immutable = ImmutableDictionary::new(immutable_path.clone());
        immutable.load()?;
        
        let mut mutable = MutableDictionary::new(db_path)?;
        mutable.load()?;
        
        let spell_checker = SymSpellChecker::new();
        
        Ok(Self {
            immutable: Arc::new(ArcSwap::from_pointee(immutable)),
            mutable: Arc::new(RwLock::new(mutable)),
            spell_checker: Arc::new(RwLock::new(spell_checker)),
            immutable_path,
        })
    }
    
    pub fn predict(&self, prefix: &str, limit: usize) -> Vec<Prediction> {
        let immut = self.immutable.load();
        let mut_dict = self.mutable.read().unwrap();
        
        let mut candidates = immut.predict(prefix, limit);
        let mut mut_candidates = mut_dict.predict(prefix, limit);
        
        // Merge candidates
        for mc in mut_candidates {
            if let Some(existing) = candidates.iter_mut().find(|c| c.text == mc.text) {
                // If it exists in both, prefer the User source and higher score
                existing.source = PredictionSource::User;
                existing.score += mc.score;
            } else {
                candidates.push(mc);
            }
        }
        
        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Normalize scores to 0.0 - 1.0
        let max_score = candidates.first().map(|c| c.score).unwrap_or(1.0).max(1.0);
        for c in &mut candidates {
            c.score /= max_score;
        }
        
        candidates.into_iter().take(limit).collect()
    }
    
    pub fn learn(&self, word: &str, freq: i64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut mut_dict = self.mutable.write().unwrap();
        mut_dict.add_word(word, freq)?;
        Ok(())
    }

    pub fn reload_dictionary_background(&self) {
        let immutable_arc = Arc::clone(&self.immutable);
        let path = self.immutable_path.clone();
        thread::spawn(move || {
            let mut new_dict = ImmutableDictionary::new(path);
            if let Ok(_) = new_dict.load() {
                immutable_arc.store(Arc::new(new_dict));
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use flate2::write::GzEncoder;
    use flate2::Compression;

    fn setup_dummy_assets() -> (String, String) {
        let test_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(&test_dir).unwrap();
        
        let dict_path = test_dir.join("dict.csv.gz").to_string_lossy().to_string();
        let db_path = test_dir.join("test.db").to_string_lossy().to_string();
        
        let file = File::create(&dict_path).unwrap();
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(b"apple,100\napplication,50\nbanana,200\n").unwrap();
        encoder.finish().unwrap();
        
        (dict_path, db_path)
    }

    #[test]
    fn test_engine_predictions_and_normalization() {
        let (dict_path, db_path) = setup_dummy_assets();
        let engine = TypeForgeEngine::new(dict_path, &db_path).unwrap();
        
        let preds = engine.predict("app", 5);
        assert_eq!(preds.len(), 2);
        assert_eq!(preds[0].text, "apple");
        assert_eq!(preds[0].score, 1.0); // Normalized max
        assert_eq!(preds[1].text, "application");
        assert_eq!(preds[1].score, 0.5); // 50 / 100
        
        // Test user learning
        engine.learn("approach", 200).unwrap();
        let preds_after = engine.predict("app", 5);
        assert_eq!(preds_after.len(), 3);
        assert_eq!(preds_after[0].text, "approach");
        assert_eq!(preds_after[0].score, 1.0); // 200/200
        assert_eq!(preds_after[1].text, "apple");
        assert_eq!(preds_after[1].score, 0.5); // 100/200
    }
}
