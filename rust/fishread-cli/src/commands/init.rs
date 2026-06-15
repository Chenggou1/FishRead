use fishread_core::protocol::{ApiResponse, InitData};

pub fn run() -> (String, i32) {
    let data = InitData {
        initialized: true,
        database_path: placeholder_db_path(),
    };
    let json = serde_json::to_string(&ApiResponse::ok(data))
        .expect("serialization is infallible for InitData");
    (json, 0)
}

fn placeholder_db_path() -> String {
    // Real path will be resolved in M2 when SQLite storage is wired up.
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_owned());
    format!("{home}/Library/Application Support/fishread/fishread.db")
}
