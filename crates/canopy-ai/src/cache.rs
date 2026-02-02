//! AI analysis cache for avoiding redundant API calls

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};
use canopy_core::GraphNode;
use super::bridge::InferredRelationship;

/// Cache entry with expiration
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub relationships: Vec<InferredRelationship>,
    pub timestamp: Instant,
    pub ttl: Duration,
}

impl CacheEntry {
    pub fn is_expired(&self) -> bool {
        self.timestamp.elapsed() > self.ttl
    }
}

/// Cache for semantic analysis results
pub struct AnalysisCache {
    entries: HashMap<CacheKey, CacheEntry>,
    default_ttl: Duration,
}

/// Key for cache lookups
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct CacheKey {
    source_node_id: u64,
    file_hash: u64,
}

impl AnalysisCache {
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            entries: HashMap::new(),
            default_ttl,
        }
    }
    
    /// Get cached analysis result if available and not expired
    pub fn get(&self, source_node: &GraphNode, file_content_hash: u64) -> Option<&CacheEntry> {
        let key = CacheKey {
            source_node_id: source_node.id.0,
            file_hash: file_content_hash,
        };
        
        self.entries.get(&key).filter(|entry| !entry.is_expired())
    }
    
    /// Store analysis result in cache
    pub fn insert(
        &mut self,
        source_node: &GraphNode,
        file_content_hash: u64,
        relationships: Vec<InferredRelationship>,
    ) {
        let key = CacheKey {
            source_node_id: source_node.id.0,
            file_hash: file_content_hash,
        };
        
        let entry = CacheEntry {
            relationships,
            timestamp: Instant::now(),
            ttl: self.default_ttl,
        };
        
        self.entries.insert(key, entry);
    }
    
    /// Clear expired entries
    pub fn cleanup_expired(&mut self) {
        self.entries.retain(|_, entry| !entry.is_expired());
    }
    
    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            total_entries: self.entries.len(),
            expired_entries: self.entries.values().filter(|e| e.is_expired()).count(),
        }
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
}

/// Compute a simple hash of file content for cache invalidation
pub fn compute_content_hash(content: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}