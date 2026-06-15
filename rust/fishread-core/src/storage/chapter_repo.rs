use anyhow::Context;
use rusqlite::params;

use crate::book::model::BookId;
use crate::chapter::model::{Chapter, ChapterId, ChapterIndex, ChapterMeta};

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

pub fn find_by_index(
    conn: &rusqlite::Connection,
    book_id: &str,
    chapter_index: i64,
) -> anyhow::Result<Option<Chapter>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, book_id, chapter_index, title, content, source_path
             FROM chapters WHERE book_id = ?1 AND chapter_index = ?2",
        )
        .context("failed to prepare find chapter query")?;

    let mut rows = stmt
        .query_map(params![book_id, chapter_index], |row| {
            Ok(Chapter {
                id: ChapterId(row.get(0)?),
                book_id: BookId(row.get(1)?),
                index: ChapterIndex(row.get(2)?),
                title: row.get(3)?,
                content: row.get(4)?,
                source_path: row.get(5)?,
            })
        })
        .context("failed to query chapter by index")?;

    match rows.next() {
        Some(row) => Ok(Some(row.context("failed to read chapter row")?)),
        None => Ok(None),
    }
}

pub fn count(conn: &rusqlite::Connection, book_id: &str) -> anyhow::Result<usize> {
    let n: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM chapters WHERE book_id = ?1",
            params![book_id],
            |row| row.get(0),
        )
        .context("failed to count chapters")?;
    Ok(n as usize)
}
