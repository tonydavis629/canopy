//! Symbol table for cross-file resolution

use crate::model::NodeId;
use dashmap::DashMap;

/// Symbol table mapping qualified names to NodeIds. Thread-safe for concurrent access.
pub struct SymbolTable {
    symbols: DashMap<String, NodeId>,
    /// For fast file lookup: file path -> list of symbol names in that file
    file_symbols: DashMap<String, Vec<String>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            symbols: DashMap::new(),
            file_symbols: DashMap::new(),
        }
    }

    /// Insert a symbol.
    pub fn insert(&self, qualified_name: String, node_id: NodeId, file_path: String) {
        self.symbols.insert(qualified_name.clone(), node_id);
        self.file_symbols
            .entry(file_path)
            .or_insert_with(Vec::new)
            .push(qualified_name);
    }

    /// Look up a symbol by qualified name.
    pub fn lookup(&self, qualified_name: &str) -> Option<NodeId> {
        self.symbols.get(qualified_name).map(|r| *r.value())
    }

    /// Get all symbols defined in a file.
    pub fn symbols_in_file(&self, file_path: &str) -> Vec<NodeId> {
        self.file_symbols
            .get(file_path)
            .map(|r| {
                r.value()
                    .iter()
                    .filter_map(|name| self.symbols.get(name).map(|n| *n.value()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Remove all symbols for a file (useful for incremental re-indexing).
    pub fn remove_file(&self, file_path: &str) {
        if let Some((_, symbols)) = self.file_symbols.remove(file_path) {
            for name in symbols {
                self.symbols.remove(&name);
            }
        }
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
