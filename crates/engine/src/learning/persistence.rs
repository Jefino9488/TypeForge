use rusqlite::Connection;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Mutex, RwLock};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct NGramKey {
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct NGramEntry {
    pub prediction: String,
    pub frequency: u32,
}

pub enum DbUpdate {
    Word {
        word: String,
        context: Option<String>,
        amount: i64,
    },
    NGram {
        context: String,
        prediction: String,
        amount: u32,
    },
}

pub struct LearningDb {
    update_tx: std::sync::mpsc::Sender<DbUpdate>,
    user_words_cache: RwLock<HashMap<String, i64>>,
    context_words_cache: RwLock<HashMap<(String, String), i64>>,
    pub ngram_cache: RwLock<HashMap<NGramKey, Vec<NGramEntry>>>,
}

impl LearningDb {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let conn = Connection::open(path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS user_words (
                word TEXT PRIMARY KEY,
                frequency INTEGER NOT NULL DEFAULT 0,
                first_seen INTEGER NOT NULL DEFAULT 0,
                last_used INTEGER NOT NULL DEFAULT 0,
                confidence REAL NOT NULL DEFAULT 0.0
            )",
            [],
        )?;

        // Ensure columns exist for older DBs
        let _ = conn.execute(
            "ALTER TABLE user_words ADD COLUMN first_seen INTEGER NOT NULL DEFAULT 0",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE user_words ADD COLUMN last_used INTEGER NOT NULL DEFAULT 0",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE user_words ADD COLUMN confidence REAL NOT NULL DEFAULT 0.0",
            [],
        );

        conn.execute(
            "CREATE TABLE IF NOT EXISTS context_frequencies (
                word TEXT NOT NULL,
                context TEXT NOT NULL,
                frequency INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (word, context)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS ngrams (
                context TEXT NOT NULL,
                prediction TEXT NOT NULL,
                order_num INTEGER NOT NULL DEFAULT 2,
                frequency INTEGER NOT NULL DEFAULT 1,
                last_updated INTEGER NOT NULL,
                PRIMARY KEY (context, prediction)
            )",
            [],
        )?;

        // Load all data into in-memory caches to avoid SQLite on the hot path
        let mut user_words_cache = HashMap::new();
        {
            let mut stmt = conn.prepare("SELECT word, frequency FROM user_words")?;
            let mut rows = stmt.query([])?;
            while let Some(row) = rows.next()? {
                let word: String = row.get(0)?;
                let freq: i64 = row.get(1)?;
                user_words_cache.insert(word, freq);
            }
        }

        let mut context_words_cache = HashMap::new();
        {
            let mut stmt =
                conn.prepare("SELECT word, context, frequency FROM context_frequencies")?;
            let mut rows = stmt.query([])?;
            while let Some(row) = rows.next()? {
                let word: String = row.get(0)?;
                let ctx: String = row.get(1)?;
                let freq: i64 = row.get(2)?;
                context_words_cache.insert((word, ctx), freq);
            }
        }

        // Populate in-memory NGram cache
        let mut ngram_cache = HashMap::new();
        {
            let mut stmt = conn.prepare(
                "SELECT context, prediction, frequency FROM ngrams ORDER BY frequency DESC",
            )?;
            let rows = stmt.query_map([], |row| {
                let context: String = row.get(0)?;
                let prediction: String = row.get(1)?;
                let frequency: u32 = row.get(2)?;
                Ok((context, prediction, frequency))
            })?;

            for row in rows.flatten() {
                let key = NGramKey { context: row.0 };
                ngram_cache
                    .entry(key)
                    .or_insert_with(Vec::new)
                    .push(NGramEntry {
                        prediction: row.1,
                        frequency: row.2,
                    });
            }
        }

        let conn_arc = std::sync::Arc::new(Mutex::new(conn));
        let (update_tx, update_rx) = std::sync::mpsc::channel::<DbUpdate>();

        let bg_conn = std::sync::Arc::clone(&conn_arc);
        std::thread::spawn(move || {
            let mut batch = Vec::new();
            let batch_size = 50;
            let timeout = std::time::Duration::from_secs(2);

            loop {
                match update_rx.recv_timeout(timeout) {
                    Ok(update) => {
                        batch.push(update);
                        if batch.len() >= batch_size {
                            Self::flush_batch(&bg_conn, &mut batch);
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        if !batch.is_empty() {
                            Self::flush_batch(&bg_conn, &mut batch);
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        if !batch.is_empty() {
                            Self::flush_batch(&bg_conn, &mut batch);
                        }
                        break;
                    }
                }
            }
        });

        Ok(Self {
            update_tx,
            user_words_cache: RwLock::new(user_words_cache),
            context_words_cache: RwLock::new(context_words_cache),
            ngram_cache: RwLock::new(ngram_cache),
        })
    }

    fn flush_batch(conn: &std::sync::Arc<Mutex<Connection>>, batch: &mut Vec<DbUpdate>) {
        let mut conn = conn.lock().unwrap();
        let tx = conn.transaction();
        if let Ok(tx) = tx {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            
            for update in batch.drain(..) {
                match update {
                    DbUpdate::Word { word, context, amount } => {
                        let _ = tx.execute(
                            "INSERT INTO user_words (word, frequency, first_seen, last_used, confidence) VALUES (?1, ?2, ?3, ?3, 1.0)
                             ON CONFLICT(word) DO UPDATE SET 
                                frequency = frequency + ?2,
                                last_used = ?3,
                                confidence = MIN(10.0, confidence + 0.1)",
                            rusqlite::params![word, amount, now],
                        );
                        if let Some(ctx) = context {
                            let _ = tx.execute(
                                "INSERT INTO context_frequencies (word, context, frequency) VALUES (?1, ?2, ?3)
                                 ON CONFLICT(word, context) DO UPDATE SET frequency = frequency + ?3",
                                rusqlite::params![word, ctx, amount],
                            );
                        }
                    }
                    DbUpdate::NGram { context, prediction, amount } => {
                        let _ = tx.execute(
                            "INSERT INTO ngrams (context, prediction, order_num, frequency, last_updated)
                             VALUES (?1, ?2, 2, ?3, ?4)
                             ON CONFLICT(context, prediction) DO UPDATE SET 
                             frequency = frequency + ?3,
                             last_updated = ?4",
                            rusqlite::params![context, prediction, amount, now],
                        );
                    }
                }
            }
            let _ = tx.commit();
        }
    }

    pub fn increase_weight(
        &self,
        word: &str,
        context: Option<&str>,
        amount: i64,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Send to background thread for batch writing
        let _ = self.update_tx.send(DbUpdate::Word {
            word: word.to_string(),
            context: context.map(|s| s.to_string()),
            amount,
        });

        // Update in-memory caches immediately for fast inference
        {
            let mut cache = self.user_words_cache.write().unwrap();
            *cache.entry(word.to_string()).or_insert(0) += amount;
        }

        if let Some(ctx) = context {
            let mut cache = self.context_words_cache.write().unwrap();
            *cache
                .entry((word.to_string(), ctx.to_string()))
                .or_insert(0) += amount;
        }

        Ok(())
    }

    pub fn increase_ngram_weight(
        &self,
        context: &str,
        prediction: &str,
        weight: u32,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let _ = self.update_tx.send(DbUpdate::NGram {
            context: context.to_string(),
            prediction: prediction.to_string(),
            amount: weight,
        });

        // Update in-memory cache immediately
        let key = NGramKey {
            context: context.to_string(),
        };
        let mut cache = self.ngram_cache.write().unwrap();
        let entries = cache.entry(key).or_default();

        if let Some(entry) = entries.iter_mut().find(|e| e.prediction == prediction) {
            entry.frequency += weight;
        } else {
            entries.push(NGramEntry {
                prediction: prediction.to_string(),
                frequency: weight,
            });
        }

        // Re-sort the entries by frequency descending
        entries.sort_by_key(|b| std::cmp::Reverse(b.frequency));

        Ok(())
    }

    pub fn get_weight(
        &self,
        word: &str,
        context: Option<&str>,
    ) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let mut total = 0i64;

        if let Some(freq) = self.user_words_cache.read().unwrap().get(word) {
            total += freq;
        }

        if let Some(ctx) = context
            && let Some(freq) = self
                .context_words_cache
                .read()
                .unwrap()
                .get(&(ctx.to_string(), word.to_string()))
        {
            total += freq;
        }

        Ok(total)
    }

    pub fn get_candidates_by_prefix(
        &self,
        prefix: &str,
        limit: usize,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let cache = self.user_words_cache.read().unwrap();

        let mut candidates: Vec<(&String, &i64)> = cache
            .iter()
            .filter(|(w, _)| w.starts_with(prefix))
            .collect();

        candidates.sort_by(|a, b| b.1.cmp(a.1));

        Ok(candidates
            .into_iter()
            .take(limit)
            .map(|(w, _)| w.clone())
            .collect())
    }

    pub fn get_ngrams(&self, context: &str, limit: usize) -> Vec<String> {
        let key = NGramKey {
            context: context.to_string(),
        };
        let cache = self.ngram_cache.read().unwrap();
        if let Some(entries) = cache.get(&key) {
            entries
                .iter()
                .take(limit)
                .map(|e| e.prediction.clone())
                .collect()
        } else {
            Vec::new()
        }
    }
}

pub struct TelemetryDb {
    _conn: Mutex<Connection>,
}

impl TelemetryDb {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let conn = Connection::open(path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS telemetry (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_type TEXT NOT NULL,
                data TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        Ok(Self {
            _conn: Mutex::new(conn),
        })
    }
}
