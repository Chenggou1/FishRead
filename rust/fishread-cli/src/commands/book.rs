use fishread_core::book::LibraryService;
use fishread_core::error::FishReadError;
use fishread_core::protocol::{ApiResponse, BookDeleteDto, BookListDto, BookUseDto};
use fishread_core::storage::db::StorageDb;

pub fn list() -> (String, i32) {
    match do_list() {
        Ok(dto) => (serde_json::to_string(&ApiResponse::ok(dto)).unwrap(), 0),
        Err(e) => (
            serde_json::to_string(&ApiResponse::<()>::err(&e)).unwrap(),
            e.exit_code(),
        ),
    }
}

pub fn use_book(book_id: &str) -> (String, i32) {
    match do_use(book_id) {
        Ok(dto) => (serde_json::to_string(&ApiResponse::ok(dto)).unwrap(), 0),
        Err(e) => (
            serde_json::to_string(&ApiResponse::<()>::err(&e)).unwrap(),
            e.exit_code(),
        ),
    }
}

pub fn delete_book(book_id: &str) -> (String, i32) {
    match do_delete(book_id) {
        Ok(dto) => (serde_json::to_string(&ApiResponse::ok(dto)).unwrap(), 0),
        Err(e) => (
            serde_json::to_string(&ApiResponse::<()>::err(&e)).unwrap(),
            e.exit_code(),
        ),
    }
}

fn do_list() -> Result<BookListDto, FishReadError> {
    let (db, _) = StorageDb::open()?;
    let svc = LibraryService::new(&db.conn);
    let result = svc.list()?;
    Ok(BookListDto::from(result))
}

fn do_use(book_id: &str) -> Result<BookUseDto, FishReadError> {
    let (db, _) = StorageDb::open()?;
    let svc = LibraryService::new(&db.conn);
    let result = svc.use_book(book_id)?;
    Ok(BookUseDto::from(result))
}

fn do_delete(book_id: &str) -> Result<BookDeleteDto, FishReadError> {
    let (db, _) = StorageDb::open()?;
    let svc = LibraryService::new(&db.conn);
    let result = svc.delete_book(book_id)?;
    Ok(BookDeleteDto::from(result))
}
