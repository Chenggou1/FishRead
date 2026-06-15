use serde::Serialize;

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
