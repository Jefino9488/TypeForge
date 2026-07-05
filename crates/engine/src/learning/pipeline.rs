use super::persistence::LearningDb;
use super::session::SessionMemory;
use std::sync::Arc;

pub struct CommitEvent {
    pub word: String,
    pub previous_word: Option<String>,
    pub context: Option<String>,
    pub is_accepted_prediction: bool,
    pub is_common_word: bool,
}

pub trait LearnerStage: Send + Sync {
    /// Returns true to continue to the next stage, false to short-circuit
    fn process(&self, event: &CommitEvent) -> bool;
}

pub struct SpamFilterStage;

impl SpamFilterStage {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SpamFilterStage {
    fn default() -> Self {
        Self::new()
    }
}

impl LearnerStage for SpamFilterStage {
    fn process(&self, event: &CommitEvent) -> bool {
        // Very basic spam filter
        if event.word.len() > 30 {
            return false;
        }

        // Check for repeated characters ratio (e.g. "aaaaa", "lollllll")
        let mut max_consecutive = 1;
        let mut current_consecutive = 1;
        let mut prev_char = '\0';

        for c in event.word.chars() {
            if c == prev_char {
                current_consecutive += 1;
                max_consecutive = max_consecutive.max(current_consecutive);
            } else {
                current_consecutive = 1;
                prev_char = c;
            }
        }

        // If a word has more than 4 consecutive identical characters, it's likely spam.
        // E.g., "aaaaa" or "loooool"
        if max_consecutive > 4 {
            return false;
        }

        // Check for too few unique characters (entropy heuristic)
        let unique_chars: std::collections::HashSet<char> = event.word.chars().collect();
        if event.word.len() > 8 && unique_chars.len() <= 2 {
            // E.g. "asdfasdfasdf" might have 4 unique chars, but "aaaaaaaaab" has 2.
            return false;
        }

        true
    }
}

pub struct CooldownStage {
    last_learned: std::sync::Mutex<Option<(String, std::time::Instant)>>,
}

impl CooldownStage {
    pub fn new() -> Self {
        Self {
            last_learned: std::sync::Mutex::new(None),
        }
    }
}

impl Default for CooldownStage {
    fn default() -> Self {
        Self::new()
    }
}

impl LearnerStage for CooldownStage {
    fn process(&self, event: &CommitEvent) -> bool {
        let mut last = self.last_learned.lock().unwrap();
        let now = std::time::Instant::now();

        if let Some((word, time)) = last.as_ref()
            && word == &event.word
            && now.duration_since(*time).as_millis() < 500
        {
            // Cooldown triggered
            return false;
        }

        *last = Some((event.word.clone(), now));
        true
    }
}

pub struct WordLearner {
    db: Arc<LearningDb>,
}

impl WordLearner {
    pub fn new(db: Arc<LearningDb>) -> Self {
        Self { db }
    }
}

impl LearnerStage for WordLearner {
    fn process(&self, event: &CommitEvent) -> bool {
        let weight = if event.is_accepted_prediction { 10 } else { 2 };

        if event.is_accepted_prediction || !event.is_common_word {
            let _ = self
                .db
                .increase_weight(&event.word, event.context.as_deref(), weight);
        }
        true
    }
}

pub struct NGramLearner {
    db: Arc<LearningDb>,
}

impl NGramLearner {
    pub fn new(db: Arc<LearningDb>) -> Self {
        Self { db }
    }
}

impl LearnerStage for NGramLearner {
    fn process(&self, event: &CommitEvent) -> bool {
        if let Some(prev) = &event.previous_word {
            // Learn the bigram: prev -> word
            let _ = self.db.increase_ngram_weight(prev, &event.word, 1);
        }
        true
    }
}

pub struct SessionLearner {
    memory: Arc<SessionMemory>,
}

impl SessionLearner {
    pub fn new(memory: Arc<SessionMemory>) -> Self {
        Self { memory }
    }
}

impl LearnerStage for SessionLearner {
    fn process(&self, event: &CommitEvent) -> bool {
        self.memory.add(&event.word);
        true
    }
}

pub struct LearningPipeline {
    stages: Vec<Box<dyn LearnerStage>>,
}

impl LearningPipeline {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }
}

impl Default for LearningPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl LearningPipeline {
    pub fn add_stage(mut self, stage: Box<dyn LearnerStage>) -> Self {
        self.stages.push(stage);
        self
    }

    pub fn process_event(&self, event: &CommitEvent) {
        for stage in &self.stages {
            if !stage.process(event) {
                break;
            }
        }
    }
}
