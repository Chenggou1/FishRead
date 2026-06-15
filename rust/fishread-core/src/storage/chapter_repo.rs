use anyhow::Context;
use rusqlite::params;

use crate::book::model::BookId;
use crate::chapter::model::{ChapterId, ChapterIndex, ChapterMeta};

pub fn list_meta_by_book(
    conn: &rusqlite::Connection,
    book_id: &str,
) -> anyhow::Result<Vec<ChapterMeta>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, book_id, chapter_index, title, source_path
             FROM chapters WHERE book_id = ?1 ORDER BY chapter_index ASC",
        )
        .context("failed to prepare chapter list query")?;

    let rows = stmt
        .query_map(params![book_id], |row| {
            Ok(ChapterMeta {
                id: ChapterId(row.get(0)?),
                book_id: BookId(row.get(1)?),
                index: ChapterIndex(row.get(2)?),
                title: row.get(3)?,
                source_path: row.get(4)?,
            })
        })
        .context("failed to query chapters")?
        .collect::<Result<Vec<_>, _>>()
        .context("failed to collect chapter rows")?;

    Ok(rows)
}
