//! Python language extractor using tree-sitter

use super::{ExtractionResult, LanguageExtractor};
use canopy_core::{GraphNode, GraphEdge, NodeKind, EdgeKind, EdgeSource, Language, NodeId, EdgeId};
use std::path::PathBuf;
use tree_sitter::{Node, Point};
use anyhow::Result;
use crate::parser_pool::{ParserPool, ParseRequest, FileType};

pub struct PythonExtractor {
    parser_pool: ParserPool,
}

impl PythonExtractor {
    pub fn new(parser_pool: ParserPool) -> Self {
        Self { parser_pool }
    }

    fn point_to_u32(point: Point) -> u32 {
        (point.row as u32) + 1
    }
    
    fn extract_function(&self, node: Node, source: &[u8], path: &PathBuf) -> Option<GraphNode> {
        if node.kind() == "function_definition" {
            if let Some(name_node) = node.child_by_field_name("name") {
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
                        language: Some(Language::Python),
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
        if node.kind() == "class_definition" {
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
                        language: Some(Language::Python),
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
    
    fn extract_method(&self, node: Node, source: &[u8], path: &PathBuf, class_name: Option<&str>) -> Option<GraphNode> {
        if node.kind() == "function_definition" {
            if let Some(name_node) = node.child_by_field_name("name") {
                if let Ok(name) = name_node.utf8_text(source) {
                    let start_pos = Self::point_to_u32(node.start_position());
                    let end_pos = Self::point_to_u32(node.end_position());
                    
                    let qualified_name = if let Some(class) = class_name {
                        format!("{}::{}::{}", path.display(), class, name)
                    } else {
                        format!("{}::{}", path.display(), name)
                    };
                    
                    return Some(GraphNode {
                        id: NodeId(0), // Will be set by graph
                        kind: NodeKind::Method,
                        name: name.to_string(),
                        qualified_name,
                        file_path: path.clone(),
                        line_start: Some(start_pos),
                        line_end: Some(end_pos),
                        language: Some(Language::Python),
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
    
    fn extract_imports(&self, node: Node, source: &[u8]) -> Vec<String> {
        let mut imports = Vec::new();
        
        if node.kind() == "import_statement" {
            // Extract module name from import statement
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "dotted_name" || child.kind() == "aliased_import" {
                    if let Ok(module) = child.utf8_text(source) {
                        imports.push(module.split_whitespace().next().unwrap_or("").to_string());
                    }
                }
            }
        } else if node.kind() == "import_from_statement" {
            // Extract module name from "from module import" statement
            if let Some(module_node) = node.child_by_field_name("module_name") {
                if let Ok(module) = module_node.utf8_text(source) {
                    imports.push(module.to_string());
                }
            }
        }
        
        imports
    }
    
    fn extract_decorators(&self, node: Node, source: &[u8]) -> Vec<String> {
        let mut decorators = Vec::new();
        
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "decorator" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    if let Ok(name) = name_node.utf8_text(source) {
                        decorators.push(name.to_string());
                    }
                }
            }
        }
        
        decorators
    }
}

impl LanguageExtractor for PythonExtractor {
    fn extract(&self, path: &PathBuf, content: &[u8]) -> Result<ExtractionResult> {
        let source_code = std::str::from_utf8(content)?;
        
        // Use the parser pool to parse the content
        let request = ParseRequest {
            file_type: FileType::Python,
            content: source_code.to_string(),
            path: path.clone(),
        };
        
        let parse_result = self.parser_pool.parse_blocking(request)?;
        let tree = parse_result.tree;
        
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut import_modules = Vec::new();
        
        // Walk the AST
        let root_node = tree.root_node();
        
        fn visit_node(
            node: Node,
            source: &str,
            path: &PathBuf,
            nodes: &mut Vec<GraphNode>,
            edges: &mut Vec<GraphEdge>,
            imports: &mut Vec<String>,
            extractor: &PythonExtractor,
            in_class: bool,
            class_name: Option<&str>,
        ) {
            // Extract functions at module level
            if !in_class {
                if let Some(function) = extractor.extract_function(node, source.as_bytes(), path) {
                    nodes.push(function);
                }
            }
            
            // Extract classes
            if node.kind() == "class_definition" {
                if let Some(class) = extractor.extract_class(node, source.as_bytes(), path) {
                    let class_name = class.name.clone();
                    nodes.push(class);
                    
                    // Extract methods within the class
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        if let Some(method) = extractor.extract_method(child, source.as_bytes(), path, Some(&class_name)) {
                            nodes.push(method);
                        }
                        
                        // Recursively visit class body
                        if child.kind() == "block" {
                            visit_node(child, source, path, nodes, edges, imports, extractor, true, Some(&class_name));
                        }
                    }
                }
            }
            
            // Extract imports
            imports.extend(extractor.extract_imports(node, source.as_bytes()));
            
            // Visit children (except in class body which we handled above)
            if node.kind() != "class_definition" {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    visit_node(child, source, path, nodes, edges, imports, extractor, in_class, class_name);
                }
            }
        }
        
        // Start visiting from root
        visit_node(root_node, source_code, path, &mut nodes, &mut edges, &mut import_modules, self, false, None);
        
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