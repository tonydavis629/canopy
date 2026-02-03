//! Language extractor trait definition

use std::path::PathBuf;
use canopy_core::{GraphNode, GraphEdge};

#[derive(Clone)]
pub struct ExtractionResult {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

pub trait LanguageExtractor: Send + Sync {
    fn extract(&self, path: &PathBuf, content: &[u8]) -> anyhow::Result<ExtractionResult>;
}
