pub const CHUNK_SIZE: usize = 600;

#[derive(Debug, Clone, PartialEq)]
pub struct ReadingChunk {
    pub index: usize,
    pub text: String,
    pub is_first: bool,
    pub is_last: bool,
}

/// Split chapter content into reading chunks.
///
/// Strategy (in priority order):
/// 1. Accumulate whole paragraphs (separated by `\n\n`) until chunk_size
/// 2. Oversized paragraphs are split at sentence boundaries (。！？ / . ! ? + space)
/// 3. Sentences still exceeding chunk_size are hard-cut by character count
pub fn split(content: &str, chunk_size: usize) -> Vec<ReadingChunk> {
    let raw = build_chunks(content, chunk_size);
    let total = raw.len();
    raw.into_iter()
        .enumerate()
        .map(|(i, text)| ReadingChunk {
            index: i,
            text,
            is_first: i == 0,
            is_last: i + 1 == total,
        })
        .collect()
}

fn build_chunks(content: &str, chunk_size: usize) -> Vec<String> {
    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();

    for para in content.split("\n\n").map(str::trim).filter(|s| !s.is_empty()) {
        let para_len = para.chars().count();

        if para_len <= chunk_size {
            let sep_cost = if current.is_empty() { 0 } else { 2 };
            if !current.is_empty() && current.chars().count() + sep_cost + para_len > chunk_size {
                flush(&mut current, &mut chunks);
            }
            if !current.is_empty() {
                current.push_str("\n\n");
            }
            current.push_str(para);
        } else {
            // Paragraph too large — flush current, then split by sentence
            flush(&mut current, &mut chunks);
            for sentence in split_sentences(para) {
                let s_len = sentence.chars().count();
                if s_len > chunk_size {
                    flush(&mut current, &mut chunks);
                    hard_cut(&sentence, chunk_size, &mut chunks);
                } else {
                    if !current.is_empty() && current.chars().count() + s_len > chunk_size {
                        flush(&mut current, &mut chunks);
                    }
                    // Sentences are contiguous — no separator
                    current.push_str(&sentence);
                }
            }
            flush(&mut current, &mut chunks);
        }
    }

    flush(&mut current, &mut chunks);

    if chunks.is_empty() {
        chunks.push(String::new());
    }

    chunks
}

/// Split text at Chinese (。！？) and English (. ! ? followed by space/end) sentence boundaries.
/// Trailing closing quotes/brackets are kept with the sentence.
fn split_sentences(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut sentences: Vec<String> = Vec::new();
    let mut start = 0;
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        let is_cn_end = matches!(c, '。' | '！' | '？');
        let is_en_end = matches!(c, '.' | '!' | '?')
            && matches!(
                chars.get(i + 1),
                None | Some(' ') | Some('\n') | Some('\r')
            );

        if is_cn_end || is_en_end {
            let mut end = i + 1;
            // Consume trailing closing quotes / brackets
            while end < chars.len()
                && matches!(
                    chars[end],
                    // ASCII and Unicode closing quotes / brackets
                    '\'' | '"' | ')' | ']'
                    | '\u{2018}' // '
                    | '\u{2019}' // '
                    | '\u{201C}' // "
                    | '\u{201D}' // "
                    | '\u{300D}' // 」
                    | '\u{FF09}' // ）
                    | '\u{3011}' // 】
                    | '\u{300B}' // 》
                    | '\u{3009}' // 〉
                )
            {
                end += 1;
            }
            let sentence: String = chars[start..end].iter().collect();
            let trimmed = sentence.trim().to_string();
            if !trimmed.is_empty() {
                sentences.push(trimmed);
            }
            start = end;
            i = end;
        } else {
            i += 1;
        }
    }

    if start < chars.len() {
        let remaining: String = chars[start..].iter().collect();
        let trimmed = remaining.trim().to_string();
        if !trimmed.is_empty() {
            sentences.push(trimmed);
        }
    }

    sentences
}

fn flush(current: &mut String, chunks: &mut Vec<String>) {
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        chunks.push(trimmed);
    }
    current.clear();
}

fn hard_cut(text: &str, chunk_size: usize, out: &mut Vec<String>) {
    let chars: Vec<char> = text.chars().collect();
    let mut start = 0;
    while start < chars.len() {
        let end = (start + chunk_size).min(chars.len());
        out.push(chars[start..end].iter().collect());
        start = end;
    }
}

pub fn chapter_percent(chunk_index: usize, total_chunks: usize) -> f64 {
    if total_chunks == 0 {
        return 100.0;
    }
    ((chunk_index + 1) as f64 / total_chunks as f64 * 100.0).min(100.0)
}

pub fn book_percent(
    chapter_index: usize,
    chunk_index: usize,
    total_chunks_in_chapter: usize,
    total_chapters: usize,
) -> f64 {
    if total_chapters == 0 {
        return 100.0;
    }
    let chapter_progress = if total_chunks_in_chapter == 0 {
        1.0
    } else {
        (chunk_index + 1) as f64 / total_chunks_in_chapter as f64
    };
    ((chapter_index as f64 + chapter_progress) / total_chapters as f64 * 100.0).min(100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_text_is_single_chunk() {
        let chunks = split("短文本内容", CHUNK_SIZE);
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].is_first);
        assert!(chunks[0].is_last);
    }

    #[test]
    fn multi_paragraph_splits_into_multiple_chunks() {
        let para = "甲".repeat(200);
        let content = format!("{}\n\n{}\n\n{}", para, para, para);
        let chunks = split(&content, CHUNK_SIZE);
        assert!(chunks.len() >= 2);
        for chunk in &chunks {
            assert!(chunk.text.chars().count() <= CHUNK_SIZE + 4);
        }
    }

    #[test]
    fn oversized_paragraph_splits_at_sentence_boundary() {
        // Two sentences, each 350 chars — should become 2 chunks, not hard-cut mid-word
        let s1 = format!("{}。", "甲".repeat(349));
        let s2 = format!("{}。", "乙".repeat(349));
        let para = format!("{}{}", s1, s2);
        let chunks = split(&para, CHUNK_SIZE);
        assert_eq!(chunks.len(), 2);
        assert!(chunks[0].text.ends_with('。'));
        assert!(chunks[1].text.ends_with('。'));
    }

    #[test]
    fn oversized_single_sentence_is_hard_cut() {
        // No punctuation — must hard-cut
        let long = "乙".repeat(1500);
        let chunks = split(&long, CHUNK_SIZE);
        assert_eq!(chunks.len(), 3);
        for chunk in &chunks {
            assert!(chunk.text.chars().count() <= CHUNK_SIZE);
        }
    }

    #[test]
    fn english_sentence_boundary_respected() {
        let s1 = format!("{} end.", "a".repeat(350));
        let s2 = format!("{} end.", "b".repeat(350));
        let para = format!("{} {}", s1, s2);
        let chunks = split(&para, CHUNK_SIZE);
        assert!(chunks.len() >= 2);
        assert!(chunks[0].text.ends_with('.'));
    }

    #[test]
    fn chunk_index_out_of_bounds_returns_none() {
        let chunks = split("短文本", CHUNK_SIZE);
        assert!(chunks.get(99).is_none());
    }

    #[test]
    fn is_first_and_is_last_single_chunk() {
        let chunks = split("只有一段", CHUNK_SIZE);
        assert!(chunks[0].is_first);
        assert!(chunks[0].is_last);
    }

    #[test]
    fn is_first_and_is_last_multi_chunk() {
        let para = "丙".repeat(400);
        let content = format!("{}\n\n{}", para, para);
        let chunks = split(&content, CHUNK_SIZE);
        assert!(chunks.len() >= 2);
        assert!(chunks.first().unwrap().is_first);
        assert!(!chunks.first().unwrap().is_last);
        assert!(!chunks.last().unwrap().is_first);
        assert!(chunks.last().unwrap().is_last);
    }

    #[test]
    fn chapter_percent_first_chunk() {
        assert!((chapter_percent(0, 5) - 20.0).abs() < 0.01);
    }

    #[test]
    fn chapter_percent_last_chunk() {
        assert!((chapter_percent(4, 5) - 100.0).abs() < 0.01);
    }

    #[test]
    fn chapter_percent_in_range() {
        for i in 0..10 {
            let pct = chapter_percent(i, 10);
            assert!((0.0..=100.0).contains(&pct));
        }
    }

    #[test]
    fn book_percent_in_range() {
        for ch in 0..5usize {
            for ck in 0..3usize {
                let pct = book_percent(ch, ck, 3, 5);
                assert!((0.0..=100.0).contains(&pct), "pct={pct}");
            }
        }
    }

    #[test]
    fn book_percent_last_chapter_last_chunk() {
        assert!((book_percent(4, 2, 3, 5) - 100.0).abs() < 0.01);
    }

    #[test]
    fn empty_content_returns_one_empty_chunk() {
        let chunks = split("", CHUNK_SIZE);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].text, "");
    }

    #[test]
    fn closing_quote_stays_with_sentence() {
        // \u{201D} = RIGHT DOUBLE QUOTATION MARK "
        let s1 = format!("{}\u{3002}\u{201D}", "甲".repeat(349)); // 349甲。"
        let s2 = format!("{}\u{3002}", "乙".repeat(349));          // 349乙。
        let para = format!("{}{}", s1, s2);
        let chunks = split(&para, CHUNK_SIZE);
        assert!(chunks[0].text.ends_with('\u{201D}'));
    }
}
