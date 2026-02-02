//! Core data structures for the code graph

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Unique, stable identifier for a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct NodeId(pub u64);

impl NodeId {
    pub fn new(file_path: &PathBuf, kind: NodeKind, qualified_name: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        file_path.hash(&mut hasher);
        kind.hash(&mut hasher);
        qualified_name.hash(&mut hasher);
        NodeId(hasher.finish())
    }
}

/// Unique edge identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct EdgeId(pub u64);

/// Discriminates what kind of code entity a node represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    // ── Structural ──────────────────────────────────────────
    Directory,
    File,

    // ── Code entities (tree-sitter extracted) ───────────────
    Module,
    Class,
    Struct,
    Enum,
    Interface,
    Function,
    Method,
    Constant,
    TypeAlias,

    // ── Config / data entities ──────────────────────────────
    ConfigBlock,
    ConfigKey,
    EnvVariable,
    Route,
    Migration,
    CIJob,
    DockerService,

    // ── Workspace / monorepo ────────────────────────────────
    WorkspaceRoot,
    Package,

    // ── Fallback ────────────────────────────────────────────
    Unknown,
}

/// A single node in the code graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphNode {
    pub id: NodeId,
    pub kind: NodeKind,
    pub name: String,
    pub qualified_name: String,
    pub file_path: PathBuf,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub language: Option<Language>,
    pub is_container: bool,
    pub child_count: u32,
    pub loc: Option<u32>,
    pub metadata: HashMap<String, String>,
}

/// Supported languages for syntax-aware parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    Java,
    C,
    Cpp,
    Yaml,
    Toml,
    Json,
    Sql,
    Dockerfile,
    Markdown,
    Protobuf,
    GraphQL,
    Other,
}

impl Language {
    /// Detect language from file extension.
    pub fn from_path(path: &PathBuf) -> Self {
        match path.extension().and_then(|e| e.to_str()) {
            Some("rs") => Language::Rust,
            Some("ts") | Some("tsx") => Language::TypeScript,
            Some("js") | Some("jsx") | Some("mjs") | Some("cjs") => Language::JavaScript,
            Some("py") | Some("pyi") => Language::Python,
            Some("go") => Language::Go,
            Some("java") => Language::Java,
            Some("c") | Some("h") => Language::C,
            Some("cpp") | Some("cc") | Some("cxx") | Some("hpp") | Some("hh") => Language::Cpp,
            Some("yml") | Some("yaml") => Language::Yaml,
            Some("toml") => Language::Toml,
            Some("json") | Some("jsonc") => Language::Json,
            Some("sql") => Language::Sql,
            Some("md") | Some("mdx") => Language::Markdown,
            Some("proto") => Language::Protobuf,
            Some("graphql") | Some("gql") => Language::GraphQL,
            _ => {
                if path.file_name().map_or(false, |n| {
                    let s = n.to_string_lossy();
                    s == "Dockerfile" || s.starts_with("Dockerfile.")
                }) {
                    Language::Dockerfile
                } else {
                    Language::Other
                }
            }
        }
    }
}

/// What kind of relationship this edge represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeKind {
    // ── Filesystem containment ──────────────────────────────
    Contains,

    // ── Structural (deterministic, from AST) ────────────────
    Imports,
    Calls,
    Inherits,
    Implements,
    TypeReference,
    Instantiates,
    Exports,

    // ── Semantic (AI-inferred) ──────────────────────────────
    ConfiguresArgument,
    EnvironmentBinding,
    RouteHandler,
    MigrationTarget,
    CITrigger,
    DockerMount,
    SemanticReference,
}

/// How this edge was determined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeSource {
    /// Determined by AST/structural analysis. Always correct.
    Structural,
    /// Determined by pattern-matching heuristics. High confidence.
    Heuristic,
    /// Determined by AI inference. Carries a confidence score.
    AI,
}

/// A directed edge in the code graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphEdge {
    /// Unique edge ID (hash of source + target + kind).
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub kind: EdgeKind,
    pub edge_source: EdgeSource,
    /// 1.0 for Structural, 0.8–1.0 for Heuristic, 0.0–1.0 for AI.
    pub confidence: f32,
    /// Human-readable label.
    pub label: Option<String>,
    /// Where in source this relationship is expressed.
    pub file_path: Option<PathBuf>,
    pub line: Option<u32>,
}

/// A summary edge shown when container nodes are collapsed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AggregatedEdge {
    /// The visible (possibly collapsed) source node.
    pub source: NodeId,
    /// The visible (possibly collapsed) target node.
    pub target: NodeId,
    /// How many underlying edges this represents.
    pub count: u32,
    /// Breakdown by edge kind.
    pub kind_counts: HashMap<EdgeKind, u32>,
    /// The underlying edge IDs (for drill-down).
    pub underlying_edge_ids: Vec<EdgeId>,
    /// Minimum confidence among underlying AI edges (if any).
    pub min_confidence: Option<f32>,
}
