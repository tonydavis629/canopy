//! JavaScript language extractor using tree-sitter

use super::{ExtractionResult, LanguageExtractor};
use canopy_core::{GraphNode, GraphEdge, NodeKind, EdgeSource, Language, NodeId, EdgeId};
use std::path::PathBuf;
use tree_sitter::{Node, Point};
use anyhow::Result;
use crate::parser_pool::{ParserPool, ParseRequest, FileType};

pub struct JavaScriptExtractor {
    parser_pool: ParserPool,
}

impl JavaScriptExtractor {
    pub fn new(parser_pool: ParserPool) -> Self {
        Self { parser_pool }
    }
    
    fn point_to_u32(point: Point) -> u32 {
        (point.row as u32) + 1
    }
    
    fn extract_function(&self, node: Node, source: &[u8], path: &PathBuf) -> Option<GraphNode> {
        if node.kind() == "function_declaration" || 
           node.kind() == "function_expression" ||
           node.kind() == "arrow_function" ||
           node.kind() == "method_definition" {
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
                            language: Some(Language::JavaScript),
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
    
    fn extract_class(&self, node: Node, source: &[u8], path: &PathBuf) -> Option<GraphNode> {
        if node.kind() == "class_declaration" {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "identifier" {
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
                            language: Some(Language::JavaScript),
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
    
    fn extract_import(&self, node: Node, source: &[u8]) -> Vec<String> {
        let mut imports = Vec::new();
        
        if node.kind() == "import_statement" {
            // Handle different import patterns
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                match child.kind() {
                    "string" => {
                        if let Ok(module_name) = child.utf8_text(source) {
                            imports.push(module_name.trim_matches('"').trim_matches('\'').to_string());
                        }
                    }
                    "identifier" => {
                        if let Ok(name) = child.utf8_text(source) {
                            imports.push(name.to_string());
                        }
                    }
                    _ => {}
                }
            }
        }
        
        imports
    }
}

impl LanguageExtractor for JavaScriptExtractor {
    fn extract(&self, path: &PathBuf, content: &[u8]) -> Result<ExtractionResult> {
        let source_code = std::str::from_utf8(content)?;
        
        // Use the parser pool to parse the content
        let request = ParseRequest {
            file_type: FileType::JavaScript,
            content: source_code.to_string(),
            path: path.clone(),
        };
        
        let parse_result = self.parser_pool.parse_blocking(request)?;
        let tree = parse_result.tree;
        
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        
        // Walk the AST
        let root_node = tree.root_node();
        
        fn visit_node(
            node: Node,
            source: &str,
            path: &PathBuf,
            nodes: &mut Vec<GraphNode>,
            edges: &mut Vec<GraphEdge>,
            extractor: &JavaScriptExtractor,
        ) {
            // Extract functions
            if let Some(function) = extractor.extract_function(node, source.as_bytes(), path) {
                nodes.push(function);
            }
            
            // Extract classes
            if let Some(class_node) = extractor.extract_class(node, source.as_bytes(), path) {
                nodes.push(class_node);
            }
            
            // Extract imports
            let imports = extractor.extract_import(node, source.as_bytes());
            for import in imports {
                edges.push(GraphEdge {
                    id: EdgeId(0), // Will be set by graph
                    source: NodeId(0), // Will be set when added to graph
                    target: NodeId(0), // Will be set when added to graph
                    kind: canopy_core::EdgeKind::Imports,
                    edge_source: EdgeSource::Heuristic,
                    confidence: 1.0,
                    label: Some(format!("imports {}", import)),
                    file_path: Some(path.clone()),
                    line: None,
                });
            }
            
            // Visit children
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                visit_node(child, source, path, nodes, edges, extractor);
            }
        }
        
        visit_node(root_node, source_code, path, &mut nodes, &mut edges, self);
        
        Ok(ExtractionResult { nodes, edges })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_extract_javascript() {
        let parser_pool = crate::parser_pool::create_parser_pool();
        let extractor = JavaScriptExtractor::new(parser_pool);
        let code = r#"
import React from 'react';
import { useState, useEffect } from 'react';

class User {
    constructor(name) {
        this.name = name;
    }
    
    getName() {
        return this.name;
    }
}

function createUser(name) {
    return new User(name);
}

const arrowFunc = (x, y) => x + y;

export default createUser;
"#;
        
        let path = PathBuf::from("test.js");
        let result = extractor.extract(&path, code.as_bytes()).unwrap();
        
        // Should extract 1 class, 3 functions, 2 imports
        assert_eq!(result.nodes.len(), 4); // 1 class + 3 functions
        assert_eq!(result.edges.len(), 2); // 2 imports
    }
}