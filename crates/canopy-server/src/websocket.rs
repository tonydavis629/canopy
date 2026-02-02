//! WebSocket handling for real-time graph updates

use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

use crate::ServerState;

/// WebSocket message types for client-server communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    /// Client requests the full graph
    #[serde(rename = "request_full_graph")]
    RequestFullGraph,
    /// Server sends the full graph
    #[serde(rename = "full_graph")]
    FullGraph { graph: GraphData },
    /// Server broadcasts a graph diff
    #[serde(rename = "graph_diff")]
    GraphDiff { diff: DiffData },
    /// Client acknowledges a diff
    #[serde(rename = "diff_ack")]
    DiffAck { sequence: u64 },
    /// Client subscribes to updates
    #[serde(rename = "subscribe")]
    Subscribe,
    /// Client unsubscribes from updates
    #[serde(rename = "unsubscribe")]
    Unsubscribe,
    /// Ping/pong for keepalive
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "pong")]
    Pong,
    /// Error message
    #[serde(rename = "error")]
    Error { message: String },
}

/// Graph data structure for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<canopy_core::model::GraphNode>,
    pub edges: Vec<canopy_core::model::GraphEdge>,
    pub sequence: u64,
}

/// Diff data structure for incremental updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffData {
    pub sequence: u64,
    pub modified_nodes: Vec<canopy_core::model::NodeId>,
    pub changes: serde_json::Value,
}

/// Convert the current graph to GraphData format expected by frontend
async fn graph_to_graph_data(state: &Arc<ServerState>) -> GraphData {
    let graph = state.graph.read().await;
    
    // Collect all nodes
    let nodes = graph.all_nodes().map(|n| n.clone()).collect();

    // Collect all edges
    let edges = graph.all_edges().map(|e| e.clone()).collect();

    GraphData {
        nodes,
        edges,
        sequence: 0,
    }
}

/// Handle WebSocket upgrade requests
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle an individual WebSocket connection
async fn handle_socket(socket: WebSocket, state: Arc<ServerState>) {
    info!("New WebSocket connection established");

    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.diff_tx.subscribe();

    // Send full graph immediately after connection
    let full_graph_data = graph_to_graph_data(&state).await;
    let full_graph_msg = WsMessage::FullGraph { graph: full_graph_data };
    
    if let Ok(json_msg) = serde_json::to_string(&full_graph_msg) {
        if sender.send(Message::Text(json_msg)).await.is_err() {
            warn!("Failed to send initial full graph to WebSocket client");
            return;
        }
        info!("Sent full graph to WebSocket client");
    } else {
        warn!("Failed to serialize full graph message");
    }

    // Spawn a task to handle incoming messages from the client
    let state_clone = Arc::clone(&state);
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                debug!("Received WebSocket message: {}", text);
                
                match serde_json::from_str::<WsMessage>(&text) {
                    Ok(ws_msg) => {
                        handle_client_message(ws_msg, &state_clone).await;
                    }
                    Err(e) => {
                        warn!("Failed to parse WebSocket message: {}", e);
                    }
                }
            } else if let Message::Close(_) = msg {
                debug!("WebSocket client disconnected");
                break;
            }
        }
    });

    // Spawn a task to broadcast diffs to the client
    let mut send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    if sender.send(Message::Text(msg)).await.is_err() {
                        debug!("Failed to send message to WebSocket client");
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    warn!("WebSocket client lagged behind");
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
        }
        _ = (&mut recv_task) => {
            send_task.abort();
        }
    }

    info!("WebSocket connection closed");
}

/// Handle messages received from the WebSocket client
async fn handle_client_message(msg: WsMessage, _state: &ServerState) {
    match msg {
        WsMessage::RequestFullGraph => {
            debug!("Client requested full graph");
            // The full graph is sent automatically on connection and can be re-requested
        }
        WsMessage::Subscribe => {
            debug!("Client subscribed to updates");
        }
        WsMessage::Unsubscribe => {
            debug!("Client unsubscribed from updates");
        }
        WsMessage::DiffAck { sequence } => {
            debug!("Client acknowledged diff with sequence: {}", sequence);
        }
        WsMessage::Ping => {
            debug!("Received ping");
        }
        _ => {
            debug!("Received message: {:?}", msg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canopy_core::Graph;

    #[test]
    fn test_ws_message_serialization() {
        let msg = WsMessage::Ping;
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("ping"));

        let msg = WsMessage::Pong;
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("pong"));
    }

    #[tokio::test]
    async fn test_broadcast() {
        let graph = Graph::new();
        let state = ServerState::new(graph);
        
        let msg = "test message".to_string();
        let result = state.broadcast(msg);
        assert!(result.is_ok());
    }
}
