//! Unit tests for canopy-core module

use canopy_core::*;
use std::path::PathBuf;

#[test]
fn test_node_id_creation() {
    let path = PathBuf::from("test.rs");
    let node_id = NodeId::new(&path, NodeKind::Function, "test_function");
    
    // NodeId should be deterministic
    let same_id = NodeId::new(&path, NodeKind::Function, "test_function");
    assert_eq!(node_id, same_id);
    
    // Different names should produce different IDs
    let different_id = NodeId::new(&path, NodeKind::Function, "different_function");
    assert_ne!(node_id, different_id);
}

#[test]
fn test_graph_node_creation() {
    let node = GraphNode {
        id: NodeId(1),
        kind: NodeKind::Function,
        name: "test_function".to_string(),
        qualified_name: "module::test_function".to_string(),
        file_path: PathBuf::from("src/lib.rs"),
        line_start: Some(10),
        line_end: Some(20),
        language: Some(Language::Rust),
        is_container: false,
        child_count: 0,
        loc: Some(10),
        metadata: std::collections::HashMap::new(),
    };
    
    assert_eq!(node.name, "test_function");
    assert_eq!(node.kind, NodeKind::Function);
    assert_eq!(node.line_start, Some(10));
}

#[test]
fn test_graph_operations() {
    let mut graph = Graph::new();
    
    // Add nodes
    let node1 = GraphNode {
        id: NodeId(0),
        kind: NodeKind::Function,
        name: "func1".to_string(),
        qualified_name: "func1".to_string(),
        file_path: PathBuf::from("test.rs"),
        line_start: None,
        line_end: None,
        language: None,
        is_container: false,
        child_count: 0,
        loc: None,
        metadata: std::collections::HashMap::new(),
    };
    
    let node2 = GraphNode {
        id: NodeId(0),
        kind: NodeKind::Function,
        name: "func2".to_string(),
        qualified_name: "func2".to_string(),
        file_path: PathBuf::from("test.rs"),
        line_start: None,
        line_end: None,
        language: None,
        is_container: false,
        child_count: 0,
        loc: None,
        metadata: std::collections::HashMap::new(),
    };
    
    let id1 = graph.add_node(node1);
    let id2 = graph.add_node(node2);
    
    assert_eq!(graph.node_count(), 2);
    
    // Add edge
    let edge = GraphEdge {
        id: EdgeId(0),
        source: id1,
        target: id2,
        kind: EdgeKind::Calls,
        edge_source: EdgeSource::Heuristic,
        confidence: 0.8,
        label: Some("calls".to_string()),
        file_path: Some(PathBuf::from("test.rs")),
        line: None,
    };
    
    graph.add_edge(edge);
    assert_eq!(graph.edge_count(), 1);
    
    // Test edge lookup
    assert!(graph.has_edge_between(id1, id2, EdgeKind::Calls));
}

#[test]
fn test_edge_kinds() {
    // Test that edge kinds can be compared and used in collections
    let kinds = vec![
        EdgeKind::Calls,
        EdgeKind::DependsOn,
        EdgeKind::Uses,
        EdgeKind::Imports,
    ];
    
    let mut set = std::collections::HashSet::new();
    for kind in kinds {
        set.insert(kind);
    }
    
    assert_eq!(set.len(), 4);
    assert!(set.contains(&EdgeKind::Calls));
}

#[test]
fn test_node_kinds() {
    // Test node kind conversion and usage
    let function_kind = NodeKind::Function;
    let class_kind = NodeKind::Class;
    
    assert_ne!(function_kind, class_kind);
    
    // Test that node kinds can be used in match statements
    match function_kind {
        NodeKind::Function => assert!(true),
        _ => panic!("Expected Function kind"),
    }
}

#[test]
fn test_graph_ancestors() {
    let mut graph = Graph::new();
    
    // Create a simple hierarchy
    let root = GraphNode {
        id: NodeId(0),
        kind: NodeKind::Directory,
        name: "src".to_string(),
        qualified_name: "src".to_string(),
        file_path: PathBuf::from("src"),
        line_start: None,
        line_end: None,
        language: None,
        is_container: true,
        child_count: 0,
        loc: None,
        metadata: std::collections::HashMap::new(),
    };
    
    let child = GraphNode {
        id: NodeId(0),
        kind: NodeKind::File,
        name: "lib.rs".to_string(),
        qualified_name: "lib.rs".to_string(),
        file_path: PathBuf::from("src/lib.rs"),
        line_start: None,
        line_end: None,
        language: None,
        is_container: false,
        child_count: 0,
        loc: None,
        metadata: std::collections::HashMap::new(),
    };
    
    let root_id = graph.add_node(root);
    let child_id = graph.add_node(child);
    
    // Add containment edge
    let edge = GraphEdge {
        id: EdgeId(0),
        source: root_id,
        target: child_id,
        kind: EdgeKind::Contains,
        edge_source: EdgeSource::Structural,
        confidence: 1.0,
        label: None,
        file_path: None,
        line: None,
    };
    
    graph.add_edge(edge);
    
    // Test ancestor finding
    let ancestors = graph.ancestors(child_id);
    assert!(ancestors.contains(&root_id));
}

#[test]
fn test_language_detection() {
    use std::path::PathBuf;
    
    let test_cases = vec![
        ("test.rs", Language::Rust),
        ("main.ts", Language::TypeScript),
        ("app.js", Language::JavaScript),
        ("lib.py", Language::Python),
        ("main.go", Language::Go),
        ("Main.java", Language::Java),
        ("main.c", Language::C),
        ("main.cpp", Language::Cpp),
        ("config.yml", Language::Yaml),
        ("config.toml", Language::Toml),
        ("package.json", Language::Json),
        ("unknown.xyz", Language::Other),
    ];
    
    for (filename, expected) in test_cases {
        let path = PathBuf::from(filename);
        let detected = Language::from_path(&path);
        assert_eq!(detected, expected, "Failed for {}", filename);
    }
}

#[test]
fn test_node_id_serialization() {
    use serde_json;
    
    let node_id = NodeId(42);
    let json = serde_json::to_string(&node_id).unwrap();
    let deserialized: NodeId = serde_json::from_str(&json).unwrap();
    
    assert_eq!(node_id, deserialized);
}

#[test]
fn test_graph_node_serialization() {
    use serde_json;
    
    let node = GraphNode {
        id: NodeId(1),
        kind: NodeKind::Function,
        name: "test".to_string(),
        qualified_name: "test".to_string(),
        file_path: PathBuf::from("test.rs"),
        line_start: Some(10),
        line_end: Some(20),
        language: Some(Language::Rust),
        is_container: false,
        child_count: 0,
        loc: Some(10),
        metadata: {
            let mut map = std::collections::HashMap::new();
            map.insert("test".to_string(), "value".to_string());
            map
        },
    };
    
    let json = serde_json::to_string(&node).unwrap();
    let deserialized: GraphNode = serde_json::from_str(&json).unwrap();
    
    assert_eq!(node.id, deserialized.id);
    assert_eq!(node.name, deserialized.name);
}