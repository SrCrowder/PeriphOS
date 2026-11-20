//! # periphos-storage
//!
//! Local persistence for PeriphOS, backed by a single SQLite database file.
//!
//! - The schema is versioned through `PRAGMA user_version` and upgraded by
//!   [`migrations`] on open. Migrations are append-only: never edit one that
//!   has been released.
//! - Repositories (`profiles`, `settings`) expose domain types from
//!   `periphos-core`, never raw rows.

mod app_rules;
mod button_defaults;
mod device_settings;
mod migrations;
mod profiles;
mod settings;

use std::path::Path;

use periphos_core::{CoreError, CoreResult};
use rusqlite::Connection;

/// Handle to the PeriphOS database.
///
/// `rusqlite::Connection` is not `Sync`; callers sharing a `Storage` between
/// threads must wrap it in a `Mutex`.
pub struct Storage {
    conn: Connection,
}

impl Storage {
    /// Open (or create) the database at `path` and apply pending migrations.
    pub fn open(path: impl AsRef<Path>) -> CoreResult<Self> {
        let conn = Connection::open(path).map_err(to_core)?;
        Self::init(conn)
    }

    /// In-memory database, used by tests.
    pub fn open_in_memory() -> CoreResult<Self> {
        let conn = Connection::open_in_memory().map_err(to_core)?;
        Self::init(conn)
    }

    fn init(mut conn: Connection) -> CoreResult<Self> {
        conn.pragma_update(None, "foreign_keys", true)
            .map_err(to_core)?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(to_core)?;
        migrations::run(&mut conn)?;
        let storage = Self { conn };
        storage.ensure_default_profile()?;
        Ok(storage)
    }

    /// Current schema version.
    pub fn schema_version(&self) -> CoreResult<u32> {
        self.conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(to_core)
    }
}

pub(crate) fn to_core(err: rusqlite::Error) -> CoreError {
    CoreError::Storage(err.to_string())
}

pub(crate) fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}
