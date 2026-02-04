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

## Current Status (2026-02-04)
TODO remove semantic zoom, its confusing. zoom should not change node expansion.
Still just showing file connections. need to break it down further into objects and functions.
need to associate config files and config values to objects/functions/files/modules. need sideway branches - its a network not a tree. no root node needed. semantic edges should connect modules.

### ✅ Frontend Stabilization Updates (2026-02-04)
1. **Semantic zoom**: zoom now auto-expands/contracts hierarchy levels.
2. **Click behavior**: clicking a container toggles expand/collapse.
3. **Edges**: containment edges are rendered and visible.
4. **Layout**: static vertical tree layout replaces orbit/force.
5. **AI summary**: remove fallback summaries when no AI data exists.

### 🔄 Remaining Immediate TODOs
1. Add a `.canopyignore` (or reuse `.gitignore`) to avoid indexing build output and vendor directories.

### ✅ Completed (Short)
1. **Core plumbing**: workspace, graph model, CLI, HTTP/WS server, static client serving
2. **Frontend baseline**: modern SVG graph, controls, legend, details panel
3. **Language extraction baseline**: Rust + TypeScript extractors wired and compiling

### 🔄 In Progress (Milestone 2)
1. **Rust-First Extraction Hardening (Priority)**
   - 🔄 Expand Rust extractor to cover modules, `mod` trees, and `use` resolution
   - 🔄 Extract traits/impl blocks, associated methods, and generics
   - 🔄 Capture macro invocations and derive-driven relationships (best-effort)
   - 🔄 Map Rust symbols to file paths and line ranges deterministically
   - 🔄 Emit structural edges: imports, calls, type references, implements/traits

2. **Semantic Understanding & AI Integration**
   - ✅ AI bridge scaffolding (OpenAI + local provider, caching, budgets)
   - 🔄 Integrate LLM summarization into the pipeline
   - 🔄 Add Anthropic provider support
   - 🔄 Persist AI summaries onto nodes for frontend display

3. **Enhanced Visualization (Priority)**
   - 🔄 Stabilize layout so nodes stay visible/readable
   - 🔄 Enforce <= 500 visible nodes with automatic abstraction
   - 🔄 Semantic zoom: workspace → packages/modules → directories → files → symbols
   - 🔄 Hierarchical/module-aware aggregation and edge rollups
   - 🔄 Expand semantic coloring + full legend (per-node-kind + edge-source)
   - 🔄 Layout presets that never collapse into unreadable lines

4. **Advanced Language + Config Support**
   - 🔄 Complete Python/Go/Java extractors
   - 🔄 Add config parsing (YAML/TOML/JSON/Dockerfile/etc.)
   - 🔄 Implement cross-file import/require resolution

### 📋 Next Steps (Milestone 2 - Explicit Stages)
1. **Stage A: Rust Extractor Completion (Rust-First)**
   - Implement full Rust symbol coverage: modules, traits, impl blocks, methods, consts, type aliases
   - Add `use`/path resolution for intra-crate references
   - Emit Imports/Calls/TypeReference/Implements edges for Rust
   - Verify with fixtures in `tests/fixtures` and snapshot tests

2. **Stage B: Module Graph Layer**
   - Add module nodes to the core graph model
   - Roll files up into modules and emit module-to-module edges
   - Extend aggregation logic for module-aware rollups

3. **Stage C: Frontend Solidification**
   - Enforce max 500 visible nodes with automatic abstraction
   - Implement semantic zoom (auto-levels by zoom)
   - Make layouts never collapse into a line; keep labels readable
   - Upgrade legend to full per-kind colors and edge-source styling
   - Show AI summary + structural metadata in details panel

4. **Stage D: AI Summaries**
   - Wire summarization into indexing or post-index pipeline
   - Cache and persist summaries on nodes
   - Surface summaries in API/WS payloads

5. **Stage E: Remaining Language + Config Support**
   - Finish Python/Go/Java extractors
   - Add config parsing and link config-to-code
   - Add cross-file reference tracking across languages

6. **Stage F: MCP Server**
   - Implement MCP endpoints for graph queries
   - Add semantic search endpoints

### 🔜 Immediate Next Steps (2026-02-03)
1. **Backend Module Graph (Stage B focus)**
   - Add explicit module nodes (directory-level and language module constructs).
   - Emit module-to-module edges derived from file/symbol edges.
2. **AI Summaries on Initial Index (Stage D focus)**
   - Generate summaries for existing graph nodes at startup (not just new changes).
   - Cache + persist summaries and include them in API/WS payloads.
3. **Frontend Diff + Aggregation Polish (Stage C focus)**
   - Improve `client/protocol.js` to apply diffs incrementally.
   - Use backend aggregation hints to avoid path-based fallback in the UI.

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
- Following test-driven development
- Committing directly to main when confident
- Manual testing with Chrome browser
- Focus on Milestone 2 with frontend stabilization first

## Frontend Behavior Requirements (Must Haves)
1. **Visibility**
   - Nodes must always be visible and readable on screen.
   - Layouts must never collapse into a single vertical or horizontal line.

2. **Scale Control**
   - Never render more than 500 nodes at once.
   - When the graph would exceed 500 nodes, automatically aggregate to a higher level.

3. **Semantic Zoom**
   - Zoomed out: show workspace/packages/modules.
   - Mid zoom: show directories and module boundaries.
   - Zoomed in: show files and code entities.
   - Aggregation must be automatic and reversible as zoom changes.

4. **Modules**
   - Represent module nodes (not just files).
   - Files roll up into modules.
   - Modules connect to other modules via aggregated edges.

5. **Edge Semantics**
   - Distinguish syntactic (structural) edges from semantic (AI/heuristic) edges.
   - Edge styling and legend must clearly reflect the distinction.

6. **Legend & Colors**
   - Legend must show distinct colors for each node kind and edge class.
   - Colors must be consistent across the graph and details panel.

7. **Details Panel**
   - Include a short AI-informed summary for each object (node).
   - Show both structural metadata (path, kind, symbols) and AI summary.

## Missing Work Plan (to close gaps)
1. **Frontend Solidification**
   - Implement zoom-driven aggregation with a hard 500-node cap.
   - Introduce module nodes and module-level edges.
   - Fix layout presets to preserve readability and always keep nodes on screen.
   - Update legend with per-kind colors and edge source styling.
   - Add AI summary to details panel.

2. **Backend Support for Frontend**
   - Add module nodes and module edges in the indexer/core graph.
   - Expose aggregation levels and node-count hints in API/WS payloads.
   - Provide AI summaries for nodes via canopy-ai (cached).

3. **AI Bridge**
   - Add Anthropic provider and wire into summarization pipeline.
   - Store and serve AI summaries in GraphNode metadata.

4. **Language & Config Extraction**
   - Finish Python/Go/Java extractors.
   - Implement config parsing (YAML/TOML/JSON) and import resolution.

5. **Testing**
   - Add browser tests for zoom/aggregation and node-cap behavior.
   - Add snapshot tests for module aggregation and edge styling metadata.

## Execution Plan (Next 2 Weeks)
1. **Week 1: Backend Structure + AI**
   - Ship module nodes + module edges (Stage B).
   - Persist AI summaries on initial index and diff updates (Stage D).
   - Define `.canopyignore` and update watcher/indexer to respect it.
2. **Week 2: Frontend + Tests**
   - Complete semantic zoom thresholds + node-cap enforcement (Stage C).
   - Add module-aware aggregation and edge rollups in UI.
   - Add browser regression tests for zoom/cap/layout stability.

## Blockers
None currently - proceeding with graph visualization

## Recent Commits
- Working CLI with filesystem indexing
- HTTP server implementation
- Browser client scaffolding
- Frontend graph overhaul: SVG nodes/edges, modern styling, no emojis
- Layout toggle + fit/reset controls
- Search, filters, and details panel improvements
- WebSocket connection establishment
- WebSocket protocol fix (full_graph message)
- Tree-sitter language extraction for Rust/TypeScript
- **Graph rendering fix**: Node ID assignment bug fixed
- **Search feature**: Real-time node search with highlighting
- **Filter controls**: Node type filtering (directories, files, symbols)
- **File watching**: Real-time updates with JavaScript extraction working
- **Milestone 2 - AI Bridge**: Implemented semantic analysis with OpenAI integration
- **Milestone 2 - AI Providers**: Added OpenAI, Anthropic (stub), and Local providers
- **Milestone 2 - Budget/Caching**: Added API budget tracking and result caching
