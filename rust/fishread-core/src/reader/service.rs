use crate::error::FishReadError;
use crate::reader::chunk;
use crate::storage::{book_repo, chapter_repo, settings_repo};

pub struct BookInfo {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
}

pub struct ChapterInfo {
    pub id: String,
    pub index: i64,
    pub title: String,
}

pub struct ChunkInfo {
    pub index: usize,
    pub text: String,
    pub is_first: bool,
    pub is_last: bool,
}

pub struct ProgressInfo {
    pub chapter_index: i64,
    pub chunk_index: i64,
    pub chapter_percent: f64,
    pub book_percent: f64,
}

pub struct ReaderState {
    pub book: BookInfo,
    pub chapter: ChapterInfo,
    pub chunk: ChunkInfo,
    pub progress: ProgressInfo,
    pub start_of_book: bool,
    pub end_of_book: bool,
}

enum Direction {
    Stay,
    Next,
    Prev,
}

pub struct ReaderService<'a> {
    conn: &'a rusqlite::Connection,
}

impl<'a> ReaderService<'a> {
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    pub fn current(&self) -> Result<ReaderState, FishReadError> {
        self.read(Direction::Stay)
    }

    pub fn next(&self) -> Result<ReaderState, FishReadError> {
        self.read(Direction::Next)
    }

    pub fn prev(&self) -> Result<ReaderState, FishReadError> {
        self.read(Direction::Prev)
    }

    fn read(&self, direction: Direction) -> Result<ReaderState, FishReadError> {
        let book_id = settings_repo::get_current_book_id(self.conn)
            .map_err(|e| FishReadError::Database(e.to_string()))?
            .ok_or(FishReadError::NoCurrentBook)?;

        let book_row = book_repo::find_by_id(self.conn, &book_id)
            .map_err(|e| FishReadError::Database(e.to_string()))?
            .ok_or_else(|| FishReadError::BookNotFound(book_id.clone()))?;

        let total_chapters = chapter_repo::count(self.conn, &book_id)
            .map_err(|e| FishReadError::Database(e.to_string()))?;

        let (mut chapter_index, mut chunk_index) =
            book_repo::get_reading_position(self.conn, &book_id)
                .map_err(|e| FishReadError::Database(e.to_string()))?
                .unwrap_or((0, 0));

        let mut chapter = chapter_repo::find_by_index(self.conn, &book_id, chapter_index)
            .map_err(|e| FishReadError::Database(e.to_string()))?
            .ok_or(FishReadError::ChapterNotFound)?;

        let mut chunks = chunk::split(&chapter.content, chunk::CHUNK_SIZE);
        let chunk_idx = (chunk_index as usize).min(chunks.len().saturating_sub(1));

        match direction {
            Direction::Stay => {}

            Direction::Next => {
                let next = chunk_idx + 1;
                if next < chunks.len() {
                    chunk_index = next as i64;
                } else if chapter_index + 1 < total_chapters as i64 {
                    chapter_index += 1;
                    chunk_index = 0;
                    chapter = chapter_repo::find_by_index(self.conn, &book_id, chapter_index)
                        .map_err(|e| FishReadError::Database(e.to_string()))?
                        .ok_or(FishReadError::ChapterNotFound)?;
                    chunks = chunk::split(&chapter.content, chunk::CHUNK_SIZE);
                }
                // else: end of book — position unchanged
                self.save_position(&book_id, chapter_index, chunk_index)?;
            }

            Direction::Prev => {
                if chunk_idx > 0 {
                    chunk_index = chunk_idx as i64 - 1;
                } else if chapter_index > 0 {
                    chapter_index -= 1;
                    chapter = chapter_repo::find_by_index(self.conn, &book_id, chapter_index)
                        .map_err(|e| FishReadError::Database(e.to_string()))?
                        .ok_or(FishReadError::ChapterNotFound)?;
                    chunks = chunk::split(&chapter.content, chunk::CHUNK_SIZE);
                    chunk_index = chunks.len().saturating_sub(1) as i64;
                }
                // else: start of book — position unchanged
                self.save_position(&book_id, chapter_index, chunk_index)?;
            }
        }

        let final_idx = (chunk_index as usize).min(chunks.len().saturating_sub(1));
        let reading_chunk = &chunks[final_idx];
        let total_chunks = chunks.len();

        let chapter_pct = chunk::chapter_percent(final_idx, total_chunks);
        let book_pct = chunk::book_percent(
            chapter_index as usize,
            final_idx,
            total_chunks,
            total_chapters,
        );

        let start_of_book = chapter_index == 0 && final_idx == 0;
        let end_of_book = chapter_index + 1 == total_chapters as i64 && reading_chunk.is_last;

        Ok(ReaderState {
            book: BookInfo {
                id: book_row.id,
                title: book_row.title,
                author: book_row.author,
            },
            chapter: ChapterInfo {
                id: chapter.id.0,
                index: chapter.index.0,
                title: chapter.title,
            },
            chunk: ChunkInfo {
                index: reading_chunk.index,
                text: reading_chunk.text.clone(),
                is_first: reading_chunk.is_first,
                is_last: reading_chunk.is_last,
            },
            progress: ProgressInfo {
                chapter_index,
                chunk_index: final_idx as i64,
                chapter_percent: chapter_pct,
                book_percent: book_pct,
            },
            start_of_book,
            end_of_book,
        })
    }

    fn save_position(
        &self,
        book_id: &str,
        chapter_index: i64,
        chunk_index: i64,
    ) -> Result<(), FishReadError> {
        let now = crate::book::model::Timestamp::now().0;
        self.conn
            .execute(
                "UPDATE reading_positions
                 SET chapter_index = ?1, chunk_index = ?2, updated_at = ?3
                 WHERE book_id = ?4",
                rusqlite::params![chapter_index, chunk_index, now, book_id],
            )
            .map_err(|e| FishReadError::Database(e.to_string()))?;
        Ok(())
    }
}
