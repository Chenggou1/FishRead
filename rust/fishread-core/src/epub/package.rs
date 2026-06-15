use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use epub::doc::EpubDoc;

pub struct ParsedChapter {
    pub source_path: String,
    pub nav_title: Option<String>,
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
        let mut doc = EpubDoc::new(epub_path).context("failed to open EPUB")?;

        let title = doc.mdata("title").map(|m| m.value.clone());
        let author = doc.mdata("creator").map(|m| m.value.clone());
        let language = doc.mdata("language").map(|m| m.value.clone());
        let identifier = doc.unique_identifier.clone();

        if doc.spine.is_empty() {
            bail!("EPUB spine is empty");
        }

        // Build path → title map from the toc (walk nested navpoints too)
        let mut toc_map: HashMap<PathBuf, String> = HashMap::new();
        collect_toc(&doc.toc.clone(), &mut toc_map);

        // Snapshot spine + resource paths before the mutable borrow from get_resource_str
        let spine: Vec<(String, Option<PathBuf>)> = doc
            .spine
            .iter()
            .map(|s| {
                let path = doc.resources.get(&s.idref).map(|r| r.path.clone());
                (s.idref.clone(), path)
            })
            .collect();

        let mut chapters = Vec::new();
        for (idref, resource_path) in &spine {
            let xhtml = match doc.get_resource_str(idref) {
                Some((content, _mime)) => content,
                None => continue, // unreadable spine item — skip, importer will warn
            };

            // Match toc title: full path first, then filename fallback
            let nav_title = resource_path
                .as_ref()
                .and_then(|p| {
                    toc_map.get(p).or_else(|| {
                        let fname = p.file_name()?;
                        toc_map
                            .iter()
                            .find(|(k, _)| k.file_name() == Some(fname))
                            .map(|(_, v)| v)
                    })
                })
                .cloned();

            let source_path = resource_path
                .as_ref()
                .and_then(|p| p.to_str())
                .unwrap_or(idref)
                .to_owned();

            chapters.push(ParsedChapter {
                source_path,
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

fn collect_toc(navpoints: &[epub::doc::NavPoint], map: &mut HashMap<PathBuf, String>) {
    for np in navpoints {
        // First entry wins — top-level nav takes priority over nested duplicates
        map.entry(np.content.clone())
            .or_insert_with(|| np.label.clone());
        collect_toc(&np.children, map);
    }
}
