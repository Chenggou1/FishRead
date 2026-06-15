pub mod epub;
pub mod model;

use std::path::Path;

pub use model::{ImportWarning, NormalizedBook, NormalizedChapter};

pub trait BookImporter {
    fn import(&self, path: &Path) -> anyhow::Result<NormalizedBook>;
}
