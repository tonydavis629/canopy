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
   - Successfully indexed 5158 nodes, 5157 edges

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

6. **WebSocket Protocol Fix** ✨
   - Server now sends `full_graph` message on WebSocket connect
   - Graph data properly serialized with nodes/edges arrays
   - Frontend receives and can process graph data

7. **Tree-sitter Language Extraction** 🌳
   - Implemented Rust language extractor with tree-sitter
   - Implemented TypeScript language extractor with tree-sitter
   - Extracts functions, classes, methods, and imports
   - Creates GraphNode and GraphEdge entries for symbols
   - All language extractors properly structured

### 🔄 In Progress
1. **Graph Visualization**
   - Nodes not rendering yet (need to verify D3.js integration)
   - Need to implement proper graph layout
   - Add interactive features (zoom, pan, click)

2. **File Watching**
   - Watch for file system changes
   - Send updates through WebSocket
   - Integrate with notify crate

3. **Language Extraction Enhancement**
   - Test tree-sitter extractors with real code
   - Improve symbol detection accuracy
   - Add more language support

### 📋 Next Steps
1. Implement D3.js graph rendering
2. Add node click handlers for sidebar details
3. Implement file watching with notify crate
4. Test tree-sitter extractors with real codebases
5. Create comprehensive test suite

### 🧪 Testing
- Manual browser testing completed
- Server accessible at http://127.0.0.1:7890
- API endpoint verified working
- WebSocket connection established
- Tree-sitter extractors compile successfully

### 📁 Key Files
- `/src/main.rs` - CLI entry point
- `/src/commands.rs` - Command implementations
- `/crates/canopy-server/src/lib.rs` - Server core
- `/crates/canopy-server/src/websocket.rs` - WebSocket protocol
- `/crates/canopy-indexer/src/languages/rust.rs` - Rust extractor
- `/crates/canopy-indexer/src/languages/typescript.rs` - TypeScript extractor
- `/client/index.html` - Browser interface
- `/client/graph.js` - Visualization logic
- `/client/protocol.js` - WebSocket handling

## Development Notes
- Using coding-agent skill (Claude Code) for implementations
- Following test-driven development
- Committing directly to main when confident
- Manual testing with Chrome browser
- Focus on M1 scope only (no M2 features yet)

## Blockers
None currently - proceeding with graph visualization

## Recent Commits
- Working CLI with filesystem indexing
- HTTP server implementation
- Browser client scaffolding
- WebSocket connection establishment
- WebSocket protocol fix (full_graph message)
- Tree-sitter language extraction for Rust/TypeScript