use crate::learning::persistence::LearningDb;
use crate::learning::session::SessionMemory;
use std::sync::Arc;
use typeforge_protocol::Prediction;

pub struct ScoreContext<'a> {
    pub application: Option<&'a str>,
}

pub struct ScorePipeline {
    learning_db: Arc<LearningDb>,
    session_memory: Arc<SessionMemory>,
}

impl ScorePipeline {
    pub fn new(learning_db: Arc<LearningDb>, session_memory: Arc<SessionMemory>) -> Self {
        Self {
            learning_db,
            session_memory,
        }
    }

    pub fn rank(&self, candidates: &mut [Prediction], ctx: &ScoreContext) {
        for candidate in candidates.iter_mut() {
            let mut score = candidate.score;

            // 1. User Learning Database (Positive reinforcement only)
            if let Ok(user_weight) = self
                .learning_db
                .get_weight(&candidate.text, ctx.application)
            {
                score += user_weight as f32;
            }

            // 2. Session Memory (Recency)
            if self.session_memory.contains(&candidate.text) {
                score += 50000.0; // Significant boost for session recency
            }

            candidate.score = score;
        }

        // Sort descending by score
        candidates.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
}
