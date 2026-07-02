use super::package::ParsedEpub;
use super::text::xhtml_to_text;
use crate::importer::model::{ImportWarning, NormalizedBook, NormalizedChapter};

/// Convert a `ParsedEpub` into the canonical `NormalizedBook`.
///
/// Title / author fallback rules and warning generation live here.
pub fn normalize(epub: ParsedEpub, file_stem: &str) -> NormalizedBook {
    let mut warnings: Vec<ImportWarning> = Vec::new();

    // Title: metadata → filename stem → "Untitled Book"
    let title = match epub.title.filter(|t| !t.is_empty()) {
        Some(t) => t,
        None => {
            let fallback = if file_stem.is_empty() {
                "Untitled Book".to_owned()
            } else {
                file_stem.to_owned()
            };
            warnings.push(ImportWarning {
                code: "TITLE_FALLBACK_TO_FILENAME".to_owned(),
                message: "EPUB metadata title is missing; file name was used as title.".to_owned(),
            });
            fallback
        }
    };

    // Author: first creator or None
    let author = epub.author.filter(|a| !a.is_empty()).or_else(|| {
        warnings.push(ImportWarning {
            code: "AUTHOR_MISSING".to_owned(),
            message: "EPUB metadata creator is missing.".to_owned(),
        });
        None
    });

    // Chapters
    let mut chapters: Vec<NormalizedChapter> = Vec::new();
    for (source_index, parsed) in epub.chapters.into_iter().enumerate() {
        let content = xhtml_to_text(&parsed.xhtml);
        if content.is_empty() {
            continue;
        }

        // Title priority: nav → h1 (already in text extraction fallback below) → "Chapter N"
        let title_str = parsed
            .nav_title
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| {
                // Try to extract first heading from content
                extract_first_heading(&content).unwrap_or_else(|| {
                    warnings.push(ImportWarning {
                        code: "CHAPTER_TITLE_FALLBACK".to_owned(),
                        message: format!(
                            "Chapter {} has no title; using fallback.",
                            source_index + 1
                        ),
                    });
                    format!("Chapter {}", source_index + 1)
                })
            });

        chapters.push(NormalizedChapter {
            source_index,
            source_path: Some(parsed.source_path),
            title: title_str,
            content,
        });
    }

    NormalizedBook {
        title,
        author,
        language: epub.language,
        identifier: epub.identifier,
        chapters,
        warnings,
    }
}

/// Extract the first non-empty line from the plain-text content as a candidate title.
fn extract_first_heading(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(|l| l.chars().take(80).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::epub::package::ParsedChapter;

    fn parsed_epub(chapters: Vec<ParsedChapter>) -> ParsedEpub {
        ParsedEpub {
            title: Some("Test Book".to_owned()),
            author: Some("Test Author".to_owned()),
            language: None,
            identifier: None,
            chapters,
        }
    }

    #[test]
    fn empty_text_spine_items_are_skipped_without_warning() {
        let book = normalize(
            parsed_epub(vec![
                ParsedChapter {
                    source_path: "titlepage.xhtml".to_owned(),
                    nav_title: None,
                    xhtml: r#"<html><body><img src="cover.jpeg"/></body></html>"#.to_owned(),
                },
                ParsedChapter {
                    source_path: "text/chapter1.xhtml".to_owned(),
                    nav_title: Some("Chapter One".to_owned()),
                    xhtml: "<html><body><p>Readable text.</p></body></html>".to_owned(),
                },
            ]),
            "test",
        );

        assert_eq!(book.chapters.len(), 1);
        assert_eq!(
            book.chapters[0].source_path.as_deref(),
            Some("text/chapter1.xhtml")
        );
        assert!(book.warnings.is_empty());
    }

    #[test]
    fn text_spine_items_without_nav_titles_use_first_text_line() {
        let book = normalize(
            parsed_epub(vec![ParsedChapter {
                source_path: "text/chapter1.xhtml".to_owned(),
                nav_title: None,
                xhtml: "<html><body><p>Readable text.</p></body></html>".to_owned(),
            }]),
            "test",
        );

        assert_eq!(book.chapters.len(), 1);
        assert_eq!(book.chapters[0].title, "Readable text.");
        assert!(book.warnings.iter().all(|w| w.code != "SPINE_ITEM_SKIPPED"));
    }
}
