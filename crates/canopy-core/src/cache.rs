//! Cache for graph serialization

use crate::graph::Graph;
use std::path::{Path, PathBuf};

/// Cache directory: .canopy/
pub const CACHE_DIR: &str = ".canopy";

/// Graph cache file
pub const GRAPH_CACHE: &str = "cache.json";

/// Get cache directory path
pub fn cache_dir(root: &Path) -> PathBuf {
    root.join(CACHE_DIR)
}

/// Get graph cache file path
pub fn graph_cache_path(root: &Path) -> PathBuf {
    root.join(CACHE_DIR).join(GRAPH_CACHE)
}

/// Ensure cache directory exists
pub fn ensure_cache_dir(root: &Path) -> std::io::Result<()> {
    let cache = cache_dir(root);
    if !cache.exists() {
        std::fs::create_dir_all(&cache)?;
    }
    Ok(())
}

/// Serialize graph to cache using JSON (for now).
/// In future, can implement more efficient serialization.
pub fn save_graph(graph: &Graph, root: &Path) -> anyhow::Result<()> {
    ensure_cache_dir(root)?;
    let path = graph_cache_path(root);
    
    // For now, just save a simple marker that cache exists
    // Real implementation would serialize graph data
    let marker = serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "node_count": graph.node_count(),
        "edge_count": graph.edge_count(),
        "cached_at": chrono::Utc::now().to_rfc3339()
    });
    
    let json_str = serde_json::to_string_pretty(&marker)?;
    std::fs::write(&path, json_str)?;
    
    tracing::debug!("Graph cache marker saved: {}", path.display());
    Ok(())
}

/// Load graph from cache
pub fn load_graph(root: &Path) -> anyhow::Result<Option<Graph>> {
    let path = graph_cache_path(root);
    if !path.exists() {
        return Ok(None);
    }

    // For now, just check if cache marker exists
    // Real implementation would deserialize graph data
    let json_str = std::fs::read_to_string(&path)?;
    let _marker: serde_json::Value = serde_json::from_str(&json_str)?;
    
    tracing::debug!("Graph cache marker loaded from: {}", path.display());
    Ok(None) // Return None for now - real graph loading not implemented
}

/// Clear cache directory
pub fn clear_cache(root: &Path) -> std::io::Result<()> {
    let cache = cache_dir(root);
    if cache.exists() {
        std::fs::remove_dir_all(&cache)?;
    }
    Ok(())
}

/// Invalidate cache for a specific file (remove .canopy/*)
pub fn invalidate_file_cache(root: &Path, _file: &Path) -> anyhow::Result<()> {
    // For now, just clear entire cache on any file change
    // In future, can implement more granular invalidation
    clear_cache(root)?;
    Ok(())
}
