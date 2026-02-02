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
use crate::extractor::{ExtractionResult, LanguageExtractor};

/// Get the appropriate extractor for a file based on its extension
pub fn get_extractor(path: &PathBuf) -> Option<Box<dyn LanguageExtractor>> {
    let ext = path.extension()?.to_str()?;
    
    // Create a parser pool for the extractors that need it
    let parser_pool = crate::parser_pool::create_parser_pool();
    
    match ext {
        "rs" => Some(Box::new(rust::RustExtractor::new(parser_pool))),
        "ts" | "tsx" => Some(Box::new(typescript::TypeScriptExtractor::new(parser_pool))),
        "js" | "jsx" => Some(Box::new(javascript::JavaScriptExtractor::new(parser_pool.clone()))),
        "py" => Some(Box::new(python::PythonExtractor::new(parser_pool.clone()))),
        "go" => Some(Box::new(go::GoExtractor::new(parser_pool.clone()))),
        "java" => Some(Box::new(java::JavaExtractor::new(parser_pool.clone()))),
        "c" => Some(Box::new(c::CExtractor::new(parser_pool.clone()))),
        "cpp" | "cc" | "cxx" | "c++" => Some(Box::new(cpp::CppExtractor::new(parser_pool.clone()))),
        _ => Some(Box::new(generic::GenericExtractor::new(parser_pool.clone()))),
    }
}