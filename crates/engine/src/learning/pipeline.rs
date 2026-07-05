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
    fn process(&self, event: &CommitEvent);
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
    fn process(&self, event: &CommitEvent) {
        let weight = if event.is_accepted_prediction { 10 } else { 2 };

        if event.is_accepted_prediction || !event.is_common_word {
            let _ = self
                .db
                .increase_weight(&event.word, event.context.as_deref(), weight);
        }
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
    fn process(&self, event: &CommitEvent) {
        if let Some(prev) = &event.previous_word {
            // Learn the bigram: prev -> word
            let _ = self.db.increase_ngram_weight(prev, &event.word, 1);
        }
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
    fn process(&self, event: &CommitEvent) {
        self.memory.add(&event.word);
    }
}

pub struct LearningPipeline {
    stages: Vec<Box<dyn LearnerStage>>,
}

impl LearningPipeline {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    pub fn add_stage(mut self, stage: Box<dyn LearnerStage>) -> Self {
        self.stages.push(stage);
        self
    }

    pub fn process_event(&self, event: &CommitEvent) {
        for stage in &self.stages {
            stage.process(event);
        }
    }
}
