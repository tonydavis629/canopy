//! File parsing and symbol extraction

pub mod coordinator;
pub mod tree_cache;
pub mod extractor;
pub mod languages;
pub mod config;
pub mod heuristics;
pub mod parser_pool;

#[cfg(test)]
pub mod tests;

pub use parser_pool::{ParserPool, ParseResult, ParseRequest, FileType, FileParseResult};
pub use extractor::{ExtractionResult, LanguageExtractor};
