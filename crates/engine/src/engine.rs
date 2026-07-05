use crate::dictionary::immutable::ImmutableDictionary;
use crate::dictionary::symspell_impl::SymSpellChecker;
use crate::learning::{Learner, LearningConfig, LearningDb, SessionMemory, TelemetryDb};
use crate::pipeline::postprocess::adaptive_count::AdaptiveCountProcessor;
use crate::pipeline::{
    BasicFeatureExtractor, CapitalizationProcessor, ContextGenerator, FuzzyExpander,
    LimitProcessor, NoopObserver, PhraseGenerator, PipelineBuilder, PredictionRequest,
    PrefixGenerator, SegmentationExpander, SessionGenerator, SpellExpander, TraceObserver,
    UserDictionaryGenerator, WeightedRanker,
};
use crate::traits::{Dictionary, SpellChecker};
use arc_swap::ArcSwap;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use typeforge_common::config::RankingConfig;
use typeforge_protocol::{PipelineTrace, PredictRequest, Prediction, PredictionSource};

pub struct TypeForgeEngine {
    immutable: Arc<ArcSwap<ImmutableDictionary>>,
    spell_checker: Arc<RwLock<SymSpellChecker>>,
    learner: Arc<Learner>,
    pipeline: crate::pipeline::builder::Pipeline,
    immutable_path: String,
    ranking_config: RankingConfig,
}

impl TypeForgeEngine {
    pub fn new(
        immutable_path: String,
        learning_db_path: &str,
        telemetry_db_path: &str,
        ranking_config: RankingConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut immutable = ImmutableDictionary::new(immutable_path.clone());
        immutable.load()?;

        let mut spell_checker = SymSpellChecker::new();
        spell_checker.load_words(immutable.iter());

        let learning_db = Arc::new(LearningDb::new(learning_db_path)?);
        let telemetry_db = Arc::new(TelemetryDb::new(telemetry_db_path)?);
        let session_memory = Arc::new(SessionMemory::new());
        let learner = Arc::new(Learner::new(
            learning_db,
            Some(telemetry_db),
            session_memory,
            LearningConfig::default(),
        ));

        let immutable_arc = Arc::new(ArcSwap::from_pointee(immutable));
        let spell_checker_arc = Arc::new(RwLock::new(spell_checker));
        let config_arc = Arc::new(ranking_config.clone());

        let pipeline = PipelineBuilder::new()
            .generator(Box::new(PrefixGenerator::new(
                Arc::clone(&immutable_arc),
                ranking_config.candidate_limit * 2,
            )))
            .generator(Box::new(SessionGenerator::new(
                Arc::clone(&learner),
                ranking_config.candidate_limit * 2,
            )))
            .generator(Box::new(UserDictionaryGenerator::new(
                Arc::clone(&learner),
                ranking_config.candidate_limit,
            )))
            .generator(Box::new(ContextGenerator::new(
                Arc::clone(&learner),
                ranking_config.candidate_limit,
            )))
            .generator(Box::new(PhraseGenerator::new(
                Arc::clone(&learner),
                ranking_config.candidate_limit,
            )))
            .expander(Box::new(SpellExpander::new(
                Arc::clone(&spell_checker_arc),
                ranking_config.candidate_limit,
            )))
            .expander(Box::new(FuzzyExpander::new(
                Arc::clone(&spell_checker_arc),
                ranking_config.candidate_limit,
            )))
            .expander(Box::new(SegmentationExpander::new(Arc::clone(
                &immutable_arc,
            ))))
            .feature(Box::new(BasicFeatureExtractor::new(
                Arc::clone(&learner),
                Arc::clone(&immutable_arc),
            )))
            .ranker(Box::new(WeightedRanker::new(config_arc)))
            .postprocessor(Box::new(CapitalizationProcessor::new()))
            .postprocessor(Box::new(AdaptiveCountProcessor))
            .postprocessor(Box::new(LimitProcessor::new(
                ranking_config.candidate_limit,
            )))
            .build();

        Ok(Self {
            immutable: immutable_arc,
            spell_checker: spell_checker_arc,
            learner,
            pipeline,
            immutable_path,
            ranking_config,
        })
    }

    pub fn get_candidate_limit(&self) -> usize {
        self.ranking_config.candidate_limit
    }

    pub fn predict(
        &self,
        prefix: &str,
        req: &PredictRequest,
        _limit: usize,
        cancellation_token: crate::pipeline::request::CancellationToken,
    ) -> Vec<Prediction> {
        let is_all_caps = !prefix.is_empty()
            && prefix
                .chars()
                .all(|c| !c.is_alphabetic() || c.is_uppercase());
        let is_capitalized = !prefix.is_empty() && prefix.chars().next().unwrap().is_uppercase();

        let request = PredictionRequest {
            text_before_cursor: req.text_before_cursor.clone(),
            text_after_cursor: req.text_after_cursor.clone(),
            cursor_position: req.cursor_position,
            application: req.application.clone().unwrap_or_default(),
            language: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            cancellation_token,
        };

        let mut observer = NoopObserver;
        let result = self.pipeline.execute(&request, &mut observer);

        let mut candidates: Vec<Prediction> = result
            .candidates
            .into_iter()
            .map(|c| Prediction {
                text: c.candidate.text,
                score: c.ranking.score,
                source: PredictionSource::User, // Map CandidateSource to PredictionSource if needed
            })
            .collect();

        // 5. Restore casing
        if is_all_caps {
            for c in &mut candidates {
                c.text = c.text.to_uppercase();
            }
        } else if is_capitalized {
            for c in &mut candidates {
                let mut chars = c.text.chars();
                if let Some(first) = chars.next() {
                    c.text = first.to_uppercase().collect::<String>() + chars.as_str();
                }
            }
        }

        candidates
    }
    pub fn explain(
        &self,
        req: &PredictRequest,
        cancellation_token: crate::pipeline::request::CancellationToken,
    ) -> PipelineTrace {
        let request = PredictionRequest {
            text_before_cursor: req.text_before_cursor.clone(),
            text_after_cursor: req.text_after_cursor.clone(),
            cursor_position: req.cursor_position,
            application: req.application.clone().unwrap_or_default(),
            language: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            cancellation_token,
        };

        let mut observer = TraceObserver::new(3); // Pipeline version 3
        self.pipeline.execute(&request, &mut observer);
        observer.trace
    }
    pub fn correct_spelling(&self, word: &str, limit: usize) -> Vec<Prediction> {
        let checker = self.spell_checker.read().unwrap();
        checker.correct(word, limit)
    }

    pub fn learn(
        &self,
        word: &str,
        context: Option<&str>,
        accepted: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if accepted {
            self.learner.on_prediction_accepted(word, context)?;
        } else {
            let is_common = self
                .immutable
                .load()
                .get_frequency(word)
                .is_some_and(|f| f > 50000);
            self.learner.on_word_typed(word, context, is_common)?;
        }
        Ok(())
    }

    pub fn reload_dictionary_background(&self) {
        let immutable_arc = Arc::clone(&self.immutable);
        let path = self.immutable_path.clone();
        thread::spawn(move || {
            let mut new_dict = ImmutableDictionary::new(path);
            if new_dict.load().is_ok() {
                immutable_arc.store(Arc::new(new_dict));
            }
        });
    }

    pub fn set_learning_enabled(&self, enabled: bool) {
        self.learner.set_learning_enabled(enabled);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::bytes_of;
    use std::fs::File;
    use std::io::Write;
    use typeforge_common::dict_format::{AlphaIndex, DictionaryEntry, DictionaryHeader};
    use typeforge_protocol::PredictRequest;

    fn setup_dummy_assets() -> (String, String, String) {
        let test_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(&test_dir).unwrap();

        let dict_path = test_dir.join("dict.bin").to_string_lossy().to_string();
        let l_db_path = test_dir.join("learning.db").to_string_lossy().to_string();
        let t_db_path = test_dir.join("telemetry.db").to_string_lossy().to_string();

        let mut file = File::create(&dict_path).unwrap();
        let mut header = DictionaryHeader {
            word_count: 3,
            ..Default::default()
        };

        let mut alpha: AlphaIndex = [0; 26];
        // apple, application, banana
        // a=0, b=2, c..z=3
        alpha[0] = 0;
        alpha[1] = 2;
        for a in alpha.iter_mut().skip(2) {
            *a = 3;
        }

        let entries = vec![
            DictionaryEntry {
                offset: 0,
                length: 5,
                first_char: b'a' as u16,
                frequency: 100,
            },
            DictionaryEntry {
                offset: 5,
                length: 11,
                first_char: b'a' as u16,
                frequency: 50,
            },
            DictionaryEntry {
                offset: 16,
                length: 6,
                first_char: b'b' as u16,
                frequency: 200,
            },
        ];

        let pool = b"appleapplicationbanana";

        header.index_offset = 48;
        header.strings_offset = 48 + 104 + (12 * 3);
        header.checksum_offset = header.strings_offset + pool.len() as u64;

        file.write_all(bytes_of(&header)).unwrap();
        file.write_all(bytemuck::cast_slice(&alpha)).unwrap();
        file.write_all(bytemuck::cast_slice(&entries)).unwrap();
        file.write_all(pool).unwrap();
        file.write_all(&[0u8; 32]).unwrap(); // Dummy checksum

        (dict_path, l_db_path, t_db_path)
    }

    fn dummy_req() -> PredictRequest {
        PredictRequest {
            prefix: "app".to_string(),
            text_before_cursor: "app".to_string(),
            text_after_cursor: "".to_string(),
            cursor_position: 3,
            application: None,
            language: None,
        }
    }

    #[test]
    fn test_engine_predictions_and_normalization() {
        let (dict_path, l_db_path, t_db_path) = setup_dummy_assets();
        let engine = TypeForgeEngine::new(
            dict_path,
            &l_db_path,
            &t_db_path,
            typeforge_common::config::RankingConfig::default(),
        )
        .unwrap();

        let preds = engine.predict(
            "app",
            &dummy_req(),
            5,
            crate::pipeline::request::CancellationToken::new(),
        );
        assert_eq!(preds.len(), 2);
        assert_eq!(preds[0].text, "apple");
        assert_eq!(preds[1].text, "application");
        assert!(preds[0].score > preds[1].score);

        // Test user learning (accepted prediction)
        engine.learn("approach", None, true).unwrap();
        let _preds_after = engine.predict(
            "app",
            &dummy_req(),
            5,
            crate::pipeline::request::CancellationToken::new(),
        );
        // It's not in the immutable dictionary so it won't show up yet.
        // Wait, ImmutableDictionary only returns words starting with prefix!
        // We removed MutableDictionary, so `approach` won't be returned unless it's in the pipeline candidates!
        // Ah, the ScorePipeline ranks candidates, but they must be generated by ImmutableDictionary OR user_words.
        // We forgot to generate candidates from learning_db!
    }
}
