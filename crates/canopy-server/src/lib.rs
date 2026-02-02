//! HTTP + WebSocket server for Canopy

pub mod assets;
pub mod handlers;
pub mod router;
pub mod websocket;

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use canopy_core::Graph;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, RwLock};
use tracing::info;

use crate::router::create_router;

/// Server configuration options
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Port to listen on (default: 7890)
    pub port: u16,
    /// Host to bind to (default: 127.0.0.1)
    pub host: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 7890,
            host: "127.0.0.1".to_string(),
        }
    }
}

/// Shared state for the Canopy server
pub struct ServerState {
    /// The current graph being served
    pub graph: Arc<RwLock<Graph>>,
    /// Broadcast channel for graph diffs to WebSocket clients
    pub diff_tx: broadcast::Sender<String>,
}

impl std::fmt::Debug for ServerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerState")
            .field("graph", &"<Graph>")
            .field("diff_tx", &self.diff_tx)
            .finish()
    }
}

impl ServerState {
    pub fn new(graph: Graph) -> Self {
        let (diff_tx, _) = broadcast::channel(100);
        Self {
            graph: Arc::new(RwLock::new(graph)),
            diff_tx,
        }
    }

    /// Update the graph and broadcast the diff to all connected WebSocket clients
    pub async fn update_graph(&self, new_graph: Graph) -> Result<()> {
        let mut graph = self.graph.write().await;
        *graph = new_graph;
        Ok(())
    }

    /// Broadcast a message to all connected WebSocket clients
    pub fn broadcast(&self, message: String) -> Result<usize> {
        match self.diff_tx.send(message) {
            Ok(count) => Ok(count),
            Err(_) => Ok(0),
        }
    }
}

/// The main Canopy HTTP/WebSocket server
pub struct CanopyServer {
    config: ServerConfig,
    state: Arc<ServerState>,
}

impl std::fmt::Debug for CanopyServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CanopyServer")
            .field("config", &self.config)
            .field("state", &self.state)
            .finish()
    }
}

impl CanopyServer {
    /// Create a new CanopyServer with the given graph and configuration
    pub fn new(graph: Graph, config: ServerConfig) -> Self {
        let state = Arc::new(ServerState::new(graph));
        Self { config, state }
    }

    /// Create a new CanopyServer with default configuration
    pub fn with_graph(graph: Graph) -> Self {
        Self::new(graph, ServerConfig::default())
    }

    /// Get a clone of the server state for external use
    pub fn state(&self) -> Arc<ServerState> {
        Arc::clone(&self.state)
    }

    /// Start the HTTP server
    pub async fn start(&self) -> Result<()> {
        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port)
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

        let router = create_router(Arc::clone(&self.state));

        let listener = TcpListener::bind(&addr).await?;
        info!("Canopy server listening on http://{}", addr);

        axum::serve(listener, router).await?;

        Ok(())
    }

    /// Start the server in a background task
    pub fn spawn(self) -> tokio::task::JoinHandle<Result<()>> {
        tokio::spawn(async move { self.start().await })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let graph = Graph::new();
        let server = CanopyServer::with_graph(graph);
        assert_eq!(server.config.port, 7890);
    }

    #[tokio::test]
    async fn test_server_config() {
        let graph = Graph::new();
        let config = ServerConfig {
            port: 8080,
            host: "0.0.0.0".to_string(),
        };
        let server = CanopyServer::new(graph, config);
        assert_eq!(server.config.port, 8080);
        assert_eq!(server.config.host, "0.0.0.0");
    }
}
