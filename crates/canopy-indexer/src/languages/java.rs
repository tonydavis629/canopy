//! Java language extractor using tree-sitter

use super::{ExtractionResult, LanguageExtractor};
use canopy_core::{GraphNode, GraphEdge, NodeKind, EdgeKind, EdgeSource, Language, NodeId, EdgeId};
use std::path::PathBuf;
use tree_sitter::{Node, Point};
use anyhow::Result;
use crate::parser_pool::{ParserPool, ParseRequest, FileType};

pub struct JavaExtractor {
    parser_pool: ParserPool,
}

impl JavaExtractor {
    pub fn new(parser_pool: ParserPool) -> Self {
        Self { parser_pool }
    }

    fn point_to_u32(point: Point) -> u32 {
        (point.row as u32) + 1
    }
    
    fn extract_method(&self, node: Node, source: &[u8], path: &PathBuf) -> Option<GraphNode> {
        if node.kind() == "method_declaration" {
            if let Some(name_node) = node.child_by_field_name("name") {
                if let Ok(name) = name_node.utf8_text(source) {
                    let start_pos = Self::point_to_u32(node.start_position());
                    let end_pos = Self::point_to_u32(node.end_position());
                    
                    return Some(GraphNode {
                        id: NodeId(0), // Will be set by graph
                        kind: NodeKind::Method,
                        name: name.to_string(),
                        qualified_name: format!("{}::{}", path.display(), name),
                        file_path: path.clone(),
                        line_start: Some(start_pos),
                        line_end: Some(end_pos),
                        language: Some(Language::Java),
                        is_container: false,
                        child_count: 0,
                        loc: Some(((end_pos - start_pos) as usize) as u32),
                        metadata: std::collections::HashMap::new(),
                    });
                }
            }
        }
        None
    }
    
    fn extract_class(&self, node: Node, source: &[u8], path: &PathBuf) -> Option<GraphNode> {
        if node.kind() == "class_declaration" {
            if let Some(name_node) = node.child_by_field_name("name") {
                if let Ok(name) = name_node.utf8_text(source) {
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
                        language: Some(Language::Java),
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
    
    fn extract_interface(&self, node: Node, source: &[u8], path: &PathBuf) -> Option<GraphNode> {
        if node.kind() == "interface_declaration" {
            if let Some(name_node) = node.child_by_field_name("name") {
                if let Ok(name) = name_node.utf8_text(source) {
                    let start_pos = Self::point_to_u32(node.start_position());
                    let end_pos = Self::point_to_u32(node.end_position());
                    
                    return Some(GraphNode {
                        id: NodeId(0), // Will be set by graph
                        kind: NodeKind::Interface,
                        name: name.to_string(),
                        qualified_name: format!("{}::{}", path.display(), name),
                        file_path: path.clone(),
                        line_start: Some(start_pos),
                        line_end: Some(end_pos),
                        language: Some(Language::Java),
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
    
    fn extract_package(&self, node: Node, source: &[u8]) -> Option<String> {
        if node.kind() == "package_declaration" {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "scoped_identifier" || child.kind() == "identifier" {
                    if let Ok(name) = child.utf8_text(source) {
                        return Some(name.to_string());
                    }
                }
            }
        }
        None
    }
    
    fn extract_imports(&self, node: Node, source: &[u8]) -> Vec<String> {
        let mut imports = Vec::new();
        
        if node.kind() == "import_declaration" {
            // Extract the import path
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "scoped_identifier" || child.kind() == "asterisk" {
                    if let Ok(path) = child.utf8_text(source) {
                        imports.push(path.to_string());
                    }
                }
            }
        }
        
        imports
    }
}

impl LanguageExtractor for JavaExtractor {
    fn extract(&self, path: &PathBuf, content: &[u8]) -> Result<ExtractionResult> {
        let source_code = std::str::from_utf8(content)?;
        
        // Use the parser pool to parse the content
        let request = ParseRequest {
            file_type: FileType::Java,
            content: source_code.to_string(),
            path: path.clone(),
        };
        
        let parse_result = self.parser_pool.parse_blocking(request)?;
        let tree = parse_result.tree;
        
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut import_modules = Vec::new();
        let mut package_name = None;
        
        // Walk the AST
        let root_node = tree.root_node();
        
        fn visit_node(
            node: Node,
            source: &str,
            path: &PathBuf,
            nodes: &mut Vec<GraphNode>,
            edges: &mut Vec<GraphEdge>,
            imports: &mut Vec<String>,
            package_name: &mut Option<String>,
            extractor: &JavaExtractor,
        ) {
            // Extract package declaration
            if package_name.is_none() {
                if let Some(pkg) = extractor.extract_package(node, source.as_bytes()) {
                    *package_name = Some(pkg);
                }
            }
            
            // Extract classes
            if let Some(class) = extractor.extract_class(node, source.as_bytes(), path) {
                nodes.push(class);
            }
            
            // Extract interfaces
            if let Some(interface) = extractor.extract_interface(node, source.as_bytes(), path) {
                nodes.push(interface);
            }
            
            // Extract methods
            if let Some(method) = extractor.extract_method(node, source.as_bytes(), path) {
                nodes.push(method);
            }
            
            // Extract imports
            imports.extend(extractor.extract_imports(node, source.as_bytes()));
            
            // Visit children
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                visit_node(child, source, path, nodes, edges, imports, package_name, extractor);
            }
        }
        
        // Start visiting from root
        visit_node(root_node, source_code, path, &mut nodes, &mut edges, &mut import_modules, &mut package_name, self);
        
        // Create edges from imports to nodes
        for import in &import_modules {
            for node in &nodes {
                // Simple heuristic: if node name appears in import or vice versa
                if import.contains(&node.name) || node.name.contains(import) {
                    edges.push(GraphEdge {
                        id: EdgeId(0), // Will be set by graph
                        source: NodeId(0), // Placeholder - would need proper resolution
                        target: node.id,
                        kind: EdgeKind::Imports,
                        edge_source: EdgeSource::Heuristic,
                        confidence: 0.7,
                        label: Some(format!("imports {}", import)),
                        file_path: Some(path.clone()),
                        line: node.line_start,
                    });
                }
            }
        }
        
        Ok(ExtractionResult { nodes, edges })
    }
}