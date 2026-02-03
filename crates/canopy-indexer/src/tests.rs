//! Unit tests for canopy-indexer module

use canopy_indexer::languages::get_extractor;
use canopy_core::{GraphNode, NodeKind, Language};
use std::path::PathBuf;

#[test]
fn test_extractor_detection() {
    let test_cases = vec![
        ("main.rs", "rust"),
        ("app.ts", "typescript"),
        ("index.js", "javascript"),
        ("lib.py", "python"),
        ("main.go", "go"),
        ("Main.java", "java"),
        ("main.c", "c"),
        ("main.cpp", "cpp"),
        ("unknown.xyz", "generic"),
    ];
    
    for (filename, expected_type) in test_cases {
        let path = PathBuf::from(filename);
        let extractor = get_extractor(&path);
        
        assert!(extractor.is_some(), "Should have extractor for {}", filename);
        
        // Test that we can extract empty content without error
        let result = extractor.unwrap().extract(&path, b"");
        assert!(result.is_ok(), "Extractor failed for {}", filename);
    }
}

#[test]
fn test_rust_extraction() {
    use canopy_indexer::languages::get_extractor;
    
    let rust_code = r#"
fn main() {
    println!("Hello, world!");
}

fn helper() -> i32 {
    42
}

struct User {
    name: String,
}

impl User {
    fn new(name: String) -> Self {
        User { name }
    }
}
"#;
    
    let path = PathBuf::from("test.rs");
    let extractor = get_extractor(&path).unwrap();
    let result = extractor.extract(&path, rust_code.as_bytes()).unwrap();
    
    // Should extract at least the function and struct
    assert!(result.nodes.len() >= 2, "Should extract at least 2 nodes");
    
    // Check for function nodes
    let functions: Vec<_> = result.nodes.iter()
        .filter(|n| n.kind == NodeKind::Function)
        .collect();
    
    assert!(functions.len() >= 2, "Should extract at least 2 functions");
    assert!(functions.iter().any(|f| f.name == "main"));
    assert!(functions.iter().any(|f| f.name == "helper"));
    
    // Check for struct
    let structs: Vec<_> = result.nodes.iter()
        .filter(|n| n.kind == NodeKind::Struct)
        .collect();
    
    assert!(structs.len() >= 1, "Should extract at least 1 struct");
    assert!(structs.iter().any(|s| s.name == "User"));
}

#[test]
fn test_javascript_extraction() {
    use canopy_indexer::languages::get_extractor;
    
    let js_code = r#"
function greet(name) {
    return "Hello, " + name;
}

class Person {
    constructor(name) {
        this.name = name;
    }
    
    greet() {
        return "Hello, I'm " + this.name;
    }
}

const arrowFunc = () => {
    return 42;
};
"#;
    
    let path = PathBuf::from("test.js");
    let extractor = get_extractor(&path).unwrap();
    let result = extractor.extract(&path, js_code.as_bytes()).unwrap();
    
    // Should extract functions and class
    let functions: Vec<_> = result.nodes.iter()
        .filter(|n| n.kind == NodeKind::Function)
        .collect();
    
    assert!(functions.len() >= 1, "Should extract at least 1 function");
    assert!(functions.iter().any(|f| f.name == "greet"));
    
    let classes: Vec<_> = result.nodes.iter()
        .filter(|n| n.kind == NodeKind::Class)
        .collect();
    
    assert!(classes.len() >= 1, "Should extract at least 1 class");
    assert!(classes.iter().any(|c| c.name == "Person"));
}

#[test]
fn test_python_extraction() {
    use canopy_indexer::languages::get_extractor;
    
    let python_code = r#"
def greet(name):
    return f"Hello, {name}"

class Person:
    def __init__(self, name):
        self.name = name
    
    def greet(self):
        return f"Hello, I'm {self.name}"
"#;
    
    let path = PathBuf::from("test.py");
    let extractor = get_extractor(&path).unwrap();
    let result = extractor.extract(&path, python_code.as_bytes()).unwrap();
    
    // Should extract function and class
    let functions: Vec<_> = result.nodes.iter()
        .filter(|n| n.kind == NodeKind::Function)
        .collect();
    
    assert!(functions.len() >= 2, "Should extract at least 2 functions");
    
    let classes: Vec<_> = result.nodes.iter()
        .filter(|n| n.kind == NodeKind::Class)
        .collect();
    
    assert!(classes.len() >= 1, "Should extract at least 1 class");
    assert!(classes.iter().any(|c| c.name == "Person"));
}

#[test]
fn test_edge_creation() {
    use canopy_indexer::languages::get_extractor;
    
    let code = r#"
import os
from pathlib import Path

def process_file(path: Path):
    return path.exists()
"#;
    
    let path = PathBuf::from("test.py");
    let extractor = get_extractor(&path).unwrap();
    let result = extractor.extract(&path, code.as_bytes()).unwrap();
    
    // Should extract import relationships
    assert!(result.edges.len() > 0, "Should extract some edges");
    
    // Check for import edges
    let imports: Vec<_> = result.edges.iter()
        .filter(|e| e.kind == canopy_core::EdgeKind::Imports)
        .collect();
    
    // Should have edges for the imports
    assert!(!imports.is_empty(), "Should extract import relationships");
}

#[test]
fn test_empty_extraction() {
    use canopy_indexer::languages::get_extractor;
    
    let path = PathBuf::from("empty.rs");
    let extractor = get_extractor(&path).unwrap();
    let result = extractor.extract(&path, b"").unwrap();
    
    // Should handle empty files gracefully
    assert_eq!(result.nodes.len(), 0);
    assert_eq!(result.edges.len(), 0);
}

#[test]
fn test_invalid_utf8_handling() {
    use canopy_indexer::languages::get_extractor;
    
    let path = PathBuf::from("binary.rs");
    let extractor = get_extractor(&path).unwrap();
    
    // Test with invalid UTF-8
    let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
    let result = extractor.extract(&path, &invalid_utf8);
    
    // Should handle invalid UTF-8 gracefully
    assert!(result.is_err() || result.unwrap().nodes.is_empty());
}