use crate::book::model::Book;

/// Warning generated during import — does not abort the import.
#[derive(Debug)]
pub struct ImportWarning {
    pub code: String,
    pub message: String,
}

#[derive(Debug)]
pub struct NormalizedChapter {
    /// Position in the EPUB spine (0-based).
    pub source_index: usize,
    /// Original XHTML file path inside the EPUB archive.
    pub source_path: Option<String>,
    pub title: String,
    /// Plain text extracted from the XHTML body.
    pub content: String,
}

#[derive(Debug)]
pub struct NormalizedBook {
    pub title: String,
    pub author: Option<String>,
    pub language: Option<String>,
    pub identifier: Option<String>,
    pub chapters: Vec<NormalizedChapter>,
    pub warnings: Vec<ImportWarning>,
}

/// Result returned by ImportService after a successful import.
#[derive(Debug)]
pub struct ImportResult {
    pub book: Book,
    pub chapters_count: usize,
    pub current: bool,
    pub warnings: Vec<ImportWarning>,
}
