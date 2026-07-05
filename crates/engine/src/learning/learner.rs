use crate::learning::persistence::{LearningDb, TelemetryDb};
use crate::learning::pipeline::{
    CommitEvent, LearningPipeline, NGramLearner, SessionLearner, WordLearner,
};
use crate::learning::scorer::ScorePipeline;
use crate::learning::session::SessionMemory;
use std::error::Error;
use std::sync::Arc;

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
    learning_pipeline: Arc<LearningPipeline>,
}

impl Learner {
    pub fn new(
        learning_db: Arc<LearningDb>,
        telemetry_db: Option<Arc<TelemetryDb>>,
        session_memory: Arc<SessionMemory>,
        config: LearningConfig,
    ) -> Self {
        let learning_pipeline = Arc::new(
            LearningPipeline::new()
                .add_stage(Box::new(WordLearner::new(learning_db.clone())))
                .add_stage(Box::new(NGramLearner::new(learning_db.clone())))
                .add_stage(Box::new(SessionLearner::new(session_memory.clone()))),
        );

        Self {
            config,
            learning_db: learning_db.clone(),
            telemetry_db,
            session_memory: session_memory.clone(),
            pipeline: ScorePipeline::new(learning_db, session_memory),
            learning_pipeline,
        }
    }

    /// Process feedback when a user accepts a prediction
    pub fn on_prediction_accepted(
        &self,
        word: &str,
        context: Option<&str>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if !self.config.learning_enabled.load(Ordering::Relaxed) {
            return Ok(());
        }

        self.learning_pipeline.process_event(&CommitEvent {
            word: word.to_string(),
            previous_word: context.map(|s| s.to_string()),
            context: context.map(|s| s.to_string()),
            is_accepted_prediction: true,
            is_common_word: false,
        });

        Ok(())
    }

    /// Process feedback when a user manually types a word (not from prediction)
    pub fn on_word_typed(
        &self,
        word: &str,
        context: Option<&str>,
        is_common: bool,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if !self.config.learning_enabled.load(Ordering::Relaxed) {
            return Ok(());
        }

        self.learning_pipeline.process_event(&CommitEvent {
            word: word.to_string(),
            previous_word: context.map(|s| s.to_string()), // We use context as previous word for now since Fcitx doesn't send separate prev word.
            context: context.map(|s| s.to_string()),
            is_accepted_prediction: false,
            is_common_word: is_common,
        });

        Ok(())
    }

    pub fn get_candidates_by_prefix(
        &self,
        prefix: &str,
        limit: usize,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        self.learning_db.get_candidates_by_prefix(prefix, limit)
    }

    pub fn get_ngrams(&self, context: &str, limit: usize) -> Vec<String> {
        self.learning_db.get_ngrams(context, limit)
    }

    pub fn set_learning_enabled(&self, enabled: bool) {
        self.config
            .learning_enabled
            .store(enabled, Ordering::Relaxed);
    }
}
