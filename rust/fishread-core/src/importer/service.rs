use std::path::Path;

use crate::book::model::{Book, BookFormat, BookId, Timestamp};
use crate::chapter::model::{Chapter, ChapterId, ChapterIndex};
use crate::error::FishReadError;
use crate::storage::{import_repo, settings_repo};

use super::epub::EpubImporter;
use super::model::{ImportResult, NormalizedChapter};
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

        let chapters = build_chapters(&book_id, &normalized.chapters);

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

fn build_chapters(book_id: &BookId, normalized_chapters: &[NormalizedChapter]) -> Vec<Chapter> {
    normalized_chapters
        .iter()
        .enumerate()
        .map(|(chapter_index, nc)| Chapter {
            id: ChapterId::new(),
            book_id: book_id.clone(),
            index: ChapterIndex(chapter_index as i64),
            title: nc.title.clone(),
            content: nc.content.clone(),
            source_path: nc.source_path.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_chapters_compresses_source_indices() {
        let book_id = BookId::new();
        let normalized = vec![
            normalized_chapter(1, "First readable"),
            normalized_chapter(4, "Second readable"),
            normalized_chapter(5, "Third readable"),
        ];

        let chapters = build_chapters(&book_id, &normalized);
        let indices: Vec<i64> = chapters.iter().map(|ch| ch.index.0).collect();

        assert_eq!(indices, vec![0, 1, 2]);
        assert_eq!(chapters[0].source_path.as_deref(), Some("chapter-1.xhtml"));
        assert_eq!(chapters[1].source_path.as_deref(), Some("chapter-4.xhtml"));
        assert_eq!(chapters[2].source_path.as_deref(), Some("chapter-5.xhtml"));
    }

    fn normalized_chapter(source_index: usize, title: &str) -> NormalizedChapter {
        NormalizedChapter {
            source_index,
            source_path: Some(format!("chapter-{source_index}.xhtml")),
            title: title.to_owned(),
            content: format!("Content for {title}"),
        }
    }
}
