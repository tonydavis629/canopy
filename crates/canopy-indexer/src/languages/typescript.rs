//! TypeScript language extractor

use super::{ExtractionResult, LanguageExtractor};
use canopy_core::{GraphNode, GraphEdge, NodeKind, EdgeSource, Language, NodeId};
use std::path::PathBuf;

pub struct TypeScriptExtractor;

impl LanguageExtractor for TypeScriptExtractor {
    fn extract(&self, _path: &PathBuf, _content: &[u8]) -> anyhow::Result<ExtractionResult> {
        // TODO: Implement tree-sitter TypeScript parsing
        Ok(ExtractionResult {
            nodes: vec![],
            edges: vec![],
        })
    }
}
