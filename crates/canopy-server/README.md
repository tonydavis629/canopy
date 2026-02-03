# canopy-server Module Documentation

## Overview

The `canopy-server` module provides the HTTP/WebSocket server that serves the Canopy visualization interface and API endpoints. It handles client connections and serves the web interface.

## Features

### HTTP Server
- **REST API** - Graph data endpoints
- **Static Files** - Serves the web interface
- **WebSocket** - Real-time updates for graph changes
- **CORS Support** - Cross-origin requests enabled

### Endpoints
- `GET /api/graph` - Returns complete graph as JSON
- `GET /` - Serves the web interface
- `WebSocket /ws` - Real-time graph updates

## Architecture

### Server Structure
- Built on Axum web framework
- Embedded static assets using rust-embed
- WebSocket support via tokio-tungstenite

### Request Flow
1. HTTP request received
2. Static assets served from embedded files
3. API requests processed and graph data returned
4. WebSocket connections maintained for real-time updates

## Usage

```rust
use canopy_server::{CanopyServer, ServerConfig};
use canopy_core::Graph;

// Create server with graph
let mut graph = Graph::new();
// ... populate graph ...

let config = ServerConfig {
    host: "127.0.0.1".to_string(),
    port: 7890,
};

let server = CanopyServer::new(graph, config);
server.start().await?;
```

## Static Assets

The server embeds the client files at compile time:
- HTML, CSS, JavaScript files from `../../client/`
- Served automatically when client requests files
- No separate web server needed

## WebSocket Protocol

### Messages from Server
- `{"type":"full_graph","graph":{...}}` - Complete graph data
- `{"type":"graph_diff","diff":{...}}` - Incremental updates

### Real-time Updates
- Graph changes are broadcast to all connected clients
- Clients receive and apply diffs to update visualization
- Efficient incremental updates for large codebases

## Configuration

```toml
[server]
host = "127.0.0.1"
port = 7890
```

## Security

- CORS enabled for development
- Host binding configurable
- No authentication required (for now)

## Testing

Run unit tests:
```bash
cargo test -p canopy-server
```

## Deployment

The server is designed for:
- **Development** - Local development with hot reload
- **Intranet** - Internal team code visualization
- **Production** - With appropriate security measures