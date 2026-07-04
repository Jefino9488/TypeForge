use std::sync::RwLock;
use std::collections::VecDeque;

const MAX_SESSION_WORDS: usize = 100;

pub struct SessionMemory {
    recent_words: RwLock<VecDeque<String>>,
}

impl SessionMemory {
    pub fn new() -> Self {
        Self {
            recent_words: RwLock::new(VecDeque::with_capacity(MAX_SESSION_WORDS)),
        }
    }

    pub fn add(&self, word: &str) {
        let mut words = self.recent_words.write().unwrap();
        // Remove if it already exists to put it at the front
        if let Some(pos) = words.iter().position(|x| x == word) {
            words.remove(pos);
        }
        words.push_front(word.to_string());
        if words.len() > MAX_SESSION_WORDS {
            words.pop_back();
        }
    }

    pub fn contains(&self, word: &str) -> bool {
        let words = self.recent_words.read().unwrap();
        words.contains(&word.to_string())
    }
}
