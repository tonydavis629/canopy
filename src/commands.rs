//! CLI command implementations

use canopy_core::{Graph, Language};
use canopy_server::{CanopyServer, ServerConfig, ServerState};
use canopy_watcher::WatcherService;
use std::path::PathBuf;
use std::sync::Arc;

pub async fn serve(root: PathBuf, host: String, port: u16, _open: bool) -> anyhow::Result<()> {
    tracing::info!("Starting Canopy server on {}:{}", host, port);
    
    // Build initial graph
    let mut graph = Graph::new();
    walk_filesystem(&root, &mut graph)?;
    
    tracing::info!("Indexed {} nodes, {} edges", graph.node_count(), graph.edge_count());
    
    // Create server with shared graph state
    let config = ServerConfig { host, port };
    let server = CanopyServer::new(graph, config);
    let state = server.state();
    
    // Start file watcher in background task
    let watcher_root = root.clone();
    let watcher_state = Arc::clone(&state);
    tokio::spawn(async move {
        if let Err(e) = run_watcher(watcher_root, watcher_state).await {
            tracing::error!("File watcher error: {}", e);
        }
    });
    
    // Start the server
    server.start().await
}

/// Run the file watcher and broadcast changes to WebSocket clients
async fn run_watcher(root: PathBuf, state: Arc<ServerState>) -> anyhow::Result<()> {
    tracing::info!("Starting file watcher for: {}", root.display());
    
    // Create watcher service with shared graph and broadcast channel
    let graph = Arc::clone(&state.graph);
    let watcher = WatcherService::with_broadcast(&root, graph, state.diff_tx.clone())?;
    
    // Start watching
    watcher.start_watching().await?;
    
    // Process events (this runs indefinitely)
    watcher.process_events().await?;
    
    Ok(())
}

pub async fn index(root: PathBuf) -> anyhow::Result<()> {
    tracing::info!("Indexing repository: {}", root.display());
    
    let mut graph = Graph::new();
    walk_filesystem(&root, &mut graph)?;
    
    tracing::info!("Indexed {} nodes, {} edges", graph.node_count(), graph.edge_count());
    
    Ok(())
}

pub fn clear(root: PathBuf) -> anyhow::Result<()> {
    tracing::info!("Clearing cache for: {}", root.display());
    
    canopy_core::clear_cache(&root)?;
    
    tracing::info!("Cache cleared");
    Ok(())
}

/// Walk filesystem and build basic directory/file structure
fn walk_filesystem(root: &PathBuf, graph: &mut Graph) -> anyhow::Result<()> {
    use std::fs;
    use std::collections::VecDeque;
    
    let mut queue = VecDeque::new();
    
    // Add root directory node
    let root_node = canopy_core::GraphNode {
        id: canopy_core::NodeId(0), // Placeholder, will be assigned by graph
        kind: canopy_core::NodeKind::Directory,
        name: root.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("root")
            .to_string(),
        qualified_name: String::new(),
        file_path: root.clone(),
        line_start: None,
        line_end: None,
        language: None,
        is_container: true,
        child_count: 0,
        loc: None,
        metadata: std::collections::HashMap::new(),
    };
    let root_id = graph.add_node(root_node);
    queue.push_back((root.clone(), root_id));
    
    while let Some((current_path, parent_id)) = queue.pop_front() {
        tracing::debug!("Processing directory: {}", current_path.display());
        
        let entries = match fs::read_dir(&current_path) {
            Ok(entries) => entries,
            Err(e) => {
                tracing::warn!("Cannot read directory {}: {}", current_path.display(), e);
                continue;
            }
        };
        
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    tracing::warn!("Cannot read entry: {}", e);
                    continue;
                }
            };
            
            let path = entry.path();
            let file_name = entry.file_name();
            
            // Skip hidden files and .git
            if file_name.to_string_lossy().starts_with('.') {
                continue;
            }
            
            if path.is_dir() {
                // Add directory node
                let dir_name_str = file_name.to_string_lossy().to_string();
                let dir_node = canopy_core::GraphNode {
                    id: canopy_core::NodeId(0),
                    kind: canopy_core::NodeKind::Directory,
                    name: dir_name_str.clone(),
                    qualified_name: dir_name_str.clone(),
                    file_path: path.clone(),
                    line_start: None,
                    line_end: None,
                    language: None,
                    is_container: true,
                    child_count: 0,
                    loc: None,
                    metadata: std::collections::HashMap::new(),
                };
                let child_id = graph.add_node(dir_node);
                
                // Add containment edge
                let label = format!("contains {}", dir_name_str);
                let edge = canopy_core::GraphEdge {
                    id: canopy_core::EdgeId(0), // Will be assigned by graph
                    source: parent_id,
                    target: child_id,
                    kind: canopy_core::EdgeKind::Contains,
                    edge_source: canopy_core::EdgeSource::Structural,
                    confidence: 1.0,
                    label: Some(label),
                    file_path: None,
                    line: None,
                };
                graph.add_edge(edge);
                
                queue.push_back((path, child_id));
            } else if path.is_file() {
                // Add file node
                let language = Language::from_path(&path);
                let file_name_str = file_name.to_string_lossy().to_string();
                let file_node = canopy_core::GraphNode {
                    id: canopy_core::NodeId(0),
                    kind: canopy_core::NodeKind::File,
                    name: file_name_str.clone(),
                    qualified_name: file_name_str.clone(),
                    file_path: path.clone(),
                    line_start: None,
                    line_end: None,
                    language: Some(language),
                    is_container: true,
                    child_count: 0,
                    loc: None,
                    metadata: std::collections::HashMap::new(),
                };
                let child_id = graph.add_node(file_node);
                
                // Add containment edge
                let label = format!("contains {}", file_name_str);
                let edge = canopy_core::GraphEdge {
                    id: canopy_core::EdgeId(0),
                    source: parent_id,
                    target: child_id,
                    kind: canopy_core::EdgeKind::Contains,
                    edge_source: canopy_core::EdgeSource::Structural,
                    confidence: 1.0,
                    label: Some(label),
                    file_path: None,
                    line: None,
                };
                graph.add_edge(edge);
            }
        }
    }
    
    Ok(())
}
