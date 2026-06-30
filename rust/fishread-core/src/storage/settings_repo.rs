use anyhow::Context;

pub fn get_current_book_id(conn: &rusqlite::Connection) -> anyhow::Result<Option<String>> {
    let result = conn.query_row(
        "SELECT value FROM settings WHERE key = 'current_book_id'",
        [],
        |row| row.get(0),
    );
    match result {
        Ok(id) => Ok(Some(id)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e).context("failed to read current_book_id"),
    }
}

pub fn set_current_book_id(conn: &rusqlite::Connection, book_id: &str) -> anyhow::Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('current_book_id', ?1)",
        rusqlite::params![book_id],
    )
    .context("failed to set current_book_id")?;
    Ok(())
}

pub fn clear_current_book_id(conn: &rusqlite::Connection) -> anyhow::Result<()> {
    conn.execute("DELETE FROM settings WHERE key = 'current_book_id'", [])
        .context("failed to clear current_book_id")?;
    Ok(())
}
