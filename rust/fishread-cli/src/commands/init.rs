use fishread_core::protocol::{ApiResponse, InitData};
use fishread_core::storage::db::StorageDb;

pub fn run() -> (String, i32) {
    match StorageDb::init() {
        Ok((_db, db_path)) => {
            let data = InitData {
                initialized: true,
                database_path: db_path,
            };
            let json = serde_json::to_string(&ApiResponse::ok(data))
                .expect("serialization is infallible for InitData");
            (json, 0)
        }
        Err(e) => {
            let json = serde_json::to_string(&ApiResponse::<()>::internal_err(e.to_string()))
                .expect("serialization is infallible for error response");
            (json, 3)
        }
    }
}
