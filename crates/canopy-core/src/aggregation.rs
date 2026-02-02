//! Edge aggregation for collapsed container nodes

use crate::graph::Graph;
use crate::model::{NodeId, EdgeKind, AggregatedEdge};
use std::collections::{HashMap, HashSet};

/// Compute aggregated edges for currently visible/collapsed nodes.
pub fn aggregate_edges(
    graph: &Graph,
    visible_nodes: &HashSet<NodeId>,
    collapsed_nodes: &HashSet<NodeId>,
) -> Vec<AggregatedEdge> {
    let mut agg_map: HashMap<(NodeId, NodeId), AggregatedEdge> = HashMap::new();

    for edge in graph.all_edges() {
        // Skip containment edges — they define hierarchy, not shown as arrows
        if edge.kind == EdgeKind::Contains {
            continue;
        }

        // Find the nearest visible ancestor of source and target
        let visible_source = nearest_visible_ancestor(graph, edge.source, visible_nodes, collapsed_nodes);
        let visible_target = nearest_visible_ancestor(graph, edge.target, visible_nodes, collapsed_nodes);

        // Skip self-loops (both endpoints inside same collapsed container)
        if visible_source == visible_target {
            continue;
        }

        let key = (visible_source, visible_target);
        let agg = agg_map.entry(key).or_insert_with(|| AggregatedEdge {
            source: visible_source,
            target: visible_target,
            count: 0,
            kind_counts: HashMap::new(),
            underlying_edge_ids: Vec::new(),
            min_confidence: None,
        });

        agg.count += 1;
        *agg.kind_counts.entry(edge.kind).or_insert(0) += 1;
        agg.underlying_edge_ids.push(edge.id);

        // Update min confidence for AI edges
        if edge.edge_source == crate::model::EdgeSource::AI {
            let conf = edge.confidence;
            if agg.min_confidence.is_none() || Some(conf) < agg.min_confidence {
                agg.min_confidence = Some(conf);
            }
        }
    }

    agg_map.into_values().collect()
}

/// Find the nearest visible ancestor of a node.
/// If the node itself is visible, return it.
/// Otherwise, walk up the Contains hierarchy until we find a visible node.
fn nearest_visible_ancestor(
    graph: &Graph,
    mut node: NodeId,
    visible_nodes: &HashSet<NodeId>,
    _collapsed_nodes: &HashSet<NodeId>,
) -> NodeId {
    while !visible_nodes.contains(&node) {
        // Find the parent (incoming Contains edge)
        let parent_edge = graph.edges_to(node).find(|e| e.kind == EdgeKind::Contains);
        if let Some(edge) = parent_edge {
            node = edge.source;
        } else {
            break;
        }
    }
    node
}
