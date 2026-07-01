use crate::error::FishReadError;
use crate::reader::chunk;
use crate::storage::{book_repo, chapter_repo, settings_repo};

const ANCHOR_CANDIDATES: &[(f64, &str)] = &[
    (0.0, "0%"),
    (25.0, "25%"),
    (50.0, "50%"),
    (75.0, "75%"),
    (90.0, "90%"),
];
const PREVIEW_CHARS: usize = 48;

pub struct ChapterListItem {
    pub id: String,
    pub index: i64,
    pub title: String,
    pub current: bool,
    pub anchors: Option<Vec<ReadingAnchor>>,
}

pub struct ReadingAnchor {
    pub label: String,
    pub chapter_percent: f64,
    pub current: bool,
    pub position: AnchorPosition,
    pub chunk: AnchorChunk,
    pub preview: String,
}

pub struct AnchorPosition {
    pub chapter_index: i64,
    pub chunk_index: i64,
}

pub struct AnchorChunk {
    pub index: i64,
    pub is_first: bool,
    pub is_last: bool,
}

pub struct ChapterListResult {
    pub book_id: String,
    pub book_title: String,
    pub chapters: Vec<ChapterListItem>,
}

pub struct ChapterService<'a> {
    conn: &'a rusqlite::Connection,
}

impl<'a> ChapterService<'a> {
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    pub fn list(&self, navigation: bool) -> Result<ChapterListResult, FishReadError> {
        let book_id = settings_repo::get_current_book_id(self.conn)
            .map_err(|e| FishReadError::Database(e.to_string()))?
            .ok_or(FishReadError::NoCurrentBook)?;

        let book = book_repo::find_by_id(self.conn, &book_id)
            .map_err(|e| FishReadError::Database(e.to_string()))?
            .ok_or_else(|| FishReadError::BookNotFound(book_id.clone()))?;

        let position = book_repo::get_reading_position(self.conn, &book_id)
            .map_err(|e| FishReadError::Database(e.to_string()))?
            .unwrap_or((0, 0));
        let current_chapter_index = position.0;
        let current_chunk_index = position.1;

        let metas = chapter_repo::list_meta_by_book(self.conn, &book_id)
            .map_err(|e| FishReadError::Database(e.to_string()))?;

        let mut chapters = Vec::with_capacity(metas.len());
        for m in metas {
            let anchors = if navigation {
                let chapter = chapter_repo::find_by_index(self.conn, &book_id, m.index.0)
                    .map_err(|e| FishReadError::Database(e.to_string()))?
                    .ok_or(FishReadError::ChapterNotFound)?;
                let chunks = chunk::split(&chapter.content, chunk::CHUNK_SIZE);
                Some(build_anchors(
                    m.index.0,
                    &chunks,
                    current_chapter_index == m.index.0,
                    current_chunk_index,
                ))
            } else {
                None
            };

            chapters.push(ChapterListItem {
                current: m.index.0 == current_chapter_index,
                id: m.id.0,
                index: m.index.0,
                title: m.title,
                anchors,
            })
        }

        Ok(ChapterListResult {
            book_id: book.id,
            book_title: book.title,
            chapters,
        })
    }
}

fn build_anchors(
    chapter_index: i64,
    chunks: &[chunk::ReadingChunk],
    current_chapter: bool,
    current_chunk_index: i64,
) -> Vec<ReadingAnchor> {
    let mut anchors = Vec::new();
    let mut last_chunk_index: Option<usize> = None;
    let current_anchor_index = if current_chapter {
        current_anchor_index(chunks.len(), current_chunk_index)
    } else {
        None
    };

    for (candidate_idx, (percent, label)) in ANCHOR_CANDIDATES.iter().enumerate() {
        let chunk_index = anchor_chunk_index(*percent, chunks.len());
        if Some(chunk_index) == last_chunk_index {
            continue;
        }
        last_chunk_index = Some(chunk_index);

        let reading_chunk = &chunks[chunk_index];
        anchors.push(ReadingAnchor {
            label: (*label).to_owned(),
            chapter_percent: *percent,
            current: current_anchor_index == Some(candidate_idx),
            position: AnchorPosition {
                chapter_index,
                chunk_index: chunk_index as i64,
            },
            chunk: AnchorChunk {
                index: reading_chunk.index as i64,
                is_first: reading_chunk.is_first,
                is_last: reading_chunk.is_last,
            },
            preview: preview(&reading_chunk.text),
        });
    }

    anchors
}

fn current_anchor_index(total_chunks: usize, current_chunk_index: i64) -> Option<usize> {
    let mut current = None;
    let mut last_chunk_index: Option<usize> = None;
    for (candidate_idx, (percent, _)) in ANCHOR_CANDIDATES.iter().enumerate() {
        let chunk_index = anchor_chunk_index(*percent, total_chunks);
        if Some(chunk_index) == last_chunk_index {
            continue;
        }
        last_chunk_index = Some(chunk_index);

        if chunk_index as i64 <= current_chunk_index {
            current = Some(candidate_idx);
        }
    }
    current
}

fn anchor_chunk_index(percent: f64, total_chunks: usize) -> usize {
    if total_chunks <= 1 {
        return 0;
    }
    let max_index = total_chunks - 1;
    ((percent / 100.0) * max_index as f64).round() as usize
}

fn preview(text: &str) -> String {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = normalized.chars();
    let truncated: String = chars.by_ref().take(PREVIEW_CHARS).collect();
    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}
