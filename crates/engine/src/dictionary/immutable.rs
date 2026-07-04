use crate::traits::{Dictionary, Predictor};
use memmap2::Mmap;
use std::error::Error;
use std::fs::File;
use std::sync::Arc;
use typeforge_common::dict_format::{AlphaIndex, DictionaryEntry, DictionaryHeader, MAGIC_NUMBER};
use typeforge_protocol::{Prediction, PredictionSource};

pub struct ImmutableDictionary {
    path: String,
    mmap: Option<Arc<Mmap>>,
}

impl ImmutableDictionary {
    pub fn new(path: String) -> Self {
        Self { path, mmap: None }
    }

    fn get_header(&self) -> Option<&DictionaryHeader> {
        let mmap = self.mmap.as_ref()?;
        if mmap.len() < 48 {
            return None;
        }
        let (header_bytes, _) = mmap.split_at(48);
        Some(bytemuck::from_bytes(header_bytes))
    }

    fn get_alpha_index(&self) -> Option<&AlphaIndex> {
        let header = self.get_header()?;
        let mmap = self.mmap.as_ref()?;
        let offset = header.index_offset as usize;
        let size = 104; // 26 * 4
        if mmap.len() < offset + size {
            return None;
        }
        let bytes = &mmap[offset..offset + size];
        Some(bytemuck::from_bytes(bytes))
    }

    fn get_entries(&self) -> Option<&[DictionaryEntry]> {
        let header = self.get_header()?;
        let mmap = self.mmap.as_ref()?;
        let offset = header.index_offset as usize + 104; // after AlphaIndex
        let size = header.word_count as usize * 12; // 12 bytes per entry
        if mmap.len() < offset + size {
            return None;
        }
        let bytes = &mmap[offset..offset + size];
        Some(bytemuck::cast_slice(bytes))
    }

    fn get_string(&self, entry: &DictionaryEntry) -> Option<&str> {
        let header = self.get_header()?;
        let mmap = self.mmap.as_ref()?;
        let offset = header.strings_offset as usize + entry.offset as usize;
        let size = entry.length as usize;
        if mmap.len() < offset + size {
            return None;
        }
        std::str::from_utf8(&mmap[offset..offset + size]).ok()
    }

    pub fn iter(&self) -> ImmutableDictionaryIter {
        ImmutableDictionaryIter {
            dict: self.clone(),
            index: 0,
        }
    }
}

impl Clone for ImmutableDictionary {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            mmap: self.mmap.clone(),
        }
    }
}

pub struct ImmutableDictionaryIter {
    dict: ImmutableDictionary,
    index: usize,
}

impl Iterator for ImmutableDictionaryIter {
    type Item = (String, i64);

    fn next(&mut self) -> Option<Self::Item> {
        let entries = self.dict.get_entries()?;
        if self.index >= entries.len() {
            return None;
        }
        let entry = &entries[self.index];
        self.index += 1;
        let word = self.dict.get_string(entry)?.to_string();
        Some((word, entry.frequency as i64))
    }
}

impl Dictionary for ImmutableDictionary {
    fn load(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let file = File::open(&self.path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        // Verify basic sanity
        if mmap.len() < 48 {
            return Err("Dictionary file too small".into());
        }
        let header_bytes = &mmap[0..48];
        let header: &DictionaryHeader = bytemuck::from_bytes(header_bytes);
        if header.magic != MAGIC_NUMBER {
            return Err("Invalid dictionary magic number".into());
        }

        self.mmap = Some(Arc::new(mmap));
        Ok(())
    }

    fn get_frequency(&self, word: &str) -> Option<i64> {
        let entries = self.get_entries()?;
        let first_char = word.chars().next()?.to_ascii_lowercase();

        let mut start_idx = 0;
        let mut end_idx = entries.len();

        if first_char.is_ascii_lowercase() {
            let alpha = self.get_alpha_index()?;
            let a_idx = (first_char as u8 - b'a') as usize;
            start_idx = alpha[a_idx] as usize;
            if a_idx < 25 {
                end_idx = alpha[a_idx + 1] as usize;
            }
        }

        let slice = &entries[start_idx..end_idx];
        let result = slice.binary_search_by(|entry| {
            let s = self.get_string(entry).unwrap_or("");
            s.cmp(word)
        });

        match result {
            Ok(idx) => Some(slice[idx].frequency as i64),
            Err(_) => None,
        }
    }

    fn add_word(&mut self, _word: &str, _freq: i64) -> Result<(), Box<dyn Error + Send + Sync>> {
        Err("Cannot add word to immutable dictionary".into())
    }
}

impl Predictor for ImmutableDictionary {
    fn predict(
        &self,
        prefix: &str,
        _req: &typeforge_protocol::PredictRequest,
        limit: usize,
    ) -> Vec<Prediction> {
        let mut results = Vec::new();
        let entries = match self.get_entries() {
            Some(e) => e,
            None => return results,
        };

        let first_char = match prefix.chars().next() {
            Some(c) => c.to_ascii_lowercase(),
            None => return results,
        };

        let mut start_idx = 0;
        let mut end_idx = entries.len();

        if first_char.is_ascii_lowercase()
            && let Some(alpha) = self.get_alpha_index()
        {
            let a_idx = (first_char as u8 - b'a') as usize;
            start_idx = alpha[a_idx] as usize;
            if a_idx < 25 {
                end_idx = alpha[a_idx + 1] as usize;
            }
        }

        let slice = &entries[start_idx..end_idx];
        let result = slice.binary_search_by(|entry| {
            let s = self.get_string(entry).unwrap_or("");
            s.cmp(prefix)
        });

        let start = match result {
            Ok(idx) => idx,
            Err(idx) => idx,
        };

        for entry in slice.iter().skip(start) {
            let s = self.get_string(entry).unwrap_or("");
            if s.starts_with(prefix) {
                // Score: Frequency is primary, but we penalize length so shorter words are favored
                // when frequencies are somewhat close. Exact matches get a massive boost.
                let mut score =
                    (entry.frequency as f32) / (s.len() as f32 - prefix.len() as f32 + 1.0).powi(2);

                if s == prefix {
                    score += 10_000_000.0; // Exact match always wins
                }

                results.push(Prediction {
                    text: s.to_string(),
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

        // Sort them descending by our new length-based score before returning
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }
}
