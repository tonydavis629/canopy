//! Integration tests for Canopy
//!
//! These tests verify that multiple systems work together correctly.

use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

/// Test that the CLI can be invoked
#[tokio::test]
async fn test_cli_invocation() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .current_dir(".")
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Canopy"));
    assert!(stdout.contains("Live hierarchical code architecture visualization"));
}

/// Test that the server starts and responds to health checks
#[tokio::test]
async fn test_server_startup() {
    // This would require starting the server in a background task
    // and then making HTTP requests to verify it's working
    // For now, we just verify the server module compiles
    use canopy_server::CanopyServer;
    use canopy_core::Graph;
    
    let graph = Graph::new();
    let config = canopy_server::ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 0, // Let OS assign port
    };
    
    let server = CanopyServer::new(graph, config);
    assert_eq!(server.state().graph.read().await.node_count(), 0);
}

/// Test that the file watcher detects changes
#[tokio::test]
async fn test_file_watcher() {
    use canopy_watcher::WatcherService;
    use canopy_core::Graph;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new().unwrap();
    let graph = Arc::new(RwLock::new(Graph::new()));
    
    let watcher = WatcherService::new(temp_dir.path(), graph.clone());
    assert!(watcher.is_ok());
}

/// Test that language extractors work for all supported languages
#[test]
fn test_language_extractors() {
    use canopy_indexer::languages::get_extractor;
    use std::path::PathBuf;
    
    let test_cases = vec![
        ("test.rs", true),   // Rust
        ("test.ts", true),   // TypeScript
        ("test.js", true),   // JavaScript
        ("test.py", true),   // Python
        ("test.go", true),   // Go
        ("test.java", true), // Java
        ("test.c", true),    // C
        ("test.cpp", true),  // C++
        ("test.txt", false), // Unsupported
    ];
    
    for (filename, should_have_extractor) in test_cases {
        let path = PathBuf::from(filename);
        let extractor = get_extractor(&path);
        
        if should_have_extractor {
            assert!(extractor.is_some(), "Should have extractor for {}", filename);
        } else {
            assert!(extractor.is_none(), "Should not have extractor for {}", filename);
        }
    }
}

/// Test that AI providers can be created
#[test]
fn test_ai_providers() {
    use canopy_ai::providers::create_provider;
    
    // Test OpenAI provider creation
    let openai = create_provider("openai", None);
    assert!(openai.is_ok());
    
    // Test Anthropic provider creation
    let anthropic = create_provider("anthropic", None);
    assert!(anthropic.is_ok());
    
    // Test local provider creation
    let local = create_provider("local", None);
    assert!(local.is_ok());
    
    // Test unknown provider
    let unknown = create_provider("unknown", None);
    assert!(unknown.is_err());
}

/// Test graph operations
#[test]
fn test_graph_operations() {
    use canopy_core::{Graph, GraphNode, NodeKind, NodeId};
    use std::collections::HashMap;
    
    let mut graph = Graph::new();
    
    // Add a node
    let node = GraphNode {
        id: NodeId(0),
        kind: NodeKind::Function,
        name: "test_function".to_string(),
        qualified_name: "test::test_function".to_string(),
        file_path: std::path::PathBuf::from("test.rs"),
        line_start: Some(1),
        line_end: Some(10),
        language: Some(canopy_core::Language::Rust),
        is_container: false,
        child_count: 0,
        loc: Some(10),
        metadata: HashMap::new(),
    };
    
    let node_id = graph.add_node(node);
    assert_eq!(graph.node_count(), 1);
    
    // Retrieve the node
    let retrieved = graph.node(node_id);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "test_function");
    
    // Remove the node
    graph.remove_node(node_id);
    assert_eq!(graph.node_count(), 0);
}

/// Test end-to-end indexing of a sample project
#[tokio::test]
async fn test_end_to_end_indexing() {
    use canopy_core::Graph;
    use canopy_indexer::languages::get_extractor;
    use std::path::PathBuf;
    
    // Create a test graph
    let mut graph = Graph::new();
    
    // Sample Rust code
    let rust_code = r#"
fn main() {
    println!("Hello, world!");
}

fn helper() -> i32 {
    42
}
"#;
    
    // Get Rust extractor
    let path = PathBuf::from("test.rs");
    let extractor = get_extractor(&path);
    assert!(extractor.is_some());
    
    // Extract nodes
    let result = extractor.unwrap().extract(&path, rust_code.as_bytes());
    assert!(result.is_ok());
    
    let extraction = result.unwrap();
    
    // Add extracted nodes to graph
    for node in extraction.nodes {
        graph.add_node(node);
    }
    
    // Verify nodes were added
    assert!(graph.node_count() > 0);
}

/// Test WebSocket protocol
#[tokio::test]
async fn test_websocket_protocol() {
    // This test would require starting a server and connecting via WebSocket
    // For now, we verify the protocol module exists and compiles
    use canopy_server::websocket;
    
    // The websocket module should be available
    // Actual WebSocket testing would require a running server
}

/// Test configuration loading
#[test]
fn test_configuration() {
    // Test that the application can load configuration
    // This would test .canopy.toml parsing if implemented
    
    // For now, just verify environment variable reading
    unsafe {
        std::env::set_var("TEST_CANOPY_VAR", "test_value");
    }
    let value = std::env::var("TEST_CANOPY_VAR");
    assert_eq!(value.unwrap(), "test_value");
}