# Canopy Development Summary

## Project Overview
**Canopy** - A Rust CLI tool for live hierarchical code visualization for the AI-agent era.

**Current Focus**: Milestone 1 (Navigable Hierarchy) only - 5-6 weeks scope

## Project Structure
5-crate workspace:
- `canopy-core` - Graph data model, NodeId/EdgeId, symbols, aggregation, cache
- `canopy-indexer` - Language extractors, config parsers, heuristics
- `canopy-ai` - AI bridge (stub)
- `canopy-server` - HTTP/WebSocket server
- `canopy-watcher` - File watcher (stub)

## Current Status (2026-02-02)

### ✅ Completed
1. **Workspace Setup**
   - All 5 crates compile successfully
   - Rust edition 2024, Linux target only
   - Dependencies: petgraph, tree-sitter, clap, axum, tokio

2. **Core Implementation**
   - Graph data structure with StableDiGraph
   - NodeId/EdgeId types with custom hashing
   - GraphNode/GraphEdge models with serialization
   - Symbols, aggregation, diff, and cache modules

3. **CLI Implementation**
   - Commands: serve, index, clear, version
   - Filesystem walking with Directory/File nodes
   - Contains edges between parent/child
   - Successfully indexed 5017 nodes, 5016 edges

4. **HTTP Server**
   - Axum-based server on port 7890
   - REST API: GET /api/graph returns JSON
   - Static file serving for client assets
   - WebSocket endpoint at /ws

5. **Browser Client**
   - HTML/CSS interface with dark theme
   - D3.js integration
   - WebSocket connection established
   - Responsive layout with sidebar

### 🔄 In Progress
1. **WebSocket Protocol Alignment**
   - Frontend expects `full_graph` / `graph_diff` messages
   - Server needs to send initial graph on connect
   - Message schema standardization

2. **Graph Visualization**
   - Nodes not rendering yet (protocol mismatch)
   - Need to implement D3.js graph rendering
   - Add interactive features (zoom, pan, click)

### 📋 Next Steps
1. Fix WebSocket protocol to send full graph on connect
2. Implement D3.js graph rendering
3. Add node click handlers for sidebar details
4. Implement file watching with notify crate
5. Add tree-sitter language extraction
6. Create comprehensive test suite

### 🧪 Testing
- Manual browser testing completed
- Server accessible at http://127.0.0.1:7890
- API endpoint verified working
- WebSocket connection established

### 📁 Key Files
- `/src/main.rs` - CLI entry point
- `/src/commands.rs` - Command implementations
- `/crates/canopy-server/src/lib.rs` - Server core
- `/client/index.html` - Browser interface
- `/client/graph.js` - Visualization logic
- `/client/protocol.js` - WebSocket handling

## Development Notes
- Using OpenCode for all code implementations
- Following test-driven development
- Committing directly to main when confident
- Manual testing with Chrome browser
- Focus on M1 scope only (no M2 features yet)

## Blockers
None currently - proceeding with WebSocket protocol fixes

## Recent Commits
- Working CLI with filesystem indexing
- HTTP server implementation
- Browser client scaffolding
- WebSocket connection establishment