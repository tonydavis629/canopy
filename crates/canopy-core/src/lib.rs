//! Canopy Core — Graph data model, symbol table, and diff engine

pub mod graph;
pub mod model;
pub mod symbols;
pub mod aggregation;
pub mod diff;
pub mod workspace;
pub mod cache;

#[cfg(test)]
pub mod tests;

#[cfg(test)]
pub mod test_utils;

pub use model::{NodeId, EdgeId, NodeKind, Language, EdgeKind, EdgeSource, GraphNode, GraphEdge, AggregatedEdge};
pub use graph::Graph;
pub use symbols::SymbolTable;
pub use diff::GraphDiff;
pub use aggregation::aggregate_edges;
pub use workspace::{WorkspaceType, detect_workspace};
pub use cache::{CACHE_DIR, GRAPH_CACHE, cache_dir, graph_cache_path, ensure_cache_dir, save_graph, load_graph, clear_cache, invalidate_file_cache};
