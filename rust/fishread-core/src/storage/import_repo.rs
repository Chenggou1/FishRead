use anyhow::Context;
use rusqlite::params;

use crate::book::model::{Book, Timestamp};
use crate::chapter::model::Chapter;

/// Write a complete book import atomically.
///
/// Inserts the book, all chapters, initialises the reading position at 0/0,
/// and optionally sets `current_book_id` in settings — all inside one transaction.
pub fn import_book(
    conn: &rusqlite::Connection,
    book: &Book,
    chapters: &[Chapter],
    set_as_current: bool,
) -> anyhow::Result<()> {
    let tx = conn
        .unchecked_transaction()
        .context("failed to begin transaction")?;

    tx.execute(
        "INSERT INTO books (id, title, author, format, source_path, imported_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            book.id.0,
            book.title,
            book.author,
            book.format.as_str(),
            book.source_path,
            book.imported_at.0,
            book.updated_at.0,
        ],
    )
    .context("failed to insert book")?;

    for ch in chapters {
        tx.execute(
            "INSERT INTO chapters
                (id, book_id, chapter_index, title, content, source_path, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                ch.id.0,
                ch.book_id.0,
                ch.index.0,
                ch.title,
                ch.content,
                ch.source_path,
                book.imported_at.0,
            ],
        )
        .with_context(|| format!("failed to insert chapter index {}", ch.index.0))?;
    }

    let initial_chapter_index = chapters.first().map(|ch| ch.index.0).unwrap_or(0);
    let now = Timestamp::now().0;
    tx.execute(
        "INSERT INTO reading_positions (book_id, chapter_index, chunk_index, updated_at)
         VALUES (?1, ?2, 0, ?3)",
        params![book.id.0, initial_chapter_index, now],
    )
    .context("failed to insert reading position")?;

    if set_as_current {
        tx.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('current_book_id', ?1)",
            params![book.id.0],
        )
        .context("failed to set current_book_id")?;
    }

    tx.commit().context("failed to commit transaction")?;
    Ok(())
}
