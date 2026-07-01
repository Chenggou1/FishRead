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
    assert_eq!(
        json["protocol_version"], 1,
        "expected protocol version 1, got: {json}"
    );
    assert_eq!(json["ok"], true, "expected ok:true, got: {json}");
    json
}

fn run_err(c: &mut Command, expected_exit: i32) -> serde_json::Value {
    let out = c.output().expect("failed to run fishread");
    assert_eq!(
        out.status.code(),
        Some(expected_exit),
        "expected exit {expected_exit}, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("output is not valid JSON");
    assert_eq!(
        json["protocol_version"], 1,
        "expected protocol version 1, got: {json}"
    );
    assert_eq!(json["ok"], false, "expected ok:false, got: {json}");
    json
}

fn count_rows(db_path: &str, sql: &str, value: &str) -> i64 {
    let conn = rusqlite::Connection::open(db_path).unwrap();
    conn.query_row(sql, [value], |row| row.get(0)).unwrap()
}

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/epub/multi-chapter.epub"
);
const SIMPLE_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/epub/simple.epub"
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

    // --- chapter list --navigation ---
    let j = run_ok(cmd(db_path).args(["chapter", "list", "--navigation"]));
    let chapters = j["data"]["chapters"].as_array().unwrap();
    let first_chapter = &chapters[0];
    assert_eq!(first_chapter["current"], true);
    let first_anchors = first_chapter["anchors"].as_array().unwrap();
    assert!(!first_anchors.is_empty());
    assert_eq!(first_anchors[0]["label"], "0%");
    assert_eq!(first_anchors[0]["current"], true);
    assert_eq!(first_anchors[0]["position"]["chapter_index"], 0);
    assert_eq!(first_anchors[0]["position"]["chunk_index"], 0);
    assert_eq!(first_anchors[0]["chunk"]["index"], 0);
    assert!(!first_anchors[0]["preview"].as_str().unwrap().is_empty());
    assert!(
        first_anchors[0]["chunk"]["text"].is_null(),
        "navigation anchors must not include full chunk text"
    );

    let jump_anchor = chapters
        .iter()
        .flat_map(|chapter| chapter["anchors"].as_array().unwrap().iter())
        .find(|anchor| anchor["position"]["chunk_index"].as_i64().unwrap() > 0)
        .unwrap_or(&first_anchors[0]);
    let jump_chapter = jump_anchor["position"]["chapter_index"]
        .as_i64()
        .unwrap()
        .to_string();
    let jump_chunk = jump_anchor["position"]["chunk_index"]
        .as_i64()
        .unwrap()
        .to_string();

    // --- read jump ---
    let j = run_ok(cmd(db_path).args([
        "read",
        "jump",
        "--chapter-index",
        &jump_chapter,
        "--chunk-index",
        &jump_chunk,
    ]));
    assert_eq!(
        j["data"]["progress"]["chapter_index"],
        jump_chapter.parse::<i64>().unwrap()
    );
    assert_eq!(
        j["data"]["progress"]["chunk_index"],
        jump_chunk.parse::<i64>().unwrap()
    );
    assert!(!j["data"]["chunk"]["text"].as_str().unwrap().is_empty());

    // --- read current ---
    let j = run_ok(cmd(db_path).args(["read", "current"]));
    assert!(!j["data"]["chunk"]["text"].as_str().unwrap().is_empty());
    assert_eq!(
        j["data"]["progress"]["chapter_index"].as_i64().unwrap(),
        jump_chapter.parse::<i64>().unwrap()
    );
    assert_eq!(
        j["data"]["progress"]["chunk_index"].as_i64().unwrap(),
        jump_chunk.parse::<i64>().unwrap()
    );
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

    let j = run_err(cmd(db_path).args(["book", "use", "book_nonexistent"]), 1);
    assert_eq!(j["error"]["code"], "BOOK_NOT_FOUND");
}

#[test]
fn read_jump_missing_chunk_returns_chunk_not_found() {
    let db = NamedTempFile::new().unwrap();
    let db_path = db.path().to_str().unwrap();
    run_ok(cmd(db_path).arg("init"));
    run_ok(cmd(db_path).args(["import", FIXTURE]));

    let j = run_err(
        cmd(db_path).args([
            "read",
            "jump",
            "--chapter-index",
            "0",
            "--chunk-index",
            "9999",
        ]),
        1,
    );
    assert_eq!(j["error"]["code"], "CHUNK_NOT_FOUND");
}

#[test]
fn book_delete_clears_current_book_and_dependent_records() {
    let db = NamedTempFile::new().unwrap();
    let db_path = db.path().to_str().unwrap();
    run_ok(cmd(db_path).arg("init"));
    let imported = run_ok(cmd(db_path).args(["import", FIXTURE]));
    let book_id = imported["data"]["book"]["id"].as_str().unwrap().to_owned();

    let j = run_ok(cmd(db_path).args(["book", "delete", &book_id]));
    assert_eq!(j["data"]["deleted"]["id"], book_id.as_str());
    assert_eq!(j["data"]["cleared_current"], true);

    let j = run_ok(cmd(db_path).args(["book", "list"]));
    assert_eq!(j["data"]["books"].as_array().unwrap().len(), 0);

    assert_eq!(
        count_rows(
            db_path,
            "SELECT COUNT(*) FROM books WHERE id = ?1",
            &book_id
        ),
        0
    );
    assert_eq!(
        count_rows(
            db_path,
            "SELECT COUNT(*) FROM chapters WHERE book_id = ?1",
            &book_id
        ),
        0
    );
    assert_eq!(
        count_rows(
            db_path,
            "SELECT COUNT(*) FROM reading_positions WHERE book_id = ?1",
            &book_id
        ),
        0
    );
    assert_eq!(
        count_rows(
            db_path,
            "SELECT COUNT(*) FROM settings WHERE key = 'current_book_id' AND value = ?1",
            &book_id
        ),
        0
    );

    let j = run_err(cmd(db_path).args(["read", "current"]), 1);
    assert_eq!(j["error"]["code"], "NO_CURRENT_BOOK");
}

#[test]
fn book_delete_non_current_keeps_current_book() {
    let db = NamedTempFile::new().unwrap();
    let db_path = db.path().to_str().unwrap();
    run_ok(cmd(db_path).arg("init"));

    let first = run_ok(cmd(db_path).args(["import", FIXTURE]));
    let current_id = first["data"]["book"]["id"].as_str().unwrap().to_owned();
    let second = run_ok(cmd(db_path).args(["import", SIMPLE_FIXTURE]));
    let deleted_id = second["data"]["book"]["id"].as_str().unwrap().to_owned();

    let j = run_ok(cmd(db_path).args(["book", "delete", &deleted_id]));
    assert_eq!(j["data"]["deleted"]["id"], deleted_id.as_str());
    assert_eq!(j["data"]["cleared_current"], false);

    let j = run_ok(cmd(db_path).args(["book", "list"]));
    let books = j["data"]["books"].as_array().unwrap();
    assert_eq!(books.len(), 1);
    assert_eq!(books[0]["id"], current_id.as_str());
    assert_eq!(books[0]["current"], true);

    assert_eq!(
        count_rows(
            db_path,
            "SELECT COUNT(*) FROM chapters WHERE book_id = ?1",
            &deleted_id
        ),
        0
    );

    let j = run_ok(cmd(db_path).args(["read", "current"]));
    assert_eq!(j["data"]["book"]["id"], current_id.as_str());
}

#[test]
fn book_delete_missing_book_returns_not_found() {
    let db = NamedTempFile::new().unwrap();
    let db_path = db.path().to_str().unwrap();
    run_ok(cmd(db_path).arg("init"));

    let j = run_err(cmd(db_path).args(["book", "delete", "book_nonexistent"]), 1);
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
    assert_eq!(j["protocol_version"], 1);
    assert_eq!(j["ok"], false);
    assert_eq!(j["error"]["code"], "DATABASE_NOT_INITIALIZED");
}
