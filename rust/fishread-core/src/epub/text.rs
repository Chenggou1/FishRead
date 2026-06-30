use scraper::{ElementRef, Html, Node};

#[derive(Clone, Copy)]
enum PendingWhitespace {
    Soft,
    Hard,
}

const SKIP_TAGS: &[&str] = &[
    "script", "style", "noscript", "head", "iframe", "form", "input", "button", "svg", "canvas",
];

const BLOCK_TAGS: &[&str] = &[
    "p",
    "div",
    "section",
    "article",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "li",
    "blockquote",
    "tr",
];

/// Convert an XHTML/HTML string to plain text.
///
/// Block elements produce paragraphs; inline whitespace is folded so source
/// formatting does not become reader-visible line breaks.
/// Script/style/head and similar tags are skipped entirely.
pub fn xhtml_to_text(xhtml: &str) -> String {
    let doc = Html::parse_document(xhtml);
    let mut blocks = Vec::new();
    collect_blocks(doc.root_element(), &mut blocks);
    blocks.join("\n\n")
}

fn collect_blocks(elem: ElementRef<'_>, blocks: &mut Vec<String>) {
    if SKIP_TAGS.contains(&elem.value().name()) {
        return;
    }

    if BLOCK_TAGS.contains(&elem.value().name()) && !has_block_child(elem) {
        push_block(blocks, collect_inline_text(elem));
        return;
    }

    for child in elem.children().filter_map(ElementRef::wrap) {
        collect_blocks(child, blocks);
    }
}

fn has_block_child(elem: ElementRef<'_>) -> bool {
    elem.children().filter_map(ElementRef::wrap).any(|child| {
        let name = child.value().name();
        !SKIP_TAGS.contains(&name) && (BLOCK_TAGS.contains(&name) || has_block_child(child))
    })
}

fn collect_inline_text(elem: ElementRef<'_>) -> String {
    let mut out = String::new();
    let mut pending_whitespace = None;
    let mut last_char: Option<char> = None;
    append_inline_text(elem, &mut out, &mut pending_whitespace, &mut last_char);
    out.trim().to_owned()
}

fn append_inline_text(
    elem: ElementRef<'_>,
    out: &mut String,
    pending_whitespace: &mut Option<PendingWhitespace>,
    last_char: &mut Option<char>,
) {
    if SKIP_TAGS.contains(&elem.value().name()) {
        return;
    }

    for child in elem.children() {
        match child.value() {
            Node::Text(t) => append_text(t, out, pending_whitespace, last_char),
            Node::Element(_) => {
                if let Some(child_elem) = ElementRef::wrap(child) {
                    if child_elem.value().name() == "br" {
                        append_line_break(out, pending_whitespace, last_char);
                    } else {
                        append_inline_text(child_elem, out, pending_whitespace, last_char);
                    }
                }
            }
            _ => {}
        }
    }
}

fn append_text(
    text: &str,
    out: &mut String,
    pending_whitespace: &mut Option<PendingWhitespace>,
    last_char: &mut Option<char>,
) {
    for c in text.chars() {
        if c.is_whitespace() {
            let whitespace = if matches!(c, '\n' | '\r') {
                PendingWhitespace::Hard
            } else {
                PendingWhitespace::Soft
            };
            *pending_whitespace = Some(match (*pending_whitespace, whitespace) {
                (Some(PendingWhitespace::Hard), _) | (_, PendingWhitespace::Hard) => {
                    PendingWhitespace::Hard
                }
                _ => PendingWhitespace::Soft,
            });
            continue;
        }

        if pending_whitespace
            .map(|w| should_insert_space(w, *last_char, c))
            .unwrap_or(false)
            && !out.ends_with('\n')
        {
            out.push(' ');
            *last_char = Some(' ');
        }

        out.push(c);
        *last_char = Some(c);
        *pending_whitespace = None;
    }
}

fn append_line_break(
    out: &mut String,
    pending_whitespace: &mut Option<PendingWhitespace>,
    last_char: &mut Option<char>,
) {
    trim_trailing_spaces(out);
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
        *last_char = Some('\n');
    }
    *pending_whitespace = None;
}

fn trim_trailing_spaces(out: &mut String) {
    while out.ends_with(' ') {
        out.pop();
    }
}

fn should_insert_space(whitespace: PendingWhitespace, prev: Option<char>, next: char) -> bool {
    let Some(prev) = prev else {
        return false;
    };

    match whitespace {
        PendingWhitespace::Soft => true,
        PendingWhitespace::Hard => is_ascii_word(prev) && is_ascii_word(next),
    }
}

fn is_ascii_word(c: char) -> bool {
    c.is_ascii_alphanumeric()
}

fn push_block(blocks: &mut Vec<String>, text: String) {
    let text = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    if !text.is_empty() {
        blocks.push(text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_html_tags() {
        let html = "<html><body><h1>Title</h1><p>Para one.</p><p>Para two.</p></body></html>";
        let text = xhtml_to_text(html);
        assert!(!text.contains('<'), "should not contain HTML tags");
        assert_eq!(text, "Title\n\nPara one.\n\nPara two.");
    }

    #[test]
    fn skips_script_and_style() {
        let html =
            "<html><head><style>body{color:red}</style></head><body><script>alert(1)</script><p>Real content.</p></body></html>";
        let text = xhtml_to_text(html);
        assert!(!text.contains("color:red"));
        assert!(!text.contains("alert"));
        assert_eq!(text, "Real content.");
    }

    #[test]
    fn collapses_whitespace() {
        let html = "<html><body><p>  spaces   everywhere  </p></body></html>";
        let text = xhtml_to_text(html);
        assert_eq!(text, "spaces everywhere");
    }

    #[test]
    fn reflows_source_line_breaks_inside_chinese_paragraph() {
        let html = "<html><body><p>宣武军\n便有一个浮浪破落户子弟\n好脚气毬。</p></body></html>";
        let text = xhtml_to_text(html);
        assert_eq!(text, "宣武军便有一个浮浪破落户子弟好脚气毬。");
    }

    #[test]
    fn preserves_block_boundaries_as_paragraphs() {
        let html = "<html><body><p>第一段</p><p>第二段</p></body></html>";
        let text = xhtml_to_text(html);
        assert_eq!(text, "第一段\n\n第二段");
    }

    #[test]
    fn preserves_explicit_br_line_breaks() {
        let html = "<html><body><p>上句<br/>下句</p></body></html>";
        let text = xhtml_to_text(html);
        assert_eq!(text, "上句\n下句");
    }
}
