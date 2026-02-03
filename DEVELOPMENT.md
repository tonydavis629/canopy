# Canopy Development Summary

## Project Overview
**Canopy** - A Rust CLI tool for live hierarchical code visualization for the AI-agent era.

**Current Focus**: Milestone 2 (Semantic Understanding) - 4-5 weeks scope

**Previous Milestone**: ✅ Milestone 1 (Navigable Hierarchy) - COMPLETED

## Project Structure
5-crate workspace:
- `canopy-core` - Graph data model, NodeId/EdgeId, symbols, aggregation, cache
- `canopy-indexer` - Language extractors, config parsers, heuristics
- `canopy-ai` - AI bridge (stub)
- `canopy-server` - HTTP/WebSocket server
- `canopy-watcher` - File watcher (stub)

## Current Status (2026-02-02)

# TONY's TODOs:
Back button should take you to a higher hierachy in the visualization.
The nodes need to be separated in space, and need to be connected with edges. The visualization is not visualizating any relationships. The point of the project is to make it clearer what the code is doing. Use the AI service to help understand the larger module uses and connections.
The only entities that are plotted are files. It should be any concept. All classes, functions, etc should have their own node.

Zoom should be implemented with scroll wheel, taking you deeper into specifics or higher in the 'canopy' (more general modules)
The pathing description at the top has a bug: Root/ sample-rust-project / src / main.rs / Cargo.toml / Cargo.toml / Cargo.toml / src / lib.rs / Cargo.toml. It is only additive.
- Comprehensive test suite (unit, integration, snapshot, browser)



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

### 🔄 In Progress (Milestone 2)
1. **Semantic Understanding & AI Integration** ✅ In Progress
   - ✅ Implemented AI-powered code analysis bridge in `canopy-ai`
   - ✅ Added semantic relationship detection (Calls, DependsOn, Uses, Configures)
   - ✅ Created OpenAI provider with GPT-4 integration
   - ✅ Added local heuristic provider for offline analysis
   - ✅ Implemented confidence scoring for AI-inferred relationships
   - ✅ Added budget tracking and caching for API usage
   - 🔄 Integrate LLM for code understanding and summarization
   - 🔄 Add Anthropic provider support

2. **Advanced Language Support**
   - 🔄 Complete Python language extractor implementation
   - 🔄 Complete Go language extractor implementation
   - 🔄 Complete Java language extractor implementation
   - 🔄 Add support for configuration files (YAML, TOML, JSON)
   - 🔄 Implement import/require resolution for all languages

3. **Enhanced Visualization**
   - 🔄 Add hierarchical layout options (tree, radial, force-directed presets)
   - 🔄 Implement semantic coloring and edge styling
   - 🔄 Add node clustering and aggregation views
   - 🔄 Create semantic zoom (show/hide details based on zoom level)

### 📋 Next Steps (Milestone 2)
1. **AI Bridge Implementation** ✅ Core Complete
   - ✅ Set up `canopy-ai` crate with LLM integration
   - ✅ Implement semantic analysis pipeline
   - ✅ Add relationship inference with confidence scoring
   - 🔄 Add Anthropic provider support
   - 🔄 Integrate AI analysis into file watcher

2. **Complete Language Extractors**
   - 🔄 Finish Python, Go, Java extractors (currently stubs)
   - 🔄 Add import resolution for all languages
   - 🔄 Implement cross-file reference tracking

3. **Semantic Visualization**
   - 🔄 Add hierarchical layout algorithms
   - 🔄 Implement semantic edge styling
   - 🔄 Create node clustering based on semantic relationships

4. **MCP Server Integration**
   - 🔄 Implement Model Context Protocol server
   - 🔄 Add graph-based context for AI agents
   - 🔄 Create semantic search endpoints

### ✅ Completed Milestones
- **Milestone 1**: Navigable Hierarchy - Live code visualization with basic interactions

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
- **Graph rendering fix**: Node ID assignment bug fixed
- **Performance optimization**: Viewport-based rendering implemented
- **Search feature**: Real-time node search with highlighting
- **Filter controls**: Node type filtering (directories, files, functions)
- **File watching**: Real-time updates with JavaScript extraction working
- **Milestone 2 - AI Bridge**: Implemented semantic analysis with OpenAI integration
- **Milestone 2 - AI Providers**: Added OpenAI, Anthropic (stub), and Local providers
- **Milestone 2 - Budget/Caching**: Added API budget tracking and result caching