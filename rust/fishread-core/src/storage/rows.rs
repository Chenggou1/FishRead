//! SQLite boundary types — only used inside the storage module.

pub struct BookRow {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub format: String,
    pub source_path: Option<String>,
    pub imported_at: i64,
    pub updated_at: i64,
}

pub struct ChapterRow {
    pub id: String,
    pub book_id: String,
    pub chapter_index: i64,
    pub title: String,
    pub content: String,
    pub source_path: Option<String>,
    pub created_at: i64,
}

pub struct ReadingPositionRow {
    pub book_id: String,
    pub chapter_index: i64,
    pub chunk_index: i64,
    pub updated_at: i64,
}
