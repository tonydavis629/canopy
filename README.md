# Canopy 🌳

A Rust CLI tool for live hierarchical code visualization with AI-powered semantic analysis for the AI-agent era.

## Overview

Canopy provides a clean, hierarchical visualization of your codebase that makes relationships between code elements clear and navigable. It uses AI to infer semantic relationships that aren't obvious from static analysis alone.

## Features

- **Live Code Visualization** - Real-time graph visualization of your codebase
- **AI-Powered Semantic Analysis** - Uses LLMs to infer relationships between code elements
- **Multi-Language Support** - Rust, TypeScript, JavaScript, Python, Go, Java, C, C++
- **Hierarchical Navigation** - Clean grid-based interface with drill-down capability
- **No Animation** - Static, diagram-like visualization for clarity
- **Zoom Navigation** - Mouse wheel for hierarchical navigation
- **AI Integration** - OpenAI and Anthropic via OpenRouter

## Quick Start

1. **Install Canopy**:
```bash
cargo install --path .
```

2. **Set up API key** (for AI features):
```bash
export OPENROUTER_API_KEY=your-api-key-here
```

3. **Run Canopy**:
```bash
# Serve current directory
canopy

# Serve specific directory
canopy /path/to/project

# Custom port and host
canopy -p 8080 --host 0.0.0.0
```

4. **Open browser** to http://localhost:7890

## Module Documentation

### [canopy-core](crates/canopy-core/)
Core graph data structures and operations. Defines nodes, edges, and graph algorithms.

### [canopy-indexer](crates/canopy-indexer/)
Language parsers using tree-sitter. Extracts code entities from 8 programming languages.

### [canopy-ai](crates/canopy-ai/)
AI-powered semantic analysis. Infers relationships between code elements using LLMs.

### [canopy-server](crates/canopy-server/)
HTTP/WebSocket server. Serves the visualization interface and API endpoints.

### [canopy-watcher](crates/canopy-watcher/)
File system monitoring. Provides real-time updates as you code.

## Architecture

```
┌─ Client (Browser) ─┐
│  Grid-based UI     │
│  Hierarchical View │
└────────┬───────────┘
         │ WebSocket
┌─ canopy-server ────┐
│  HTTP API          │
│  Static Files      │
└────────┬───────────┘
         │
┌─ canopy-watcher ───┐
│  File Monitoring   │
│  Change Detection  │
└────────┬───────────┘
         │
┌─ canopy-indexer ───┐
│  Tree-sitter Parsers│
│  AST Extraction    │
└────────┬───────────┘
         │
┌─ canopy-core ──────┐
│  Graph Structure   │
│  Node/Edge Model   │
└────────────────────┘
```

## Configuration

Create a `.canopy.toml` file in your project root:

```toml
[ai]
provider = "openai"  # or "anthropic"
api_key = "your-api-key"
enabled = true

[server]
host = "127.0.0.1"
port = 7890

[watch]
ignore_patterns = ["target", "node_modules", ".git"]
```

## Testing

Run all tests:
```bash
cargo test
```

Run specific module tests:
```bash
cargo test -p canopy-core
cargo test -p canopy-indexer
cargo test -p canopy-ai
cargo test -p canopy-server
cargo test -p canopy-watcher
```

## Development

```bash
# Build all crates
cargo build --workspace

# Run with debug logging
RUST_LOG=debug cargo run

# Run specific example
cargo run -- examples/sample-rust-project
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.