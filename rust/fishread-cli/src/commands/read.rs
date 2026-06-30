use fishread_core::error::FishReadError;
use fishread_core::protocol::{ApiResponse, ReaderStateDto};
use fishread_core::reader::service::ReaderState;
use fishread_core::reader::ReaderService;
use fishread_core::storage::db::StorageDb;

pub fn current() -> (String, i32) {
    run(|svc| svc.current())
}

pub fn next() -> (String, i32) {
    run(|svc| svc.next())
}

pub fn prev() -> (String, i32) {
    run(|svc| svc.prev())
}

fn run<F>(f: F) -> (String, i32)
where
    F: FnOnce(&ReaderService) -> Result<ReaderState, FishReadError>,
{
    match do_read(f) {
        Ok(dto) => (serde_json::to_string(&ApiResponse::ok(dto)).unwrap(), 0),
        Err(e) => (
            serde_json::to_string(&ApiResponse::<()>::err(&e)).unwrap(),
            e.exit_code(),
        ),
    }
}

fn do_read<F>(f: F) -> Result<ReaderStateDto, FishReadError>
where
    F: FnOnce(&ReaderService) -> Result<ReaderState, FishReadError>,
{
    let (db, _) = StorageDb::open()?;
    let svc = ReaderService::new(&db.conn);
    Ok(ReaderStateDto::from(f(&svc)?))
}
