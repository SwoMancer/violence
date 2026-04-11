use rusqlite::Connection;
use std::sync::{Mutex, OnceLock};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to execute query: {0}")]
    Rusqlite(#[from] rusqlite::Error),
    #[error("failed to lock mutex: {0}")]
    Mutex(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub struct Alarm {
    pub id: i64,
    pub time: String,
    pub enabled: bool,
    pub recurrence: String,
    pub monday: bool,
    pub tuesday: bool,
    pub wednesday: bool,
    pub thursday: bool,
    pub friday: bool,
    pub saturday: bool,
    pub sunday: bool,
    pub confirm_delay_ms: Option<i64>,
    pub refire_delay_ms: Option<i64>,
}

static DB: OnceLock<Mutex<Connection>> = OnceLock::new();

pub fn get_connection() -> &'static Mutex<Connection> {
    DB.get_or_init(|| {
        #[cfg(target_os = "android")]
        let path = crate::scheduler::android_db_path();
        #[cfg(not(target_os = "android"))]
        let path = std::env::temp_dir().join("alarms.db");

        let conn = Connection::open(&path).unwrap_or_else(|e| {
            log::error!("get_connection: open {:?} failed: {e}, falling back to :memory:", path);
            Connection::open_in_memory().expect("in-memory db failed")
        });
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
                sunday BOOLEAN NOT NULL DEFAULT 0,
                confirm_delay_ms INTEGER,
                refire_delay_ms INTEGER
            )",
            [],
        )
        .unwrap_or_else(|e| { log::error!("get_connection: create table failed: {e}"); 0 });
        // Idempotent migrations for installs that predate the override columns.
        // `ALTER TABLE ADD COLUMN` errors with "duplicate column name" on re-run — fine, ignore.
        conn.execute("ALTER TABLE alarms ADD COLUMN confirm_delay_ms INTEGER", []).ok();
        conn.execute("ALTER TABLE alarms ADD COLUMN refire_delay_ms INTEGER", []).ok();
        Mutex::new(conn)
    })
}

pub fn load_alarms() -> Result<Vec<Alarm>> {
    let conn = get_connection()
        .lock()
        .map_err(|e| Error::Mutex(e.to_string()))?;
    let mut stmt = conn.prepare(
        "SELECT id, time, enabled, recurrence, monday, tuesday, wednesday, thursday, friday, saturday, sunday, confirm_delay_ms, refire_delay_ms FROM alarms",
    )?;
    let alarms_iter = stmt.query_map([], |row| {
        Ok(Alarm {
            id: row.get(0)?,
            time: row.get(1)?,
            enabled: row.get(2)?,
            recurrence: row.get(3)?,
            monday: row.get(4)?,
            tuesday: row.get(5)?,
            wednesday: row.get(6)?,
            thursday: row.get(7)?,
            friday: row.get(8)?,
            saturday: row.get(9)?,
            sunday: row.get(10)?,
            confirm_delay_ms: row.get(11)?,
            refire_delay_ms: row.get(12)?,
        })
    })?;

    let mut alarms = Vec::new();
    for alarm in alarms_iter {
        alarms.push(alarm?);
    }
    Ok(alarms)
}

pub fn get_alarm(id: i64) -> Result<Alarm> {
    let conn = get_connection()
        .lock()
        .map_err(|e| Error::Mutex(e.to_string()))?;
    let alarm = conn.query_row(
        "SELECT id, time, enabled, recurrence, monday, tuesday, wednesday, thursday, friday, saturday, sunday, confirm_delay_ms, refire_delay_ms FROM alarms WHERE id = ?1",
        [id],
        |row| {
            Ok(Alarm {
                id: row.get(0)?,
                time: row.get(1)?,
                enabled: row.get(2)?,
                recurrence: row.get(3)?,
                monday: row.get(4)?,
                tuesday: row.get(5)?,
                wednesday: row.get(6)?,
                thursday: row.get(7)?,
                friday: row.get(8)?,
                saturday: row.get(9)?,
                sunday: row.get(10)?,
                confirm_delay_ms: row.get(11)?,
                refire_delay_ms: row.get(12)?,
            })
        },
    )?;
    Ok(alarm)
}

pub fn delete_alarm(id: i64) -> Result<()> {
    let conn = get_connection()
        .lock()
        .map_err(|e| Error::Mutex(e.to_string()))?;
    conn.execute("DELETE FROM alarms WHERE id = ?1", [id])?;
    Ok(())
}

pub fn set_enabled(id: i64, enabled: bool) -> Result<()> {
    let conn = get_connection()
        .lock()
        .map_err(|e| Error::Mutex(e.to_string()))?;
    conn.execute(
        "UPDATE alarms SET enabled = ?1 WHERE id = ?2",
        rusqlite::params![enabled, id],
    )?;
    Ok(())
}

pub fn insert_alarm(
    time: &str,
    enabled: bool,
    recurrence: &str,
    monday: bool,
    tuesday: bool,
    wednesday: bool,
    thursday: bool,
    friday: bool,
    saturday: bool,
    sunday: bool,
    confirm_delay_ms: Option<i64>,
    refire_delay_ms: Option<i64>,
) -> Result<i64> {
    let conn = get_connection()
        .lock()
        .map_err(|e| Error::Mutex(e.to_string()))?;
    conn.execute(
        "INSERT INTO alarms (time, enabled, recurrence, monday, tuesday, wednesday, thursday, friday, saturday, sunday, confirm_delay_ms, refire_delay_ms) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        rusqlite::params![time, enabled, recurrence, monday, tuesday, wednesday, thursday, friday, saturday, sunday, confirm_delay_ms, refire_delay_ms],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_alarm(
    id: i64,
    time: &str,
    enabled: bool,
    recurrence: &str,
    monday: bool,
    tuesday: bool,
    wednesday: bool,
    thursday: bool,
    friday: bool,
    saturday: bool,
    sunday: bool,
    confirm_delay_ms: Option<i64>,
    refire_delay_ms: Option<i64>,
) -> Result<()> {
    let conn = get_connection()
        .lock()
        .map_err(|e| Error::Mutex(e.to_string()))?;
    conn.execute(
        "UPDATE alarms SET time = ?1, enabled = ?2, recurrence = ?3, monday = ?4, tuesday = ?5, wednesday = ?6, thursday = ?7, friday = ?8, saturday = ?9, sunday = ?10, confirm_delay_ms = ?11, refire_delay_ms = ?12 WHERE id = ?13",
        rusqlite::params![time, enabled, recurrence, monday, tuesday, wednesday, thursday, friday, saturday, sunday, confirm_delay_ms, refire_delay_ms, id],
    )?;
    Ok(())
}

