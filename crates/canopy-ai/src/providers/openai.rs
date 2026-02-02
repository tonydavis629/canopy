//! OpenAI provider implementation

use super::super::bridge::{AIProvider, SemanticAnalysisRequest, SemanticAnalysisResult, InferredRelationship, SemanticRelationship, AnalysisContext};
use anyhow::{Result, Context};
use canopy_core::{GraphNode, GraphEdge, NodeId};
use serde::{Deserialize, Serialize};

pub struct OpenAIProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl OpenAIProvider {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.unwrap_or_else(|| std::env::var("OPENAI_API_KEY").unwrap_or_default()),
            model: "gpt-4o-mini".to_string(),
        }
    }
    
    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
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
impl AIProvider for OpenAIProvider {
    async fn analyze_semantic_relationships(
        &self,
        request: SemanticAnalysisRequest,
    ) -> Result<SemanticAnalysisResult> {
        let prompt = format!(
            r#"You are a code analysis expert. Analyze the following code and identify semantic relationships between the source function and other code elements.

Source code file: {}
Language: {}
Source function: {} (lines {}-{})

Source code:
```{}```

Candidate code elements to analyze relationships with:
{}

Look for these types of relationships:
- Calls: Does the source function call any of these functions?
- DependsOn: Does it depend on any types/classes?
- Uses: Does it use/import any modules?
- Configures: Does it configure or consume any configs?

For each relationship found, provide:
1. The target element ID
2. Type of relationship
3. Confidence score (0.0-1.0)
4. Brief explanation
5. Line number where evident

Return JSON in this format:
{{
  "relationships": [
    {{
      "source_id": {},
      "target_id": <target_node_id>,
      "relationship": "Calls|DependsOn|Uses|Configures",
      "confidence": 0.85,
      "explanation": "Function calls target on line 42",
      "line_reference": 42
    }}
  ],
  "explanation": "Overall analysis summary"
}}"#,
            request.context.file_path.display(),
            request.context.language,
            request.source_node.name,
            request.source_node.line_start.unwrap_or(0),
            request.source_node.line_end.unwrap_or(0),
            request.source_node.qualified_name,
            request.candidate_nodes.iter()
                .map(|n| format!("- {} (ID: {}, kind: {}, lines: {}-{})", 
                    n.name, n.id.0, format!("{:?}", n.kind), 
                    n.line_start.unwrap_or(0), n.line_end.unwrap_or(0)))
                .collect::<Vec<_>>()
                .join("\n"),
            request.source_node.id.0
        );

        let openai_request = OpenAIRequest {
            model: self.model.clone(),
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: "You are a code analysis expert. Analyze code relationships accurately and return valid JSON.".to_string(),
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            temperature: 0.1,
            max_tokens: 2000,
        };

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await
            .context("Failed to send request to OpenAI")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI API error: {}", error_text);
        }

        let openai_response: OpenAIResponse = response.json().await?;
        let content = &openai_response.choices[0].message.content;
        
        // Extract JSON from the response
        let json_start = content.find('{').unwrap_or(0);
        let json_end = content.rfind('}').unwrap_or(content.len() - 1) + 1;
        let json_str = &content[json_start..json_end];
        
        let analysis_response: SemanticAnalysisResponse = serde_json::from_str(json_str)
            .context("Failed to parse OpenAI response JSON")?;

        let tokens_used = openai_response.usage.map(|u| u.total_tokens).unwrap_or(0);
        
        let relationships = analysis_response.relationships.into_iter()
            .map(|rel| InferredRelationship {
                source_id: NodeId(rel.source_id),
                target_id: NodeId(rel.target_id),
                relationship: match rel.relationship.as_str() {
                    "Calls" => SemanticRelationship::Calls,
                    "DependsOn" => SemanticRelationship::DependsOn,
                    "Uses" => SemanticRelationship::Uses,
                    "Configures" => SemanticRelationship::Configures,
                    _ => SemanticRelationship::SemanticReference,
                },
                confidence: rel.confidence,
                explanation: rel.explanation,
                line_reference: rel.line_reference,
            })
            .collect();

        Ok(SemanticAnalysisResult {
            relationships,
            explanation: analysis_response.explanation,
            tokens_used,
        })
    }
    
    async fn generate_node_summary(
        &self,
        node: &GraphNode,
        context: &AnalysisContext,
    ) -> Result<String> {
        let prompt = format!(
            r#"Summarize what this {} does in one sentence:

File: {}
Name: {}
Lines: {}-{}
Code: {}

Context: {:?}"#,
            format!("{:?}", node.kind),
            context.file_path.display(),
            node.name,
            node.line_start.unwrap_or(0),
            node.line_end.unwrap_or(0),
            node.qualified_name,
            context.enclosing_context
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
                },
            ],
            temperature: 0.3,
            max_tokens: 150,
        };

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await?;

        let openai_response: OpenAIResponse = response.json().await?;
        Ok(openai_response.choices[0].message.content.trim().to_string())
    }
    
    async fn answer_code_question(
        &self,
        question: &str,
        relevant_nodes: &[GraphNode],
        relevant_edges: &[GraphEdge],
    ) -> Result<String> {
        let nodes_desc = relevant_nodes.iter()
            .map(|n| format!("- {} ({}): {}", n.name, format!("{:?}", n.kind), n.qualified_name))
            .collect::<Vec<_>>()
            .join("\n");
            
        let edges_desc = relevant_edges.iter()
            .map(|e| format!("- {} -> {} ({})", 
                e.source.0, e.target.0, format!("{:?}", e.kind)))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            r#"Based on this code graph information, answer the question:

Question: {}

Relevant code elements:
{}

Relationships:
{}

Provide a clear, accurate answer based on the graph data."#,
            question, nodes_desc, edges_desc
        );

        let openai_request = OpenAIRequest {
            model: self.model.clone(),
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: "You are a code analysis assistant. Answer questions accurately based on provided code graph data.".to_string(),
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            temperature: 0.2,
            max_tokens: 500,
        };

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await?;

        let openai_response: OpenAIResponse = response.json().await?;
        Ok(openai_response.choices[0].message.content.trim().to_string())
    }
    
    fn name(&self) -> &str {
        "OpenAI"
    }
}