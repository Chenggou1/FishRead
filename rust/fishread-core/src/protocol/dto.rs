use serde::Serialize;

use crate::book::service::{BookListResult, BookUseResult};
use crate::importer::model::{ImportResult, ImportWarning};

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
pub struct BookListItemDto {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub format: String,
    pub current: bool,
    pub imported_at: i64,
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
