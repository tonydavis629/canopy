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
        // Use OpenRouter API key from .env file for consistency
        let api_key = api_key.or_else(|| std::env::var("OPENROUTER_API_KEY").ok())
            .or_else(|| std::env::var("openrouter_api_key").ok())
            .unwrap_or_default();
        
        Self {
            client: reqwest::Client::new(),
            api_key,
            model: "anthropic/claude-3-haiku-20240307".to_string(), // OpenRouter format
        }
    }
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    total_tokens: u32,
}

// Using OpenAI-compatible format for OpenRouter

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

        // Convert to OpenAI-compatible format for OpenRouter
        let openai_request = OpenAIRequest {
            model: self.model.clone(),
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: "You are a code analysis expert. Respond only with valid JSON.".to_string(),
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: prompt,
                }
            ],
            temperature: 0.1,
            max_tokens: 2000,
        };

        let response = self.client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://github.com/openclaw/openclaw")
            .header("X-Title", "Canopy")
            .json(&openai_request)
            .send()
            .await
            .context("Failed to send request to OpenRouter")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenRouter API error: {}", error_text);
        }

        let openai_response: OpenAIResponse = response.json().await.context("Failed to parse OpenRouter response")?;
        
        let content = &openai_response.choices[0].message.content;

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
            tokens_used: openai_response.usage.map(|u| u.total_tokens).unwrap_or(0),
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

        let openai_request = OpenAIRequest {
            model: self.model.clone(),
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: "You are a code documentation expert. Provide concise, clear summaries.".to_string(),
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: prompt,
                }
            ],
            temperature: 0.3,
            max_tokens: 150,
        };

        let response = self.client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://github.com/openclaw/openclaw")
            .header("X-Title", "Canopy")
            .json(&openai_request)
            .send()
            .await
            .context("Failed to send request to OpenRouter")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenRouter API error: {}", error_text);
        }

        let openai_response: OpenAIResponse = response.json().await.context("Failed to parse OpenRouter response")?;
        
        let summary = openai_response.choices[0].message.content.trim().to_string();

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

        let openai_request = OpenAIRequest {
            model: self.model.clone(),
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: "You are a helpful code analysis assistant. Answer questions clearly and concisely.".to_string(),
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: prompt,
                }
            ],
            temperature: 0.2,
            max_tokens: 1000,
        };

        let response = self.client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://github.com/openclaw/openclaw")
            .header("X-Title", "Canopy")
            .json(&openai_request)
            .send()
            .await
            .context("Failed to send request to OpenRouter")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Anthropic API error: {}", error_text);
        }

        let openai_response: OpenAIResponse = response.json().await.context("Failed to parse OpenRouter response")?;
        
        let answer = openai_response.choices[0].message.content.trim().to_string();

        Ok(answer)
    }
    
    fn name(&self) -> &str {
        "Anthropic (via OpenRouter)"
    }
}