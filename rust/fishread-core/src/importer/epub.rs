use std::ffi::OsStr;
use std::path::Path;

use anyhow::bail;

use crate::epub::{normalize::normalize, package::ParsedEpub};
use crate::error::FishReadError;

use super::{BookImporter, NormalizedBook};

pub struct EpubImporter;

impl BookImporter for EpubImporter {
    fn import(&self, path: &Path) -> anyhow::Result<NormalizedBook> {
        if !path.exists() {
            bail!(FishReadError::InvalidArgument(format!(
                "file not found: {}",
                path.display()
            )));
        }
        if path.extension() != Some(OsStr::new("epub")) {
            bail!(FishReadError::UnsupportedFormat(
                path.extension()
                    .and_then(OsStr::to_str)
                    .unwrap_or("unknown")
                    .to_owned()
            ));
        }

        let file_stem = path
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or("Unknown");

        let parsed = ParsedEpub::parse(path)?;
        let book = normalize(parsed, file_stem);

        if book.chapters.is_empty() {
            bail!(FishReadError::EpubNoReadableChapters);
        }

        Ok(book)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("fixtures/epub")
    }

    #[test]
    fn parse_simple_epub_title_and_author() {
        let path = fixtures_dir().join("simple.epub");
        let book = EpubImporter.import(&path).unwrap();
        assert_eq!(book.title, "Simple Book");
        assert_eq!(book.author.as_deref(), Some("Test Author"));
    }

    #[test]
    fn parse_simple_epub_chapter_count_and_order() {
        let path = fixtures_dir().join("simple.epub");
        let book = EpubImporter.import(&path).unwrap();
        assert_eq!(book.chapters.len(), 1);
        assert_eq!(book.chapters[0].source_index, 0);
    }

    #[test]
    fn parse_simple_epub_chapter_title_from_nav() {
        let path = fixtures_dir().join("simple.epub");
        let book = EpubImporter.import(&path).unwrap();
        assert_eq!(book.chapters[0].title, "Chapter One");
    }

    #[test]
    fn parse_simple_epub_content_is_plain_text() {
        let path = fixtures_dir().join("simple.epub");
        let book = EpubImporter.import(&path).unwrap();
        let content = &book.chapters[0].content;
        assert!(
            !content.contains('<'),
            "content should not contain HTML tags"
        );
        assert!(!content.is_empty(), "content should not be empty");
        assert!(content.contains("first paragraph"));
    }

    #[test]
    fn no_author_epub_returns_none_author_with_warning() {
        let path = fixtures_dir().join("no-author.epub");
        let book = EpubImporter.import(&path).unwrap();
        assert!(book.author.is_none());
        assert!(book.warnings.iter().any(|w| w.code == "AUTHOR_MISSING"));
    }

    #[test]
    fn multi_chapter_epub_preserves_spine_order() {
        let path = fixtures_dir().join("multi-chapter.epub");
        let book = EpubImporter.import(&path).unwrap();
        assert_eq!(book.chapters.len(), 3);
        assert_eq!(book.chapters[0].title, "First Chapter");
        assert_eq!(book.chapters[1].title, "Second Chapter");
        assert_eq!(book.chapters[2].title, "Third Chapter");
        // source_index must match spine order
        assert_eq!(book.chapters[0].source_index, 0);
        assert_eq!(book.chapters[1].source_index, 1);
        assert_eq!(book.chapters[2].source_index, 2);
    }

    #[test]
    fn multi_chapter_each_has_non_empty_content() {
        let path = fixtures_dir().join("multi-chapter.epub");
        let book = EpubImporter.import(&path).unwrap();
        for ch in &book.chapters {
            assert!(
                !ch.content.is_empty(),
                "chapter '{}' has empty content",
                ch.title
            );
            assert!(
                !ch.content.contains('<'),
                "chapter '{}' contains HTML",
                ch.title
            );
        }
    }

    #[test]
    fn missing_file_returns_error() {
        let path = std::path::Path::new("/nonexistent/path.epub");
        let result = EpubImporter.import(path);
        assert!(result.is_err());
    }

    #[test]
    fn non_epub_extension_returns_unsupported_format() {
        let result = EpubImporter.import(std::path::Path::new("/tmp/book.pdf"));
        let err = result.unwrap_err();
        assert!(err.to_string().contains("pdf") || err.to_string().contains("unsupported"));
    }
}
