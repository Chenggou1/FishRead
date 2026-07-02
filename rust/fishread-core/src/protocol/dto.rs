use serde::Serialize;

use crate::book::service::{BookDeleteResult, BookListResult, BookUseResult};
use crate::chapter::service::{AnchorChunk, AnchorPosition, ChapterListResult, ReadingAnchor};
use crate::importer::model::{ImportResult, ImportWarning};
use crate::reader::service::ReaderState;
use crate::storage::migrations::{MigrationInfo, MigrationRun, MigrationStatus};

#[derive(Debug, Serialize)]
pub struct BookDto {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub format: String,
}

#[derive(Debug, Serialize)]
pub struct ImportWarningDto {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ImportResultDto {
    pub book: BookDto,
    pub chapters_count: usize,
    pub current: bool,
    pub warnings: Vec<ImportWarningDto>,
}

impl From<ImportResult> for ImportResultDto {
    fn from(r: ImportResult) -> Self {
        Self {
            book: BookDto {
                id: r.book.id.0,
                title: r.book.title,
                author: r.book.author,
                format: r.book.format.as_str().to_owned(),
            },
            chapters_count: r.chapters_count,
            current: r.current,
            warnings: r.warnings.into_iter().map(ImportWarningDto::from).collect(),
        }
    }
}

impl From<ImportWarning> for ImportWarningDto {
    fn from(w: ImportWarning) -> Self {
        Self {
            code: w.code,
            message: w.message,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MigrationInfoDto {
    pub version: i64,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct MigrationRunDto {
    pub database_path: String,
    pub applied: Vec<MigrationInfoDto>,
    pub current_version: i64,
    pub latest_version: i64,
}

#[derive(Debug, Serialize)]
pub struct MigrationStatusDto {
    pub database_path: String,
    pub applied: Vec<MigrationInfoDto>,
    pub pending: Vec<MigrationInfoDto>,
    pub current_version: i64,
    pub latest_version: i64,
}

impl MigrationRunDto {
    pub fn from_run(database_path: String, run: MigrationRun) -> Self {
        Self {
            database_path,
            applied: run
                .applied
                .into_iter()
                .map(MigrationInfoDto::from)
                .collect(),
            current_version: run.current_version,
            latest_version: run.latest_version,
        }
    }
}

impl MigrationStatusDto {
    pub fn from_status(database_path: String, status: MigrationStatus) -> Self {
        Self {
            database_path,
            applied: status
                .applied
                .into_iter()
                .map(MigrationInfoDto::from)
                .collect(),
            pending: status
                .pending
                .into_iter()
                .map(MigrationInfoDto::from)
                .collect(),
            current_version: status.current_version,
            latest_version: status.latest_version,
        }
    }
}

impl From<MigrationInfo> for MigrationInfoDto {
    fn from(m: MigrationInfo) -> Self {
        Self {
            version: m.version,
            name: m.name,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BookListItemDto {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub format: String,
    pub current: bool,
    pub imported_at: i64,
    pub position: PositionDto,
    pub reading_anchor_label: String,
}

#[derive(Debug, Serialize)]
pub struct BookListDto {
    pub books: Vec<BookListItemDto>,
}

impl From<BookListResult> for BookListDto {
    fn from(r: BookListResult) -> Self {
        Self {
            books: r
                .books
                .into_iter()
                .map(|b| BookListItemDto {
                    id: b.id,
                    title: b.title,
                    author: b.author,
                    format: b.format,
                    current: b.current,
                    imported_at: b.imported_at,
                    position: PositionDto {
                        chapter_index: b.position.chapter_index,
                        chunk_index: b.position.chunk_index,
                    },
                    reading_anchor_label: b.reading_anchor_label,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PositionDto {
    pub chapter_index: i64,
    pub chunk_index: i64,
}

#[derive(Debug, Serialize)]
pub struct BookUseDto {
    pub book: BookDto,
    pub position: PositionDto,
}

#[derive(Debug, Serialize)]
pub struct BookDeleteDto {
    pub deleted: BookDto,
    pub cleared_current: bool,
}

#[derive(Debug, Serialize)]
pub struct BookRefDto {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Serialize)]
pub struct ChapterListItemDto {
    pub id: String,
    pub index: i64,
    pub title: String,
    pub current: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchors: Option<Vec<ReadingAnchorDto>>,
}

#[derive(Debug, Serialize)]
pub struct ReadingAnchorDto {
    pub label: String,
    pub chapter_percent: f64,
    pub current: bool,
    pub position: PositionDto,
    pub chunk: AnchorChunkDto,
    pub preview: String,
}

#[derive(Debug, Serialize)]
pub struct AnchorChunkDto {
    pub index: i64,
    pub is_first: bool,
    pub is_last: bool,
}

#[derive(Debug, Serialize)]
pub struct ChapterListDto {
    pub book: BookRefDto,
    pub chapters: Vec<ChapterListItemDto>,
}

impl From<ChapterListResult> for ChapterListDto {
    fn from(r: ChapterListResult) -> Self {
        Self {
            book: BookRefDto {
                id: r.book_id,
                title: r.book_title,
            },
            chapters: r
                .chapters
                .into_iter()
                .map(|c| ChapterListItemDto {
                    id: c.id,
                    index: c.index,
                    title: c.title,
                    current: c.current,
                    anchors: c
                        .anchors
                        .map(|anchors| anchors.into_iter().map(ReadingAnchorDto::from).collect()),
                })
                .collect(),
        }
    }
}

impl From<ReadingAnchor> for ReadingAnchorDto {
    fn from(a: ReadingAnchor) -> Self {
        Self {
            label: a.label,
            chapter_percent: a.chapter_percent,
            current: a.current,
            position: PositionDto::from(a.position),
            chunk: AnchorChunkDto::from(a.chunk),
            preview: a.preview,
        }
    }
}

impl From<AnchorPosition> for PositionDto {
    fn from(p: AnchorPosition) -> Self {
        Self {
            chapter_index: p.chapter_index,
            chunk_index: p.chunk_index,
        }
    }
}

impl From<AnchorChunk> for AnchorChunkDto {
    fn from(c: AnchorChunk) -> Self {
        Self {
            index: c.index,
            is_first: c.is_first,
            is_last: c.is_last,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BookReaderDto {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChapterRefDto {
    pub id: String,
    pub index: i64,
    pub title: String,
}

#[derive(Debug, Serialize)]
pub struct ChunkDto {
    pub index: i64,
    pub text: String,
    pub is_first: bool,
    pub is_last: bool,
}

#[derive(Debug, Serialize)]
pub struct ProgressDto {
    pub chapter_index: i64,
    pub chunk_index: i64,
    pub chapter_percent: f64,
    pub book_percent: f64,
}

#[derive(Debug, Serialize)]
pub struct ReaderStateDto {
    pub book: BookReaderDto,
    pub chapter: ChapterRefDto,
    pub chunk: ChunkDto,
    pub progress: ProgressDto,
    pub start_of_book: bool,
    pub end_of_book: bool,
}

impl From<ReaderState> for ReaderStateDto {
    fn from(s: ReaderState) -> Self {
        Self {
            book: BookReaderDto {
                id: s.book.id,
                title: s.book.title,
                author: s.book.author,
            },
            chapter: ChapterRefDto {
                id: s.chapter.id,
                index: s.chapter.index,
                title: s.chapter.title,
            },
            chunk: ChunkDto {
                index: s.chunk.index as i64,
                text: s.chunk.text,
                is_first: s.chunk.is_first,
                is_last: s.chunk.is_last,
            },
            progress: ProgressDto {
                chapter_index: s.progress.chapter_index,
                chunk_index: s.progress.chunk_index,
                chapter_percent: s.progress.chapter_percent,
                book_percent: s.progress.book_percent,
            },
            start_of_book: s.start_of_book,
            end_of_book: s.end_of_book,
        }
    }
}

impl From<BookUseResult> for BookUseDto {
    fn from(r: BookUseResult) -> Self {
        Self {
            book: BookDto {
                id: r.id,
                title: r.title,
                author: r.author,
                format: r.format,
            },
            position: PositionDto {
                chapter_index: r.position.chapter_index,
                chunk_index: r.position.chunk_index,
            },
        }
    }
}

impl From<BookDeleteResult> for BookDeleteDto {
    fn from(r: BookDeleteResult) -> Self {
        Self {
            deleted: BookDto {
                id: r.id,
                title: r.title,
                author: r.author,
                format: r.format,
            },
            cleared_current: r.cleared_current,
        }
    }
}
