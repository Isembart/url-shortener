use rusqlite::{params, Connection, Result};
use std::sync::Mutex;


use bcrypt::{hash, DEFAULT_COST};


pub struct DbConn {
    pub conn: Mutex<Connection>, // Thread-safe shared connection
}

#[derive(Debug)]
pub enum UserError {
    UserAlreadyExists,
    InvalidCredentials,
    DatabaseError(rusqlite::Error),
}

impl From<rusqlite::Error> for UserError {
    fn from(err: rusqlite::Error) -> Self {
        UserError::DatabaseError(err)
    }
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
        
        //initiate users table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                password TEXT NOT NULL
                )",
                [],
            )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS urls (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER,
                short TEXT UNIQUE NOT NULL,
                long TEXT NOT NULL,
                FOREIGN KEY(user_id) REFERENCES users(id)
            )",
            [],
        )?;


            Ok(())
        }

    pub fn insert_url(&self, short: &str, long: &str, userid: u32) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO urls (short, long, user_id) VALUES (?1, ?2, ?3) 
             ON CONFLICT(short) DO NOTHING",
            params![short, long, userid],
        )?;
        Ok(())
    }

    pub fn get_long_url(&self, short: String) -> Option<String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT long FROM urls WHERE short = ?1").ok()?;
        let mut rows = stmt.query(params![short]).ok()?;

        if let Some(row) = rows.next().ok()? {
            let long_url: String = row.get(0).ok()?;
            Some(long_url)
        } else {
            None
        }
    }


    pub fn create_user(&self, username: &str, password: &str) -> Result<String, UserError> {
        let conn = self.conn.lock().unwrap();
        let password = hash(password, DEFAULT_COST).unwrap_or_else(|_| {
            panic!("Failed to hash password")
        });
    
        let affected_rows = conn.execute(
            "INSERT INTO users (username, password) VALUES (?1, ?2) 
             ON CONFLICT(username) DO NOTHING",
            params![username, password],
        )?;
    
        if affected_rows == 0 {
            return Err(UserError::UserAlreadyExists);
        }
    
        Ok("User created".to_string())
    }

    pub fn login(&self, username: &str, password: &str) -> Result<String, UserError> {
        let conn = self.conn.lock().unwrap();
        let db_password: String = conn.query_row(
            "SELECT password FROM users WHERE username = ?1",
            params![username],
            |row| row.get(0),
        ).map_err(|_| UserError::InvalidCredentials)?;

        if bcrypt::verify(password, &db_password).unwrap_or(false) {
            Ok("Login successful".to_string())
        } else {
            Err(UserError::InvalidCredentials)
        }
    }

    pub fn get_user_id(&self, username: &str) -> Result<Option<u32>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id FROM users WHERE username = ?1")?;
        let mut rows = stmt.query(params![username])?;

        if let Some(row) = rows.next()? {
            let user_id: u32 = row.get(0)?;
            Ok(Some(user_id))
        } else {
            Ok(None)
        }
    }

}
