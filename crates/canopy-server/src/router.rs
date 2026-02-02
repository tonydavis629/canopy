//! Axum router setup for the Canopy server

use std::sync::Arc;

use axum::{routing::get, Router};
use tower_http::cors::CorsLayer;

use crate::{
    assets::static_handler,
    handlers::{get_graph, health_check},
    websocket::ws_handler,
    ServerState,
};

/// Create the axum router with all routes
pub fn create_router(state: Arc<ServerState>) -> Router {
    Router::new()
        // WebSocket endpoint for real-time updates
        .route("/ws", get(ws_handler))
        // REST API endpoints
        .route("/api/graph", get(get_graph))
        .route("/api/health", get(health_check))
        // Static file serving
        .route("/", get(static_handler))
        .route("/*path", get(static_handler))
        // Add CORS support
        .layer(CorsLayer::permissive())
        // Add state
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use canopy_core::Graph;

    #[test]
    fn test_router_creation() {
        let graph = Graph::new();
        let state = Arc::new(ServerState::new(graph));
        let _router = create_router(state);
        // Router creation should succeed
        assert!(true);
    }
}
