use crate::db::Database;
use crate::traits::{Dictionary, Predictor};
use std::collections::HashMap;
use std::error::Error;
use typeforge_protocol::{Prediction, PredictionSource};

pub struct MutableDictionary {
    db: Database,
    words: Vec<String>,
    frequencies: HashMap<String, i64>,
}

impl MutableDictionary {
    pub fn new(db_path: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let db = Database::new(db_path)?;
        Ok(Self {
            db,
            words: Vec::new(),
            frequencies: HashMap::new(),
        })
    }
}

impl Dictionary for MutableDictionary {
    fn load(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let rows = self.db.load_all()?;
        for (word, freq) in rows {
            if !self.frequencies.contains_key(&word) {
                self.words.push(word.clone());
            }
            self.frequencies.insert(word, freq);
        }
        self.words.sort();
        Ok(())
    }

    fn get_frequency(&self, word: &str) -> Option<i64> {
        self.frequencies.get(word).copied()
    }

    fn add_word(&mut self, word: &str, freq: i64) -> Result<(), Box<dyn Error + Send + Sync>> {
        if !self.frequencies.contains_key(word) {
            self.words.push(word.to_string());
            self.words.sort();
        }

        let current_freq = self.frequencies.entry(word.to_string()).or_insert(0);
        *current_freq += freq;

        self.db.upsert_word(word, freq)?;
        Ok(())
    }
}

impl Predictor for MutableDictionary {
    fn predict(&self, prefix: &str, limit: usize) -> Vec<Prediction> {
        let start = self.words.partition_point(|x| x.as_str() < prefix);
        let mut results = Vec::new();
        for i in start..self.words.len() {
            if self.words[i].starts_with(prefix) {
                let text = self.words[i].clone();
                let score = self.get_frequency(&text).unwrap_or(0) as f32;
                results.push(Prediction {
                    text,
                    score,
                    source: PredictionSource::User,
                });
                if results.len() >= limit * 10 {
                    break;
                }
            } else {
                break;
            }
        }
        results
    }
}
