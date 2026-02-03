# canopy-core Module Documentation

## Overview

The `canopy-core` module provides the fundamental data structures and operations for the Canopy code visualization system. It defines the graph model, node/edge types, and core algorithms for code analysis.

## Key Components

### Graph Data Structure
- `Graph`: The main graph structure using petgraph::StableDiGraph
- `GraphNode`: Represents code entities (functions, classes, files, etc.)
- `GraphEdge`: Represents relationships between nodes

### Node Types
- `NodeKind`: Enum defining different types of code entities
- `NodeId`: Unique identifier for nodes in the graph

### Edge Types
- `EdgeKind`: Defines relationship types (Calls, DependsOn, Uses, etc.)
- `EdgeId`: Unique identifier for edges in the graph
- `EdgeSource`: Indicates how the edge was determined (Structural, Heuristic, AI)

### Language Support
- `Language`: Enum for supported programming languages
- Automatic language detection from file extensions

## Usage

```rust
use canopy_core::{Graph, GraphNode, NodeKind};

// Create a new graph
let mut graph = Graph::new();

// Add a node
let node_id = graph.add_node(GraphNode {
    id: NodeId(0),
    kind: NodeKind::Function,
    name: "main".to_string(),
    // ... other fields
});

// Add an edge
graph.add_edge(GraphEdge {
    source: source_id,
    target: target_id,
    kind: EdgeKind::Calls,
    // ... other fields
});
```

## Testing

Run unit tests:
```bash
cargo test -p canopy-core
```

## Architecture

The core module is designed to be:
- **Fast**: Uses efficient graph algorithms
- **Extensible**: Easy to add new node/edge types
- **Serializable**: Full serde support for persistence
- **Type-safe**: Strong typing with Rust's type system