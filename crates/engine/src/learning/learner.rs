use std::error::Error;
use std::sync::Arc;
use crate::learning::persistence::{LearningDb, TelemetryDb};
use crate::learning::session::SessionMemory;
use crate::learning::scorer::ScorePipeline;

use std::sync::atomic::{AtomicBool, Ordering};

pub struct LearningConfig {
    pub learning_enabled: AtomicBool,
    pub telemetry_enabled: AtomicBool,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            learning_enabled: AtomicBool::new(true),
            telemetry_enabled: AtomicBool::new(true),
        }
    }
}

pub struct Learner {
    pub config: LearningConfig,
    pub learning_db: Arc<LearningDb>,
    pub telemetry_db: Option<Arc<TelemetryDb>>,
    pub session_memory: Arc<SessionMemory>,
    pub pipeline: ScorePipeline,
}

impl Learner {
    pub fn new(
        learning_db: Arc<LearningDb>,
        telemetry_db: Option<Arc<TelemetryDb>>,
        session_memory: Arc<SessionMemory>,
        config: LearningConfig,
    ) -> Self {
        Self {
            config,
            learning_db: learning_db.clone(),
            telemetry_db,
            session_memory: session_memory.clone(),
            pipeline: ScorePipeline::new(learning_db, session_memory),
        }
    }

    /// Process feedback when a user accepts a prediction
    pub fn on_prediction_accepted(&self, word: &str, context: Option<&str>) -> Result<(), Box<dyn Error + Send + Sync>> {
        if !self.config.learning_enabled.load(Ordering::Relaxed) {
            return Ok(());
        }

        // Add to session memory
        self.session_memory.add(word);

        // Positive reinforcement in the learning database
        self.learning_db.increase_weight(word, context, 10)?;

        Ok(())
    }

    /// Process feedback when a user manually types a word (not from prediction)
    pub fn on_word_typed(&self, word: &str, context: Option<&str>, is_common: bool) -> Result<(), Box<dyn Error + Send + Sync>> {
        if !self.config.learning_enabled.load(Ordering::Relaxed) {
            return Ok(());
        }
        
        self.session_memory.add(word);
        
        if is_common {
            // "Common dictionary word typed -> no learning needed."
            return Ok(());
        }
        
        // Smaller positive reinforcement for manual typing
        self.learning_db.increase_weight(word, context, 2)?;
        
        Ok(())
    }

    pub fn get_candidates_by_prefix(&self, prefix: &str, limit: usize) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        self.learning_db.get_candidates_by_prefix(prefix, limit)
    }

    pub fn set_learning_enabled(&self, enabled: bool) {
        self.config.learning_enabled.store(enabled, Ordering::Relaxed);
    }
}
