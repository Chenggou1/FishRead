use thiserror::Error;

#[derive(Debug, Error)]
pub enum FishReadError {
    #[error("database not initialized; run `fishread init` first")]
    DatabaseNotInitialized,

    #[error("database error: {0}")]
    Database(#[from] anyhow::Error),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("epub parse error: {0}")]
    EpubParse(String),

    #[error("epub has no readable chapters")]
    EpubNoReadableChapters,

    #[error("book not found: {0}")]
    BookNotFound(String),

    #[error("no current book set")]
    NoCurrentBook,

    #[error("chapter not found")]
    ChapterNotFound,

    #[error("reading position not found")]
    ReadingPositionNotFound,

    #[error("library is empty")]
    EmptyLibrary,
}

impl FishReadError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::DatabaseNotInitialized => "DATABASE_NOT_INITIALIZED",
            Self::Database(_) => "DATABASE_ERROR",
            Self::InvalidArgument(_) => "INVALID_ARGUMENT",
            Self::UnsupportedFormat(_) => "UNSUPPORTED_FORMAT",
            Self::EpubParse(_) => "EPUB_PARSE_ERROR",
            Self::EpubNoReadableChapters => "EPUB_NO_READABLE_CHAPTERS",
            Self::BookNotFound(_) => "BOOK_NOT_FOUND",
            Self::NoCurrentBook => "NO_CURRENT_BOOK",
            Self::ChapterNotFound => "CHAPTER_NOT_FOUND",
            Self::ReadingPositionNotFound => "READING_POSITION_NOT_FOUND",
            Self::EmptyLibrary => "EMPTY_LIBRARY",
        }
    }
}
