# Canopy Usage Guide

This guide demonstrates how to use Canopy to visualize and analyze your codebase.

## Quick Start

1. **Install Canopy**:
```bash
cargo install --path .
```

2. **Set up API key** (for AI features):
```bash
export OPENROUTER_API_KEY=your-api-key-here
```

3. **Run on your project**:
```bash
cd /path/to/your/project
canopy serve
```

4. **Open browser**: http://localhost:7890

## Basic Usage Examples

### Example 1: Simple Rust Project

```bash
# Navigate to a Rust project
cd my-rust-app

# Start Canopy
canopy serve

# Canopy will extract:
# - Functions and their calls
# - Struct and enum definitions
# - Module imports
# - Trait implementations
```

### Example 2: TypeScript/React Project

```bash
# Navigate to a TypeScript project
cd my-react-app

# Start Canopy
canopy serve

# Canopy will extract:
# - React components
# - Functions and methods
# - Import statements
# - Type definitions
```

### Example 3: Python Project

```bash
# Navigate to a Python project
cd my-python-app

# Start Canopy
canopy serve

# Canopy will extract:
# - Functions and classes
# - Methods within classes
# - Import statements
# - Decorators
```

## AI-Powered Analysis

When AI is enabled, Canopy will infer semantic relationships:

### What AI Detects:
- **Calls**: Function A calls function B
- **DependsOn**: Type A depends on type B
- **Uses**: Module A uses module B
- **Configures**: Function A configures component B

### Example Analysis:
```rust
// Canopy AI might infer:
fn process_data() {
    let data = validate_input();  // Calls relationship
    let db = Database::new();     // DependsOn relationship
    db.save(data);                // Uses relationship
}
```

## Web Interface Features

### Navigation
- **Pan**: Click and drag the canvas
- **Zoom**: Mouse wheel or pinch gesture
- **Select**: Click on nodes to highlight relationships

### Search
- Use the search box to find nodes by name
- Results are highlighted in the graph

### Filters
- Toggle visibility of:
  - Directories
  - Files
  - Functions
  - Classes
  - AI-inferred relationships

### Node Information
Click on any node to see:
- Node type and location
- Connected nodes
- AI-generated summary (if available)

## Configuration

Create a `.canopy.toml` file in your project root:

```toml
[ai]
provider = "openai"  # or "anthropic"
api_key = "your-api-key"
enabled = true
confidence_threshold = 0.7  # Minimum confidence for AI relationships

[server]
host = "127.0.0.1"
port = 7890

[watch]
ignore_patterns = ["target", "node_modules", ".git", "build"]
debounce_ms = 500

[limits]
max_nodes = 10000
max_edges = 50000
ai_batch_size = 10
```

## Advanced Usage

### Without AI
```bash
# Skip AI analysis for faster startup
canopy serve --no-ai

# Or disable in config
[ai]
enabled = false
```

### Custom Analysis
```bash
# Only analyze specific file types
canopy serve --include "*.rs,*.ts"

# Exclude certain directories
canopy serve --exclude "tests,benches"
```

### Export Graph Data
```bash
# Get graph as JSON
curl http://localhost:7890/api/graph > graph.json

# Use with other tools
jq '.nodes[] | select(.kind == "Function")' graph.json
```

## Performance Tips

### Large Codebases
- Increase `max_nodes` and `max_edges` in config
- Disable AI for initial indexing
- Use include/exclude patterns to focus on specific areas

### AI Optimization
- Increase `ai_batch_size` for faster analysis
- Adjust `confidence_threshold` to filter results
- Use specific relationship types to reduce API calls

## Troubleshooting

### AI Not Working
```bash
# Check API key
echo $OPENROUTER_API_KEY

# Check logs
RUST_LOG=debug canopy serve

# Test with local provider
canopy serve --provider local
```

### File Watcher Issues
```bash
# Check permissions
ls -la /path/to/project

# Try manual indexing
canopy index --path /path/to/project
```

### Performance Issues
```bash
# Reduce graph complexity
canopy serve --max-depth 3

# Disable certain node types
canopy serve --no-directories
```

## Examples by Language

### Rust
```rust
// Extracted: User struct, new() method, UserService struct
pub struct User {
    pub name: String,
}

impl User {
    pub fn new(name: String) -> Self {
        User { name }
    }
}
```

### TypeScript
```typescript
// Extracted: User class, constructor, methods
export class User {
    constructor(public name: string) {}
    
    greet() {
        return `Hello ${this.name}`;
    }
}
```

### Python
```python
# Extracted: User class, __init__ method, greet method
class User:
    def __init__(self, name: str):
        self.name = name
    
    def greet(self):
        return f"Hello {self.name}"
```

## Next Steps

1. **Explore your codebase**: Use the search and filter features
2. **Analyze relationships**: Click on nodes to see connections
3. **Customize views**: Adjust layouts and filters
4. **Integrate with workflow**: Use Canopy during code reviews

For more examples, see the `examples/` directory in the repository.