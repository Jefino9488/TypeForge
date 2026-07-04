use crate::traits::{Dictionary, Predictor};
use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use typeforge_protocol::{Prediction, PredictionSource};

pub struct ImmutableDictionary {
    path: String,
    words: Vec<String>,
    frequencies: HashMap<String, i64>,
}

impl ImmutableDictionary {
    pub fn new(path: String) -> Self {
        Self {
            path,
            words: Vec::new(),
            frequencies: HashMap::new(),
        }
    }
}

impl Dictionary for ImmutableDictionary {
    fn load(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let file = File::open(&self.path)?;
        let decoder = GzDecoder::new(file);
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(decoder);

        for result in rdr.records() {
            let record = result?;
            if let (Some(word), Some(freq_str)) = (record.get(0), record.get(1))
                && let Ok(freq) = freq_str.parse::<i64>()
            {
                let w = word.to_string();
                self.words.push(w.clone());
                self.frequencies.insert(w, freq);
            }
        }
        self.words.sort();
        Ok(())
    }

    fn get_frequency(&self, word: &str) -> Option<i64> {
        self.frequencies.get(word).copied()
    }

    fn add_word(&mut self, _word: &str, _freq: i64) -> Result<(), Box<dyn Error + Send + Sync>> {
        Err("Cannot add word to immutable dictionary".into())
    }
}

impl Predictor for ImmutableDictionary {
    fn predict(&self, prefix: &str, _req: &typeforge_protocol::PredictRequest, limit: usize) -> Vec<Prediction> {
        let start = self.words.partition_point(|x| x.as_str() < prefix);
        let mut results = Vec::new();
        for i in start..self.words.len() {
            if self.words[i].starts_with(prefix) {
                let text = self.words[i].clone();
                let score = self.get_frequency(&text).unwrap_or(0) as f32;
                results.push(Prediction {
                    text,
                    score,
                    source: PredictionSource::Dictionary,
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
