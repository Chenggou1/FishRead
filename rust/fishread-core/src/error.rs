use thiserror::Error;

#[derive(Debug, Error)]
pub enum FishReadError {
    #[error("database not initialized; run `fishread init` first")]
    DatabaseNotInitialized,

    #[error("database error: {0}")]
    Database(String),

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

    #[error("chunk not found")]
    ChunkNotFound,

    #[error("reading position not found")]
    ReadingPositionNotFound,

    #[error("library is empty")]
    EmptyLibrary,
}

impl FishReadError {
    /// Maps to CLI exit code: 1 = business error, 2 = argument error, 3 = internal error.
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::InvalidArgument(_) => 2,
            Self::Database(_) | Self::DatabaseNotInitialized => 3,
            _ => 1,
        }
    }

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
            Self::ChunkNotFound => "CHUNK_NOT_FOUND",
            Self::ReadingPositionNotFound => "READING_POSITION_NOT_FOUND",
            Self::EmptyLibrary => "EMPTY_LIBRARY",
        }
    }
}
