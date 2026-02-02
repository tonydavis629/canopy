//! Prompt templates for AI analysis

use super::bridge::{SemanticRelationship, AnalysisContext};
use canopy_core::{GraphNode, GraphEdge};

/// Generate a prompt for semantic relationship analysis
pub fn semantic_analysis_prompt(
    source_node: &GraphNode,
    candidate_nodes: &[GraphNode],
    context: &AnalysisContext,
    relationships: &[SemanticRelationship],
) -> String {
    let relationship_types = relationships.iter()
        .map(|r| format!("{:?}", r))
        .collect::<Vec<_>>()
        .join(", ");
    
    let candidates_desc = candidate_nodes.iter()
        .map(|n| format!(
            "- {} (ID: {}, kind: {}, lines: {}-{})",
            n.name,
            n.id.0,
            format!("{:?}", n.kind),
            n.line_start.unwrap_or(0),
            n.line_end.unwrap_or(0)
        ))
        .collect::<Vec<_>>()
        .join("\n");

    format!(r#"You are analyzing code relationships in a software project. 

File: {}
Language: {}
Source element: {} (ID: {}, kind: {}, lines: {}-{})

Source code context:
```
{}
```

Surrounding context: {:?}

Related elements to analyze:
{}

Look for these types of relationships: {}

Instructions:
1. Analyze if the source element has any semantic relationships with the related elements
2. Consider imports, function calls, type usage, configuration, etc.
3. Provide confidence scores (0.0-1.0) based on evidence in the code
4. Include line numbers where relationships are evident
5. Return only relationships with confidence > 0.5

Return a JSON object with:
{{
  "relationships": [
    {{
      "source_id": {},
      "target_id": <target_id>,
      "relationship": "<relationship_type>",
      "confidence": 0.85,
      "explanation": "Brief explanation of the relationship",
      "line_reference": 42
    }}
  ],
  "explanation": "Overall analysis summary"
}}"#,
        context.file_path.display(),
        context.language,
        source_node.name,
        source_node.id.0,
        format!("{:?}", source_node.kind),
        source_node.line_start.unwrap_or(0),
        source_node.line_end.unwrap_or(0),
        source_node.qualified_name,
        context.enclosing_context,
        candidates_desc,
        relationship_types,
        source_node.id.0
    )
}

/// Generate a prompt for node summarization
pub fn node_summary_prompt(node: &GraphNode, context: &AnalysisContext) -> String {
    format!(r#"Summarize what this {} does in one concise sentence:

File: {}
Name: {}
Type: {:?}
Lines: {}-{}
Qualified name: {}

Context: {:?}

Provide a clear, technical summary of its purpose and functionality."#,
        format!("{:?}", node.kind),
        context.file_path.display(),
        node.name,
        node.kind,
        node.line_start.unwrap_or(0),
        node.line_end.unwrap_or(0),
        node.qualified_name,
        context.enclosing_context
    )
}

/// Generate a prompt for code question answering
pub fn code_question_prompt(
    question: &str,
    relevant_nodes: &[GraphNode],
    relevant_edges: &[GraphEdge],
) -> String {
    let nodes_desc = relevant_nodes.iter()
        .map(|n| format!("- {} ({}): {}", n.name, format!("{:?}", n.kind), n.qualified_name))
        .collect::<Vec<_>>()
        .join("\n");
        
    let edges_desc = relevant_edges.iter()
        .map(|e| format!("- {} -> {} ({})", e.source.0, e.target.0, format!("{:?}", e.kind)))
        .collect::<Vec<_>>()
        .join("\n");

    format!(r#"Based on this code graph information, answer the following question:

Question: {}

Relevant code elements:
{}

Relationships:
{}

Provide a clear, accurate answer based on the graph data. If the information is insufficient, explain what additional context would be needed."#,
        question,
        if nodes_desc.is_empty() { "No relevant nodes found." } else { &nodes_desc },
        if edges_desc.is_empty() { "No relevant relationships found." } else { &edges_desc }
    )
}

/// System prompt for code analysis
pub const CODE_ANALYSIS_SYSTEM_PROMPT: &str = r#"You are an expert code analysis AI assistant. Your role is to:

1. Accurately identify semantic relationships in code
2. Provide confidence scores based on evidence
3. Be conservative - only report relationships you're confident about
4. Focus on actual code patterns, not assumptions
5. Return valid JSON in the specified format

Common relationship patterns to look for:
- Function calls: functionA() { functionB(); }
- Type dependencies: let x: TypeB = ...
- Imports: import { something } from './module'
- Inheritance: class A extends B
- Interface implementation: class A implements B
- Configuration usage: config.get('key') or process.env.VAR"#;