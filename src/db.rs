use rusqlite::{params, Connection, Result};
use std::sync::Mutex;

pub struct DbConn {
    pub conn: Mutex<Connection>, // Thread-safe shared connection
}

impl DbConn {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn init_db(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS urls (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                short TEXT UNIQUE NOT NULL,
                long TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn insert_url(&self, short: &str, long: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO urls (short, long) VALUES (?1, ?2) 
             ON CONFLICT(short) DO NOTHING",
            params![short, long],
        )?;
        Ok(())
    }

    pub fn get_long_url(&self, short: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT long FROM urls WHERE short = ?1")?;
        let mut rows = stmt.query(params![short])?;

        if let Some(row) = rows.next()? {
            let long_url: String = row.get(0)?;
            Ok(Some(long_url))
        } else {
            Ok(None)
        }
    }
}
