pub mod epub;
pub mod model;
pub mod service;

use std::path::Path;

pub use model::{ImportResult, ImportWarning, NormalizedBook, NormalizedChapter};
pub use service::ImportService;

pub trait BookImporter {
    fn import(&self, path: &Path) -> anyhow::Result<NormalizedBook>;
}
