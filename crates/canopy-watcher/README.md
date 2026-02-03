# canopy-watcher Module Documentation

## Overview

The `canopy-watcher` module provides real-time file system monitoring for automatic graph updates. It watches for file changes and updates the visualization graph accordingly.

## Features

### File System Monitoring
- **Real-time Updates** - Detects file creation, modification, and deletion
- **Multi-language Support** - Watches all supported programming languages
- **Debounced Updates** - Batches rapid changes to avoid excessive updates
- **Ignore Patterns** - Skips build directories, git folders, etc.

### Integration
- Works seamlessly with the graph data structure
- Integrates with AI analysis for new files
- Broadcasts updates via WebSocket to connected clients

## Architecture

### Watcher Service
- Uses notify crate for cross-platform file watching
- Maintains file-to-node mapping for incremental updates
- Handles both file and directory changes

### Event Processing
1. File system event detected
2. File content read and parsed
3. Language-specific extractor used
4. Graph updated incrementally
5. WebSocket broadcast sent to clients

## Usage

```rust
use canopy_watcher::WatcherService;
use canopy_core::Graph;
use std::sync::Arc;
use tokio::sync::RwLock;

// Create watcher with shared graph
let graph = Arc::new(RwLock::new(Graph::new()));
let watcher = WatcherService::new(".", graph)?;

// Start watching
watcher.start_watching().await?;

// Process events (runs indefinitely)
watcher.process_events().await?;
```

## Configuration

```toml
[watch]
ignore_patterns = ["target", "node_modules", ".git"]
debounce_ms = 500
```

## Event Types

- **Created** - New files/directories added
- **Modified** - Existing files changed
- **Removed** - Files/directories deleted
- **ChangesFlushed** - Batch of changes completed

## Performance

The watcher is optimized for:
- **Low Latency** - Minimal delay between file change and graph update
- **Efficiency** - Only processes changed files
- **Scalability** - Handles large codebases with many files

## Testing

Run unit tests:
```bash
cargo test -p canopy-watcher
```

## Troubleshooting

Common issues:
- **Permission Errors** - Ensure read access to watched directories
- **High CPU Usage** - Check ignore patterns are properly configured
- **Missed Changes** - Verify file system supports inotify/fsevents