//! Anthropic Claude provider implementation

use super::super::bridge::{AIProvider, SemanticAnalysisRequest, SemanticAnalysisResult, InferredRelationship, SemanticRelationship, AnalysisContext};
use anyhow::{Result, Context};
use canopy_core::{GraphNode, GraphEdge, NodeId};
use serde::{Deserialize, Serialize};

pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl AnthropicProvider {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.unwrap_or_else(|| std::env::var("ANTHROPIC_API_KEY").unwrap_or_default()),
            model: "claude-3-haiku-20240307".to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    system: String,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    text: String,
    #[serde(rename = "type")]
    content_type: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct SemanticAnalysisResponse {
    relationships: Vec<InferredRelationshipJson>,
    explanation: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct InferredRelationshipJson {
    source_id: u64,
    target_id: u64,
    relationship: String,
    confidence: f32,
    explanation: String,
    line_reference: Option<u32>,
}

#[async_trait::async_trait]
impl AIProvider for AnthropicProvider {
    async fn analyze_semantic_relationships(
        &self,
        request: SemanticAnalysisRequest,
    ) -> Result<SemanticAnalysisResult> {
        let prompt = format!(
            r#"You are a code analysis expert. Analyze the following code and identify semantic relationships between the source function and other code elements.

Source code file: {}
Language: {:?}
Source function: {} (lines {:?}-{:?})

Source code:
```{}```

Candidate code elements to analyze relationships with:
{}

Look for these types of relationships:
- Calls: Does the source function call any of these functions?
- DependsOn: Does it depend on any types/classes?
- Uses: Does it use/import any modules?
- Configures: Does it configure or consume any configs?

Respond with a JSON object in this exact format:
{{
  "relationships": [
    {{
      "source_id": {},
      "target_id": <target_node_id>,
      "relationship": "Calls|DependsOn|Uses|Configures",
      "confidence": 0.0-1.0,
      "explanation": "Brief explanation of why this relationship exists",
      "line_reference": <line_number_or_null>
    }}
  ],
  "explanation": "Overall analysis summary"
}}"#,
            request.source_node.file_path.display(),
            request.source_node.language,
            request.source_node.name,
            request.source_node.line_start,
            request.source_node.line_end,
            "Source code not available in request",
            request.candidate_nodes.iter()
                .map(|n| format!("- {} ({}): {} lines {:?}-{:?}", 
                    n.name, 
                    format!("{:?}", n.kind).to_lowercase(),
                    n.file_path.display(),
                    n.line_start,
                    n.line_end))
                .collect::<Vec<_>>()
                .join("\n"),
            request.source_node.id.0
        );

        let anthropic_request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 2000,
            messages: vec![
                AnthropicMessage {
                    role: "user".to_string(),
                    content: prompt,
                }
            ],
            system: "You are a code analysis expert. Respond only with valid JSON.".to_string(),
        };

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("content-type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&anthropic_request)
            .send()
            .await
            .context("Failed to send request to Anthropic API")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Anthropic API error: {}", error_text);
        }

        let anthropic_response: AnthropicResponse = response.json().await.context("Failed to parse Anthropic response")?;
        
        let content = &anthropic_response.content.first()
            .context("No content in Anthropic response")?
            .text;

        // Parse the JSON response
        let analysis_response: SemanticAnalysisResponse = serde_json::from_str(content)
            .context("Failed to parse semantic analysis response from Anthropic")?;

        // Convert JSON relationships to proper InferredRelationship objects
        let mut relationships = Vec::new();
        for rel_json in analysis_response.relationships {
            let relationship = match rel_json.relationship.as_str() {
                "Calls" => SemanticRelationship::Calls,
                "DependsOn" => SemanticRelationship::DependsOn,
                "Uses" => SemanticRelationship::Uses,
                "Configures" => SemanticRelationship::Configures,
                _ => continue, // Skip unknown relationships
            };

            relationships.push(InferredRelationship {
                source_id: NodeId(rel_json.source_id),
                target_id: NodeId(rel_json.target_id),
                relationship,
                confidence: rel_json.confidence,
                explanation: rel_json.explanation,
                line_reference: rel_json.line_reference,
            });
        }

        Ok(SemanticAnalysisResult {
            relationships,
            explanation: analysis_response.explanation,
            tokens_used: anthropic_response.usage.input_tokens + anthropic_response.usage.output_tokens,
        })
    }
    
    async fn generate_node_summary(
        &self,
        node: &GraphNode,
        _context: &AnalysisContext,
    ) -> Result<String> {
        let prompt = format!(
            r#"Please provide a concise summary of this code element:

Name: {}
Type: {:?}
File: {}
Lines: {:?}-{:?}
Language: {:?}

Provide a brief summary (1-2 sentences) explaining what this code does and its purpose in the codebase."#,
            node.name,
            node.kind,
            node.file_path.display(),
            node.line_start,
            node.line_end,
            node.language
        );

        let anthropic_request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 150,
            messages: vec![
                AnthropicMessage {
                    role: "user".to_string(),
                    content: prompt,
                }
            ],
            system: "You are a code documentation expert. Provide concise, clear summaries.".to_string(),
        };

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("content-type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&anthropic_request)
            .send()
            .await
            .context("Failed to send request to Anthropic API")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Anthropic API error: {}", error_text);
        }

        let anthropic_response: AnthropicResponse = response.json().await.context("Failed to parse Anthropic response")?;
        
        let summary = anthropic_response.content.first()
            .context("No content in Anthropic response")?
            .text.trim()
            .to_string();

        Ok(summary)
    }
    
    async fn answer_code_question(
        &self,
        question: &str,
        relevant_nodes: &[GraphNode],
        relevant_edges: &[GraphEdge],
    ) -> Result<String> {
        let nodes_info = relevant_nodes.iter()
            .map(|n| format!("- {} ({}): {} at {:?}:{:?}", 
                n.name, 
                format!("{:?}", n.kind).to_lowercase(),
                n.file_path.display(),
                n.line_start,
                n.line_end))
            .collect::<Vec<_>>()
            .join("\n");

        let edges_info = relevant_edges.iter()
            .map(|e| format!("- {} -> {} ({:?})", 
                e.source.0, 
                e.target.0,
                e.kind))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            r#"You are a code analysis expert. Answer the following question about the codebase.

Question: {}

Relevant code elements:
{}

Connections between elements:
{}

Provide a clear, concise answer based on the provided code context. If the information is insufficient to answer accurately, explain what additional context would be needed."#,
            question,
            nodes_info,
            edges_info
        );

        let anthropic_request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 1000,
            messages: vec![
                AnthropicMessage {
                    role: "user".to_string(),
                    content: prompt,
                }
            ],
            system: "You are a helpful code analysis assistant. Answer questions clearly and concisely.".to_string(),
        };

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("content-type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&anthropic_request)
            .send()
            .await
            .context("Failed to send request to Anthropic API")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Anthropic API error: {}", error_text);
        }

        let anthropic_response: AnthropicResponse = response.json().await.context("Failed to parse Anthropic response")?;
        
        let answer = anthropic_response.content.first()
            .context("No content in Anthropic response")?
            .text.trim()
            .to_string();

        Ok(answer)
    }
    
    fn name(&self) -> &str {
        "Anthropic"
    }
}