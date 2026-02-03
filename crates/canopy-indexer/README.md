# canopy-indexer Module Documentation

## Overview

The `canopy-indexer` module is responsible for parsing source code files and extracting semantic information using tree-sitter parsers. It supports multiple programming languages and creates the graph structure that Canopy visualizes.

## Supported Languages

- **Rust** - Functions, structs, enums, traits, impl blocks
- **TypeScript/JavaScript** - Functions, classes, methods, interfaces, imports
- **Python** - Functions, classes, methods, decorators, imports
- **Go** - Functions, methods, structs, interfaces, imports
- **Java** - Classes, interfaces, methods, fields, imports
- **C/C++** - Functions, structs, enums, includes

## Architecture

### Parser Pool
- Thread-safe pool of tree-sitter parsers
- Avoids creating new parsers for each file
- Supports concurrent parsing operations

### Language Extractors
Each language has its own extractor that:
1. Parses the source code using tree-sitter
2. Walks the AST to find relevant nodes
3. Creates GraphNode entries for each concept
4. Creates GraphEdge entries for relationships

### Extraction Process
1. File is read and passed to the appropriate language extractor
2. Tree-sitter parses the code into an AST
3. The extractor walks the AST to find code entities
4. Nodes and edges are created and returned

## Usage

```rust
use canopy_indexer::languages::get_extractor;
use std::path::PathBuf;

let path = PathBuf::from("main.rs");
let extractor = get_extractor(&path).unwrap();
let result = extractor.extract(&path, source_code.as_bytes())?;

// Process the extracted nodes and edges
for node in result.nodes {
    println!("Found {}: {}", format!("{:?}", node.kind), node.name);
}
```

## Adding a New Language

1. Create a new extractor in `src/languages/`
2. Implement the `LanguageExtractor` trait
3. Register it in `src/languages/mod.rs`
4. Add the tree-sitter language dependency

## Testing

Run unit tests:
```bash
cargo test -p canopy-indexer
```

## Performance

The indexer is designed to be:
- **Fast**: Uses tree-sitter for efficient parsing
- **Concurrent**: Thread-safe parser pool
- **Incremental**: Only re-parses changed files
- **Memory-efficient**: Streams large files