use periphos_core::CoreResult;
use rusqlite::Connection;

use crate::to_core;

/// Ordered list of schema migrations. Index `i` upgrades the schema from
/// version `i` to `i + 1`. Append only.
const MIGRATIONS: &[&str] = &[
    // v1: profiles and key/value settings.
    r#"
    CREATE TABLE profiles (
        id          TEXT PRIMARY KEY NOT NULL,
        name        TEXT NOT NULL COLLATE NOCASE UNIQUE,
        description TEXT,
        is_default  INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
        created_at  INTEGER NOT NULL,
        updated_at  INTEGER NOT NULL
    );
    -- At most one default profile.
    CREATE UNIQUE INDEX profiles_single_default ON profiles (is_default) WHERE is_default = 1;

    CREATE TABLE settings (
        key   TEXT PRIMARY KEY NOT NULL,
        value TEXT NOT NULL
    );
    "#,
    // v2: device settings stored per profile.
    r#"
    CREATE TABLE profile_device_settings (
        profile_id     TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
        device_id      TEXT NOT NULL,
        device_name    TEXT NOT NULL,
        dpi            INTEGER,
        report_rate_hz INTEGER,
        updated_at     INTEGER NOT NULL,
        PRIMARY KEY (profile_id, device_id)
    );
    "#,
];

pub(crate) fn run(conn: &mut Connection) -> CoreResult<()> {
    let current: usize = conn
        .pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))
        .map_err(to_core)? as usize;

    if current > MIGRATIONS.len() {
        return Err(periphos_core::CoreError::Storage(format!(
            "database schema v{current} is newer than this version of PeriphOS supports (v{}). \
             Please update PeriphOS.",
            MIGRATIONS.len()
        )));
    }

    for (index, sql) in MIGRATIONS.iter().enumerate().skip(current) {
        let tx = conn.transaction().map_err(to_core)?;
        tx.execute_batch(sql).map_err(to_core)?;
        tx.pragma_update(None, "user_version", (index + 1) as i64)
            .map_err(to_core)?;
        tx.commit().map_err(to_core)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_idempotent() {
        let mut conn = Connection::open_in_memory().unwrap();
        run(&mut conn).unwrap();
        run(&mut conn).unwrap();
        let v: i64 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(v as usize, MIGRATIONS.len());
    }

    #[test]
    fn refuses_newer_schema() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "user_version", 999).unwrap();
        assert!(run(&mut conn).is_err());
    }
}
