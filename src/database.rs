use lazy_static::lazy_static;
use rusqlite::Connection;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to execute query: {0}")]
    Rusqlite(#[from] rusqlite::Error),
    #[error("failed to lock mutex: {0}")]
    Mutex(String),
}

pub type Result<T> = std::result::Result<T, Error>;

lazy_static! {
    static ref DB_CONNECTION: Mutex<Connection> = {
        let path = std::env::temp_dir().join("alarms.db");
        let conn = Connection::open(&path).expect("failed to open database connection");
        conn.execute(
            "CREATE TABLE IF NOT EXISTS alarms (
                id INTEGER PRIMARY KEY,
                time TEXT NOT NULL,
                enabled BOOLEAN NOT NULL,
                recurrence TEXT NOT NULL DEFAULT 'one-off',
                monday BOOLEAN NOT NULL DEFAULT 0,
                tuesday BOOLEAN NOT NULL DEFAULT 0,
                wednesday BOOLEAN NOT NULL DEFAULT 0,
                thursday BOOLEAN NOT NULL DEFAULT 0,
                friday BOOLEAN NOT NULL DEFAULT 0,
                saturday BOOLEAN NOT NULL DEFAULT 0,
                sunday BOOLEAN NOT NULL DEFAULT 0
            )",
            [],
        )
        .expect("failed to create alarms table");
        Mutex::new(conn)
    };
}

pub fn get_connection() -> &'static Mutex<Connection> {
    &DB_CONNECTION
}