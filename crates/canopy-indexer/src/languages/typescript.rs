//! TypeScript language extractor using tree-sitter

use super::{ExtractionResult, LanguageExtractor};
use canopy_core::{GraphNode, GraphEdge, NodeKind, EdgeSource, Language, NodeId, EdgeId};
use std::path::PathBuf;
use tree_sitter::{Node, Point};
use anyhow::Result;
use crate::parser_pool::{ParserPool, ParseRequest, FileType};

pub struct TypeScriptExtractor {
    parser_pool: ParserPool,
}

impl TypeScriptExtractor {
    pub fn new(parser_pool: ParserPool) -> Self {
        Self { parser_pool }
    }
    
    fn point_to_u32(point: Point) -> u32 {
        (point.row as u32) + 1
    }
    
    fn extract_function(&self, node: Node, source: &[u8], path: &PathBuf) -> Option<GraphNode> {
        if node.kind() == "function_declaration" || node.kind() == "method_definition" {
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
                        language: Some(Language::TypeScript),
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
                        language: Some(Language::TypeScript),
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
    
    fn extract_imports(&self, node: Node, source: &[u8]) -> Vec<String> {
        let mut imports = Vec::new();
        
        if node.kind() == "import_statement" {
            // Walk through the import statement to find module names
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "string" {
                    if let Ok(module) = child.utf8_text(source) {
                        imports.push(module.trim_matches('"').trim_matches('\'').to_string());
                    }
                }
            }
        }
        
        imports
    }
}

impl LanguageExtractor for TypeScriptExtractor {
    fn extract(&self, path: &PathBuf, content: &[u8]) -> Result<ExtractionResult> {
        let source_code = std::str::from_utf8(content)?;
        
        // Use the parser pool to parse the content
        // Since LanguageExtractor is not async, we use block_in_place to handle the async call
        let request = ParseRequest {
            file_type: FileType::TypeScript,
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
            extractor: &TypeScriptExtractor,
        ) {
            // Extract functions
            if let Some(function) = extractor.extract_function(node, source.as_bytes(), path) {
                nodes.push(function);
            }
            
            // Extract classes
            if let Some(class) = extractor.extract_class(node, source.as_bytes(), path) {
                nodes.push(class);
            }
            
            // Extract imports
            imports.extend(extractor.extract_imports(node, source.as_bytes()));
            
            // Visit children
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                visit_node(child, source, path, nodes, edges, imports, extractor);
            }
        }
        
        visit_node(root_node, source_code, path, &mut nodes, &mut edges, &mut import_modules, self);
        
        // Create edges for imports
        for import in import_modules {
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
        
        Ok(ExtractionResult { nodes, edges })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_extract_typescript() {
        let parser_pool = crate::parser_pool::create_parser_pool();
        let extractor = TypeScriptExtractor::new(parser_pool);
        let code = r#"
import { UserService } from './services/user';
import * as utils from './utils';

export class UserController {
    private service: UserService;
    
    constructor(service: UserService) {
        this.service = service;
    }
    
    getUser(id: string): User {
        return this.service.findById(id);
    }
}

export function createController(service: UserService): UserController {
    return new UserController(service);
}
"#;
        
        let path = PathBuf::from("test.ts");
        let result = extractor.extract(&path, code.as_bytes()).unwrap();
        
        // Should extract 1 class, 2 methods, and 1 function
        assert_eq!(result.nodes.len(), 4);
        assert_eq!(result.edges.len(), 2); // 2 imports
    }
}