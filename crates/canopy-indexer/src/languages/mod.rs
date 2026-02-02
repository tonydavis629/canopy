//! Language extractors for different programming languages

pub mod javascript;
pub mod python;
pub mod go;
pub mod java;
pub mod c;
pub mod cpp;
pub mod generic;
pub mod rust;
pub mod typescript;

use std::path::PathBuf;
use anyhow::Result;
use canopy_core::{GraphNode, GraphEdge};

/// Result of extracting symbols from a source file
#[derive(Debug, Clone)]
pub struct ExtractionResult {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

/// Trait for language-specific symbol extractors
pub trait LanguageExtractor {
    /// Extract symbols and relationships from source code
    fn extract(&self, path: &PathBuf, content: &[u8]) -> Result<ExtractionResult>;
}

/// Get the appropriate extractor for a file based on its extension
pub fn get_extractor(path: &PathBuf) -> Option<Box<dyn LanguageExtractor>> {
    let ext = path.extension()?.to_str()?;
    
    match ext {
        "rs" => Some(Box::new(rust::RustExtractor)),
        "ts" | "tsx" => Some(Box::new(typescript::TypeScriptExtractor)),
        "js" | "jsx" => Some(Box::new(javascript::JavaScriptExtractor)),
        "py" => Some(Box::new(python::pythonExtractor)),
        "go" => Some(Box::new(go::goExtractor)),
        "java" => Some(Box::new(java::javaExtractor)),
        "c" => Some(Box::new(c::cExtractor)),
        "cpp" | "cc" | "cxx" | "c++" => Some(Box::new(cpp::cppExtractor)),
        _ => Some(Box::new(generic::GenericExtractor)),
    }
}
