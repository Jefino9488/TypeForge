use rusqlite::{Connection, Result as SqlResult};
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: &str) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS learned_words (
                word TEXT PRIMARY KEY,
                frequency INTEGER NOT NULL
            )",
            [],
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn load_all(&self) -> SqlResult<Vec<(String, i64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT word, frequency FROM learned_words")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn upsert_word(&self, word: &str, freq: i64) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO learned_words (word, frequency) VALUES (?1, ?2)
             ON CONFLICT(word) DO UPDATE SET frequency = frequency + ?2",
            (word, freq),
        )?;
        Ok(())
    }
}
