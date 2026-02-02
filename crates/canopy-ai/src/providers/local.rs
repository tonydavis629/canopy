//! Local AI provider for offline semantic analysis

use super::super::bridge::{AIProvider, SemanticAnalysisRequest, SemanticAnalysisResult, InferredRelationship, SemanticRelationship};
use anyhow::Result;
use canopy_core::{GraphNode, GraphEdge};

pub struct LocalProvider;

impl LocalProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl AIProvider for LocalProvider {
    async fn analyze_semantic_relationships(
        &self,
        request: SemanticAnalysisRequest,
    ) -> Result<SemanticAnalysisResult> {
        // Simple heuristic-based analysis without AI
        let mut relationships = Vec::new();
        
        // Look for obvious relationships based on naming and structure
        for candidate in &request.candidate_nodes {
            // Check if source calls target (based on name patterns)
            if request.source_node.kind == canopy_core::NodeKind::Function 
                && candidate.kind == canopy_core::NodeKind::Function
                && candidate.name.starts_with(&request.source_node.name)
            {
                relationships.push(InferredRelationship {
                    source_id: request.source_node.id,
                    target_id: candidate.id,
                    relationship: SemanticRelationship::Calls,
                    confidence: 0.6,
                    explanation: format!("Function name suggests it calls {}", candidate.name),
                    line_reference: None,
                });
            }
            
            // Check for type usage (simple heuristic)
            if request.source_node.qualified_name.contains(&candidate.name) {
                relationships.push(InferredRelationship {
                    source_id: request.source_node.id,
                    target_id: candidate.id,
                    relationship: SemanticRelationship::DependsOn,
                    confidence: 0.5,
                    explanation: format!("Source references {} in its qualified name", candidate.name),
                    line_reference: None,
                });
            }
        }
        
        Ok(SemanticAnalysisResult {
            relationships,
            explanation: "Heuristic-based analysis without AI".to_string(),
            tokens_used: 0,
        })
    }
    
    async fn generate_node_summary(
        &self,
        node: &GraphNode,
        _context: &super::super::bridge::AnalysisContext,
    ) -> Result<String> {
        // Simple template-based summary
        let summary = match node.kind {
            canopy_core::NodeKind::Function => {
                format!("Function {} that performs operations related to its name.", node.name)
            }
            canopy_core::NodeKind::Class => {
                format!("Class {} that encapsulates related functionality.", node.name)
            }
            _ => {
                format!("{} {} in the codebase.", format!("{:?}", node.kind), node.name)
            }
        };
        Ok(summary)
    }
    
    async fn answer_code_question(
        &self,
        question: &str,
        relevant_nodes: &[GraphNode],
        relevant_edges: &[GraphEdge],
    ) -> Result<String> {
        // Simple template-based answers
        let answer = if question.to_lowercase().contains("what does") {
            if let Some(node) = relevant_nodes.first() {
                format!("{} {} appears to be a {} that {} based on its name and connections.",
                    format!("{:?}", node.kind),
                    node.name,
                    format!("{:?}", node.kind).to_lowercase(),
                    if node.kind == canopy_core::NodeKind::Function { "performs specific operations" } else { "encapsulates functionality" }
                )
            } else {
                "I need more context to answer that question.".to_string()
            }
        } else if question.to_lowercase().contains("how many") {
            format!("The graph contains {} nodes and {} edges in the relevant context.",
                relevant_nodes.len(),
                relevant_edges.len()
            )
        } else {
            "I'm a local provider with limited capabilities. Please use an AI provider for more sophisticated analysis.".to_string()
        };
        Ok(answer)
    }
    
    fn name(&self) -> &str {
        "Local (Heuristic)"
    }
}