//! TypeScript language extractor using tree-sitter

use super::{ExtractionResult, LanguageExtractor};
use canopy_core::{GraphNode, GraphEdge, NodeKind, EdgeSource, Language, NodeId, EdgeId};
use std::path::PathBuf;
use tree_sitter::{Parser, Language as TSLanguage, Node, Point};
use anyhow::Result;

pub struct TypeScriptExtractor;

impl TypeScriptExtractor {
    fn get_language() -> TSLanguage {
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
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
        let mut parser = Parser::new();
        parser.set_language(&Self::get_language())?;
        
        let source_code = std::str::from_utf8(content)?;
        let tree = parser.parse(source_code, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse TypeScript file"))?;
        
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut import_modules = Vec::new();
        
        // Walk the AST
        let root_node = tree.root_node();
        let mut cursor = root_node.walk();
        
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
    
    #[test]
    fn test_extract_typescript() {
        let extractor = TypeScriptExtractor;
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
        
        // Should extract 1 class and 2 functions
        assert_eq!(result.nodes.len(), 3);
        assert_eq!(result.edges.len(), 2); // 2 imports
    }
}