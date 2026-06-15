CREATE TABLE IF NOT EXISTS books (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    author      TEXT,
    format      TEXT NOT NULL,
    source_path TEXT,
    imported_at INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS chapters (
    id            TEXT PRIMARY KEY,
    book_id       TEXT NOT NULL,
    chapter_index INTEGER NOT NULL,
    title         TEXT NOT NULL,
    content       TEXT NOT NULL,
    source_path   TEXT,
    created_at    INTEGER NOT NULL,
    FOREIGN KEY(book_id) REFERENCES books(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_chapters_book_index
    ON chapters(book_id, chapter_index);

CREATE TABLE IF NOT EXISTS reading_positions (
    book_id       TEXT PRIMARY KEY,
    chapter_index INTEGER NOT NULL,
    chunk_index   INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    FOREIGN KEY(book_id) REFERENCES books(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
