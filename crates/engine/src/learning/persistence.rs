use rusqlite::Connection;
use std::error::Error;
use std::sync::Mutex;

pub struct LearningDb {
    conn: Mutex<Connection>,
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
        let _ = conn.execute("ALTER TABLE user_words ADD COLUMN first_seen INTEGER NOT NULL DEFAULT 0", []);
        let _ = conn.execute("ALTER TABLE user_words ADD COLUMN last_used INTEGER NOT NULL DEFAULT 0", []);
        let _ = conn.execute("ALTER TABLE user_words ADD COLUMN confidence REAL NOT NULL DEFAULT 0.0", []);

        conn.execute(
            "CREATE TABLE IF NOT EXISTS context_frequencies (
                word TEXT NOT NULL,
                context TEXT NOT NULL,
                frequency INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (word, context)
            )",
            [],
        )?;

        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn increase_weight(&self, word: &str, context: Option<&str>, amount: i64) -> Result<(), Box<dyn Error + Send + Sync>> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;
        
        conn.execute(
            "INSERT INTO user_words (word, frequency, first_seen, last_used, confidence) VALUES (?1, ?2, ?3, ?3, 1.0)
             ON CONFLICT(word) DO UPDATE SET 
                frequency = frequency + ?2,
                last_used = ?3,
                confidence = MIN(10.0, confidence + 0.1)",
            rusqlite::params![word, amount, now],
        )?;

        if let Some(ctx) = context {
            conn.execute(
                "INSERT INTO context_frequencies (word, context, frequency) VALUES (?1, ?2, ?3)
                 ON CONFLICT(word, context) DO UPDATE SET frequency = frequency + ?3",
                rusqlite::params![word, ctx, amount],
            )?;
        }

        Ok(())
    }

    pub fn get_weight(&self, word: &str, context: Option<&str>) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let conn = self.conn.lock().unwrap();
        
        let mut total = 0i64;
        
        if let Ok(freq) = conn.query_row(
            "SELECT frequency FROM user_words WHERE word = ?1",
            rusqlite::params![word],
            |row| row.get::<usize, i64>(0),
        ) {
            total += freq;
        }

        if let Some(ctx) = context {
            if let Ok(ctx_freq) = conn.query_row(
                "SELECT frequency FROM context_frequencies WHERE word = ?1 AND context = ?2",
                rusqlite::params![word, ctx],
                |row| row.get::<usize, i64>(0),
            ) {
                total += ctx_freq;
            }
        }

        Ok(total)
    }

    pub fn get_candidates_by_prefix(&self, prefix: &str, limit: usize) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT word FROM user_words WHERE word LIKE ?1 ORDER BY frequency DESC LIMIT ?2")?;
        let prefix_like = format!("{}%", prefix);
        
        let mut rows = stmt.query(rusqlite::params![prefix_like, limit])?;
        let mut candidates = Vec::new();
        while let Some(row) = rows.next()? {
            candidates.push(row.get(0)?);
        }
        
        Ok(candidates)
    }
}

pub struct TelemetryDb {
    conn: Mutex<Connection>,
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
        
        Ok(Self { conn: Mutex::new(conn) })
    }
}
