use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use anyhow::{bail, Context};

pub struct ParsedChapter {
    /// Path relative to the OPF directory.
    pub source_path: String,
    /// Title from nav.xhtml TOC, if found.
    pub nav_title: Option<String>,
    /// Raw XHTML content.
    pub xhtml: String,
}

pub struct ParsedEpub {
    pub title: Option<String>,
    pub author: Option<String>,
    pub language: Option<String>,
    pub identifier: Option<String>,
    pub chapters: Vec<ParsedChapter>,
}

impl ParsedEpub {
    pub fn parse(epub_path: &Path) -> anyhow::Result<Self> {
        let file = std::fs::File::open(epub_path)
            .with_context(|| format!("cannot open {}", epub_path.display()))?;
        let mut archive = zip::ZipArchive::new(file).context("not a valid zip/epub file")?;

        // 1. container.xml → OPF path
        let opf_path = read_container_xml(&mut archive)?;
        let opf_dir = opf_path
            .rfind('/')
            .map(|i| opf_path[..i].to_owned())
            .unwrap_or_default();

        // 2. OPF → metadata + manifest + spine
        let opf_xml = read_entry(&mut archive, &opf_path)?;
        let opf_doc = roxmltree::Document::parse_with_options(
            &opf_xml,
            roxmltree::ParsingOptions {
                allow_dtd: true,
                ..Default::default()
            },
        )
        .context("failed to parse OPF document")?;

        let pkg = opf_doc.root_element();
        let dc = "http://purl.org/dc/elements/1.1/";

        let title = find_dc_text(&pkg, "title", dc);
        let author = find_dc_text(&pkg, "creator", dc);
        let language = find_dc_text(&pkg, "language", dc);
        let identifier = find_dc_text(&pkg, "identifier", dc);

        // manifest: id → href
        let manifest = parse_manifest(&pkg);

        // nav href (EPUB3) or toc ncx href (EPUB2 fallback)
        let nav_href = find_nav_href(&pkg, &manifest);

        // nav/toc titles: normalized-href → title
        let nav_titles = nav_href
            .as_deref()
            .and_then(|href| {
                let path = resolve_path(&opf_dir, href);
                read_entry(&mut archive, &path).ok()
            })
            .map(|xml| parse_nav_titles(&xml))
            .unwrap_or_default();

        // spine → ordered idrefs
        let spine = parse_spine(&pkg);
        if spine.is_empty() {
            bail!("EPUB spine is empty");
        }

        // Read XHTML for each spine item
        let mut chapters = Vec::new();
        for idref in &spine {
            let href = match manifest.get(idref) {
                Some(h) => h.clone(),
                None => continue,
            };
            let xhtml_path = resolve_path(&opf_dir, &href);
            let xhtml = match read_entry(&mut archive, &xhtml_path) {
                Ok(x) => x,
                Err(_) => continue,
            };
            // Look up title by href (try full relative href, then filename only)
            let nav_title = nav_titles
                .get(&href)
                .or_else(|| {
                    let fname = href.rsplit('/').next().unwrap_or(&href);
                    nav_titles.get(fname)
                })
                .cloned();

            chapters.push(ParsedChapter {
                source_path: href,
                nav_title,
                xhtml,
            });
        }

        Ok(Self {
            title,
            author,
            language,
            identifier,
            chapters,
        })
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

type Archive = zip::ZipArchive<std::fs::File>;

fn read_entry(archive: &mut Archive, path: &str) -> anyhow::Result<String> {
    let mut entry = archive
        .by_name(path)
        .with_context(|| format!("entry not found in epub: {path}"))?;
    let mut s = String::new();
    entry.read_to_string(&mut s)?;
    Ok(s)
}

fn read_container_xml(archive: &mut Archive) -> anyhow::Result<String> {
    let xml = read_entry(archive, "META-INF/container.xml")?;
    let doc = roxmltree::Document::parse_with_options(
        &xml,
        roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        },
    )
    .context("failed to parse container.xml")?;

    doc.descendants()
        .find(|n| n.tag_name().name() == "rootfile")
        .and_then(|n| n.attribute("full-path"))
        .map(str::to_owned)
        .context("container.xml has no rootfile full-path")
}

fn find_dc_text(pkg: &roxmltree::Node<'_, '_>, local: &str, ns: &str) -> Option<String> {
    pkg.descendants()
        .find(|n| n.tag_name().name() == local && n.tag_name().namespace() == Some(ns))
        .and_then(|n| n.text())
        .map(|t| t.trim().to_owned())
        .filter(|s| !s.is_empty())
}

fn parse_manifest(pkg: &roxmltree::Node<'_, '_>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Some(mf) = pkg.children().find(|n| n.tag_name().name() == "manifest") {
        for item in mf.children().filter(|n| n.tag_name().name() == "item") {
            if let (Some(id), Some(href)) = (item.attribute("id"), item.attribute("href")) {
                map.insert(id.to_owned(), href.to_owned());
            }
        }
    }
    map
}

fn find_nav_href(
    pkg: &roxmltree::Node<'_, '_>,
    manifest: &HashMap<String, String>,
) -> Option<String> {
    // EPUB3: item with properties="nav"
    if let Some(mf) = pkg.children().find(|n| n.tag_name().name() == "manifest") {
        for item in mf.children().filter(|n| n.tag_name().name() == "item") {
            if item
                .attribute("properties")
                .map(|p| p.split_whitespace().any(|v| v == "nav"))
                .unwrap_or(false)
            {
                if let Some(href) = item.attribute("href") {
                    return Some(href.to_owned());
                }
            }
        }
    }
    // EPUB2 fallback: spine toc attribute → manifest id
    if let Some(spine) = pkg.children().find(|n| n.tag_name().name() == "spine") {
        if let Some(toc_id) = spine.attribute("toc") {
            return manifest.get(toc_id).cloned();
        }
    }
    None
}

fn parse_spine(pkg: &roxmltree::Node<'_, '_>) -> Vec<String> {
    pkg.children()
        .find(|n| n.tag_name().name() == "spine")
        .map(|spine| {
            spine
                .children()
                .filter(|n| n.tag_name().name() == "itemref")
                .filter_map(|n| n.attribute("idref"))
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// Parse nav.xhtml (EPUB3) or toc.ncx (EPUB2) and return href → title map.
fn parse_nav_titles(xml: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    // Try EPUB3 nav.xhtml
    if let Ok(doc) = roxmltree::Document::parse_with_options(
        xml,
        roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        },
    ) {
        // <a href="...">title</a> inside <nav epub:type="toc">
        for a in doc.descendants().filter(|n| n.tag_name().name() == "a") {
            if let (Some(href), Some(text)) = (a.attribute("href"), a.text()) {
                let text = text.trim();
                if !text.is_empty() {
                    // strip fragment identifier
                    let href_base = href.split('#').next().unwrap_or(href);
                    map.insert(href_base.to_owned(), text.to_owned());
                }
            }
        }
        if !map.is_empty() {
            return map;
        }

        // EPUB2 toc.ncx: <navPoint><navLabel><text>...</text></navLabel><content src="..."/>
        for nav_point in doc
            .descendants()
            .filter(|n| n.tag_name().name() == "navPoint")
        {
            let label = nav_point
                .descendants()
                .find(|n| n.tag_name().name() == "text")
                .and_then(|n| n.text())
                .map(str::trim)
                .filter(|s| !s.is_empty());
            let src = nav_point
                .descendants()
                .find(|n| n.tag_name().name() == "content")
                .and_then(|n| n.attribute("src"))
                .map(|s| s.split('#').next().unwrap_or(s));

            if let (Some(title), Some(src)) = (label, src) {
                map.insert(src.to_owned(), title.to_owned());
            }
        }
    }

    map
}

/// Resolve an href relative to a base directory inside the zip.
pub fn resolve_path(base_dir: &str, href: &str) -> String {
    if href.starts_with('/') {
        return href.trim_start_matches('/').to_owned();
    }
    let combined = if base_dir.is_empty() {
        href.to_owned()
    } else {
        format!("{base_dir}/{href}")
    };
    // Normalize: collapse `..` and `.`
    let mut parts: Vec<&str> = Vec::new();
    for seg in combined.split('/') {
        match seg {
            ".." => {
                parts.pop();
            }
            "." | "" => {}
            s => parts.push(s),
        }
    }
    parts.join("/")
}
