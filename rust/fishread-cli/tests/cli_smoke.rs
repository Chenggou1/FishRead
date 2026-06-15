use std::process::Command;
use tempfile::NamedTempFile;

fn cmd(db_path: &str) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_fishread"));
    c.env("FISHREAD_DB_PATH", db_path);
    c
}

fn run_ok(c: &mut Command) -> serde_json::Value {
    let out = c.output().expect("failed to run fishread");
    assert_eq!(
        out.status.code(),
        Some(0),
        "expected exit 0, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("output is not valid JSON");
    assert_eq!(json["ok"], true, "expected ok:true, got: {json}");
    json
}

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/epub/multi-chapter.epub"
);

#[test]
fn smoke_full_workflow() {
    let db = NamedTempFile::new().unwrap();
    let db_path = db.path().to_str().unwrap();

    // --- init ---
    let j = run_ok(cmd(db_path).arg("init"));
    assert!(!j["data"]["database_path"].as_str().unwrap().is_empty());

    // --- import ---
    let j = run_ok(cmd(db_path).args(["import", FIXTURE]));
    assert!(j["data"]["chapters_count"].as_u64().unwrap() > 0);
    assert_eq!(j["data"]["current"], true);
    let book_id = j["data"]["book"]["id"].as_str().unwrap().to_string();

    // --- book list ---
    let j = run_ok(cmd(db_path).args(["book", "list"]));
    let books = j["data"]["books"].as_array().unwrap();
    assert!(!books.is_empty(), "book list must not be empty");
    assert!(
        books.iter().any(|b| b["current"] == true),
        "at least one book must be current"
    );

    // --- book use ---
    let j = run_ok(cmd(db_path).args(["book", "use", &book_id]));
    assert_eq!(j["data"]["book"]["id"], book_id.as_str());
    assert_eq!(j["data"]["position"]["chapter_index"], 0);
    assert_eq!(j["data"]["position"]["chunk_index"], 0);

    // --- chapter list ---
    let j = run_ok(cmd(db_path).args(["chapter", "list"]));
    let chapters = j["data"]["chapters"].as_array().unwrap();
    assert!(!chapters.is_empty());
    let indices: Vec<i64> = chapters
        .iter()
        .map(|c| c["index"].as_i64().unwrap())
        .collect();
    let mut sorted = indices.clone();
    sorted.sort();
    assert_eq!(indices, sorted, "chapters must be sorted by index");

    // --- read current ---
    let j = run_ok(cmd(db_path).args(["read", "current"]));
    assert!(!j["data"]["chunk"]["text"].as_str().unwrap().is_empty());
    assert_eq!(j["data"]["start_of_book"], true);
    let ch0 = j["data"]["progress"]["chapter_index"].as_i64().unwrap();
    let ck0 = j["data"]["progress"]["chunk_index"].as_i64().unwrap();

    // --- read next ---
    let j = run_ok(cmd(db_path).args(["read", "next"]));
    let ch1 = j["data"]["progress"]["chapter_index"].as_i64().unwrap();
    let ck1 = j["data"]["progress"]["chunk_index"].as_i64().unwrap();
    assert!(
        ch1 > ch0 || ck1 > ck0,
        "next must advance position: ({ch0},{ck0}) -> ({ch1},{ck1})"
    );

    // current should reflect the advanced position
    let j = run_ok(cmd(db_path).args(["read", "current"]));
    assert_eq!(j["data"]["progress"]["chapter_index"], ch1);
    assert_eq!(j["data"]["progress"]["chunk_index"], ck1);

    // --- read prev ---
    let j = run_ok(cmd(db_path).args(["read", "prev"]));
    let ch2 = j["data"]["progress"]["chapter_index"].as_i64().unwrap();
    let ck2 = j["data"]["progress"]["chunk_index"].as_i64().unwrap();
    assert_eq!(
        (ch2, ck2),
        (ch0, ck0),
        "prev must return to position before next"
    );
}

#[test]
fn error_book_not_found() {
    let db = NamedTempFile::new().unwrap();
    let db_path = db.path().to_str().unwrap();
    run_ok(cmd(db_path).arg("init"));
    run_ok(cmd(db_path).args(["import", FIXTURE]));

    let out = cmd(db_path)
        .args(["book", "use", "book_nonexistent"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let j: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(j["ok"], false);
    assert_eq!(j["error"]["code"], "BOOK_NOT_FOUND");
}

#[test]
fn error_database_not_initialized() {
    // Use a path that does not exist — StorageDb::open() checks for file existence
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("nonexistent.db");
    let db_path = db_path.to_str().unwrap();

    let out = cmd(db_path).args(["book", "list"]).output().unwrap();
    assert_eq!(out.status.code(), Some(3));
    let j: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(j["ok"], false);
    assert_eq!(j["error"]["code"], "DATABASE_NOT_INITIALIZED");
}
