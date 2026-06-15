use scraper::{ElementRef, Html, Node};

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
    "br",
];

/// Convert an XHTML/HTML string to plain text.
///
/// Block elements produce newlines; inline elements are joined without separation.
/// Script/style/head and similar tags are skipped entirely.
pub fn xhtml_to_text(xhtml: &str) -> String {
    let doc = Html::parse_document(xhtml);
    let mut buf = String::new();
    walk(doc.root_element(), &mut buf);
    normalize_whitespace(&buf)
}

fn walk(elem: ElementRef<'_>, buf: &mut String) {
    if SKIP_TAGS.contains(&elem.value().name()) {
        return;
    }

    for child in elem.children() {
        match child.value() {
            Node::Text(t) => buf.push_str(t),
            Node::Element(_) => {
                if let Some(child_elem) = ElementRef::wrap(child) {
                    let name = child_elem.value().name();
                    if BLOCK_TAGS.contains(&name) {
                        buf.push('\n');
                        walk(child_elem, buf);
                        buf.push('\n');
                    } else {
                        walk(child_elem, buf);
                    }
                }
            }
            _ => {}
        }
    }
}

fn normalize_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut blank_lines = 0u32;

    for line in s.lines() {
        let trimmed = line.split_whitespace().collect::<Vec<_>>().join(" ");
        if trimmed.is_empty() {
            blank_lines += 1;
            if blank_lines == 1 {
                out.push('\n');
            }
        } else {
            blank_lines = 0;
            out.push_str(&trimmed);
            out.push('\n');
        }
    }

    out.trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_html_tags() {
        let html = "<html><body><h1>Title</h1><p>Para one.</p><p>Para two.</p></body></html>";
        let text = xhtml_to_text(html);
        assert!(!text.contains('<'), "should not contain HTML tags");
        assert!(text.contains("Title"));
        assert!(text.contains("Para one."));
        assert!(text.contains("Para two."));
    }

    #[test]
    fn skips_script_and_style() {
        let html =
            "<html><head><style>body{color:red}</style></head><body><script>alert(1)</script><p>Real content.</p></body></html>";
        let text = xhtml_to_text(html);
        assert!(!text.contains("color:red"));
        assert!(!text.contains("alert"));
        assert!(text.contains("Real content."));
    }

    #[test]
    fn collapses_whitespace() {
        let html = "<html><body><p>  spaces   everywhere  </p></body></html>";
        let text = xhtml_to_text(html);
        assert_eq!(text, "spaces everywhere");
    }
}
