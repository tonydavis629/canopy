//! Go language extractor using tree-sitter

use super::{ExtractionResult, LanguageExtractor};
use canopy_core::GraphNode;
use std::path::PathBuf;
use anyhow::Result;
use crate::parser_pool::{ParserPool, ParseRequest, FileType};

pub struct GoExtractor {
    parser_pool: ParserPool,
}

impl GoExtractor {
    pub fn new(parser_pool: ParserPool) -> Self {
        Self { parser_pool }
    }
}

impl LanguageExtractor for GoExtractor {
    fn extract(&self, path: &PathBuf, content: &[u8]) -> Result<ExtractionResult> {
        let source_code = std::str::from_utf8(content)?;
        
        // Use the parser pool to parse the content
        let request = ParseRequest {
            file_type: FileType::Go,
            content: source_code.to_string(),
            path: path.clone(),
        };
        
        let _parse_result = self.parser_pool.parse_blocking(request)?;
        
        // TODO: Implement actual Go AST extraction
        Ok(ExtractionResult {
            nodes: vec![],
            edges: vec![],
        })
    }
}
