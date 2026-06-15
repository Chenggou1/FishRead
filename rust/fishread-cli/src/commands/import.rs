use std::path::Path;

use fishread_core::error::FishReadError;
use fishread_core::importer::ImportService;
use fishread_core::protocol::{ApiResponse, ImportResultDto};
use fishread_core::storage::db::StorageDb;

pub fn run(path: &str) -> (String, i32) {
    match do_import(path) {
        Ok(dto) => {
            let json =
                serde_json::to_string(&ApiResponse::ok(dto)).expect("ImportResultDto is Serialize");
            (json, 0)
        }
        Err(e) => {
            let exit_code = e.exit_code();
            let json = serde_json::to_string(&ApiResponse::<()>::err(&e))
                .expect("error response is Serialize");
            (json, exit_code)
        }
    }
}

fn do_import(path_str: &str) -> Result<ImportResultDto, FishReadError> {
    let (db, _) = StorageDb::open()?;
    let service = ImportService::new(&db.conn);
    let result = service.import(Path::new(path_str))?;
    Ok(ImportResultDto::from(result))
}
