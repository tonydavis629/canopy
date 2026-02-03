# Canopy 🌳

A Rust CLI tool for live hierarchical code visualization with AI-powered semantic analysis for the AI-agent era.

## Features

- **Live Code Visualization**: Real-time graph visualization of your codebase
- **AI-Powered Semantic Analysis**: Uses LLMs to infer relationships between code elements
- **Multi-Language Support**: Rust, TypeScript, JavaScript, Python, Go, Java, C, C++
- **File Watching**: Automatic updates as you code
- **WebSocket Integration**: Live updates in your browser
- **AI Providers**: OpenAI and Anthropic via OpenRouter
- **Hierarchical Layouts**: Tree, radial, and force-directed graph layouts
- **Semantic Relationships**: Calls, DependsOn, Uses, Configures, and more

## Installation

### From Source

```bash
git clone https://github.com/your-username/canopy.git
cd canopy
cargo build --release
```

The binary will be available at `target/release/canopy`

### Prerequisites

- Rust 1.70+ 
- OpenRouter API key (for AI features)

## Quick Start

1. **Set up your API key** (for AI features):
```bash
export OPENROUTER_API_KEY=your-api-key-here
```

2. **Index and serve your project**:
```bash
# Navigate to your project directory
cd /path/to/your/project

# Run Canopy
canopy serve
```

3. **Open your browser** to `http://localhost:7890`

## Usage Examples

### Basic Usage

```bash
# Serve current directory
canopy serve

# Serve specific directory
canopy serve --path /path/to/project

# Custom host and port
canopy serve --host 0.0.0.0 --port 8080
```

### Advanced Usage

```bash
# Clear the graph cache
canopy clear

# Index without serving
canopy index --path /path/to/project

# Enable debug logging
RUST_LOG=debug canopy serve
```

### Configuration File

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

## How It Works

### 1. Code Extraction
Canopy uses tree-sitter parsers to extract code entities:
- Functions and methods
- Classes and structs
- Imports and dependencies
- Configuration blocks

### 2. AI Semantic Analysis
When AI is enabled, Canopy analyzes your code to infer relationships:
- **Calls**: Function A calls function B
- **DependsOn**: Type A depends on type B
- **Uses**: Module A uses module B
- **Configures**: Function A configures component B

### 3. Real-time Updates
The file watcher monitors your codebase and:
- Updates the graph when files change
- Performs AI analysis on new code
- Broadcasts updates via WebSocket

### 4. Visualization
The web interface provides:
- Interactive node selection
- Relationship highlighting
- Search functionality
- Filter controls

## Project Structure

```
canopy/
├── crates/
│   ├── canopy-core/     # Graph data model and primitives
│   ├── canopy-indexer/  # Language-specific extractors
│   ├── canopy-ai/       # AI semantic analysis
│   ├── canopy-server/   # HTTP/WebSocket server
│   └── canopy-watcher/  # File system watcher
├── client/              # Web interface (HTML/CSS/JS)
└── src/                 # CLI entry point
```

## Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/your-username/canopy.git
cd canopy

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run with debug logging
RUST_LOG=debug cargo run -- serve
```

### Adding a New Language

1. Create a new extractor in `crates/canopy-indexer/src/languages/`
2. Implement the `LanguageExtractor` trait
3. Register it in `crates/canopy-indexer/src/languages/mod.rs`

Example:
```rust
pub struct MyExtractor {
    parser_pool: ParserPool,
}

impl LanguageExtractor for MyExtractor {
    fn extract(&self, path: &PathBuf, content: &[u8]) -> anyhow::Result<ExtractionResult> {
        // Your extraction logic here
    }
}
```

### AI Provider Integration

Canopy supports multiple AI providers through OpenRouter:

```rust
use canopy_ai::providers::create_provider;

let provider = create_provider("openai", Some(api_key))?;
// or
let provider = create_provider("anthropic", Some(api_key))?;
```

## Examples

### Analyzing a Rust Project

```bash
cd my-rust-project
canopy serve
```

Canopy will extract:
- Functions and their calls
- Struct definitions
- Module imports
- Trait implementations

### AI-Powered Relationship Detection

With AI enabled, Canopy can infer:
```rust
// Canopy AI might detect that:
// - `process_data` calls `validate_input`
// - `Database` depends on `ConnectionPool`
// - `handle_request` uses `json_parser`
```

### Web Interface Features

- **Search**: Find nodes by name or type
- **Filter**: Show/hide specific node types
- **Zoom**: Focus on specific areas
- **Relationships**: Click nodes to see connections

## Troubleshooting

### AI Analysis Not Working
- Check your `OPENROUTER_API_KEY` is set
- Verify network connectivity
- Check logs for AI provider errors

### File Watcher Issues
- Ensure you have read permissions
- Check ignored patterns in `.canopy.toml`
- Try disabling other file watchers

### Performance
- For large codebases, increase `batch_size` in config
- Consider disabling AI for initial indexing
- Use `--no-ai` flag to skip AI analysis

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [tree-sitter](https://tree-sitter.github.io/) for parsing
- Uses [petgraph](https://github.com/petgraph/petgraph) for graph operations
- AI powered by OpenRouter
- Visualization with D3.js