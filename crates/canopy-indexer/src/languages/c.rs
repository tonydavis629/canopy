//! C language extractor using tree-sitter

use super::{ExtractionResult, LanguageExtractor};
use canopy_core::{GraphNode, GraphEdge, NodeKind, EdgeKind, EdgeSource, Language, NodeId, EdgeId};
use std::path::PathBuf;
use tree_sitter::{Node, Point};
use anyhow::Result;
use crate::parser_pool::{ParserPool, ParseRequest, FileType};

pub struct CExtractor {
    parser_pool: ParserPool,
}

impl CExtractor {
    pub fn new(parser_pool: ParserPool) -> Self {
        Self { parser_pool }
    }

    fn point_to_u32(point: Point) -> u32 {
        (point.row as u32) + 1
    }
    
    fn extract_function(&self, node: Node, source: &[u8], path: &PathBuf) -> Option<GraphNode> {
        if node.kind() == "function_definition" {
            if let Some(declarator) = node.child_by_field_name("declarator") {
                // Find the function name in the declarator
                let mut cursor = declarator.walk();
                for child in declarator.children(&mut cursor) {
                    if child.kind() == "function_declarator" {
                        if let Some(name_node) = child.child_by_field_name("declarator") {
                            if let Ok(name) = name_node.utf8_text(source) {
                                let start_pos = Self::point_to_u32(node.start_position());
                                let end_pos = Self::point_to_u32(node.end_position());
                                
                                return Some(GraphNode {
                                    id: NodeId(0), // Will be set by graph
                                    kind: NodeKind::Function,
                                    name: name.to_string(),
                                    qualified_name: format!("{}::{}", path.display(), name),
                                    file_path: path.clone(),
                                    line_start: Some(start_pos),
                                    line_end: Some(end_pos),
                                    language: Some(Language::C),
                                    is_container: false,
                                    child_count: 0,
                                    loc: Some(((end_pos - start_pos) as usize) as u32),
                                    metadata: std::collections::HashMap::new(),
                                });
                            }
                        }
                    }
                }
            }
        }
        None
    }
    
    fn extract_struct(&self, node: Node, source: &[u8], path: &PathBuf) -> Option<GraphNode> {
        if node.kind() == "struct_specifier" {
            if let Some(name_node) = node.child_by_field_name("name") {
                if let Ok(name) = name_node.utf8_text(source) {
                    let start_pos = Self::point_to_u32(node.start_position());
                    let end_pos = Self::point_to_u32(node.end_position());
                    
                    return Some(GraphNode {
                        id: NodeId(0), // Will be set by graph
                        kind: NodeKind::Struct,
                        name: name.to_string(),
                        qualified_name: format!("{}::{}", path.display(), name),
                        file_path: path.clone(),
                        line_start: Some(start_pos),
                        line_end: Some(end_pos),
                        language: Some(Language::C),
                        is_container: true,
                        child_count: 0,
                        loc: Some(((end_pos - start_pos) as usize) as u32),
                        metadata: std::collections::HashMap::new(),
                    });
                }
            }
        }
        None
    }
    
    fn extract_typedef(&self, node: Node, source: &[u8], path: &PathBuf) -> Option<GraphNode> {
        if node.kind() == "type_definition" {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "type_identifier" {
                    if let Ok(name) = child.utf8_text(source) {
                        let start_pos = Self::point_to_u32(node.start_position());
                        let end_pos = Self::point_to_u32(node.end_position());
                        
                        return Some(GraphNode {
                            id: NodeId(0), // Will be set by graph
                            kind: NodeKind::TypeAlias,
                            name: name.to_string(),
                            qualified_name: format!("{}::{}", path.display(), name),
                            file_path: path.clone(),
                            line_start: Some(start_pos),
                            line_end: Some(end_pos),
                            language: Some(Language::C),
                            is_container: false,
                            child_count: 0,
                            loc: Some(((end_pos - start_pos) as usize) as u32),
                            metadata: std::collections::HashMap::new(),
                        });
                    }
                }
            }
        }
        None
    }
    
    fn extract_enum(&self, node: Node, source: &[u8], path: &PathBuf) -> Option<GraphNode> {
        if node.kind() == "enum_specifier" {
            if let Some(name_node) = node.child_by_field_name("name") {
                if let Ok(name) = name_node.utf8_text(source) {
                    let start_pos = Self::point_to_u32(node.start_position());
                    let end_pos = Self::point_to_u32(node.end_position());
                    
                    return Some(GraphNode {
                        id: NodeId(0), // Will be set by graph
                        kind: NodeKind::Enum,
                        name: name.to_string(),
                        qualified_name: format!("{}::{}", path.display(), name),
                        file_path: path.clone(),
                        line_start: Some(start_pos),
                        line_end: Some(end_pos),
                        language: Some(Language::C),
                        is_container: true,
                        child_count: 0,
                        loc: Some(((end_pos - start_pos) as usize) as u32),
                        metadata: std::collections::HashMap::new(),
                    });
                }
            }
        }
        None
    }
    
    fn extract_include(&self, node: Node, source: &[u8]) -> Vec<String> {
        let mut includes = Vec::new();
        
        if node.kind() == "preproc_include" {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "string_literal" || child.kind() == "system_lib_string" {
                    if let Ok(header) = child.utf8_text(source) {
                        // Remove quotes or angle brackets
                        includes.push(header.trim_matches('"').trim_matches('<').trim_matches('>').to_string());
                    }
                }
            }
        }
        
        includes
    }
}

impl LanguageExtractor for CExtractor {
    fn extract(&self, path: &PathBuf, content: &[u8]) -> Result<ExtractionResult> {
        let source_code = std::str::from_utf8(content)?;
        
        // Use the parser pool to parse the content
        let request = ParseRequest {
            file_type: FileType::C,
            content: source_code.to_string(),
            path: path.clone(),
        };
        
        let parse_result = self.parser_pool.parse_blocking(request)?;
        let tree = parse_result.tree;
        
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut include_files = Vec::new();
        
        // Walk the AST
        let root_node = tree.root_node();
        
        fn visit_node(
            node: Node,
            source: &str,
            path: &PathBuf,
            nodes: &mut Vec<GraphNode>,
            edges: &mut Vec<GraphEdge>,
            includes: &mut Vec<String>,
            extractor: &CExtractor,
        ) {
            // Extract functions
            if let Some(function) = extractor.extract_function(node, source.as_bytes(), path) {
                nodes.push(function);
            }
            
            // Extract structs
            if let Some(struct_type) = extractor.extract_struct(node, source.as_bytes(), path) {
                nodes.push(struct_type);
            }
            
            // Extract typedefs
            if let Some(typedef) = extractor.extract_typedef(node, source.as_bytes(), path) {
                nodes.push(typedef);
            }
            
            // Extract enums
            if let Some(enum_type) = extractor.extract_enum(node, source.as_bytes(), path) {
                nodes.push(enum_type);
            }
            
            // Extract includes
            includes.extend(extractor.extract_include(node, source.as_bytes()));
            
            // Visit children
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                visit_node(child, source, path, nodes, edges, includes, extractor);
            }
        }
        
        // Start visiting from root
        visit_node(root_node, source_code, path, &mut nodes, &mut edges, &mut include_files, self);
        
        // Create edges from includes to nodes
        for include in &include_files {
            for node in &nodes {
                // Simple heuristic: create a relationship
                edges.push(GraphEdge {
                    id: EdgeId(0), // Will be set by graph
                    source: NodeId(0), // Placeholder
                    target: node.id,
                    kind: EdgeKind::Imports,
                    edge_source: EdgeSource::Heuristic,
                    confidence: 0.5,
                    label: Some(format!("includes {}", include)),
                    file_path: Some(path.clone()),
                    line: node.line_start,
                });
            }
        }
        
        Ok(ExtractionResult { nodes, edges })
    }
}