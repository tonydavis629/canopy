//! Filesystem watcher implementation

use anyhow::Result;
use canopy_core::{Graph, GraphDiff, NodeId, EdgeId, GraphNode, GraphEdge, EdgeSource};
use canopy_core::diff::DiffEngine;
use canopy_indexer::ExtractionResult;
use canopy_ai::bridge::{AIProvider, SemanticAnalysisRequest, AnalysisContext, SemanticRelationship};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::{HashSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

/// Events emitted by the file watcher
#[derive(Debug, Clone)]
pub enum WatchEvent {
    /// File or directory created
    Created(PathBuf),
    /// File or directory modified
    Modified(PathBuf),
    /// File or directory removed
    Removed(PathBuf),
    /// Batch of changes completed (debounced)
    ChangesFlushed,
}

/// File system watcher for monitoring code changes
pub struct FileWatcher {
    watcher: RecommendedWatcher,
    event_rx: mpsc::UnboundedReceiver<WatchEvent>,
    watched_paths: HashSet<PathBuf>,
    root_path: PathBuf,
}

impl FileWatcher {
    /// Create a new file watcher for the given root path
    pub fn new(root_path: impl AsRef<Path>) -> Result<Self> {
        let root_path = root_path.as_ref().to_path_buf();
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        
        let event_tx_clone = event_tx.clone();
        let watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            match res {
                Ok(event) => {
                    debug!("File system event: {:?}", event);
                    Self::handle_notify_event(event, &event_tx_clone);
                }
                Err(e) => {
                    error!("File system watch error: {}", e);
                }
            }
        })?;

        Ok(Self {
            watcher,
            event_rx,
            watched_paths: HashSet::new(),
            root_path,
        })
    }

    /// Handle a notify event and convert to our watch events
    fn handle_notify_event(event: notify::Event, event_tx: &mpsc::UnboundedSender<WatchEvent>) {
        match event.kind {
            notify::EventKind::Create(_) => {
                for path in event.paths {
                    if should_ignore_path(&path) {
                        continue;
                    }
                    if let Err(e) = event_tx.send(WatchEvent::Created(path)) {
                        warn!("Failed to send create event: {}", e);
                    }
                }
            }
            notify::EventKind::Modify(_) => {
                for path in event.paths {
                    if should_ignore_path(&path) {
                        continue;
                    }
                    if let Err(e) = event_tx.send(WatchEvent::Modified(path)) {
                        warn!("Failed to send modify event: {}", e);
                    }
                }
            }
            notify::EventKind::Remove(_) => {
                for path in event.paths {
                    if should_ignore_path(&path) {
                        continue;
                    }
                    if let Err(e) = event_tx.send(WatchEvent::Removed(path)) {
                        warn!("Failed to send remove event: {}", e);
                    }
                }
            }
            _ => {}
        }
    }

    /// Watch a directory recursively
    pub fn watch_directory(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        info!("Watching directory: {:?}", path);
        
        self.watcher.watch(path, RecursiveMode::Recursive)?;
        self.watched_paths.insert(path.to_path_buf());
        Ok(())
    }

    /// Watch a single file
    pub fn watch_file(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        info!("Watching file: {:?}", path);
        
        self.watcher.watch(path, RecursiveMode::NonRecursive)?;
        self.watched_paths.insert(path.to_path_buf());
        Ok(())
    }

    /// Stop watching a path
    pub fn unwatch(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        info!("Stopping watch for: {:?}", path);
        
        self.watcher.unwatch(path)?;
        self.watched_paths.remove(path);
        Ok(())
    }

    /// Get the event receiver
    pub fn event_receiver(&mut self) -> &mut mpsc::UnboundedReceiver<WatchEvent> {
        &mut self.event_rx
    }

    /// Check if a path is being watched
    pub fn is_watching(&self, path: &Path) -> bool {
        self.watched_paths.contains(path)
    }

    /// Get all watched paths
    pub fn watched_paths(&self) -> &HashSet<PathBuf> {
        &self.watched_paths
    }
}

/// Watcher service that manages file watching and graph updates
pub struct WatcherService {
    watcher: Arc<RwLock<FileWatcher>>,
    graph: Arc<RwLock<Graph>>,
    diff_tx: Option<tokio::sync::broadcast::Sender<String>>,
    diff_engine: Arc<RwLock<DiffEngine>>,
    /// Track which nodes belong to which file for incremental updates
    file_to_nodes: Arc<RwLock<HashMap<PathBuf, Vec<NodeId>>>>,
    file_to_edges: Arc<RwLock<HashMap<PathBuf, Vec<EdgeId>>>>,
    /// AI provider for semantic analysis
    ai_provider: Option<Arc<dyn AIProvider>>,
}

impl WatcherService {
    /// Create a new watcher service
    pub fn new(root_path: impl AsRef<Path>, graph: Arc<RwLock<Graph>>) -> Result<Self> {
        let watcher = Arc::new(RwLock::new(FileWatcher::new(root_path)?));
        let diff_engine = Arc::new(RwLock::new(DiffEngine::new()));
        Ok(Self {
            watcher,
            graph,
            diff_tx: None,
            diff_engine,
            file_to_nodes: Arc::new(RwLock::new(HashMap::new())),
            file_to_edges: Arc::new(RwLock::new(HashMap::new())),
            ai_provider: None,
        })
    }

    /// Create a new watcher service with a broadcast channel for graph diffs
    pub fn with_broadcast(
        root_path: impl AsRef<Path>,
        graph: Arc<RwLock<Graph>>,
        diff_tx: tokio::sync::broadcast::Sender<String>
    ) -> Result<Self> {
        let watcher = Arc::new(RwLock::new(FileWatcher::new(root_path)?));
        let diff_engine = Arc::new(RwLock::new(DiffEngine::new()));
        Ok(Self {
            watcher,
            graph,
            diff_tx: Some(diff_tx),
            diff_engine,
            file_to_nodes: Arc::new(RwLock::new(HashMap::new())),
            file_to_edges: Arc::new(RwLock::new(HashMap::new())),
            ai_provider: None,
        })
    }

    /// Set the AI provider for semantic analysis
    pub fn with_ai_provider(mut self, provider: Arc<dyn AIProvider>) -> Self {
        self.ai_provider = Some(provider);
        self
    }

    /// Start watching the project directory
    pub async fn start_watching(&self) -> Result<()> {
        let mut watcher = self.watcher.write().await;
        let root_path = watcher.root_path.clone();
        
        // Watch the root directory
        watcher.watch_directory(&root_path)?;
        
        info!("Started watching project directory: {:?}", root_path);
        Ok(())
    }

    /// Process file system events and update the graph
    pub async fn process_events(&self) -> Result<()> {
        let mut watcher = self.watcher.write().await;
        let event_rx = watcher.event_receiver();
        
        while let Some(event) = event_rx.recv().await {
            debug!("Processing watch event: {:?}", event);
            
            match event {
                WatchEvent::Created(path) => {
                    info!("File created: {:?}", path);
                    self.handle_file_change(&path).await?;
                }
                WatchEvent::Modified(path) => {
                    info!("File modified: {:?}", path);
                    self.handle_file_change(&path).await?;
                }
                WatchEvent::Removed(path) => {
                    info!("File removed: {:?}", path);
                    self.handle_file_removal(&path).await?;
                }
                WatchEvent::ChangesFlushed => {
                    info!("Batch of changes completed");
                }
            }
        }
        
        Ok(())
    }

    /// Handle a file change event
    async fn handle_file_change(&self, path: &Path) -> Result<()> {
        // Only process code files
        if !is_code_file(path) {
            return Ok(());
        }

        info!("Processing code file change: {:?}", path);

        // Read file content
        let content = match tokio::fs::read_to_string(path).await {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to read file {}: {}", path.display(), e);
                return Ok(());
            }
        };

        // Extract nodes and edges from the file using language-specific extractors
        let extraction_result = match self.extract_from_file(path, &content).await {
            Ok(result) => result,
            Err(e) => {
                error!("Failed to extract symbols from file {}: {}", path.display(), e);
                return Ok(());
            }
        };

        // Get the old nodes and edges for this file before updating
        let old_nodes = {
            let file_to_nodes = self.file_to_nodes.read().await;
            file_to_nodes.get(path).cloned().unwrap_or_default()
        };
        let old_edges = {
            let file_to_edges = self.file_to_edges.read().await;
            file_to_edges.get(path).cloned().unwrap_or_default()
        };

        // Update the graph incrementally
        let graph_diff = self.update_graph_incrementally(path, extraction_result.clone(), old_nodes, old_edges).await?;

        // Perform AI semantic analysis on newly added nodes
        if self.ai_provider.is_some() && !extraction_result.nodes.is_empty() {
            match self.perform_ai_analysis(path, &content, &graph_diff.added_nodes).await {
                Ok(ai_edges) => {
                    if !ai_edges.is_empty() {
                        // Add AI-inferred edges to the graph
                        let mut graph = self.graph.write().await;
                        let mut new_edge_ids = Vec::new();
                        for mut edge in ai_edges {
                            let edge_id = graph.add_edge(edge.clone());
                            edge.id = edge_id;
                            new_edge_ids.push(edge_id);
                        }
                        drop(graph);

                        // Update file_to_edges tracking
                        {
                            let mut file_to_edges = self.file_to_edges.write().await;
                            if let Some(edges) = file_to_edges.get_mut(path) {
                                edges.extend(new_edge_ids.clone());
                            }
                        }

                        info!("Added {} AI-inferred edges for {:?}", new_edge_ids.len(), path);
                    }
                }
                Err(e) => {
                    warn!("AI analysis failed for {:?}: {}", path, e);
                }
            }
        }

        // Broadcast the graph diff to WebSocket clients
        if let Some(ref diff_tx) = self.diff_tx {
            let diff_json = match serde_json::to_string(&graph_diff) {
                Ok(json) => json,
                Err(e) => {
                    error!("Failed to serialize graph diff: {}", e);
                    return Ok(());
                }
            };
            let message = format!(
                r#"{{"type":"graph_diff","diff":{}}}"#,
                diff_json
            );
            // It's okay if there are no receivers - just means no WebSocket clients connected
            let _ = diff_tx.send(message);
        }

        Ok(())
    }

    /// Handle a file removal event
    async fn handle_file_removal(&self, path: &Path) -> Result<()> {
        if !is_code_file(path) {
            return Ok(());
        }

        info!("Processing code file removal: {:?}", path);

        // Get the nodes and edges to remove
        let nodes_to_remove = {
            let file_to_nodes = self.file_to_nodes.read().await;
            file_to_nodes.get(path).cloned().unwrap_or_default()
        };
        let edges_to_remove = {
            let file_to_edges = self.file_to_edges.read().await;
            file_to_edges.get(path).cloned().unwrap_or_default()
        };

        // Remove nodes and edges from the graph
        let mut graph = self.graph.write().await;
        for edge_id in &edges_to_remove {
            graph.remove_edge(*edge_id);
        }
        for node_id in &nodes_to_remove {
            graph.remove_node(*node_id);
        }
        drop(graph);

        // Update tracking maps
        {
            let mut file_to_nodes = self.file_to_nodes.write().await;
            file_to_nodes.remove(path);
        }
        {
            let mut file_to_edges = self.file_to_edges.write().await;
            file_to_edges.remove(path);
        }

        // Create a diff for the removal
        let mut diff = GraphDiff::new(0);
        diff.removed_nodes = nodes_to_remove;
        diff.removed_edges = edges_to_remove;

        // Increment sequence and update
        let mut diff_engine = self.diff_engine.write().await;
        diff_engine.compute_diff(&Graph::new(), &Graph::new()); // Just to increment sequence
        drop(diff_engine);

        // Broadcast the graph diff to WebSocket clients
        if let Some(ref diff_tx) = self.diff_tx {
            let diff_json = match serde_json::to_string(&diff) {
                Ok(json) => json,
                Err(e) => {
                    error!("Failed to serialize graph diff: {}", e);
                    return Ok(());
                }
            };
            let message = format!(
                r#"{{"type":"graph_diff","diff":{}}}"#,
                diff_json
            );
            let _ = diff_tx.send(message);
        }

        Ok(())
    }

    /// Extract nodes and edges from a file using language-specific extractors
    async fn extract_from_file(&self, path: &Path, content: &str) -> Result<ExtractionResult> {
        let path_buf = path.to_path_buf();

        // Get the appropriate extractor based on file extension
        let extractor = canopy_indexer::languages::get_extractor(&path_buf);

        if let Some(extractor) = extractor {
            // Use the extractor to get nodes and edges
            extractor.extract(&path_buf, content.as_bytes())
        } else {
            // No extractor available, return empty result
            Ok(ExtractionResult {
                nodes: Vec::new(),
                edges: Vec::new(),
            })
        }
    }

    /// Update the graph incrementally with new nodes and edges
    async fn update_graph_incrementally(
        &self,
        path: &Path,
        extraction_result: ExtractionResult,
        old_nodes: Vec<NodeId>,
        old_edges: Vec<EdgeId>,
    ) -> Result<GraphDiff> {
        let mut graph = self.graph.write().await;

        // Remove old nodes and edges for this file
        for edge_id in &old_edges {
            graph.remove_edge(*edge_id);
        }
        for node_id in &old_nodes {
            graph.remove_node(*node_id);
        }

        // Add new nodes and collect their IDs
        let mut new_node_ids = Vec::new();
        let mut added_nodes = Vec::new();
        for mut node in extraction_result.nodes {
            let node_id = graph.add_node(node.clone());
            node.id = node_id;
            new_node_ids.push(node_id);
            added_nodes.push(node);
        }

        // Add new edges and collect their IDs
        let mut new_edge_ids = Vec::new();
        let mut added_edges = Vec::new();
        for mut edge in extraction_result.edges {
            // Update edge source/target to point to actual node IDs if needed
            // For now, edges reference nodes by their position in the extraction result
            // This is a simplified approach - in production, you'd need proper node resolution
            let edge_id = graph.add_edge(edge.clone());
            edge.id = edge_id;
            new_edge_ids.push(edge_id);
            added_edges.push(edge);
        }

        drop(graph);

        // Update tracking maps
        {
            let mut file_to_nodes = self.file_to_nodes.write().await;
            file_to_nodes.insert(path.to_path_buf(), new_node_ids);
        }
        {
            let mut file_to_edges = self.file_to_edges.write().await;
            file_to_edges.insert(path.to_path_buf(), new_edge_ids);
        }

        // Create the diff
        let mut diff = GraphDiff::new(0);
        diff.added_nodes = added_nodes;
        diff.removed_nodes = old_nodes;
        diff.added_edges = added_edges;
        diff.removed_edges = old_edges;

        // Update sequence number
        let mut diff_engine = self.diff_engine.write().await;
        diff.sequence = diff_engine.sequence() + 1;
        diff_engine.compute_diff(&Graph::new(), &Graph::new()); // Just to increment sequence
        drop(diff_engine);

        Ok(diff)
    }

    /// Get the current graph diff sequence number
    pub async fn sequence(&self) -> u64 {
        let diff_engine = self.diff_engine.read().await;
        diff_engine.sequence()
    }

    /// Perform AI semantic analysis on newly added nodes
    async fn perform_ai_analysis(
        &self,
        path: &Path,
        _content: &str,
        added_nodes: &[GraphNode],
    ) -> Result<Vec<GraphEdge>> {
        let Some(ai_provider) = &self.ai_provider else {
            return Ok(Vec::new());
        };

        if added_nodes.is_empty() {
            return Ok(Vec::new());
        }

        info!("Performing AI semantic analysis on {} nodes from {:?}", added_nodes.len(), path);

        let mut ai_edges = Vec::new();

        // Get all nodes in the graph as candidates for relationships
        let candidate_nodes = {
            let graph = self.graph.read().await;
            graph.all_nodes().cloned().collect::<Vec<_>>()
        };

        // Analyze each function/method node
        for source_node in added_nodes.iter().filter(|n| {
            matches!(n.kind, canopy_core::NodeKind::Function | canopy_core::NodeKind::Method)
        }) {
            // Build context for the analysis
            let context = AnalysisContext {
                file_path: path.to_path_buf(),
                language: format!("{:?}", source_node.language.unwrap_or(canopy_core::Language::Other)),
                enclosing_context: Vec::new(),
                imports: Vec::new(),
                project_context: HashMap::new(),
            };

            // Create analysis request
            let request = SemanticAnalysisRequest {
                source_node: source_node.clone(),
                candidate_nodes: candidate_nodes.clone(),
                context,
                relationship_types: vec![
                    SemanticRelationship::Calls,
                    SemanticRelationship::DependsOn,
                    SemanticRelationship::Uses,
                ],
            };

            // Call AI provider
            match ai_provider.analyze_semantic_relationships(request).await {
                Ok(result) => {
                    info!("AI analysis found {} relationships for {}", result.relationships.len(), source_node.name);
                    
                    for rel in result.relationships {
                        // Only accept high-confidence relationships
                        if rel.confidence >= 0.7 {
                            ai_edges.push(GraphEdge {
                                id: EdgeId(0), // Will be set by graph
                                source: rel.source_id,
                                target: rel.target_id,
                                kind: rel.relationship.into(),
                                edge_source: EdgeSource::AI,
                                confidence: rel.confidence,
                                label: Some(rel.explanation),
                                file_path: Some(path.to_path_buf()),
                                line: rel.line_reference,
                            });
                        }
                    }
                }
                Err(e) => {
                    warn!("AI analysis failed for {}: {}", source_node.name, e);
                }
            }
        }

        info!("AI analysis complete: {} semantic edges inferred", ai_edges.len());
        Ok(ai_edges)
    }
}

/// Check if a path is a code file we should process
fn is_code_file(path: &Path) -> bool {
    match path.extension().and_then(|s| s.to_str()) {
        Some("rs") | Some("ts") | Some("js") | Some("jsx") | Some("tsx") | Some("py") | Some("go") | Some("java") | Some("cpp") | Some("c") | Some("h") => true,
        _ => false,
    }
}

/// Check if a path should be ignored (e.g., target/, .git/, etc.)
fn should_ignore_path(path: &Path) -> bool {
    // Check if any component of the path is a directory we should ignore
    for component in path.components() {
        if let Some(name) = component.as_os_str().to_str() {
            if name == "target" || name == ".git" || name == "node_modules" || name == ".openclaw" {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_file_watcher_creation() {
        let temp_dir = TempDir::new().unwrap();
        let watcher = FileWatcher::new(temp_dir.path());
        assert!(watcher.is_ok());
    }

    #[tokio::test]
    async fn test_watch_events() {
        let temp_dir = TempDir::new().unwrap();
        let mut watcher = FileWatcher::new(temp_dir.path()).unwrap();
        
        // Create a test file
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").unwrap();
        
        // Watch the file
        watcher.watch_file(&test_file).unwrap();
        
        // Modify the file
        std::fs::write(&test_file, "fn main() { println!(\"Hello\"); }").unwrap();
        
        // Give the watcher time to detect the change
        sleep(Duration::from_millis(100)).await;
        
        // Check if we received the event
        if let Ok(event) = watcher.event_receiver().try_recv() {
            match event {
                WatchEvent::Modified(path) => assert_eq!(path, test_file),
                _ => panic!("Expected modified event"),
            }
        }
    }

    #[test]
    fn test_is_code_file() {
        assert!(is_code_file(Path::new("test.rs")));
        assert!(is_code_file(Path::new("main.ts")));
        assert!(is_code_file(Path::new("app.js")));
        assert!(!is_code_file(Path::new("readme.md")));
        assert!(!is_code_file(Path::new("image.png")));
    }
}