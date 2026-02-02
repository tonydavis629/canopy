//! AI semantic analysis bridge for understanding code relationships

use anyhow::Result;
use canopy_core::{GraphNode, GraphEdge, NodeId, EdgeKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Confidence score for AI-inferred relationships (0.0 - 1.0)
pub type Confidence = f32;

/// Semantic relationship types that AI can infer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SemanticRelationship {
    /// Function A calls function B
    Calls,
    /// Type A depends on/uses type B
    DependsOn,
    /// Class A implements interface B
    Implements,
    /// Class A extends class B
    Extends,
    /// Function A is tested by test B
    TestedBy,
    /// Module A imports/uses module B
    Uses,
    /// Function A configures/consumes config B
    Configures,
    /// API endpoint A handles route B
    HandlesRoute,
    /// Migration A depends on migration B
    MigrationDepends,
    /// Generic semantic reference
    SemanticReference,
}

impl From<SemanticRelationship> for EdgeKind {
    fn from(rel: SemanticRelationship) -> Self {
        match rel {
            SemanticRelationship::Calls => EdgeKind::Calls,
            SemanticRelationship::DependsOn => EdgeKind::TypeReference,
            SemanticRelationship::Implements => EdgeKind::Implements,
            SemanticRelationship::Extends => EdgeKind::Inherits,
            SemanticRelationship::TestedBy => EdgeKind::SemanticReference,
            SemanticRelationship::Uses => EdgeKind::Imports,
            SemanticRelationship::Configures => EdgeKind::ConfiguresArgument,
            SemanticRelationship::HandlesRoute => EdgeKind::RouteHandler,
            SemanticRelationship::MigrationDepends => EdgeKind::MigrationTarget,
            SemanticRelationship::SemanticReference => EdgeKind::SemanticReference,
        }
    }
}

/// Context for AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisContext {
    /// The source code file being analyzed
    pub file_path: PathBuf,
    /// Programming language detected
    pub language: String,
    /// Surrounding code context (enclosing functions, classes, etc.)
    pub enclosing_context: Vec<String>,
    /// Import statements in the file
    pub imports: Vec<String>,
    /// Project-wide context (package.json, Cargo.toml, etc.)
    pub project_context: HashMap<String, String>,
}

/// Request for semantic analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAnalysisRequest {
    /// The source node to analyze
    pub source_node: GraphNode,
    /// Related nodes that might have relationships
    pub candidate_nodes: Vec<GraphNode>,
    /// Source code context
    pub context: AnalysisContext,
    /// Specific relationships to look for
    pub relationship_types: Vec<SemanticRelationship>,
}

/// Result of semantic analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAnalysisResult {
    /// Inferred relationships with confidence scores
    pub relationships: Vec<InferredRelationship>,
    /// Natural language explanation of the analysis
    pub explanation: String,
    /// Tokens used for this analysis
    pub tokens_used: u32,
}

/// A single inferred relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferredRelationship {
    /// Source node ID
    pub source_id: NodeId,
    /// Target node ID
    pub target_id: NodeId,
    /// Type of relationship
    pub relationship: SemanticRelationship,
    /// Confidence score (0.0 - 1.0)
    pub confidence: Confidence,
    /// Natural language explanation
    pub explanation: String,
    /// Line number where this relationship is evident
    pub line_reference: Option<u32>,
}

/// AI provider trait for different LLM backends
#[async_trait::async_trait]
pub trait AIProvider: Send + Sync {
    /// Analyze semantic relationships in code
    async fn analyze_semantic_relationships(
        &self,
        request: SemanticAnalysisRequest,
    ) -> Result<SemanticAnalysisResult>;
    
    /// Generate a summary of what a node does
    async fn generate_node_summary(
        &self,
        node: &GraphNode,
        context: &AnalysisContext,
    ) -> Result<String>;
    
    /// Answer questions about the codebase
    async fn answer_code_question(
        &self,
        question: &str,
        relevant_nodes: &[GraphNode],
        relevant_edges: &[GraphEdge],
    ) -> Result<String>;
    
    /// Get provider name
    fn name(&self) -> &str;
}

/// Budget tracking for AI API usage
#[derive(Debug, Clone)]
pub struct AIBudget {
    /// Total tokens available
    pub total_tokens: u32,
    /// Tokens used so far
    pub tokens_used: u32,
    /// Maximum confidence threshold for auto-accepting relationships
    pub auto_accept_threshold: Confidence,
}

impl AIBudget {
    pub fn new(total_tokens: u32) -> Self {
        Self {
            total_tokens,
            tokens_used: 0,
            auto_accept_threshold: 0.8,
        }
    }
    
    pub fn has_budget(&self, estimated_tokens: u32) -> bool {
        self.tokens_used + estimated_tokens <= self.total_tokens
    }
    
    pub fn use_tokens(&mut self, tokens: u32) {
        self.tokens_used += tokens;
    }
    
    pub fn remaining_tokens(&self) -> u32 {
        self.total_tokens.saturating_sub(self.tokens_used)
    }
}

/// Semantic analysis configuration
#[derive(Debug, Clone)]
pub struct SemanticConfig {
    /// Whether to enable AI analysis
    pub enabled: bool,
    /// AI provider to use
    pub provider: String,
    /// API key for the provider
    pub api_key: Option<String>,
    /// Budget configuration
    pub budget: AIBudget,
    /// Batch size for processing multiple nodes
    pub batch_size: usize,
    /// Delay between API calls to avoid rate limiting
    pub api_delay_ms: u64,
}

impl Default for SemanticConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            provider: "openai".to_string(),
            api_key: None,
            budget: AIBudget::new(100_000),
            batch_size: 10,
            api_delay_ms: 1000,
        }
    }
}