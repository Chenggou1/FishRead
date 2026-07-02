use crate::error::FishReadError;
use crate::reader::chunk;
use crate::storage::{book_repo, chapter_repo, settings_repo};

const ANCHOR_CANDIDATES: &[(f64, &str)] = &[
    (0.0, "0%"),
    (25.0, "25%"),
    (50.0, "50%"),
    (75.0, "75%"),
    (90.0, "90%"),
];

pub struct BookListItem {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub format: String,
    pub current: bool,
    pub imported_at: i64,
    pub position: PositionInfo,
    pub reading_anchor_label: String,
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

pub struct BookDeleteResult {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub format: String,
    pub cleared_current: bool,
}

pub struct LibraryService<'a> {
    conn: &'a rusqlite::Connection,
}

impl<'a> LibraryService<'a> {
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    pub fn list(&self) -> Result<BookListResult, FishReadError> {
        let rows =
            book_repo::list_all(self.conn).map_err(|e| FishReadError::Database(e.to_string()))?;

        let current_id = settings_repo::get_current_book_id(self.conn)
            .map_err(|e| FishReadError::Database(e.to_string()))?;

        let mut books = Vec::with_capacity(rows.len());
        for r in rows {
            let (chapter_index, chunk_index) = book_repo::get_reading_position(self.conn, &r.id)
                .map_err(|e| FishReadError::Database(e.to_string()))?
                .unwrap_or((0, 0));
            let reading_anchor_label =
                self.reading_anchor_label(&r.id, chapter_index, chunk_index)?;

            books.push(BookListItem {
                current: current_id.as_deref() == Some(&r.id),
                id: r.id,
                title: r.title,
                author: r.author,
                format: r.format,
                imported_at: r.imported_at,
                position: PositionInfo {
                    chapter_index,
                    chunk_index,
                },
                reading_anchor_label,
            });
        }

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

        let (chapter_index, chunk_index) = book_repo::get_reading_position(self.conn, book_id)
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

    pub fn delete_book(&self, book_id: &str) -> Result<BookDeleteResult, FishReadError> {
        let row = book_repo::find_by_id(self.conn, book_id)
            .map_err(|e| FishReadError::Database(e.to_string()))?
            .ok_or_else(|| FishReadError::BookNotFound(book_id.to_owned()))?;

        let current_id = settings_repo::get_current_book_id(self.conn)
            .map_err(|e| FishReadError::Database(e.to_string()))?;
        let cleared_current = current_id.as_deref() == Some(book_id);

        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| FishReadError::Database(e.to_string()))?;

        book_repo::delete_chapters(&tx, book_id)
            .map_err(|e| FishReadError::Database(e.to_string()))?;
        book_repo::delete_reading_position(&tx, book_id)
            .map_err(|e| FishReadError::Database(e.to_string()))?;
        book_repo::delete_by_id(&tx, book_id)
            .map_err(|e| FishReadError::Database(e.to_string()))?;

        if cleared_current {
            settings_repo::clear_current_book_id(&tx)
                .map_err(|e| FishReadError::Database(e.to_string()))?;
        }

        tx.commit()
            .map_err(|e| FishReadError::Database(e.to_string()))?;

        Ok(BookDeleteResult {
            id: row.id,
            title: row.title,
            author: row.author,
            format: row.format,
            cleared_current,
        })
    }

    fn reading_anchor_label(
        &self,
        book_id: &str,
        chapter_index: i64,
        chunk_index: i64,
    ) -> Result<String, FishReadError> {
        let chapter = chapter_repo::find_by_index(self.conn, book_id, chapter_index)
            .map_err(|e| FishReadError::Database(e.to_string()))?;
        let Some(chapter) = chapter else {
            return Ok("0%".to_owned());
        };

        let chunks = chunk::split(&chapter.content, chunk::CHUNK_SIZE);
        Ok(anchor_label_for_chunk(chunks.len(), chunk_index).to_owned())
    }
}

fn anchor_label_for_chunk(total_chunks: usize, chunk_index: i64) -> &'static str {
    let mut current = ANCHOR_CANDIDATES[0].1;
    let mut last_chunk_index: Option<usize> = None;
    for (percent, label) in ANCHOR_CANDIDATES {
        let anchor_chunk_index = anchor_chunk_index(*percent, total_chunks);
        if Some(anchor_chunk_index) == last_chunk_index {
            continue;
        }
        last_chunk_index = Some(anchor_chunk_index);

        if anchor_chunk_index as i64 <= chunk_index {
            current = label;
        }
    }
    current
}

fn anchor_chunk_index(percent: f64, total_chunks: usize) -> usize {
    if total_chunks <= 1 {
        return 0;
    }
    let max_index = total_chunks - 1;
    ((percent / 100.0) * max_index as f64).round() as usize
}
