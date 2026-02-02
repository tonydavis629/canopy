//! REST API handlers for the Canopy server

use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Serialize;

use crate::ServerState;

/// Response structure for the graph API
#[derive(Debug, Serialize)]
pub struct GraphResponse {
    pub nodes: Vec<NodeResponse>,
    pub edges: Vec<EdgeResponse>,
}

/// Simplified node representation for the API
#[derive(Debug, Serialize)]
pub struct NodeResponse {
    pub id: u64,
    pub kind: String,
    pub name: String,
    pub qualified_name: String,
    pub file_path: String,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub language: Option<String>,
    pub is_container: bool,
    pub child_count: u32,
    pub loc: Option<u32>,
}

/// Simplified edge representation for the API
#[derive(Debug, Serialize)]
pub struct EdgeResponse {
    pub id: u64,
    pub source: u64,
    pub target: u64,
    pub kind: String,
    pub edge_source: String,
    pub confidence: f32,
    pub label: Option<String>,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Get the current graph as JSON
pub async fn get_graph(
    State(state): State<Arc<ServerState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let graph = state.graph.read().await;
    
    // Collect all nodes
    let mut nodes = Vec::new();
    // We need to iterate through the graph to get all nodes
    // Since Graph doesn't expose a direct nodes iterator, we'll use a workaround
    // by checking node indices from 0 to node_count
    for i in 0..graph.node_count() {
        let node_id = canopy_core::NodeId(i as u64);
        if let Some(node) = graph.node(node_id) {
            nodes.push(NodeResponse {
                id: node.id.0,
                kind: format!("{:?}", node.kind),
                name: node.name.clone(),
                qualified_name: node.qualified_name.clone(),
                file_path: node.file_path.to_string_lossy().to_string(),
                line_start: node.line_start,
                line_end: node.line_end,
                language: node.language.map(|l| format!("{:?}", l)),
                is_container: node.is_container,
                child_count: node.child_count,
                loc: node.loc,
            });
        }
    }

    // Collect all edges
    let mut edges = Vec::new();
    // We need to iterate through all possible edge indices
    for edge_ref in graph.all_edges() {
        edges.push(EdgeResponse {
            id: edge_ref.id.0,
            source: edge_ref.source.0,
            target: edge_ref.target.0,
            kind: format!("{:?}", edge_ref.kind),
            edge_source: format!("{:?}", edge_ref.edge_source),
            confidence: edge_ref.confidence,
            label: edge_ref.label.clone(),
        });
    }

    let response = GraphResponse { nodes, edges };
    Ok(Json(response))
}

/// Health check endpoint
pub async fn health_check() -> impl IntoResponse {
    let health = HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    Json(health)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let _response = health_check().await;
        // Should succeed
    }
}
