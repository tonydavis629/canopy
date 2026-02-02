//! Graph diff computation for incremental updates

use crate::model::*;
use serde::{Deserialize, Serialize};

/// Represents a change to the graph that should be broadcast to clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDiff {
    /// Monotonically increasing diff sequence number.
    pub sequence: u64,
    /// Nodes added in this update.
    pub added_nodes: Vec<GraphNode>,
    /// Nodes removed in this update.
    pub removed_nodes: Vec<NodeId>,
    /// Edges added in this update.
    pub added_edges: Vec<GraphEdge>,
    /// Edges removed in this update.
    pub removed_edges: Vec<EdgeId>,
    /// Nodes that were modified (metadata changed).
    pub modified_nodes: Vec<NodeId>,
}

impl GraphDiff {
    /// Create an empty diff with given sequence number.
    pub fn new(sequence: u64) -> Self {
        GraphDiff {
            sequence,
            added_nodes: Vec::new(),
            removed_nodes: Vec::new(),
            added_edges: Vec::new(),
            removed_edges: Vec::new(),
            modified_nodes: Vec::new(),
        }
    }

    /// Check if this diff is empty (no changes).
    pub fn is_empty(&self) -> bool {
        self.added_nodes.is_empty()
            && self.removed_nodes.is_empty()
            && self.added_edges.is_empty()
            && self.removed_edges.is_empty()
            && self.modified_nodes.is_empty()
    }
}

/// Diff state for incremental updates.
pub struct DiffEngine {
    sequence: u64,
}

impl DiffEngine {
    pub fn new() -> Self {
        DiffEngine { sequence: 0 }
    }

    /// Compute the difference between two graph states.
    /// Returns a GraphDiff with the sequence number incremented.
    pub fn compute_diff(
        &mut self,
        old_graph: &crate::graph::Graph,
        new_graph: &crate::graph::Graph,
    ) -> GraphDiff {
        let mut diff = GraphDiff::new(self.sequence);

        // Find added nodes
        for node_id in new_graph.nodes_of_kind(NodeKind::Unknown) {
            if old_graph.node(node_id).is_none() {
                if let Some(node) = new_graph.node(node_id) {
                    diff.added_nodes.push(node.clone());
                }
            }
        }

        // Find removed nodes
        for node_id in old_graph.nodes_of_kind(NodeKind::Unknown) {
            if new_graph.node(node_id).is_none() {
                diff.removed_nodes.push(node_id);
            }
        }

        // Find added edges
        for edge in new_graph.all_edges() {
            if old_graph.edge(edge.id).is_none() {
                diff.added_edges.push(edge.clone());
            }
        }

        // Find removed edges
        for edge in old_graph.all_edges() {
            if new_graph.edge(edge.id).is_none() {
                diff.removed_edges.push(edge.id);
            }
        }

        // Increment sequence
        self.sequence += 1;
        diff.sequence = self.sequence;

        diff
    }

    /// Get current sequence number.
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
}

impl Default for DiffEngine {
    fn default() -> Self {
        Self::new()
    }
}
