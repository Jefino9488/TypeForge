use rusqlite::Connection;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Mutex, RwLock};

pub struct LearningDb {
    conn: Mutex<Connection>,
    // In-Memory Caches for hot-path zero-SQLite predictions
    user_words_cache: RwLock<HashMap<String, i64>>,
    context_words_cache: RwLock<HashMap<(String, String), i64>>,
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

        Ok(Self {
            conn: Mutex::new(conn),
            user_words_cache: RwLock::new(user_words_cache),
            context_words_cache: RwLock::new(context_words_cache),
        })
    }

    pub fn increase_weight(
        &self,
        word: &str,
        context: Option<&str>,
        amount: i64,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        conn.execute(
            "INSERT INTO user_words (word, frequency, first_seen, last_used, confidence) VALUES (?1, ?2, ?3, ?3, 1.0)
             ON CONFLICT(word) DO UPDATE SET 
                frequency = frequency + ?2,
                last_used = ?3,
                confidence = MIN(10.0, confidence + 0.1)",
            rusqlite::params![word, amount, now],
        )?;

        // Update in-memory caches
        {
            let mut cache = self.user_words_cache.write().unwrap();
            *cache.entry(word.to_string()).or_insert(0) += amount;
        }

        if let Some(ctx) = context {
            conn.execute(
                "INSERT INTO context_frequencies (word, context, frequency) VALUES (?1, ?2, ?3)
                 ON CONFLICT(word, context) DO UPDATE SET frequency = frequency + ?3",
                rusqlite::params![word, ctx, amount],
            )?;
            let mut cache = self.context_words_cache.write().unwrap();
            *cache
                .entry((word.to_string(), ctx.to_string()))
                .or_insert(0) += amount;
        }

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
