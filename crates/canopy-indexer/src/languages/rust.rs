//! Rust language extractor using tree-sitter

use super::{ExtractionResult, LanguageExtractor};
use canopy_core::{GraphNode, GraphEdge, NodeKind, EdgeSource, Language, NodeId, EdgeId};
use std::path::PathBuf;
use tree_sitter::{Node, Point};
use anyhow::Result;
use crate::parser_pool::{ParserPool, ParseRequest, FileType};

pub struct RustExtractor {
    parser_pool: ParserPool,
}

impl RustExtractor {
    pub fn new(parser_pool: ParserPool) -> Self {
        Self { parser_pool }
    }
    
    fn point_to_u32(point: Point) -> u32 {
        (point.row as u32) + 1
    }
    
    fn extract_function(&self, node: Node, source: &[u8], path: &PathBuf) -> Option<GraphNode> {
        if node.kind() == "function_item" || node.kind() == "method_definition" {
            // Find the identifier node
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "identifier" {
                    if let Ok(name) = child.utf8_text(source) {
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
                            language: Some(Language::Rust),
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
    
    fn extract_struct(&self, node: Node, source: &[u8], path: &PathBuf) -> Option<GraphNode> {
        if node.kind() == "struct_item" {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "type_identifier" || child.kind() == "identifier" {
                    if let Ok(name) = child.utf8_text(source) {
                        let start_pos = Self::point_to_u32(node.start_position());
                        let end_pos = Self::point_to_u32(node.end_position());
                        
                        return Some(GraphNode {
                            id: NodeId(0), // Will be set by graph
                            kind: NodeKind::Class,
                            name: name.to_string(),
                            qualified_name: format!("{}::{}", path.display(), name),
                            file_path: path.clone(),
                            line_start: Some(start_pos),
                            line_end: Some(end_pos),
                            language: Some(Language::Rust),
                            is_container: true,
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
    
    fn extract_impl_block(&self, node: Node, source: &[u8], path: &PathBuf) -> Vec<GraphNode> {
        let mut methods = Vec::new();
        
        if node.kind() == "impl_item" {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "declaration_list" {
                    let mut method_cursor = child.walk();
                    for method in child.children(&mut method_cursor) {
                        if let Some(method_node) = self.extract_function(method, source, path) {
                            methods.push(method_node);
                        }
                    }
                }
            }
        }
        
        methods
    }
    
    fn extract_use_statement(&self, node: Node, source: &[u8]) -> Vec<String> {
        let mut imports = Vec::new();
        
        if node.kind() == "use_declaration" {
            // Extract the path from use statement
            if let Some(path_node) = node.child_by_field_name("argument") {
                if let Some(path) = self.extract_use_path(path_node, source) {
                    imports.push(path);
                }
            }
        }
        
        imports
    }
    
    fn extract_use_path(&self, node: Node, source: &[u8]) -> Option<String> {
        match node.kind() {
            "scoped_identifier" | "identifier" => {
                node.utf8_text(source).ok().map(|s| s.to_string())
            }
            "use_wildcard" => {
                if let Some(prefix) = node.child_by_field_name("prefix") {
                    prefix.utf8_text(source).ok().map(|s| format!("{}::*", s))
                } else {
                    None
                }
            }
            _ => {
                // Recursively build the path
                let mut path_parts = Vec::new();
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if let Some(part) = self.extract_use_path(child, source) {
                        path_parts.push(part);
                    }
                }
                if path_parts.is_empty() {
                    None
                } else {
                    Some(path_parts.join("::"))
                }
            }
        }
    }
}

impl LanguageExtractor for RustExtractor {
    fn extract(&self, path: &PathBuf, content: &[u8]) -> Result<ExtractionResult> {
        let source_code = std::str::from_utf8(content)?;
        
        // Use the parser pool to parse the content
        // Since LanguageExtractor is not async, we use block_in_place to handle the async call
        let request = ParseRequest {
            file_type: FileType::Rust,
            content: source_code.to_string(),
            path: path.clone(),
        };
        
        let parse_result = self.parser_pool.parse_blocking(request)?;
        let tree = parse_result.tree;
        
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut imports = Vec::new();
        
        // Walk the AST
        let root_node = tree.root_node();
        
        fn visit_node(
            node: Node,
            source: &str,
            path: &PathBuf,
            nodes: &mut Vec<GraphNode>,
            edges: &mut Vec<GraphEdge>,
            imports: &mut Vec<String>,
            extractor: &RustExtractor,
        ) {
            // Extract functions
            if let Some(function) = extractor.extract_function(node, source.as_bytes(), path) {
                nodes.push(function);
            }
            
            // Extract structs
            if let Some(struct_node) = extractor.extract_struct(node, source.as_bytes(), path) {
                nodes.push(struct_node);
            }
            
            // Extract impl methods
            if node.kind() == "impl_item" {
                let methods = extractor.extract_impl_block(node, source.as_bytes(), path);
                nodes.extend(methods);
            }
            
            // Extract imports
            imports.extend(extractor.extract_use_statement(node, source.as_bytes()));
            
            // Visit children
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                visit_node(child, source, path, nodes, edges, imports, extractor);
            }
        }
        
        visit_node(root_node, source_code, path, &mut nodes, &mut edges, &mut imports, self);
        
        // Create edges for imports
        for import in imports {
            edges.push(GraphEdge {
                id: EdgeId(0), // Will be set by graph
                source: NodeId(0), // Will be set when added to graph
                target: NodeId(0), // Will be set when added to graph
                kind: canopy_core::EdgeKind::Imports,
                edge_source: EdgeSource::Heuristic,
                confidence: 1.0,
                label: Some(format!("uses {}", import)),
                file_path: Some(path.clone()),
                line: None,
            });
        }
        
        Ok(ExtractionResult { nodes, edges })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_extract_rust() {
        let parser_pool = crate::parser_pool::create_parser_pool();
        let extractor = RustExtractor::new(parser_pool);
        let code = r#"
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

pub struct User {
    pub id: u64,
    pub name: String,
}

impl User {
    pub fn new(id: u64, name: String) -> Self {
        User { id, name }
    }
    
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

pub fn create_user(id: u64, name: String) -> User {
    User::new(id, name)
}
"#;
        
        let path = PathBuf::from("test.rs");
        let result = extractor.extract(&path, code.as_bytes()).unwrap();
        
        // Should extract 1 struct, 2 methods, 2 functions, 1 impl block
        assert_eq!(result.nodes.len(), 6);
        assert_eq!(result.edges.len(), 2); // 2 imports
    }
}