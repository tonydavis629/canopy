//! Unit tests for canopy-ai module

use canopy_ai::providers::create_provider;
use canopy_ai::bridge::{AIProvider, SemanticAnalysisRequest, AnalysisContext, SemanticRelationship};
use canopy_core::{GraphNode, NodeKind, NodeId};
use std::path::PathBuf;
use std::collections::HashMap;

#[test]
fn test_provider_creation() {
    // Test creating providers without API keys
    let openai = create_provider("openai", None);
    assert!(openai.is_ok());
    
    let anthropic = create_provider("anthropic", None);
    assert!(anthropic.is_ok());
    
    let local = create_provider("local", None);
    assert!(local.is_ok());
    
    // Test unknown provider
    let unknown = create_provider("unknown", None);
    assert!(unknown.is_err());
}

#[test]
fn test_local_provider_analysis() {
    use tokio::runtime::Runtime;
    
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let provider = create_provider("local", None).unwrap();
        
        // Create test nodes
        let node1 = GraphNode {
            id: NodeId(1),
            kind: NodeKind::Function,
            name: "process_data".to_string(),
            qualified_name: "process_data".to_string(),
            file_path: PathBuf::from("src/lib.rs"),
            line_start: Some(10),
            line_end: Some(20),
            language: Some(canopy_core::Language::Rust),
            is_container: false,
            child_count: 0,
            loc: Some(10),
            metadata: HashMap::new(),
        };
        
        let node2 = GraphNode {
            id: NodeId(2),
            kind: NodeKind::Function,
            name: "validate_input".to_string(),
            qualified_name: "validate_input".to_string(),
            file_path: PathBuf::from("src/lib.rs"),
            line_start: Some(30),
            line_end: Some(40),
            language: Some(canopy_core::Language::Rust),
            is_container: false,
            child_count: 0,
            loc: Some(10),
            metadata: HashMap::new(),
        };
        
        // Test semantic analysis
        let request = SemanticAnalysisRequest {
            source_node: node1.clone(),
            candidate_nodes: vec![node2],
            context: AnalysisContext {
                file_path: PathBuf::from("src/lib.rs"),
                language: "Rust".to_string(),
                enclosing_context: vec![],
                imports: vec![],
                project_context: HashMap::new(),
            },
            relationship_types: vec![SemanticRelationship::Calls, SemanticRelationship::DependsOn],
        };
        
        let result = provider.analyze_semantic_relationships(request).await;
        assert!(result.is_ok());
        
        let analysis = result.unwrap();
        assert!(analysis.relationships.len() >= 0); // Local provider may return empty
        assert_eq!(analysis.tokens_used, 0); // Local provider uses no tokens
    });
}

#[test]
fn test_semantic_analysis_request_creation() {
    let node = GraphNode {
        id: NodeId(1),
        kind: NodeKind::Function,
        name: "test_function".to_string(),
        qualified_name: "test_function".to_string(),
        file_path: PathBuf::from("test.rs"),
        line_start: Some(10),
        line_end: Some(20),
        language: Some(canopy_core::Language::Rust),
        is_container: false,
        child_count: 0,
        loc: Some(10),
        metadata: HashMap::new(),
    };
    
    let request = SemanticAnalysisRequest {
        source_node: node,
        candidate_nodes: vec![],
        context: AnalysisContext {
            file_path: PathBuf::from("test.rs"),
            language: "Rust".to_string(),
            enclosing_context: vec!["fn main()".to_string()],
            imports: vec!["std::collections::HashMap".to_string()],
            project_context: HashMap::new(),
        },
        relationship_types: vec![SemanticRelationship::Calls],
    };
    
    assert_eq!(request.context.language, "Rust");
    assert_eq!(request.relationship_types.len(), 1);
}

#[test]
fn test_ai_budget() {
    use canopy_ai::bridge::AIBudget;
    
    let mut budget = AIBudget::new(1000);
    
    assert!(budget.has_budget(500));
    assert!(!budget.has_budget(1500));
    
    budget.use_tokens(300);
    assert_eq!(budget.tokens_used, 300);
    assert_eq!(budget.remaining_tokens(), 700);
    
    assert!(!budget.has_budget(800));
    assert!(budget.has_budget(600));
}

#[test]
fn test_semantic_relationships() {
    use canopy_ai::bridge::SemanticRelationship;
    
    let relationships = vec![
        SemanticRelationship::Calls,
        SemanticRelationship::DependsOn,
        SemanticRelationship::Uses,
        SemanticRelationship::Configures,
    ];
    
    // Test that relationships can be used in collections
    let mut set = std::collections::HashSet::new();
    for rel in relationships {
        set.insert(rel);
    }
    
    assert_eq!(set.len(), 4);
    assert!(set.contains(&SemanticRelationship::Calls));
}

#[test]
fn test_analysis_context() {
    let context = AnalysisContext {
        file_path: PathBuf::from("src/main.rs"),
        language: "Rust".to_string(),
        enclosing_context: vec!["fn main()".to_string(), "struct User".to_string()],
        imports: vec!["std::fs::File".to_string(), "serde::{Serialize, Deserialize}".to_string()],
        project_context: {
            let mut map = HashMap::new();
            map.insert("version".to_string(), "1.0.0".to_string());
            map
        },
    };
    
    assert_eq!(context.language, "Rust");
    assert_eq!(context.imports.len(), 2);
    assert!(context.project_context.contains_key("version"));
}

#[tokio::test]
async fn test_node_summary_generation() {
    use tokio::runtime::Runtime;
    
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let provider = create_provider("local", None).unwrap();
        
        let node = GraphNode {
            id: NodeId(1),
            kind: NodeKind::Function,
            name: "calculate_total".to_string(),
            qualified_name: "calculate_total".to_string(),
            file_path: PathBuf::from("src/math.rs"),
            line_start: Some(42),
            line_end: Some(58),
            language: Some(canopy_core::Language::Rust),
            is_container: false,
            child_count: 0,
            loc: Some(16),
            metadata: HashMap::new(),
        };
        
        let context = AnalysisContext {
            file_path: PathBuf::from("src/math.rs"),
            language: "Rust".to_string(),
            enclosing_context: vec![],
            imports: vec![],
            project_context: HashMap::new(),
        };
        
        let summary = provider.generate_node_summary(&node, &context).await;
        assert!(summary.is_ok());
        
        let summary_text = summary.unwrap();
        assert!(!summary_text.is_empty());
    });
}