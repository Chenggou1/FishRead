use anyhow::Context;
use rusqlite::Connection;

const MIGRATION_0001: &str = include_str!("../../../../migrations/0001_init.sql");

pub fn run(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(MIGRATION_0001)
        .context("failed to run migration 0001_init")?;
    Ok(())
}
