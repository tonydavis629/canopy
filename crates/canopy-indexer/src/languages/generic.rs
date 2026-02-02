//! Generic fallback extractor

use super::{ExtractionResult, LanguageExtractor};
use canopy_core::{GraphNode, GraphEdge};
use std::path::PathBuf;
use anyhow::Result;

pub struct GenericExtractor;

impl LanguageExtractor for GenericExtractor {
    fn extract(&self, _path: &PathBuf, _content: &[u8]) -> Result<ExtractionResult> {
        Ok(ExtractionResult {
            nodes: vec![],
            edges: vec![],
        })
    }
}