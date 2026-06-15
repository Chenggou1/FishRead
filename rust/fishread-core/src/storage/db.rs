use std::path::PathBuf;

use anyhow::Context;
use rusqlite::Connection;

use crate::error::FishReadError;

#[derive(Debug)]
pub struct StorageDb {
    pub conn: Connection,
}

/// Resolves the database file path.
///
/// Priority:
/// 1. `FISHREAD_DB_PATH` environment variable (for testing and custom installs)
/// 2. Platform default data directory:
///    - macOS:   ~/Library/Application Support/fishread/fishread.db
///    - Linux:   ~/.local/share/fishread/fishread.db
///    - Windows: %APPDATA%\fishread\fishread.db
pub fn resolve_db_path() -> anyhow::Result<PathBuf> {
    if let Ok(custom) = std::env::var("FISHREAD_DB_PATH") {
        return Ok(PathBuf::from(custom));
    }
    let base =
        dirs::data_dir().ok_or_else(|| anyhow::anyhow!("cannot determine app data directory"))?;
    Ok(base.join("fishread").join("fishread.db"))
}

impl StorageDb {
    /// Used by `fishread init`: creates the data directory + database file, runs migrations.
    /// Idempotent — safe to call multiple times.
    pub fn init() -> anyhow::Result<(Self, String)> {
        let path = resolve_db_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create data dir: {}", parent.display()))?;
        }
        let conn = Connection::open(&path)
            .with_context(|| format!("failed to open database at {}", path.display()))?;
        super::migrations::run(&conn)?;
        let db_path_str = path.to_string_lossy().into_owned();
        Ok((Self { conn }, db_path_str))
    }

    /// Used by all commands other than `init`: opens an existing database.
    /// Returns `DatabaseNotInitialized` when the file is absent.
    pub fn open() -> Result<(Self, String), FishReadError> {
        let path = resolve_db_path().map_err(|e| FishReadError::Database(e.to_string()))?;
        if !path.exists() {
            return Err(FishReadError::DatabaseNotInitialized);
        }
        let conn = Connection::open(&path)
            .with_context(|| format!("failed to open database at {}", path.display()))
            .map_err(|e| FishReadError::Database(e.to_string()))?;
        let db_path_str = path.to_string_lossy().into_owned();
        Ok((Self { conn }, db_path_str))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // env var tests must not run in parallel — they mutate process-global state.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn env_var_overrides_default_path() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("FISHREAD_DB_PATH", "/tmp/test_fishread.db");
        let result = resolve_db_path().unwrap();
        std::env::remove_var("FISHREAD_DB_PATH");

        assert_eq!(result, PathBuf::from("/tmp/test_fishread.db"));
    }

    #[test]
    fn default_path_ends_with_fishread_db() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var("FISHREAD_DB_PATH");

        // dirs::data_dir() may be None in some CI environments; skip gracefully.
        if dirs::data_dir().is_none() {
            return;
        }
        let path = resolve_db_path().unwrap();
        assert!(
            path.ends_with("fishread/fishread.db"),
            "expected path to end with fishread/fishread.db, got: {}",
            path.display()
        );
    }

    #[test]
    fn init_creates_db_and_tables() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("fishread.db");
        std::env::set_var("FISHREAD_DB_PATH", db_path.to_str().unwrap());

        let (_db, returned_path) = StorageDb::init().unwrap();
        std::env::remove_var("FISHREAD_DB_PATH");

        assert!(db_path.exists(), "database file should be created");
        assert_eq!(returned_path, db_path.to_str().unwrap());

        // Verify all four tables exist.
        let conn = Connection::open(&db_path).unwrap();
        let tables: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
                .unwrap();
            stmt.query_map([], |row| row.get(0))
                .unwrap()
                .map(|r| r.unwrap())
                .collect()
        };
        assert_eq!(
            tables,
            ["books", "chapters", "reading_positions", "settings"]
        );
    }

    #[test]
    fn init_is_idempotent() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("fishread.db");
        std::env::set_var("FISHREAD_DB_PATH", db_path.to_str().unwrap());

        StorageDb::init().unwrap();
        // Second call must not error or wipe existing data.
        let result = StorageDb::init();
        std::env::remove_var("FISHREAD_DB_PATH");

        assert!(result.is_ok(), "second init should succeed: {:?}", result);
    }

    #[test]
    fn open_returns_not_initialized_when_db_absent() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("FISHREAD_DB_PATH", "/tmp/fishread_nonexistent_test.db");
        // Ensure the file doesn't exist.
        let _ = std::fs::remove_file("/tmp/fishread_nonexistent_test.db");

        let result = StorageDb::open();
        std::env::remove_var("FISHREAD_DB_PATH");

        assert!(
            matches!(result, Err(FishReadError::DatabaseNotInitialized)),
            "expected DatabaseNotInitialized, got: {:?}",
            result
        );
    }
}
