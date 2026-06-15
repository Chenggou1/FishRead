use crate::error::FishReadError;
use crate::storage::{book_repo, chapter_repo, settings_repo};

pub struct ChapterListItem {
    pub id: String,
    pub index: i64,
    pub title: String,
    pub current: bool,
}

pub struct ChapterListResult {
    pub book_id: String,
    pub book_title: String,
    pub chapters: Vec<ChapterListItem>,
}

pub struct ChapterService<'a> {
    conn: &'a rusqlite::Connection,
}

impl<'a> ChapterService<'a> {
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    pub fn list(&self) -> Result<ChapterListResult, FishReadError> {
        let book_id = settings_repo::get_current_book_id(self.conn)
            .map_err(|e| FishReadError::Database(e.to_string()))?
            .ok_or(FishReadError::NoCurrentBook)?;

        let book = book_repo::find_by_id(self.conn, &book_id)
            .map_err(|e| FishReadError::Database(e.to_string()))?
            .ok_or_else(|| FishReadError::BookNotFound(book_id.clone()))?;

        let position = book_repo::get_reading_position(self.conn, &book_id)
            .map_err(|e| FishReadError::Database(e.to_string()))?
            .unwrap_or((0, 0));
        let current_chapter_index = position.0;

        let metas = chapter_repo::list_meta_by_book(self.conn, &book_id)
            .map_err(|e| FishReadError::Database(e.to_string()))?;

        let chapters = metas
            .into_iter()
            .map(|m| ChapterListItem {
                current: m.index.0 == current_chapter_index,
                id: m.id.0,
                index: m.index.0,
                title: m.title,
            })
            .collect();

        Ok(ChapterListResult {
            book_id: book.id,
            book_title: book.title,
            chapters,
        })
    }
}
