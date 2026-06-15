use anyhow::Context;
use rusqlite::params;

use crate::storage::rows::BookRow;

pub fn list_all(conn: &rusqlite::Connection) -> anyhow::Result<Vec<BookRow>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, author, format, source_path, imported_at, updated_at
             FROM books ORDER BY imported_at ASC",
        )
        .context("failed to prepare book list query")?;

    let rows = stmt
        .query_map([], |row| {
            Ok(BookRow {
                id: row.get(0)?,
                title: row.get(1)?,
                author: row.get(2)?,
                format: row.get(3)?,
                source_path: row.get(4)?,
                imported_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .context("failed to query books")?
        .collect::<Result<Vec<_>, _>>()
        .context("failed to collect book rows")?;

    Ok(rows)
}

pub fn find_by_id(conn: &rusqlite::Connection, id: &str) -> anyhow::Result<Option<BookRow>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, author, format, source_path, imported_at, updated_at
             FROM books WHERE id = ?1",
        )
        .context("failed to prepare find book query")?;

    let mut rows = stmt
        .query_map(params![id], |row| {
            Ok(BookRow {
                id: row.get(0)?,
                title: row.get(1)?,
                author: row.get(2)?,
                format: row.get(3)?,
                source_path: row.get(4)?,
                imported_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .context("failed to query book by id")?;

    match rows.next() {
        Some(row) => Ok(Some(row.context("failed to read book row")?)),
        None => Ok(None),
    }
}

pub fn get_reading_position(
    conn: &rusqlite::Connection,
    book_id: &str,
) -> anyhow::Result<Option<(i64, i64)>> {
    let mut stmt = conn
        .prepare(
            "SELECT chapter_index, chunk_index FROM reading_positions WHERE book_id = ?1",
        )
        .context("failed to prepare reading position query")?;

    let mut rows = stmt
        .query_map(params![book_id], |row| Ok((row.get(0)?, row.get(1)?)))
        .context("failed to query reading position")?;

    match rows.next() {
        Some(pos) => Ok(Some(pos.context("failed to read position row")?)),
        None => Ok(None),
    }
}

pub fn upsert_reading_position(
    conn: &rusqlite::Connection,
    book_id: &str,
) -> anyhow::Result<()> {
    let now = crate::book::model::Timestamp::now().0;
    conn.execute(
        "INSERT OR IGNORE INTO reading_positions (book_id, chapter_index, chunk_index, updated_at)
         VALUES (?1, 0, 0, ?2)",
        params![book_id, now],
    )
    .context("failed to upsert reading position")?;
    Ok(())
}
