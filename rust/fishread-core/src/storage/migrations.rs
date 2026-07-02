use anyhow::Context;
use rusqlite::{params, Connection};

const MIGRATION_0001: &str = include_str!("../../../../migrations/0001_init.sql");

const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "init",
    sql: MIGRATION_0001,
}];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationInfo {
    pub version: i64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationRun {
    pub applied: Vec<MigrationInfo>,
    pub current_version: i64,
    pub latest_version: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationStatus {
    pub applied: Vec<MigrationInfo>,
    pub pending: Vec<MigrationInfo>,
    pub current_version: i64,
    pub latest_version: i64,
}

struct Migration {
    version: i64,
    name: &'static str,
    sql: &'static str,
}

pub fn run(conn: &mut Connection) -> anyhow::Result<MigrationRun> {
    ensure_migrations_table(conn)?;

    let tx = conn
        .transaction()
        .context("failed to start migration transaction")?;
    let mut applied = Vec::new();

    for migration in MIGRATIONS {
        if migration_applied(&tx, migration.version)? {
            continue;
        }

        tx.execute_batch(migration.sql).with_context(|| {
            format!(
                "failed to run migration {:04}_{}",
                migration.version, migration.name
            )
        })?;
        tx.execute(
            "INSERT INTO _fishread_migrations (version, name, applied_at) VALUES (?1, ?2, strftime('%s','now'))",
            params![migration.version, migration.name],
        )
        .with_context(|| format!("failed to record migration {:04}_{}", migration.version, migration.name))?;
        applied.push(MigrationInfo {
            version: migration.version,
            name: migration.name.to_owned(),
        });
    }

    tx.commit()
        .context("failed to commit migration transaction")?;

    let current_version = current_version(conn)?;
    Ok(MigrationRun {
        applied,
        current_version,
        latest_version: latest_version(),
    })
}

pub fn status(conn: &Connection) -> anyhow::Result<MigrationStatus> {
    if !migrations_table_exists(conn)? {
        return Ok(MigrationStatus {
            applied: Vec::new(),
            pending: known_migrations(),
            current_version: 0,
            latest_version: latest_version(),
        });
    }

    let applied = applied_migrations(conn)?;
    let pending = MIGRATIONS
        .iter()
        .filter(|migration| !applied.iter().any(|a| a.version == migration.version))
        .map(|migration| MigrationInfo {
            version: migration.version,
            name: migration.name.to_owned(),
        })
        .collect();
    let current_version = applied.iter().map(|m| m.version).max().unwrap_or(0);

    Ok(MigrationStatus {
        applied,
        pending,
        current_version,
        latest_version: latest_version(),
    })
}

fn ensure_migrations_table(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _fishread_migrations (
            version    INTEGER PRIMARY KEY,
            name       TEXT NOT NULL,
            applied_at INTEGER NOT NULL
        );",
    )
    .context("failed to create migration metadata table")?;
    Ok(())
}

fn migrations_table_exists(conn: &Connection) -> anyhow::Result<bool> {
    let exists = conn
        .query_row(
            "SELECT EXISTS (
                SELECT 1
                FROM sqlite_master
                WHERE type = 'table' AND name = '_fishread_migrations'
            )",
            [],
            |row| row.get::<_, i64>(0),
        )
        .context("failed to inspect migration metadata table")?;
    Ok(exists == 1)
}

fn migration_applied(conn: &Connection, version: i64) -> anyhow::Result<bool> {
    let applied = conn
        .query_row(
            "SELECT EXISTS (SELECT 1 FROM _fishread_migrations WHERE version = ?1)",
            [version],
            |row| row.get::<_, i64>(0),
        )
        .with_context(|| format!("failed to inspect migration version {version}"))?;
    Ok(applied == 1)
}

fn applied_migrations(conn: &Connection) -> anyhow::Result<Vec<MigrationInfo>> {
    let mut stmt = conn
        .prepare("SELECT version, name FROM _fishread_migrations ORDER BY version")
        .context("failed to read applied migrations")?;
    let migrations = stmt
        .query_map([], |row| {
            Ok(MigrationInfo {
                version: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .context("failed to query applied migrations")?
        .collect::<Result<Vec<_>, _>>()
        .context("failed to decode applied migrations")?;
    Ok(migrations)
}

fn known_migrations() -> Vec<MigrationInfo> {
    MIGRATIONS
        .iter()
        .map(|migration| MigrationInfo {
            version: migration.version,
            name: migration.name.to_owned(),
        })
        .collect()
}

fn current_version(conn: &Connection) -> anyhow::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM _fishread_migrations",
        [],
        |row| row.get(0),
    )
    .context("failed to read current migration version")
}

fn latest_version() -> i64 {
    MIGRATIONS
        .iter()
        .map(|migration| migration.version)
        .max()
        .unwrap_or(0)
}
