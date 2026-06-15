pub mod chunk;
pub mod service;

pub use chunk::{split, ReadingChunk, CHUNK_SIZE};
pub use service::ReaderService;
