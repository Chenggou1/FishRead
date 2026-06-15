use std::path::Path;

use crate::book::model::{Book, BookFormat, BookId, Timestamp};
use crate::chapter::model::{Chapter, ChapterId, ChapterIndex};
use crate::error::FishReadError;
use crate::storage::{import_repo, settings_repo};

use super::epub::EpubImporter;
use super::model::ImportResult;
use super::BookImporter;

pub struct ImportService<'a> {
    conn: &'a rusqlite::Connection,
}

impl<'a> ImportService<'a> {
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    pub fn import(&self, path: &Path) -> Result<ImportResult, FishReadError> {
        // 1. Parse EPUB — downcast anyhow errors to FishReadError where possible
        let normalized = EpubImporter.import(path).map_err(|e| {
            e.downcast::<FishReadError>()
                .unwrap_or_else(|e| FishReadError::EpubParse(e.to_string()))
        })?;

        // 2. Build domain objects
        let now = Timestamp::now();
        let book_id = BookId::new();

        let book = Book {
            id: book_id.clone(),
            title: normalized.title,
            author: normalized.author,
            format: BookFormat::Epub,
            source_path: path.to_str().map(str::to_owned),
            imported_at: now,
            updated_at: now,
        };

        let chapters: Vec<Chapter> = normalized
            .chapters
            .iter()
            .map(|nc| Chapter {
                id: ChapterId::new(),
                book_id: book_id.clone(),
                index: ChapterIndex(nc.source_index as i64),
                title: nc.title.clone(),
                content: nc.content.clone(),
                source_path: nc.source_path.clone(),
            })
            .collect();

        let chapters_count = chapters.len();

        // 3. Determine if this should become the current book
        let has_current = settings_repo::get_current_book_id(self.conn)
            .map_err(|e| FishReadError::Database(e.to_string()))?
            .is_some();
        let set_as_current = !has_current;

        // 4. Write everything in a single transaction
        import_repo::import_book(self.conn, &book, &chapters, set_as_current)
            .map_err(|e| FishReadError::Database(e.to_string()))?;

        Ok(ImportResult {
            book,
            chapters_count,
            current: set_as_current,
            warnings: normalized.warnings,
        })
    }
}
