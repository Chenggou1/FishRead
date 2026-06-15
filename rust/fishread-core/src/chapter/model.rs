use ulid::Ulid;

use crate::book::model::BookId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChapterId(pub String);

impl ChapterId {
    pub fn new() -> Self {
        Self(format!("chapter_{}", Ulid::new()))
    }
}

impl Default for ChapterId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChapterIndex(pub i64);

#[derive(Debug, Clone)]
pub struct Chapter {
    pub id: ChapterId,
    pub book_id: BookId,
    pub index: ChapterIndex,
    pub title: String,
    pub content: String,
    pub source_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ChapterMeta {
    pub id: ChapterId,
    pub book_id: BookId,
    pub index: ChapterIndex,
    pub title: String,
    pub source_path: Option<String>,
}
