use fishread_core::error::FishReadError;
use fishread_core::protocol::{ApiResponse, MigrationRunDto, MigrationStatusDto};
use fishread_core::storage::{db::StorageDb, migrations};

pub fn run() -> (String, i32) {
    match do_run() {
        Ok(dto) => (serde_json::to_string(&ApiResponse::ok(dto)).unwrap(), 0),
        Err(e) => (
            serde_json::to_string(&ApiResponse::<()>::err(&e)).unwrap(),
            e.exit_code(),
        ),
    }
}

pub fn status() -> (String, i32) {
    match do_status() {
        Ok(dto) => (serde_json::to_string(&ApiResponse::ok(dto)).unwrap(), 0),
        Err(e) => (
            serde_json::to_string(&ApiResponse::<()>::err(&e)).unwrap(),
            e.exit_code(),
        ),
    }
}

fn do_run() -> Result<MigrationRunDto, FishReadError> {
    let (_db, database_path, run) =
        StorageDb::migrate().map_err(|e| FishReadError::Database(e.to_string()))?;
    Ok(MigrationRunDto::from_run(database_path, run))
}

fn do_status() -> Result<MigrationStatusDto, FishReadError> {
    let (db, database_path) = StorageDb::open()?;
    let status =
        migrations::status(&db.conn).map_err(|e| FishReadError::Database(e.to_string()))?;
    Ok(MigrationStatusDto::from_status(database_path, status))
}
