use crate::error::FishReadError;
use crate::storage::{book_repo, settings_repo};

pub struct BookListItem {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub format: String,
    pub current: bool,
    pub imported_at: i64,
}

pub struct BookListResult {
    pub books: Vec<BookListItem>,
}

pub struct PositionInfo {
    pub chapter_index: i64,
    pub chunk_index: i64,
}

pub struct BookUseResult {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub format: String,
    pub position: PositionInfo,
}

pub struct LibraryService<'a> {
    conn: &'a rusqlite::Connection,
}

impl<'a> LibraryService<'a> {
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    pub fn list(&self) -> Result<BookListResult, FishReadError> {
        let rows = book_repo::list_all(self.conn)
            .map_err(|e| FishReadError::Database(e.to_string()))?;

        let current_id = settings_repo::get_current_book_id(self.conn)
            .map_err(|e| FishReadError::Database(e.to_string()))?;

        let books = rows
            .into_iter()
            .map(|r| BookListItem {
                current: current_id.as_deref() == Some(&r.id),
                id: r.id,
                title: r.title,
                author: r.author,
                format: r.format,
                imported_at: r.imported_at,
            })
            .collect();

        Ok(BookListResult { books })
    }

    pub fn use_book(&self, book_id: &str) -> Result<BookUseResult, FishReadError> {
        let row = book_repo::find_by_id(self.conn, book_id)
            .map_err(|e| FishReadError::Database(e.to_string()))?
            .ok_or_else(|| FishReadError::BookNotFound(book_id.to_owned()))?;

        settings_repo::set_current_book_id(self.conn, book_id)
            .map_err(|e| FishReadError::Database(e.to_string()))?;

        book_repo::upsert_reading_position(self.conn, book_id)
            .map_err(|e| FishReadError::Database(e.to_string()))?;

        let (chapter_index, chunk_index) =
            book_repo::get_reading_position(self.conn, book_id)
                .map_err(|e| FishReadError::Database(e.to_string()))?
                .unwrap_or((0, 0));

        Ok(BookUseResult {
            id: row.id,
            title: row.title,
            author: row.author,
            format: row.format,
            position: PositionInfo {
                chapter_index,
                chunk_index,
            },
        })
    }
}
