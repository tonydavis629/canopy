//! Graph wrapper using petgraph::StableDiGraph with custom NodeId/EdgeId

use crate::model::*;
use petgraph::stable_graph::{EdgeIndex, NodeIndex, StableDiGraph};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use std::collections::HashSet;

/// The code graph — a directed multigraph with stable node/edge indices.
pub struct Graph {
    inner: StableDiGraph<GraphNode, GraphEdge>,
}

impl std::fmt::Debug for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Graph")
            .field("node_count", &self.inner.node_count())
            .field("edge_count", &self.inner.edge_count())
            .finish()
    }
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            inner: StableDiGraph::new(),
        }
    }

    /// Add a node to graph. Returns assigned NodeId.
    pub fn add_node(&mut self, node: GraphNode) -> NodeId {
        let idx = self.inner.add_node(node);
        NodeId(idx.index() as u64)
    }

    /// Add an edge to graph. Returns assigned EdgeId.
    pub fn add_edge(&mut self, edge: GraphEdge) -> EdgeId {
        let source = NodeIndex::new(edge.source.0 as usize);
        let target = NodeIndex::new(edge.target.0 as usize);
        let idx = self.inner.add_edge(source, target, edge);
        EdgeId(idx.index() as u64)
    }

    /// Get a node by ID.
    pub fn node(&self, id: NodeId) -> Option<&GraphNode> {
        let idx = NodeIndex::new(id.0 as usize);
        self.inner.node_weight(idx)
    }

    /// Get a mutable node by ID.
    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut GraphNode> {
        let idx = NodeIndex::new(id.0 as usize);
        self.inner.node_weight_mut(idx)
    }

    /// Get an edge by ID.
    pub fn edge(&self, id: EdgeId) -> Option<&GraphEdge> {
        let idx = EdgeIndex::new(id.0 as usize);
        self.inner.edge_weight(idx)
    }

    /// Total number of nodes.
    pub fn node_count(&self) -> usize {
        self.inner.node_count()
    }

    /// Total number of edges.
    pub fn edge_count(&self) -> usize {
        self.inner.edge_count()
    }

    /// Iterate over all nodes.
    pub fn all_nodes(&self) -> impl Iterator<Item = &GraphNode> {
        self.inner
            .node_indices()
            .filter_map(move |idx| self.inner.node_weight(idx))
    }

    /// Iterate over all edges.
    pub fn all_edges(&self) -> impl Iterator<Item = &GraphEdge> {
        self.inner
            .edge_indices()
            .filter_map(move |idx| self.inner.edge_weight(idx))
    }

    /// Get all outgoing edges from a node.
    pub fn edges_from(&self, source: NodeId) -> impl Iterator<Item = &GraphEdge> {
        let idx = NodeIndex::new(source.0 as usize);
        self.inner
            .edges_directed(idx, Direction::Outgoing)
            .filter_map(move |edge_ref| self.inner.edge_weight(edge_ref.id()))
    }

    /// Get all incoming edges to a node.
    pub fn edges_to(&self, target: NodeId) -> impl Iterator<Item = &GraphEdge> {
        let idx = NodeIndex::new(target.0 as usize);
        self.inner
            .edges_directed(idx, Direction::Incoming)
            .filter_map(move |edge_ref| self.inner.edge_weight(edge_ref.id()))
    }

    /// Check if an edge exists between two nodes of a specific kind.
    pub fn has_edge_between(&self, source: NodeId, target: NodeId, kind: EdgeKind) -> bool {
        self.edges_from(source)
            .any(|e| e.target == target && e.kind == kind)
    }

    /// Find a node by name (first match).
    pub fn find_node_by_name(&self, name: &str) -> Option<NodeId> {
        self.inner
            .node_indices()
            .find(|&idx| {
                self.inner
                    .node_weight(idx)
                    .map_or(false, |n| n.name == name)
            })
            .map(|idx| NodeId(idx.index() as u64))
    }

    /// Find a node by fully qualified name.
    pub fn find_node_by_qualified(&self, qualified_name: &str) -> Option<NodeId> {
        self.inner
            .node_indices()
            .find(|&idx| {
                self.inner
                    .node_weight(idx)
                    .map_or(false, |n| n.qualified_name == qualified_name)
            })
            .map(|idx| NodeId(idx.index() as u64))
    }

    /// Get all nodes of a specific kind.
    pub fn nodes_of_kind(&self, kind: NodeKind) -> impl Iterator<Item = NodeId> + '_ {
        self.inner
            .node_indices()
            .filter(move |&idx| {
                self.inner
                    .node_weight(idx)
                    .map_or(false, |n| n.kind == kind)
            })
            .map(|idx| NodeId(idx.index() as u64))
    }

    /// Remove a node and all its edges.
    pub fn remove_node(&mut self, id: NodeId) -> Option<GraphNode> {
        let idx = NodeIndex::new(id.0 as usize);
        self.inner.remove_node(idx)
    }

    /// Remove an edge by ID.
    pub fn remove_edge(&mut self, id: EdgeId) -> Option<GraphEdge> {
        let idx = EdgeIndex::new(id.0 as usize);
        self.inner.remove_edge(idx)
    }

    /// Get all nodes that are ancestors of a given node (following Contains edges).
    pub fn ancestors(&self, node: NodeId) -> HashSet<NodeId> {
        let mut ancestors = HashSet::new();
        let mut to_visit = vec![node];

        while let Some(current) = to_visit.pop() {
            let current_idx = NodeIndex::new(current.0 as usize);
            for edge_ref in self.inner.edges_directed(current_idx, Direction::Incoming) {
                if let Some(edge) = self.inner.edge_weight(edge_ref.id()) {
                    if edge.kind == EdgeKind::Contains && !ancestors.contains(&edge.source) {
                        ancestors.insert(edge.source);
                        to_visit.push(edge.source);
                    }
                }
            }
        }

        ancestors
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}
