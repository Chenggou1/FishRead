use fishread_core::chapter::ChapterService;
use fishread_core::error::FishReadError;
use fishread_core::protocol::{ApiResponse, ChapterListDto};
use fishread_core::storage::db::StorageDb;

pub fn list(navigation: bool) -> (String, i32) {
    match do_list(navigation) {
        Ok(dto) => (serde_json::to_string(&ApiResponse::ok(dto)).unwrap(), 0),
        Err(e) => (
            serde_json::to_string(&ApiResponse::<()>::err(&e)).unwrap(),
            e.exit_code(),
        ),
    }
}

fn do_list(navigation: bool) -> Result<ChapterListDto, FishReadError> {
    let (db, _) = StorageDb::open()?;
    let svc = ChapterService::new(&db.conn);
    let result = svc.list(navigation)?;
    Ok(ChapterListDto::from(result))
}
