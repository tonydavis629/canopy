# Canopy — Technical Specification v1.0

## Live hierarchical code architecture visualization for the AI-agent era

**Version:** 1.0.0
**Date:** February 2026

---

## Table of Contents

1. [Product Summary](#1-product-summary)
2. [Design Principles](#2-design-principles)
3. [Architecture Overview](#3-architecture-overview)
4. [Graph Data Model](#4-graph-data-model)
5. [Indexing Pipeline](#5-indexing-pipeline)
6. [Config-to-Code Linking](#6-config-to-code-linking)
7. [AI Semantic Bridge](#7-ai-semantic-bridge)
8. [File Watcher and Incremental Updates](#8-file-watcher-and-incremental-updates)
9. [Server](#9-server)
10. [Browser Client](#10-browser-client)
11. [CLI Interface](#11-cli-interface)
12. [Project Structure](#12-project-structure)
13. [Rust Dependencies](#13-rust-dependencies)
14. [Concurrency Model](#14-concurrency-model)
15. [Testing Strategy](#15-testing-strategy)
16. [Build and Distribution](#16-build-and-distribution)
17. [Enterprise Tier](#17-enterprise-tier)
18. [Milestone Plan](#18-milestone-plan)

---

## 1. Product Summary

Canopy  is a Rust CLI tool that runs from a project root, watches the filesystem in real time, builds a whole-repository code graph, and serves an interactive hierarchical visualization in the browser. It is designed for the AI-agent era: developers split-screen Canopy  alongside Cursor, Claude Code, or any other coding agent and watch their codebase's architecture evolve live as code is generated.

### Core experience

```
$ cd my-project
$ canopy
  ✓ Indexed 847 files in 1.2s
  ✓ Resolved 2,341 structural edges
  ✓ AI enrichment: 186 semantic edges added (23 cached)
  → http://localhost:7890
  hint: add .canopy/ to your .gitignore
```

A browser tab opens. The developer sees their repo's top-level architecture as a clean hierarchical graph — directories as expandable boxes, aggregate dependency counts as edges between them. They click to expand a directory, and its children fan out with edges splitting to reveal individual connections. When any file changes — whether saved by a human or written by an AI agent — the affected nodes flash with a numbered change indicator, and edges update within milliseconds.

---

## 2. Design Principles

1. **Zero config.** Run `canopy` in any repo and it works. No project files, no manifest, no build step. A single self-contained binary.
2. **Every file is visible.** Not just source code — config files, markdown, dockerfiles, CI manifests, migrations, everything gets a node. Binary files get a placeholder node. Nothing is invisible.
3. **Hierarchical, not flat.** The default view is a collapsible tree-of-boxes reflecting directory structure, not a force-directed hairball. Users expand on demand. The graph shows ~30–50 nodes at a time. Edges aggregate when containers are collapsed and fan out when expanded.
4. **Structural first, AI second.** Deterministic AST-based analysis provides the foundation. AI fills gaps only for connections that static analysis cannot infer. AI-inferred edges are visually distinct and carry confidence scores.
5. **Real time.** Sub-200ms update latency from file save to graph re-render.
6. **Clickable everything.** Every node and edge links back to the exact file and line in the user's editor.
7. **Agent-aware.** Change indicators are bold and sequenced, designed for the split-screen agent-watching workflow.

---

## 3. Architecture Overview

```
┌──────────────────────────────────────────────────────────────┐
│                     canopy (single Rust binary)            │
│                                                               │
│  ┌──────────────┐  ┌────────────────┐  ┌──────────────────┐  │
│  │   Watcher    │→ │    Indexer      │→ │   Graph Engine   │  │
│  │   (notify)   │  │ (tree-sitter   │  │   (petgraph)     │  │
│  │              │  │  + cfg parsers) │  │                  │  │
│  └──────────────┘  └────────────────┘  └────────┬─────────┘  │
│                                                  │            │
│  ┌──────────────┐                    ┌───────────▼──────────┐│
│  │   AI Bridge  │─ ─ semantic ─ ─ → │    Diff Engine       ││
│  │  (optional)  │   edges           │  (incremental deltas) ││
│  └──────────────┘                    └───────────┬──────────┘│
│                                                  │            │
│                                      ┌───────────▼──────────┐│
│                                      │   Server (axum)      ││
│                                      │   HTTP + WebSocket   ││
│                                      └───────────┬──────────┘│
└──────────────────────────────────────────────────┼───────────┘
                                                   │
                                       ┌───────────▼──────────┐
                                       │   Browser Client     │
                                       │   (SVG + vanilla JS) │
                                       │   embedded in binary │
                                       └──────────────────────┘
```

### Component Responsibilities

| Component | Crate / Tech | Responsibility |
|-----------|-------------|----------------|
| **Watcher** | `notify` + `ignore` | Recursive filesystem monitoring, debounced event batching, .gitignore-aware |
| **Indexer** | `tree-sitter` (incremental) + `serde_yaml`/`toml`/`serde_json` | Per-file AST parsing with retained parse trees, symbol extraction, structural edge detection |
| **Graph Engine** | `petgraph::stable_graph` | In-memory directed multigraph, incremental mutation, subgraph queries, edge aggregation |
| **AI Bridge** | `reqwest` + Anthropic/OpenAI/Ollama | Semantic edge inference for non-parseable connections, cached and budget-limited |
| **Diff Engine** | Custom | Computes minimal graph deltas per file change, aggregates edges for collapsed containers |
| **Server** | `axum` + `tokio-tungstenite` | Subgraph-oriented REST API, WebSocket for live diffs, embedded static assets |
| **Browser Client** | ELK.js + D3.js (SVG) + vanilla JS | Hierarchical layout, expand/collapse, edge aggregation animation, code navigation |

---

## 4. Graph Data Model

### 4.1 Node Types

Every file in the repository gets at least one node. Source code files are decomposed into finer-grained symbol nodes.

```rust
/// Discriminates what kind of code entity a node represents.
/// Used for rendering (shape, color) and filtering in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum NodeKind {
    // ── Structural ──────────────────────────────────────────
    Directory,
    File,

    // ── Code entities (tree-sitter extracted) ───────────────
    Module,
    Class,
    Struct,
    Enum,
    Interface,        // includes traits, protocols
    Function,
    Method,
    Constant,
    TypeAlias,

    // ── Config / data entities ──────────────────────────────
    ConfigBlock,      // named section in YAML/TOML/JSON/INI
    ConfigKey,        // specific key-value pair of interest
    EnvVariable,      // from .env files
    Route,            // from routing configs, OpenAPI specs
    Migration,        // from DB migration files
    CIJob,            // from GitHub Actions, GitLab CI, etc.
    DockerService,    // from docker-compose

    // ── Workspace / monorepo ────────────────────────────────
    WorkspaceRoot,    // top-level workspace container
    Package,          // a workspace member / package / service

    // ── Fallback ────────────────────────────────────────────
    Unknown,          // non-parseable files still get a File node
}
```

```rust
/// Unique, stable identifier for a node. Survives incremental updates
/// as long as the underlying entity still exists.
///
/// Format: deterministic hash of (file_path, kind, qualified_name).
/// This means the same function in the same file always gets the same ID,
/// even across full re-indexes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct NodeId(u64);

impl NodeId {
    fn new(file_path: &Path, kind: NodeKind, qualified_name: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        file_path.hash(&mut hasher);
        kind.hash(&mut hasher);
        qualified_name.hash(&mut hasher);
        NodeId(hasher.finish())
    }
}

/// A single node in the code graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GraphNode {
    id: NodeId,
    kind: NodeKind,

    /// Short display name. e.g. "DatabasePool", "connect", "pool_size"
    name: String,

    /// Fully qualified name for resolution. e.g. "src::db::pool::DatabasePool"
    qualified_name: String,

    /// Relative path from repo root.
    file_path: PathBuf,

    /// Line range in the source file. None for directories.
    line_start: Option<u32>,
    line_end: Option<u32>,

    /// Detected programming language (None for directories, binary files).
    language: Option<Language>,

    /// True if this node can contain children (Directory, File, Class, etc.)
    is_container: bool,

    /// Number of direct children. Used for sizing collapsed container nodes.
    child_count: u32,

    /// Lines of code (for files) or lines in body (for functions/classes).
    /// Used for proportional node sizing.
    loc: Option<u32>,

    /// Flexible metadata. Examples:
    ///   "visibility" → "pub", "private"
    ///   "async" → "true"
    ///   "binary" → "true"
    ///   "decorator" → "@app.route('/api/users')"
    metadata: HashMap<String, String>,
}
```

```rust
/// Supported languages for syntax-aware parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum Language {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    Java,
    C,
    Cpp,
    // Config / data formats
    Yaml,
    Toml,
    Json,
    Sql,
    Dockerfile,
    Markdown,
    Protobuf,
    GraphQL,
    // Fallback
    Other,
}

impl Language {
    /// Detect language from file extension.
    fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|e| e.to_str()) {
            Some("rs") => Language::Rust,
            Some("ts" | "tsx") => Language::TypeScript,
            Some("js" | "jsx" | "mjs" | "cjs") => Language::JavaScript,
            Some("py" | "pyi") => Language::Python,
            Some("go") => Language::Go,
            Some("java") => Language::Java,
            Some("c" | "h") => Language::C,
            Some("cpp" | "cc" | "cxx" | "hpp" | "hh") => Language::Cpp,
            Some("yml" | "yaml") => Language::Yaml,
            Some("toml") => Language::Toml,
            Some("json" | "jsonc") => Language::Json,
            Some("sql") => Language::Sql,
            Some("md" | "mdx") => Language::Markdown,
            Some("proto") => Language::Protobuf,
            Some("graphql" | "gql") => Language::GraphQL,
            _ => {
                // Check filename for Dockerfile
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
```

### 4.2 Edge Types

```rust
/// What kind of relationship this edge represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum EdgeKind {
    // ── Filesystem containment ──────────────────────────────
    Contains,           // Directory → File, File → Class, Class → Method

    // ── Structural (deterministic, from AST) ────────────────
    Imports,            // File/module A imports symbol from File/module B
    Calls,              // Function A calls Function B
    Inherits,           // Class A extends Class B
    Implements,         // Class A implements Interface/Trait B
    TypeReference,      // param/return/field type references a type definition
    Instantiates,       // code creates instance of a class/struct
    Exports,            // module explicitly exports a symbol

    // ── Semantic (AI-inferred) ──────────────────────────────
    ConfiguresArgument, // config key → code that reads it
    EnvironmentBinding, // .env variable → code that reads it
    RouteHandler,       // route definition → handler function
    MigrationTarget,    // migration file → model/schema it affects
    CITrigger,          // CI job → files/tests it references
    DockerMount,        // docker-compose volume → code path
    SemanticReference,  // catch-all AI-inferred connection
}

/// How this edge was determined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum EdgeSource {
    /// Determined by AST/structural analysis. Always correct.
    Structural,
    /// Determined by pattern-matching heuristics (e.g. env var matching).
    /// High confidence but not guaranteed.
    Heuristic,
    /// Determined by AI inference. Carries a confidence score.
    AI,
}

/// A directed edge in the code graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GraphEdge {
    /// Unique edge ID (hash of source + target + kind).
    id: EdgeId,
    source: NodeId,
    target: NodeId,
    kind: EdgeKind,
    edge_source: EdgeSource,

    /// 1.0 for Structural, 0.8–1.0 for Heuristic, 0.0–1.0 for AI.
    confidence: f32,

    /// Human-readable label. e.g. "imports DatabasePool", "reads DATABASE_URL".
    label: Option<String>,

    /// Where in the source this relationship is expressed.
    /// For an import edge, this is the file and line of the import statement.
    file_path: Option<PathBuf>,
    line: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct EdgeId(u64);
```

### 4.3 Edge Aggregation

When a container node (directory, package, file) is collapsed in the UI, edges between its children and the outside world must be aggregated into summary edges. This is computed server-side.

```rust
/// A summary edge shown when container nodes are collapsed.
/// Represents N underlying edges between children of the source
/// container and children of the target container.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AggregatedEdge {
    /// The visible (possibly collapsed) source node.
    source: NodeId,
    /// The visible (possibly collapsed) target node.
    target: NodeId,
    /// How many underlying edges this represents.
    count: u32,
    /// Breakdown by edge kind.
    kind_counts: HashMap<EdgeKind, u32>,
    /// The underlying edge IDs (for drill-down).
    underlying_edge_ids: Vec<EdgeId>,
    /// Minimum confidence among underlying AI edges (if any).
    min_confidence: Option<f32>,
}
```

The aggregation algorithm:

```
fn aggregate_edges(
    graph: &Graph,
    visible_nodes: &HashSet<NodeId>,   // currently expanded/visible
    collapsed_nodes: &HashSet<NodeId>, // containers that are collapsed
) -> Vec<AggregatedEdge> {
    let mut agg_map: HashMap<(NodeId, NodeId), AggregatedEdge> = HashMap::new();

    for edge in graph.all_edges() {
        // Skip containment edges — they define the hierarchy, not shown as arrows
        if edge.kind == EdgeKind::Contains { continue; }

        // Find the nearest visible ancestor of source and target
        let visible_source = nearest_visible_ancestor(graph, edge.source, &visible_nodes);
        let visible_target = nearest_visible_ancestor(graph, edge.target, &visible_nodes);

        // Skip self-loops (both endpoints inside the same collapsed container)
        if visible_source == visible_target { continue; }

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
        if edge.edge_source == EdgeSource::AI {
            agg.min_confidence = Some(
                agg.min_confidence.map_or(edge.confidence, |c| c.min(edge.confidence))
            );
        }
    }

    agg_map.into_values().collect()
}

/// Walk up the containment tree until we find a node that is currently
/// visible (either directly visible or is itself a collapsed container).
fn nearest_visible_ancestor(
    graph: &Graph,
    node: NodeId,
    visible_nodes: &HashSet<NodeId>,
) -> NodeId {
    let mut current = node;
    loop {
        if visible_nodes.contains(&current) {
            return current;
        }
        match graph.parent(current) {
            Some(parent) => current = parent,
            None => return current, // root
        }
    }
}
```

### 4.4 Graph Storage

The graph lives in memory using `petgraph::stable_graph::StableDiGraph<GraphNode, GraphEdge>`. Stable indices are required because nodes are added/removed during incremental updates and IDs must remain valid for WebSocket clients holding references.

A parallel `HashMap<NodeId, petgraph::NodeIndex>` maps our deterministic `NodeId` to petgraph's internal index.

The containment hierarchy (Directory → File → Class → Method) is stored as `Contains` edges in the same graph. A separate `HashMap<NodeId, NodeId>` caches child → parent for fast ancestor lookups during aggregation.

**Persistence:**

On shutdown (or periodically every 60s), the graph is serialized to `.canopy/cache.bincode` using `bincode`. On startup:

```
1. If .canopy/cache.bincode exists:
   a. Deserialize the cached graph
   b. Walk the filesystem and compare file modification times against cached timestamps
   c. Re-index only files that changed since the cache was written
   d. Remove nodes for files that no longer exist
2. If no cache exists:
   a. Full index from scratch
3. Write updated cache
```

This makes cold starts after the first run near-instant for repos where few files changed.

### 4.5 Symbol Table

The symbol table is the central data structure for cross-file reference resolution.

```rust
/// Global symbol table built during indexing.
/// Supports concurrent reads during resolution via DashMap.
struct SymbolTable {
    /// Fully qualified name → node IDs.
    /// Multiple IDs possible for overloaded names or re-exports.
    symbols: DashMap<String, Vec<NodeId>>,

    /// File path → list of symbols exported from that file.
    exports: DashMap<PathBuf, Vec<ExportedSymbol>>,

    /// Unresolved references from pass 1, to be resolved in pass 2.
    pending: Mutex<Vec<UnresolvedReference>>,
}

struct ExportedSymbol {
    name: String,
    qualified_name: String,
    node_id: NodeId,
    kind: NodeKind,
}

struct UnresolvedReference {
    /// The node that contains this reference (e.g. a function that calls something).
    from_node: NodeId,
    /// The raw reference text (e.g. "DatabasePool", "super::utils::hash").
    reference_text: String,
    /// What kind of edge this would create if resolved.
    edge_kind: EdgeKind,
    /// File and line where the reference appears.
    file_path: PathBuf,
    line: u32,
    /// Resolution hints from the language extractor.
    hints: ResolutionHints,
}

struct ResolutionHints {
    /// The import context: what modules/packages are in scope.
    in_scope_modules: Vec<String>,
    /// For config references: the pattern that matched (e.g. "process.env.KEY").
    pattern: Option<String>,
    /// For type references: expected kind (Class, Interface, etc.).
    expected_kind: Option<NodeKind>,
}
```

---

## 5. Indexing Pipeline

### 5.1 Overview

Indexing proceeds in phases:

```
Phase 1: Filesystem scan
    Walk repo, build Directory and File nodes, detect workspace structure.

Phase 2: Parallel file parsing (rayon)
    For each file, using the retained tree-sitter parse tree:
      - Extract symbol nodes (classes, functions, etc.)
      - Extract intra-file edges (Contains, method calls within file)
      - Extract references to external symbols (imports, type refs, calls)
      - Populate symbol table with exported symbols

Phase 3: Cross-file resolution
    Resolve pending references against the completed symbol table.
    Create Imports, Calls, TypeReference, Inherits, Implements edges.

Phase 4: Config-to-code linking (heuristic)
    Pattern-match config keys against code patterns.
    Create EnvironmentBinding, ConfiguresArgument edges.

Phase 5: AI semantic bridge (optional, async)
    Batch remaining unresolved references to AI.
    Create SemanticReference, RouteHandler, etc. edges.
```

### 5.2 Workspace / Monorepo Detection

Before parsing individual files, detect workspace structure to define the top-level hierarchy.

```rust
/// Detected workspace configuration.
enum WorkspaceKind {
    /// Not a workspace — single project.
    SingleProject,
    /// Cargo workspace: root Cargo.toml has [workspace] with members.
    CargoWorkspace { members: Vec<PathBuf> },
    /// npm/pnpm/yarn workspace: package.json has "workspaces" field.
    NpmWorkspace { members: Vec<PathBuf> },
    /// Go workspace: go.work file with use directives.
    GoWorkspace { members: Vec<PathBuf> },
    /// Python monorepo: multiple pyproject.toml / setup.py files.
    PythonMonorepo { members: Vec<PathBuf> },
    /// Generic: multiple top-level directories that each look like projects
    /// (contain their own manifest files).
    GenericMonorepo { members: Vec<PathBuf> },
}

fn detect_workspace(root: &Path) -> WorkspaceKind {
    // Check Cargo.toml for [workspace]
    if let Ok(content) = fs::read_to_string(root.join("Cargo.toml")) {
        if let Ok(parsed) = content.parse::<toml::Value>() {
            if let Some(workspace) = parsed.get("workspace") {
                if let Some(members) = workspace.get("members") {
                    // Expand globs in members array
                    return WorkspaceKind::CargoWorkspace {
                        members: expand_workspace_globs(root, members),
                    };
                }
            }
        }
    }

    // Check package.json for "workspaces"
    if let Ok(content) = fs::read_to_string(root.join("package.json")) {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(workspaces) = parsed.get("workspaces") {
                return WorkspaceKind::NpmWorkspace {
                    members: expand_workspace_globs(root, workspaces),
                };
            }
        }
    }

    // Check go.work
    if root.join("go.work").exists() {
        // Parse "use" directives
        return WorkspaceKind::GoWorkspace {
            members: parse_go_work(root),
        };
    }

    // Check pnpm-workspace.yaml
    if let Ok(content) = fs::read_to_string(root.join("pnpm-workspace.yaml")) {
        if let Ok(parsed) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
            if let Some(packages) = parsed.get("packages") {
                return WorkspaceKind::NpmWorkspace {
                    members: expand_workspace_globs(root, packages),
                };
            }
        }
    }

    WorkspaceKind::SingleProject
}
```

When a workspace is detected:
- The root gets a `WorkspaceRoot` node.
- Each member gets a `Package` node (instead of a plain `Directory`).
- The initial collapsed view shows packages as top-level boxes, not raw directories.
- Cross-package edges are visually prominent (thicker, different color).

### 5.3 Language Support Matrix

**Tier 1 — full structural extraction (launch):**

| Language | Grammar | Extracted Symbols | Extracted References |
|----------|---------|-------------------|----------------------|
| Rust | `tree-sitter-rust` | `mod`, `struct`, `enum`, `trait`, `impl` blocks, `fn`, `const`, `type` | `use` imports, function calls, type annotations, trait implementations |
| TypeScript/JS | `tree-sitter-typescript`, `tree-sitter-javascript` | `class`, `interface`, `function`, `const`/`let` declarations, React components, `export` | `import` statements, function calls, `new` expressions, type annotations, JSX component usage |
| Python | `tree-sitter-python` | `class`, `def`, module-level assignments, `__all__` exports | `import`/`from...import`, function calls, decorator references, type hints |
| Go | `tree-sitter-go` | `package`, `struct`, `interface`, `func`, `type`, `const` | `import`, function calls, type assertions, interface satisfaction (structural) |
| Java | `tree-sitter-java` | `class`, `interface`, `enum`, `method`, `field`, `annotation` | `import`, method invocations, `new` expressions, `extends`/`implements`, annotations |
| C/C++ | `tree-sitter-c`, `tree-sitter-cpp` | `struct`, `class`, `function`, `typedef`, `enum`, `#define` | `#include`, function calls, type references |

**Tier 2 — config and data files (launch):**

| File Type | Parser | Extracted Entities |
|-----------|--------|-------------------|
| YAML | `serde_yaml` | Nested key blocks as `ConfigBlock`, leaf key-value pairs as `ConfigKey` |
| TOML | `toml` crate | Sections as `ConfigBlock`, key-value pairs as `ConfigKey` |
| JSON | `serde_json` | Top-level keys as `ConfigBlock`, notable nested keys as `ConfigKey` |
| `.env` | Line parser | Each `KEY=value` as `EnvVariable` |
| Dockerfile | Line parser | `FROM`, `EXPOSE`, `ENV`, `COPY`/`ADD` targets as metadata |
| `docker-compose.yml` | YAML + service detection | Each service as `DockerService`, volumes/ports/depends_on as edges |
| GitHub Actions | YAML + job detection | Each job as `CIJob`, `on` triggers and path filters as metadata |
| SQL migrations | Regex | `CREATE TABLE`, `ALTER TABLE` as `Migration` with table name |
| Markdown | `tree-sitter-markdown` | H1/H2 headings as `ConfigBlock` (for docs linking) |
| Protobuf | `tree-sitter-proto` | `message`, `service`, `rpc` as typed nodes |
| GraphQL | `tree-sitter-graphql` | `type`, `query`, `mutation`, `subscription` as typed nodes |

**Tier 3 — fallback:**

Any file not matching Tier 1 or Tier 2 gets a `File` node with `kind: Unknown`. Binary files (detected by checking for null bytes in the first 8KB) get `metadata: { "binary": "true" }` and a distinct visual style. They still appear in the hierarchy but have no outgoing structural edges.

### 5.4 Per-Language Extractor Interface

Each language implements a common trait:

```rust
/// Trait implemented by each language-specific extractor.
trait LanguageExtractor: Send + Sync {
    /// Which Language this extractor handles.
    fn language(&self) -> Language;

    /// Parse a file and extract symbols and references.
    ///
    /// `old_tree` is the previous tree-sitter parse tree for this file,
    /// if available, for incremental parsing.
    ///
    /// Returns the new parse tree (to be retained for future incremental
    /// parses) and the extraction results.
    fn extract(
        &self,
        file_path: &Path,
        source: &[u8],
        old_tree: Option<&Tree>,
    ) -> Result<(Tree, ExtractionResult)>;
}

struct ExtractionResult {
    /// Symbol nodes found in this file.
    symbols: Vec<ExtractedSymbol>,
    /// References to external symbols (to be resolved in pass 2/3).
    references: Vec<ExtractedReference>,
    /// Intra-file edges (e.g. method calls within the same file).
    local_edges: Vec<(String, String, EdgeKind)>, // (from_qualified, to_qualified, kind)
}

struct ExtractedSymbol {
    name: String,
    qualified_name: String,
    kind: NodeKind,
    line_start: u32,
    line_end: u32,
    is_exported: bool,
    metadata: HashMap<String, String>,
}

struct ExtractedReference {
    /// Qualified name of the symbol containing this reference.
    from_qualified_name: String,
    /// The reference text as it appears in code.
    reference_text: String,
    /// What kind of edge this would create.
    edge_kind: EdgeKind,
    line: u32,
    hints: ResolutionHints,
}
```

### 5.5 Incremental Tree-Sitter Parsing

Each file's most recent `tree_sitter::Tree` is retained in memory:

```rust
/// Cache of parse trees for incremental re-parsing.
struct ParseTreeCache {
    /// file path → (last modified time, parse tree, source content)
    trees: DashMap<PathBuf, (SystemTime, Tree, Vec<u8>)>,
}
```

When a file changes:

```rust
fn reparse_file(
    cache: &ParseTreeCache,
    parser: &mut Parser,
    file_path: &Path,
    new_source: &[u8],
) -> Result<Tree> {
    if let Some(entry) = cache.get(file_path) {
        let (_, ref old_tree, ref old_source) = *entry;

        // Compute the edit that transforms old_source into new_source.
        // Uses a fast diff algorithm (similar to what editors use).
        let edits = compute_tree_sitter_edits(old_source, new_source);

        // Clone the old tree and apply edits to get a modified tree
        // that tree-sitter can use as a baseline for incremental parsing.
        let mut edited_tree = old_tree.clone();
        for edit in &edits {
            edited_tree.edit(edit);
        }

        // Incremental parse: tree-sitter only re-parses the changed regions.
        let new_tree = parser.parse(new_source, Some(&edited_tree))?;
        Ok(new_tree)
    } else {
        // No cached tree — full parse.
        let new_tree = parser.parse(new_source, None)?;
        Ok(new_tree)
    }
}
```

Memory cost: a tree-sitter `Tree` for a typical source file is 50–200KB. For a 10,000-file repo, that's ~500MB–2GB of retained trees. This is acceptable for development machines. A CLI flag `--low-memory` can disable tree retention and fall back to full re-parses.

### 5.6 Cross-File Resolution (Pass 2)

After all files are parsed and the symbol table is populated:

```rust
fn resolve_pending_references(
    symbol_table: &SymbolTable,
    graph: &mut Graph,
) -> Vec<UnresolvedReference> {
    let mut still_unresolved = Vec::new();

    for reference in symbol_table.drain_pending() {
        let candidates = find_candidates(symbol_table, &reference);

        match candidates.len() {
            0 => still_unresolved.push(reference),
            1 => {
                graph.add_edge(GraphEdge {
                    source: reference.from_node,
                    target: candidates[0],
                    kind: reference.edge_kind,
                    edge_source: EdgeSource::Structural,
                    confidence: 1.0,
                    label: Some(format!("{}", reference.reference_text)),
                    file_path: Some(reference.file_path),
                    line: Some(reference.line),
                    ..default()
                });
            }
            _ => {
                // Multiple candidates: use heuristics to pick the best one.
                // Heuristics: same directory > same package > closest in tree.
                if let Some(best) = rank_candidates(&reference, &candidates, graph) {
                    graph.add_edge(/* ... */);
                } else {
                    still_unresolved.push(reference);
                }
            }
        }
    }

    still_unresolved
}

fn find_candidates(
    symbol_table: &SymbolTable,
    reference: &UnresolvedReference,
) -> Vec<NodeId> {
    let ref_text = &reference.reference_text;

    // Strategy 1: exact qualified name match
    if let Some(ids) = symbol_table.symbols.get(ref_text) {
        return ids.clone();
    }

    // Strategy 2: match against exported symbols from imported modules
    for module in &reference.hints.in_scope_modules {
        let qualified = format!("{}::{}", module, ref_text);
        if let Some(ids) = symbol_table.symbols.get(&qualified) {
            return ids.clone();
        }
    }

    // Strategy 3: fuzzy match — last segment of qualified name
    let mut matches = Vec::new();
    for entry in symbol_table.symbols.iter() {
        let name = entry.key();
        if name.ends_with(&format!("::{}", ref_text))
            || name.ends_with(&format!(".{}", ref_text))
            || name.ends_with(&format!("/{}", ref_text))
        {
            matches.extend(entry.value().iter().copied());
        }
    }

    // Filter by expected kind if specified
    if let Some(expected) = reference.hints.expected_kind {
        matches.retain(|id| {
            // look up node kind in graph
            true // simplified
        });
    }

    matches
}
```

---

## 6. Config-to-Code Linking

Config-to-code linking runs after cross-file resolution (Phase 4). It uses deterministic pattern matching — no AI needed for these common patterns.

### 6.1 Environment Variables

Scan all code files for environment variable access patterns. Match against `.env` file entries.

```rust
/// Patterns for environment variable access, by language.
const ENV_PATTERNS: &[(&str, &[&str])] = &[
    // Rust
    ("rs", &[
        r#"std::env::var\("([^"]+)"\)"#,
        r#"env::var\("([^"]+)"\)"#,
        r#"env!?\("([^"]+)"\)"#,
        r#"dotenvy::var\("([^"]+)"\)"#,
    ]),
    // JavaScript/TypeScript
    ("js,ts,jsx,tsx,mjs,cjs", &[
        r#"process\.env\.(\w+)"#,
        r#"process\.env\["([^"]+)"\]"#,
        r#"process\.env\['([^']+)'\]"#,
        r#"import\.meta\.env\.(\w+)"#,  // Vite
    ]),
    // Python
    ("py", &[
        r#"os\.environ\["([^"]+)"\]"#,
        r#"os\.environ\.get\("([^"]+)""#,
        r#"os\.getenv\("([^"]+)""#,
    ]),
    // Go
    ("go", &[
        r#"os\.Getenv\("([^"]+)"\)"#,
        r#"os\.LookupEnv\("([^"]+)"\)"#,
    ]),
    // Java
    ("java", &[
        r#"System\.getenv\("([^"]+)"\)"#,
        r#"System\.getProperty\("([^"]+)"\)"#,
    ]),
];
```

For each match, create an `EnvironmentBinding` edge from the `.env` `EnvVariable` node to the function/method node containing the access.

### 6.2 Config File Key Access

Match common config library access patterns against config file keys.

**Rust (config crate / serde):**
- Match `config.get::<T>("key.path")` against TOML/YAML key paths.
- Match `#[serde(rename = "key")]` struct fields against config keys.

**Python (pydantic, dynaconf, configparser):**
- Match `settings.KEY_NAME` against config keys.
- Match `config["section"]["key"]` against INI/TOML sections.

**JavaScript/TypeScript:**
- Match `config.get("key.path")` against JSON/YAML config keys.
- Match destructured imports from config files.

**Go (viper):**
- Match `viper.GetString("key.path")` against config keys.

### 6.3 Route-to-Handler Matching

For web frameworks with explicit route registration:

```
// Express/Fastify: app.get("/path", handler) → match handler to function node
// Flask: @app.route("/path") → match decorated function
// Actix/Axum: .route("/path", get(handler)) → match handler to function node
// Go net/http: http.HandleFunc("/path", handler) → match handler
// Spring: @GetMapping("/path") on method → match method
```

These are extracted during Phase 2 (the language extractor captures route decorators/registrations) and resolved structurally. Route strings become `Route` nodes, connected to their handler functions with `RouteHandler` edges.

### 6.4 Docker and CI

**docker-compose.yml:**
- `volumes: ["./src:/app"]` → `DockerMount` edge from service to `./src` directory node.
- `depends_on: [db]` → edge between `DockerService` nodes.
- `build: { context: ./services/api }` → `Contains`-like edge from service to that directory.

**GitHub Actions:**
- `on: push: paths: ["src/**"]` → `CITrigger` edges from the job to matching directory nodes.
- `uses: actions/checkout@v4` → metadata on the job node.
- `run: cargo test -p my-crate` → `CITrigger` edge to the package node if detectable.

---

## 7. AI Semantic Bridge

### 7.1 Purpose

After Phases 1–4, some references remain unresolved — typically config-to-code bindings that don't follow standard library conventions, framework-specific magic, indirect references through dependency injection, or cross-cutting concerns that can't be detected by pattern matching.

The AI bridge is **optional** and **conservative**: it only processes unresolved items, every result carries a confidence score, and AI-inferred edges are visually distinct.

### 7.2 Configuration

```rust
struct AIBridgeConfig {
    enabled: bool,                    // default: true
    provider: AIProvider,             // default: Anthropic
    model: String,                    // default: "claude-sonnet-4-20250514"
    max_batch_size: usize,            // default: 50 items per API call
    confidence_threshold: f32,        // default: 0.7 — edges below this are discarded
    daily_budget: u32,                // default: 500 API calls per day
    cache_ttl: Duration,              // default: 24 hours
}

enum AIProvider {
    Anthropic { api_key: String },
    OpenAI { api_key: String, endpoint: Option<String> },
    Ollama { endpoint: String, model: String },
}
```

API keys are read from (in order of precedence):
1. `.canopy.toml` config file
2. Environment variables: `CODEGRAPH_AI_KEY`, `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`
3. If no key found and AI is enabled, print a hint and continue without AI.

### 7.3 Batching and Prompt Construction

Unresolved references are batched by proximity (same directory first) to maximize context reuse.

```rust
struct AIBatch {
    /// The unresolved items in this batch.
    items: Vec<UnresolvedReference>,
    /// Summarized repo structure for context.
    repo_summary: String,
    /// Nearby code snippets for each item (±5 lines around the reference).
    context_snippets: Vec<String>,
    /// Candidate target symbols (from the symbol table, filtered by plausibility).
    candidate_targets: Vec<Vec<(String, NodeId)>>,
}
```

**System prompt:**

```
You are a code architecture analyzer. Given unresolved references from a
codebase, determine which source code symbols they connect to.

Rules:
- Only suggest connections you are confident about.
- Rate your confidence from 0.0 to 1.0.
- If you are not sure, set confidence below 0.5 — these will be discarded.
- Respond ONLY with the JSON object specified. No explanation.
```

**User prompt (per batch):**

```
Repository structure:
{repo_tree, truncated to 3 levels}

Unresolved items:

[1] Config key "database.pool_size" at config/database.yml:14
    Context: (5 lines of surrounding YAML)
    Candidate targets:
      - DatabasePool::new at src/db/pool.rs:23
      - Settings::database at src/config.rs:45
      - DB_POOL_SIZE constant at src/constants.rs:12

[2] Route "/api/users/:id" at routes.yaml:8
    Context: (5 lines of surrounding YAML)
    Candidate targets:
      - UserController::get_by_id at src/controllers/user.rs:67
      - user_router at src/routes/user.rs:3

Respond with:
{
  "resolutions": [
    {
      "item": <number>,
      "target": "<qualified_name>",
      "target_file": "<file_path>",
      "confidence": <0.0-1.0>,
      "edge_kind": "<ConfiguresArgument|RouteHandler|SemanticReference|...>",
      "reasoning": "<one sentence>"
    }
  ]
}
```

### 7.4 Response Processing

```rust
async fn process_ai_response(
    response: &AIResponse,
    graph: &mut Graph,
    symbol_table: &SymbolTable,
    config: &AIBridgeConfig,
) {
    for resolution in &response.resolutions {
        // Validate: target must exist in symbol table
        let target_id = match symbol_table.lookup(&resolution.target) {
            Some(id) => id,
            None => {
                tracing::warn!(
                    "AI suggested target {} which doesn't exist in symbol table, skipping",
                    resolution.target
                );
                continue;
            }
        };

        // Apply confidence threshold
        if resolution.confidence < config.confidence_threshold {
            tracing::debug!(
                "AI edge below threshold ({:.2} < {:.2}), discarding",
                resolution.confidence, config.confidence_threshold
            );
            continue;
        }

        // Create the edge
        graph.add_edge(GraphEdge {
            source: resolution.item.from_node,
            target: target_id,
            kind: resolution.edge_kind,
            edge_source: EdgeSource::AI,
            confidence: resolution.confidence,
            label: Some(resolution.reasoning.clone()),
            file_path: Some(resolution.item.file_path.clone()),
            line: Some(resolution.item.line),
            ..default()
        });
    }
}
```

### 7.5 Caching

AI results are cached in `.canopy/ai_cache.json`:

```json
{
  "version": 1,
  "entries": {
    "<hash(source_content + reference_text + candidate_names)>": {
      "created_at": "2026-02-01T12:00:00Z",
      "target": "src::db::pool::DatabasePool::new",
      "confidence": 0.92,
      "edge_kind": "ConfiguresArgument",
      "reasoning": "pool_size config read by DatabasePool constructor"
    }
  }
}
```

On file change, cache entries whose source content hash changed are invalidated.

### 7.6 Budget Tracking

Usage is logged to `.canopy/usage.log`:

```
2026-02-01T12:00:00Z batch_size=12 tokens_in=3400 tokens_out=850 cost_usd=0.012
2026-02-01T12:05:00Z batch_size=8 tokens_in=2100 tokens_out=600 cost_usd=0.008
```

When daily budget is exhausted, AI bridge logs a warning and stops until midnight UTC. Structural analysis continues unaffected.

---

## 8. File Watcher and Incremental Updates

### 8.1 Watcher Configuration

```rust
struct WatcherConfig {
    /// Debounce window: batch rapid changes into a single update.
    debounce: Duration,                  // default: 150ms

    /// Always ignored (hardcoded).
    builtin_ignores: Vec<&'static str>,  // .git, node_modules, target, __pycache__,
                                         // .next, dist, build, .canopy, .DS_Store,
                                         // *.pyc, *.o, *.so, *.dylib

    /// Additional user-configured ignores.
    user_ignores: Vec<String>,

    /// Respect .gitignore files. Default: true.
    gitignore_aware: bool,
}
```

Built on the `notify` crate (v6) with the `ignore` crate (from ripgrep) for .gitignore-aware file walking.

### 8.2 Event Processing Pipeline

```
FileSystem event(s)
    │
    ▼
Debounce buffer (150ms window)
    │
    ▼
Deduplicate: collapse multiple events for the same file
    │
    ▼
Classify events:
    - Created(path)  → index new file
    - Modified(path) → re-index file
    - Deleted(path)  → remove file's nodes and edges
    - Renamed(old, new) → remove old, index new
    │
    ▼
Batch reindex:
    For each affected file:
      1. Remove all nodes originating from this file (except the File node itself for modify)
      2. Remove all edges originating from these nodes
      3. Re-parse with incremental tree-sitter
      4. Re-extract symbols and references
      5. Re-resolve references against symbol table
      6. Re-run config-to-code heuristics for this file
      7. If AI bridge enabled, queue new unresolved refs
    │
    ▼
Compute GraphDiff
    │
    ▼
Broadcast diff to all WebSocket clients
    │
    ▼
Update parse tree cache
```

### 8.3 GraphDiff

```rust
/// Minimal diff describing what changed in the graph.
/// Sent over WebSocket to all connected clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GraphDiff {
    /// Monotonically increasing sequence number.
    sequence: u64,

    /// Timestamp of this update.
    timestamp: f64,

    /// Nodes added to the graph.
    added_nodes: Vec<GraphNode>,

    /// Node IDs removed from the graph.
    removed_nodes: Vec<NodeId>,

    /// Nodes whose properties changed (e.g. LOC count, metadata).
    /// Only the changed fields are populated.
    modified_nodes: Vec<NodePatch>,

    /// Edges added.
    added_edges: Vec<GraphEdge>,

    /// Edge IDs removed.
    removed_edges: Vec<EdgeId>,

    /// Which files triggered this update (for change indicators).
    changed_files: Vec<ChangedFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NodePatch {
    id: NodeId,
    name: Option<String>,
    loc: Option<u32>,
    child_count: Option<u32>,
    metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChangedFile {
    path: PathBuf,
    /// Sequential change number since server start.
    /// Used for the numbered change indicators in the UI.
    change_number: u32,
    /// What kind of change.
    kind: FileChangeKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum FileChangeKind {
    Created,
    Modified,
    Deleted,
}
```

---

## 9. Server

Built with `axum` on `tokio`. Static assets embedded with `rust-embed`.

### 9.1 HTTP API

All responses are JSON. CORS is enabled for `localhost` origins.

```
GET /
    Serves index.html (embedded static asset).

GET /assets/{path}
    Serves embedded JS/CSS/font assets.

GET /api/graph/stats
    Returns high-level graph statistics.
    Response: {
        "total_nodes": 2847,
        "total_edges": 5231,
        "structural_edges": 5045,
        "ai_edges": 186,
        "files": 847,
        "languages": { "rust": 312, "typescript": 245, ... },
        "workspace_kind": "CargoWorkspace",
        "last_indexed": "2026-02-01T12:00:00Z"
    }

GET /api/node/{id}/children
    Returns the direct children of a container node and the aggregated
    edges between them (and from them to outside the container).
    Query params:
      - include_external_edges=true (default true)
    Response: {
        "parent": GraphNode,
        "children": [GraphNode, ...],
        "internal_edges": [AggregatedEdge, ...],  // edges between children
        "external_edges": [AggregatedEdge, ...],  // edges to/from outside
    }

GET /api/roots
    Returns the top-level nodes (workspace packages or top-level directories)
    and aggregated edges between them. This is what the client loads on
    initial page load.
    Response: {
        "workspace_kind": "CargoWorkspace",
        "roots": [GraphNode, ...],
        "edges": [AggregatedEdge, ...],
    }

GET /api/node/{id}
    Returns full details for a single node including all direct edges.
    Response: {
        "node": GraphNode,
        "edges_out": [GraphEdge, ...],
        "edges_in": [GraphEdge, ...],
        "parent": Option<NodeId>,
        "children_count": u32,
    }

GET /api/search?q={query}&limit={n}
    Fuzzy search across all node names.
    Default limit: 20.
    Response: {
        "results": [
            { "node": GraphNode, "score": 0.95, "ancestors": [NodeId, ...] },
            ...
        ]
    }

GET /api/file/{path...}
    Returns syntax-highlighted file contents.
    Query params:
      - line_start (optional)
      - line_end (optional)
    Response: {
        "path": "src/db/pool.rs",
        "language": "rust",
        "total_lines": 145,
        "content_html": "<pre>...</pre>",
        "content_plain": "...",
    }

GET /api/paths?from={id}&to={id}&max_depth={n}
    Finds all paths between two nodes up to max_depth hops.
    Default max_depth: 6.
    Response: {
        "paths": [
            { "nodes": [NodeId, ...], "edges": [EdgeId, ...] },
            ...
        ]
    }

GET /api/edge/{id}
    Returns full details for a single edge.
    Response: GraphEdge

GET /api/export?format={json|dot}
    Exports the full graph. For tooling integration, not for the UI.
    format=json: full graph as JSON.
    format=dot: Graphviz DOT format.

WS /ws
    WebSocket endpoint for live graph diffs.
    After connection, server sends current sequence number.
    On each graph update, server sends a GraphDiff message.
    Client can send: { "type": "ping" } to keep alive.
```

### 9.2 Static Asset Embedding

```rust
#[derive(RustEmbed)]
#[folder = "client/"]
struct ClientAssets;

// In axum router:
async fn serve_asset(Path(path): Path<String>) -> impl IntoResponse {
    match ClientAssets::get(&path) {
        Some(file) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            (
                [(header::CONTENT_TYPE, mime.as_ref())],
                file.data.into_owned(),
            ).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
```

### 9.3 WebSocket Broadcasting

```rust
/// Shared state between the indexer and WebSocket connections.
struct AppState {
    graph: Arc<RwLock<Graph>>,
    symbol_table: Arc<SymbolTable>,
    parse_cache: Arc<ParseTreeCache>,
    /// Broadcast channel for graph diffs.
    diff_tx: broadcast::Sender<GraphDiff>,
    /// Current sequence number.
    sequence: AtomicU64,
    /// Global change counter for file change numbering.
    change_counter: AtomicU32,
}
```

Each WebSocket client subscribes to `diff_tx`. When the indexer produces a `GraphDiff`, it sends on the broadcast channel. The WebSocket handler serializes and sends to the client.

### 9.4 Port Selection

```rust
async fn find_available_port(preferred: u16) -> u16 {
    for port in preferred..preferred + 100 {
        if TcpListener::bind(("127.0.0.1", port)).await.is_ok() {
            return port;
        }
    }
    panic!("No available port found in range {}–{}", preferred, preferred + 99);
}
```

Default: 7890. If busy, tries 7891, 7892, etc. The chosen port is printed to stdout.

---

## 10. Browser Client

The entire browser client is 4 files embedded in the binary. No build step, no npm, no framework.

### 10.1 Files

```
client/
├── index.html      # shell HTML, loads the other files
├── graph.js        # hierarchical layout, SVG rendering, expand/collapse, edge aggregation
├── ui.js           # search, filters, detail panel, history, keyboard shortcuts
├── protocol.js     # WebSocket client, diff application, local graph state
└── style.css       # all styles, including node/edge visual language
```

### 10.2 External Dependencies (loaded from CDN with local fallback)

```html
<!-- ELK.js for hierarchical layout computation -->
<script src="https://cdn.jsdelivr.net/npm/elkjs@0.9.3/lib/elk.bundled.js"></script>

<!-- D3.js for SVG rendering and transitions -->
<script src="https://cdn.jsdelivr.net/npm/d3@7/dist/d3.min.js"></script>

<!-- highlight.js for code panel syntax highlighting -->
<script src="https://cdn.jsdelivr.net/npm/highlight.js@11/highlight.min.js"></script>
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/highlight.js@11/styles/github-dark.min.css">
```

Each library is also bundled into the binary as a fallback for offline/air-gapped environments. On load, the client tries the CDN first with a 2-second timeout, then falls back to `/assets/vendor/`.

### 10.3 Layout Engine

**ELK.js** (Eclipse Layout Kernel compiled to JS) computes the hierarchical layout. It handles:
- Nested boxes (directories containing files containing classes)
- Edge routing between nested levels
- Automatic spacing and sizing
- Port-based edge attachment (edges connect at specific sides of nodes)

Layout is recomputed when nodes are expanded/collapsed. ELK is fast enough (<50ms for 50 nodes) to run synchronously on user interaction.

```javascript
const elk = new ELK();

async function computeLayout(visibleNodes, visibleEdges) {
    const elkGraph = {
        id: "root",
        layoutOptions: {
            "elk.algorithm": "layered",
            "elk.direction": "DOWN",
            "elk.spacing.nodeNode": "30",
            "elk.spacing.edgeNode": "20",
            "elk.layered.spacing.baseValue": "40",
            "elk.hierarchyHandling": "INCLUDE_CHILDREN",
            "elk.layered.mergeEdges": true,
        },
        children: visibleNodes.map(nodeToElk),
        edges: visibleEdges.map(edgeToElk),
    };

    return await elk.layout(elkGraph);
}

function nodeToElk(node) {
    const elkNode = {
        id: node.id,
        width: computeNodeWidth(node),
        height: computeNodeHeight(node),
        labels: [{ text: node.name }],
    };

    // If this node is expanded and has visible children,
    // include them as nested children for ELK to lay out.
    if (isExpanded(node.id) && node.children) {
        elkNode.children = node.children.map(nodeToElk);
        elkNode.edges = getInternalEdges(node.id).map(edgeToElk);
    }

    return elkNode;
}
```

### 10.4 Screen Layout

```
┌──────────────────────────────────────────────────────────────────┐
│ [🔍 Search...              ] [Layers ▾] [AI edges ○] [⚙ Settings]│
├──────────────────────────────────────────┬───────────────────────┤
│                                          │                       │
│                                          │ DETAIL PANEL           │
│      GRAPH VIEWPORT                      │                       │
│      (SVG, pannable, zoomable)           │ ┌─ src/db/pool.rs ─┐ │
│                                          │ │ ln 23-45           │ │
│    ┌─────────────────┐                   │ │                   │ │
│    │   src/          │ ←── 12 imports ── │ │ pub fn connect()  │ │
│    │  ┌───────┐      │                   │ │   -> Pool {       │ │
│    │  │db/    │──┐   │    ┌──────────┐   │ │   let size =      │ │
│    │  │ pool ◉│  │   │    │ config/  │   │ │     config.get(   │ │
│    │  │ conn  │  │   │    │ db.yml   │   │ │       "pool_size" │ │
│    │  └───────┘  │   │    └──────────┘   │ │     );            │ │
│    │  ┌───────┐  │   │                   │ │ }                 │ │
│    │  │api/   │◄─┘   │                   │ └───────────────────┘ │
│    │  │       │      │                   │                       │
│    │  └───────┘      │                   │ Edges:                │
│    └─────────────────┘                   │  → calls Pool::new    │
│                                          │  ← called by main()   │
│                                          │  ~ reads pool_size    │
│                                          │    (AI, 0.91) ⓘ      │
│                                          │                       │
│                                          │ [Open in editor ↗]    │
├──────────────────────────────────────────┴───────────────────────┤
│ ①②③④ ← change trail │ 847 files │ 2,527 edges │ ● connected    │
└──────────────────────────────────────────────────────────────────┘
```

**Left (graph viewport):** SVG rendering of the hierarchical graph. Pan with drag, zoom with scroll wheel. Nodes are clickable and expandable.

**Right (detail panel):** Opens when a node or edge is clicked. Shows syntax-highlighted code snippet, list of all connected edges, and an "Open in editor" button. Collapsed by default on narrow viewports.

**Top bar:** Search input, layer selector, AI edge toggle, settings button.

**Bottom bar:** Change trail showing numbered recently-changed files. Click a number to navigate to that file's node. Stats. Connection status indicator.

### 10.5 Visual Language

#### Node Shapes and Colors

| Node Kind | Shape | Color | Border | Sizing |
|-----------|-------|-------|--------|--------|
| WorkspaceRoot | large rounded rect | `gray-100` | `gray-300` 2px solid | fixed |
| Package | large rounded rect | `gray-50` | `blue-400` 2px solid | proportional to child count |
| Directory | rounded rect | `gray-50` | `gray-300` 1px solid | proportional to child count |
| File (source) | rect | language color, 15% opacity fill | language color, 1px solid | proportional to LOC |
| File (config) | rect | `amber-50` | `amber-400` 1px dashed | fixed small |
| File (binary) | rect | `gray-100` | `gray-300` 1px dotted | fixed small |
| File (unknown) | rect | `gray-50` | `gray-300` 1px dotted | fixed small |
| Class / Struct | rounded rect | language color, 25% opacity | language color, 2px solid | proportional to method count |
| Function / Method | small rounded rect | language color, 10% opacity | language color, 1px solid | fixed |
| Interface / Trait | rounded rect | language color, 15% opacity | language color, 2px dashed | proportional to implementor count |
| Enum | rounded rect | language color, 20% opacity | language color, 1px solid | fixed |
| Constant | small rect | language color, 5% opacity | language color, 1px solid | fixed |
| ConfigBlock | diamond | `amber-100` | `amber-500` 1px solid | fixed |
| ConfigKey | small diamond | `amber-50` | `amber-400` 1px solid | fixed |
| EnvVariable | small diamond | `green-50` | `green-400` 1px solid | fixed |
| Route | pill | `emerald-100` | `emerald-500` 1px solid | fixed |
| Migration | cylinder | `purple-100` | `purple-400` 1px solid | fixed |
| CIJob | hexagon | `sky-100` | `sky-400` 1px solid | fixed |
| DockerService | hexagon | `cyan-100` | `cyan-400` 1px solid | fixed |

#### Language Colors

```javascript
const LANGUAGE_COLORS = {
    rust:       { primary: "#DEA584", text: "#7C3F00" },
    typescript: { primary: "#3178C6", text: "#FFFFFF" },
    javascript: { primary: "#F7DF1E", text: "#000000" },
    python:     { primary: "#3776AB", text: "#FFFFFF" },
    go:         { primary: "#00ADD8", text: "#FFFFFF" },
    java:       { primary: "#B07219", text: "#FFFFFF" },
    c:          { primary: "#555555", text: "#FFFFFF" },
    cpp:        { primary: "#F34B7D", text: "#FFFFFF" },
    config:     { primary: "#F59E0B", text: "#000000" },
    other:      { primary: "#9CA3AF", text: "#000000" },
};
```

#### Edge Styles

| Edge Source | Line Style | Opacity | Arrow |
|-------------|-----------|---------|-------|
| Structural | solid, 1.5px | 0.7 | filled triangle |
| Heuristic | solid, 1px | 0.5 | filled triangle |
| AI (confidence ≥ 0.8) | dashed (5,5), 1px | 0.5 | open triangle |
| AI (confidence < 0.8) | dotted (2,4), 1px | 0.3 | open triangle |
| Aggregated (count > 1) | solid, thickness = log2(count) + 1 | 0.6 | filled triangle, label showing count |

Edge colors follow the predominant kind in the aggregation:
- `Imports` → `blue-400`
- `Calls` → `gray-500`
- `Inherits`/`Implements` → `purple-400`
- `TypeReference` → `teal-400`
- Config-related → `amber-400`
- AI semantic → `orange-400`

### 10.6 Expand / Collapse Interaction

```javascript
// When a user clicks the expand toggle on a container node:
async function toggleExpand(nodeId) {
    if (expandedNodes.has(nodeId)) {
        // ── COLLAPSE ─────────────────────────────────────────
        expandedNodes.delete(nodeId);

        // 1. Animate children fading out + shrinking into parent
        const children = getVisibleChildren(nodeId);
        await animateCollapse(nodeId, children);

        // 2. Remove children from visible set
        removeFromVisible(children);

        // 3. Request updated aggregated edges from server
        const response = await fetch(`/api/node/${nodeId}/children`);
        const data = await response.json();
        updateExternalEdges(nodeId, data.external_edges);

        // 4. Recompute layout and animate to new positions
        await relayout();

    } else {
        // ── EXPAND ───────────────────────────────────────────
        expandedNodes.add(nodeId);

        // 1. Fetch children and edges from server
        const response = await fetch(`/api/node/${nodeId}/children`);
        const data = await response.json();

        // 2. Add children to visible set
        addToVisible(data.children);

        // 3. Replace aggregated external edges with disaggregated ones
        updateInternalEdges(nodeId, data.internal_edges);
        updateExternalEdges(nodeId, data.external_edges);

        // 4. Recompute layout
        await relayout();

        // 5. Animate children appearing (fade in + expand from parent center)
        await animateExpand(nodeId, data.children);
    }
}
```

**Animation details:**
- Collapse: children scale to 0 and fade out over 200ms, then edges morph into aggregated edges over 150ms.
- Expand: parent grows to accommodate children over 200ms, children fade in at scale 0 and grow to full size over 200ms, then edges fan out over 150ms.
- Layout transition: all nodes smoothly interpolate to new positions over 300ms using D3 transitions.

### 10.7 Agent-Aware Change Indicators

When a file changes (received via WebSocket `GraphDiff`):

```javascript
function applyChangeIndicator(nodeId, changeNumber, changeKind) {
    const node = getNodeElement(nodeId);

    // 1. Add a bright colored ring around the node
    const ring = createChangeRing(node, changeKind);
    // Created = green, Modified = blue, Deleted = red

    // 2. Add a numbered badge showing the change sequence
    const badge = createBadge(changeNumber);
    // Position: top-right corner of the node
    // Style: small circle with number, bold font, bright background

    // 3. Flash animation: pulse the ring 3 times over 1 second
    ring.animate([
        { opacity: 1, transform: "scale(1)" },
        { opacity: 0.3, transform: "scale(1.15)" },
        { opacity: 1, transform: "scale(1)" },
    ], { duration: 1000 });

    // 4. If the changed node is inside a collapsed container,
    //    propagate the indicator UP to the nearest visible ancestor.
    //    The ancestor node gets the ring + badge.
    if (!isVisible(nodeId)) {
        const ancestor = nearestVisibleAncestor(nodeId);
        applyChangeIndicator(ancestor, changeNumber, changeKind);
        // Badge shows "③ (in db/pool.rs)" when propagated
    }

    // 5. Persist the indicator until the NEXT change elsewhere.
    //    Previous indicators dim (opacity 0.3) but remain visible.
    dimPreviousIndicators();

    // 6. Add to the change trail in the bottom bar.
    addToChangeTrail(changeNumber, nodeId, changeKind);
}
```

**Change trail (bottom bar):**
A horizontal sequence of clickable numbered circles: `① ② ③ ④`. Each shows the filename on hover. Clicking one navigates to that node (expanding parent containers if needed). Shows the last 20 changes. Older entries scroll off to the left.

### 10.8 Code Navigation — Editor Integration

When "Open in editor" is clicked (or a node/edge is double-clicked):

```javascript
function openInEditor(filePath, line) {
    const absPath = `${repoRoot}/${filePath}`;

    // Try each editor protocol in order.
    // The user can set their preference in settings.
    const editors = getEditorPreference(); // default: ["vscode", "jetbrains", "generic"]

    for (const editor of editors) {
        switch (editor) {
            case "vscode":
                window.open(`vscode://file/${absPath}:${line}`, "_blank");
                return;
            case "cursor":
                window.open(`cursor://file/${absPath}:${line}`, "_blank");
                return;
            case "jetbrains":
                // Requires JetBrains Toolbox with protocol handler
                window.open(`jetbrains://open?file=${absPath}&line=${line}`, "_blank");
                return;
            case "zed":
                window.open(`zed://open/${absPath}:${line}`, "_blank");
                return;
            case "generic":
                // Send to server, which spawns $CODEGRAPH_EDITOR
                fetch(`/api/open?path=${encodeURIComponent(filePath)}&line=${line}`, {
                    method: "POST",
                });
                return;
        }
    }

    // Fallback: show in the built-in code panel
    showCodePanel(filePath, line);
}
```

The server-side `/api/open` endpoint:

```rust
async fn open_in_editor(Query(params): Query<OpenParams>) -> StatusCode {
    let editor = std::env::var("CODEGRAPH_EDITOR")
        .unwrap_or_else(|_| std::env::var("EDITOR").unwrap_or_default());

    if editor.is_empty() {
        return StatusCode::NOT_FOUND;
    }

    let path = repo_root.join(&params.path);
    let arg = format!("+{}", params.line);

    tokio::process::Command::new(&editor)
        .arg(&arg)
        .arg(&path)
        .spawn()
        .ok();

    StatusCode::OK
}
```

### 10.9 Search

```javascript
let searchDebounce = null;

function onSearchInput(query) {
    clearTimeout(searchDebounce);
    searchDebounce = setTimeout(async () => {
        if (query.length < 2) {
            clearSearchHighlights();
            return;
        }

        const response = await fetch(`/api/search?q=${encodeURIComponent(query)}&limit=20`);
        const data = await response.json();

        // 1. Show dropdown with results
        showSearchResults(data.results);

        // 2. Highlight matching nodes in the graph (if visible).
        //    Dim non-matching nodes to 20% opacity.
        highlightNodes(data.results.map(r => r.node.id));

    }, 150); // 150ms debounce
}

function onSearchResultClick(result) {
    // Expand all ancestor containers to make this node visible.
    const ancestors = result.ancestors; // returned by the API
    for (const ancestorId of ancestors) {
        if (!expandedNodes.has(ancestorId)) {
            await toggleExpand(ancestorId);
        }
    }

    // Center the viewport on this node.
    centerOnNode(result.node.id);

    // Select the node (open detail panel).
    selectNode(result.node.id);

    // Close search dropdown.
    closeSearch();
}
```

### 10.10 Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `/` or `Cmd+K` | Focus search |
| `Escape` | Close search / detail panel / deselect |
| `Enter` (on selected node) | Toggle expand/collapse |
| `Space` (on selected node) | Open in editor |
| `1`, `2`, `3` | Switch layer (L1 Files, L2 Modules+Types, L3 Full) |
| `A` | Toggle AI edges visibility |
| `F` | Fit graph to viewport |
| `H` | Toggle detail panel |
| `?` | Show keyboard shortcuts help |
| `Cmd+scroll` | Zoom |
| Arrow keys | Navigate between sibling nodes |

### 10.11 Layer Views

Layers control which `NodeKind`s are visible, providing different levels of detail.

```javascript
const LAYERS = {
    L1: {
        name: "Files",
        description: "Files and directories only",
        visible_kinds: [
            "WorkspaceRoot", "Package", "Directory", "File",
        ],
    },
    L2: {
        name: "Modules & Types",
        description: "Files, classes, structs, interfaces, and their relationships",
        visible_kinds: [
            "WorkspaceRoot", "Package", "Directory", "File",
            "Module", "Class", "Struct", "Enum", "Interface",
            "ConfigBlock", "Route", "Migration", "CIJob", "DockerService",
        ],
    },
    L3: {
        name: "Full Detail",
        description: "Everything including individual functions and config keys",
        visible_kinds: null, // null = show all
    },
};
```

Switching layers does not affect the expand/collapse state — it only filters which children are shown when a container is expanded. The ELK layout is recomputed on layer change.

### 10.12 Settings Panel

Accessible via the gear icon. Stored in `localStorage`.

```javascript
const DEFAULT_SETTINGS = {
    editor: "vscode",        // vscode, cursor, jetbrains, zed, generic
    theme: "dark",           // dark, light
    default_layer: "L2",
    show_ai_edges: true,
    ai_confidence_threshold: 0.7,
    animation_speed: 1.0,    // 0.5 = half speed, 2.0 = double speed, 0 = instant
    change_trail_length: 20,
    max_visible_nodes: 100,  // warn if expanding would show more than this
};
```

---

## 11. CLI Interface

```
canopy [OPTIONS] [PATH]

ARGUMENTS:
  [PATH]  Path to project root [default: .]

OPTIONS:
  -p, --port <PORT>              Server port [default: 7890]
      --no-open                  Don't auto-open browser tab
      --no-watch                 Index once and serve statically (no live updates)
      --no-ai                    Disable AI semantic bridge entirely
      --ai-provider <PROVIDER>   anthropic | openai | ollama [default: anthropic]
      --ai-model <MODEL>         Model name [default: claude-sonnet-4-20250514]
      --ai-endpoint <URL>        Custom endpoint URL (for Ollama or proxies)
      --depth <N>                Max directory depth to scan
      --include <GLOB>           Additional file patterns to include (repeatable)
      --exclude <GLOB>           Additional file patterns to exclude (repeatable)
      --languages <L1,L2,...>    Only index these languages
      --export <FORMAT>          Export graph and exit (json | dot)
      --ci                       CI mode: index → run structural checks → exit with code
      --low-memory               Don't retain parse trees (slower incremental updates)
  -v, --verbose                  Verbose logging (repeat for more: -vv, -vvv)
  -q, --quiet                    Suppress all output except errors
  -h, --help                     Print help
  -V, --version                  Print version

EXAMPLES:
  canopy                       # index current directory, serve on :7890
  canopy ~/projects/myapp      # index specific path
  canopy --no-ai --port 3000   # no AI, custom port
  canopy --export json > g.json  # export full graph as JSON
  canopy --ci                  # CI mode for structural checks
```

### Configuration File

Optional `.canopy.toml` in project root. CLI flags override config file values.

```toml
[general]
port = 7890
open_browser = true
default_layer = "L2"

[watch]
debounce_ms = 150
ignore = ["*.generated.ts", "vendor/**", "*.snap"]

[ai]
enabled = true
provider = "anthropic"       # "anthropic" | "openai" | "ollama"
model = "claude-sonnet-4-20250514"
# endpoint = "http://localhost:11434"  # for ollama
confidence_threshold = 0.7
daily_budget = 500

[editor]
default = "vscode"           # "vscode" | "cursor" | "jetbrains" | "zed" | "generic"
# command = "nvim"           # for generic editor

[display]
theme = "dark"

[display.language_colors]
# Override default colors
rust = "#DEA584"
python = "#3776AB"

[ci]
# Structural rules checked in CI mode.
# Each rule that fails causes a non-zero exit code.
fail_on_circular_imports = true
max_dependency_depth = 10
# forbidden_edges: list of {from_glob, to_glob} pairs
# Example: API layer should not import from database layer directly
[[ci.forbidden_edges]]
from = "src/api/**"
to = "src/db/**"
```

### `.canopy/` Directory

Created in the project root on first run.

```
.canopy/
├── cache.bincode       # serialized graph for fast cold starts
├── ai_cache.json       # cached AI bridge responses
├── usage.log           # AI API usage tracking
└── parse_trees/        # (if --low-memory is NOT set, trees are in memory instead)
```

On first run, if `.canopy/` is not in `.gitignore`, print:

```
hint: add .canopy/ to your .gitignore
```

Support `CODEGRAPH_CACHE_DIR` env var to relocate the cache directory elsewhere (e.g. `~/.cache/canopy/project-hash/`).

---

## 12. Project Structure

```
canopy/
├── Cargo.toml                    # workspace root
├── Cargo.lock
├── README.md
├── LICENSE-MIT
├── LICENSE-APACHE
├── .github/
│   └── workflows/
│       ├── ci.yml                # test + lint on push/PR
│       └── release.yml           # cross-compile + publish on tag
│
├── crates/
│   ├── canopy-core/           # graph data model, symbol table, resolution
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── graph.rs          # StableDiGraph wrapper, node/edge CRUD, subgraph queries
│   │       ├── model.rs          # NodeKind, EdgeKind, GraphNode, GraphEdge, etc.
│   │       ├── symbols.rs        # SymbolTable, cross-file resolution
│   │       ├── aggregation.rs    # edge aggregation for collapsed containers
│   │       ├── diff.rs           # GraphDiff computation
│   │       ├── workspace.rs      # workspace/monorepo detection
│   │       └── cache.rs          # bincode serialization, cache invalidation
│   │
│   ├── canopy-indexer/        # file parsing and symbol extraction
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── coordinator.rs    # orchestrates parallel indexing, phase management
│   │       ├── tree_cache.rs     # ParseTreeCache, incremental tree-sitter management
│   │       ├── extractor.rs      # LanguageExtractor trait definition
│   │       ├── languages/        # per-language extractors
│   │       │   ├── mod.rs        # registry: extension → extractor mapping
│   │       │   ├── rust.rs
│   │       │   ├── typescript.rs
│   │       │   ├── javascript.rs
│   │       │   ├── python.rs
│   │       │   ├── go.rs
│   │       │   ├── java.rs
│   │       │   ├── c.rs
│   │       │   ├── cpp.rs
│   │       │   └── generic.rs    # fallback: File node, no symbols extracted
│   │       ├── config/           # config file parsers
│   │       │   ├── mod.rs
│   │       │   ├── yaml.rs
│   │       │   ├── toml_parser.rs
│   │       │   ├── json.rs
│   │       │   ├── dotenv.rs
│   │       │   ├── dockerfile.rs
│   │       │   ├── github_actions.rs
│   │       │   └── sql_migration.rs
│   │       └── heuristics/       # config-to-code pattern matching
│   │           ├── mod.rs
│   │           ├── env_vars.rs   # .env → process.env / os.environ matching
│   │           ├── config_keys.rs # config key → config library access patterns
│   │           ├── routes.rs     # route definition → handler matching
│   │           └── docker.rs     # docker-compose → code path matching
│   │
│   ├── canopy-ai/            # AI semantic bridge
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── bridge.rs        # batch processing, orchestration
│   │       ├── prompt.rs        # prompt templates, response parsing
│   │       ├── providers/
│   │       │   ├── mod.rs       # AIProvider trait
│   │       │   ├── anthropic.rs
│   │       │   ├── openai.rs
│   │       │   └── ollama.rs
│   │       ├── cache.rs         # JSON response cache
│   │       └── budget.rs        # daily budget tracking and enforcement
│   │
│   ├── canopy-server/        # HTTP + WebSocket server
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── router.rs        # axum router definition, all routes
│   │       ├── handlers.rs      # request handlers for each endpoint
│   │       ├── websocket.rs     # WebSocket upgrade, diff broadcasting
│   │       ├── highlight.rs     # syntect-based code highlighting
│   │       ├── editor.rs        # editor protocol launching
│   │       └── assets.rs        # rust-embed static file serving
│   │
│   └── canopy-watcher/       # filesystem monitoring
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           └── watcher.rs       # notify setup, debouncing, event classification
│
├── client/                      # browser client (embedded at build time)
│   ├── index.html
│   ├── graph.js                 # ELK layout, SVG rendering, expand/collapse
│   ├── ui.js                    # search, filters, detail panel, settings
│   ├── protocol.js              # WebSocket, diff application, local state
│   ├── style.css
│   └── vendor/                  # bundled CDN fallbacks
│       ├── elk.bundled.js
│       ├── d3.min.js
│       └── highlight.min.js
│
├── src/
│   └── main.rs                  # CLI entry point: parse args, wire components, run
│
└── tests/
    ├── fixtures/                 # sample repos for integration tests
    │   ├── rust-workspace/       # Cargo workspace with 3 crates
    │   ├── ts-monorepo/          # npm workspace with packages
    │   ├── python-project/       # single Python project with .env
    │   ├── mixed-project/        # multi-language with docker-compose
    │   └── config-heavy/         # lots of YAML/TOML/JSON to test config linking
    └── integration/
        ├── indexing_test.rs      # full index pipeline tests
        ├── resolution_test.rs    # cross-file resolution tests
        ├── incremental_test.rs   # file change → correct diff tests
        ├── config_link_test.rs   # config-to-code heuristic tests
        ├── aggregation_test.rs   # edge aggregation correctness
        ├── server_test.rs        # HTTP API contract tests
        └── workspace_test.rs     # monorepo detection tests
```

---

## 13. Rust Dependencies

```toml
[workspace]
members = [
    "crates/canopy-core",
    "crates/canopy-indexer",
    "crates/canopy-ai",
    "crates/canopy-server",
    "crates/canopy-watcher",
]
resolver = "2"

[workspace.dependencies]
# ── Async runtime ────────────────────────────────────────
tokio = { version = "1", features = ["full"] }

# ── Web server ───────────────────────────────────────────
axum = "0.7"
axum-extra = { version = "0.9", features = ["query"] }
tokio-tungstenite = "0.24"
tower-http = { version = "0.6", features = ["cors"] }

# ── Tree-sitter parsing ─────────────────────────────────
tree-sitter = "0.24"
tree-sitter-rust = "0.23"
tree-sitter-typescript = "0.23"
tree-sitter-javascript = "0.23"
tree-sitter-python = "0.23"
tree-sitter-go = "0.23"
tree-sitter-java = "0.23"
tree-sitter-c = "0.23"
tree-sitter-cpp = "0.23"

# ── Config file parsing ─────────────────────────────────
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
toml = "0.8"

# ── Graph data structure ────────────────────────────────
petgraph = "0.6"

# ── Filesystem watching ─────────────────────────────────
notify = { version = "6", features = ["macos_fsevent"] }
ignore = "0.4"           # gitignore-aware walking (from ripgrep)
globset = "0.4"

# ── HTTP client (AI bridge) ─────────────────────────────
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# ── Serialization ───────────────────────────────────────
bincode = "1"

# ── Syntax highlighting ─────────────────────────────────
syntect = "5"

# ── Static asset embedding ──────────────────────────────
rust-embed = "8"

# ── CLI ─────────────────────────────────────────────────
clap = { version = "4", features = ["derive"] }

# ── Concurrency ─────────────────────────────────────────
dashmap = "6"
rayon = "1"

# ── Utilities ───────────────────────────────────────────
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
anyhow = "1"
thiserror = "2"
fuzzy-matcher = "0.3"
open = "5"               # open browser
regex = "1"
mime_guess = "2"

# ── Testing ─────────────────────────────────────────────
[workspace.dev-dependencies]
insta = { version = "1", features = ["json"] }
tempfile = "3"
tokio-test = "0.4"
```

---

## 14. Concurrency Model

```
┌─────────────────────────────────────────────────────────────┐
│                        main thread                           │
│                                                              │
│  1. Parse CLI args                                           │
│  2. Detect workspace                                         │
│  3. Run initial full index (uses rayon for parallelism)      │
│  4. Start tokio runtime                                      │
│  5. Spawn all async tasks                                    │
│  6. Block on signal handler (SIGINT/SIGTERM)                 │
└──────────────────────────────┬──────────────────────────────┘
                               │
            ┌──────────────────┴──────────────────┐
            │           tokio runtime              │
            │                                      │
            │  ┌──────────────────────────────┐    │
            │  │    Watcher task               │    │
            │  │    (spawns OS thread via      │    │
            │  │     notify, sends events      │    │
            │  │     through mpsc channel)     │    │
            │  └──────────────┬───────────────┘    │
            │                 │ FileEvent           │
            │  ┌──────────────▼───────────────┐    │
            │  │    Reindex task               │    │
            │  │    Receives batched events,   │    │
            │  │    spawns rayon tasks for     │    │
            │  │    parallel file re-parsing.  │    │
            │  │    Acquires write lock on     │    │
            │  │    graph briefly to apply     │    │
            │  │    changes. Computes diff.    │    │
            │  └──────────────┬───────────────┘    │
            │                 │ GraphDiff           │
            │  ┌──────────────▼───────────────┐    │
            │  │    Broadcast task             │    │
            │  │    Sends diff to all WS       │    │
            │  │    clients via broadcast::    │    │
            │  │    Sender.                    │    │
            │  └──────────────────────────────┘    │
            │                                      │
            │  ┌──────────────────────────────┐    │
            │  │    AI Bridge task             │    │
            │  │    Async HTTP calls, batched. │    │
            │  │    Runs after reindex if      │    │
            │  │    there are unresolved refs. │    │
            │  │    Results trigger another    │    │
            │  │    small GraphDiff broadcast. │    │
            │  └──────────────────────────────┘    │
            │                                      │
            │  ┌──────────────────────────────┐    │
            │  │    HTTP server (axum)         │    │
            │  │    Acquires read lock on      │    │
            │  │    graph for API requests.    │    │
            │  └──────────────────────────────┘    │
            │                                      │
            └──────────────────────────────────────┘
```

**Shared state:**

```rust
struct SharedState {
    /// The code graph. Write lock held briefly during reindex.
    /// Read lock for all API requests and WebSocket reads.
    graph: Arc<RwLock<Graph>>,

    /// Symbol table. DashMap for concurrent read access.
    /// Rebuilt during reindex (swap-based: build new, swap pointer).
    symbol_table: Arc<ArcSwap<SymbolTable>>,

    /// Parse tree cache. DashMap, updated per-file.
    parse_cache: Arc<ParseTreeCache>,

    /// Broadcast channel for graph diffs.
    diff_tx: broadcast::Sender<GraphDiff>,

    /// Monotonically increasing diff sequence number.
    sequence: AtomicU64,

    /// File change counter for UI change trail.
    change_counter: AtomicU32,

    /// Repo root path.
    repo_root: PathBuf,

    /// Configuration.
    config: Arc<Config>,
}
```

The write lock on `graph` is held only for the duration of applying node/edge additions and removals — not during parsing. Parsing happens in rayon threads that produce a `Vec<GraphMutation>`, which is then applied to the graph under the lock in a single batch. This keeps write-lock contention minimal.

---

## 15. Testing Strategy

### Unit Tests

Each crate has its own unit tests. Key areas:

| Crate | Test Focus |
|-------|-----------|
| `canopy-core` | Graph CRUD, edge aggregation correctness, symbol table resolution, diff computation |
| `canopy-indexer` | Per-language extraction correctness (given source → expected symbols/references), incremental re-parse correctness |
| `canopy-ai` | Prompt construction, response JSON parsing, cache hit/miss, budget enforcement |
| `canopy-server` | Route handler correctness, WebSocket message serialization |
| `canopy-watcher` | Debounce behavior, ignore pattern matching |

### Integration Tests

Located in `tests/integration/`. Each test:
1. Creates a temp directory with a fixture repo (or copies from `tests/fixtures/`).
2. Runs the full indexing pipeline.
3. Asserts on expected graph structure (specific nodes exist, specific edges exist, edge counts).

```rust
#[test]
fn test_rust_workspace_indexing() {
    let fixture = load_fixture("rust-workspace");
    let graph = index_project(&fixture.path, &Config::default()).unwrap();

    // Should detect workspace with 3 packages
    assert_eq!(graph.nodes_of_kind(NodeKind::Package).count(), 3);

    // Package "api" should import from package "core"
    let api = graph.find_node_by_name("api").unwrap();
    let core = graph.find_node_by_name("core").unwrap();
    assert!(graph.has_edge_between(api, core, EdgeKind::Imports));

    // Should have resolved cross-crate type reference
    let handler = graph.find_node_by_qualified("api::handlers::create_user").unwrap();
    let user_model = graph.find_node_by_qualified("core::models::User").unwrap();
    assert!(graph.has_edge_between(handler, user_model, EdgeKind::TypeReference));
}

#[test]
fn test_env_var_linking() {
    let fixture = load_fixture("python-project");
    let graph = index_project(&fixture.path, &Config::default()).unwrap();

    // .env has DATABASE_URL, code has os.environ["DATABASE_URL"]
    let env_node = graph.find_node_by_name("DATABASE_URL").unwrap();
    assert_eq!(graph.node(env_node).kind, NodeKind::EnvVariable);

    let edges: Vec<_> = graph.edges_from(env_node)
        .filter(|e| e.kind == EdgeKind::EnvironmentBinding)
        .collect();
    assert_eq!(edges.len(), 1);
    assert!(edges[0].target_name().contains("get_db_connection"));
}

#[test]
fn test_incremental_update() {
    let fixture = load_fixture("ts-monorepo");
    let (mut graph, mut cache) = index_project(&fixture.path, &Config::default()).unwrap();

    let initial_node_count = graph.node_count();

    // Simulate adding a new function to an existing file
    let file = fixture.path.join("packages/api/src/handlers.ts");
    let mut content = fs::read_to_string(&file).unwrap();
    content.push_str("\nexport function newHandler() { return 42; }\n");
    fs::write(&file, &content).unwrap();

    // Reindex just this file
    let diff = reindex_file(&mut graph, &mut cache, &file).unwrap();

    // Should have exactly one new node
    assert_eq!(diff.added_nodes.len(), 1);
    assert_eq!(diff.added_nodes[0].name, "newHandler");
    assert_eq!(graph.node_count(), initial_node_count + 1);
}
```

### Snapshot Tests

Using `insta` for JSON snapshots of graph structure. Catches unintentional changes to extraction behavior.

```rust
#[test]
fn test_python_extraction_snapshot() {
    let source = r#"
class UserService:
    def __init__(self, db: Database):
        self.db = db

    def get_user(self, user_id: int) -> User:
        return self.db.query(User, user_id)
    "#;

    let result = PythonExtractor::new().extract(Path::new("service.py"), source.as_bytes(), None).unwrap();
    insta::assert_json_snapshot!(result.1); // snapshot the ExtractionResult
}
```

### Browser Tests

Playwright tests against a running `canopy` instance:

```javascript
test('initial load shows top-level directories', async ({ page }) => {
    await page.goto('http://localhost:7890');
    await page.waitForSelector('[data-node-kind="Directory"]');
    const nodes = await page.$$('[data-node-kind]');
    expect(nodes.length).toBeGreaterThan(0);
    expect(nodes.length).toBeLessThan(30); // not showing everything
});

test('clicking directory expands children', async ({ page }) => {
    await page.goto('http://localhost:7890');
    const srcDir = await page.waitForSelector('[data-node-name="src"]');
    const initialCount = (await page.$$('[data-node-kind]')).length;

    await srcDir.click();
    await page.waitForTimeout(500); // wait for animation

    const expandedCount = (await page.$$('[data-node-kind]')).length;
    expect(expandedCount).toBeGreaterThan(initialCount);
});

test('file change triggers visual update', async ({ page }) => {
    await page.goto('http://localhost:7890');
    await page.waitForSelector('[data-node-kind="Directory"]');

    // Modify a file externally
    fs.appendFileSync('test-repo/src/main.rs', '\nfn new_func() {}\n');

    // Wait for change indicator
    const badge = await page.waitForSelector('.change-badge', { timeout: 2000 });
    expect(badge).toBeTruthy();
});
```

### Performance Benchmarks

Using `criterion`:

```rust
fn bench_full_index(c: &mut Criterion) {
    let sizes = [100, 1_000, 5_000];
    for size in sizes {
        let fixture = generate_synthetic_repo(size);
        c.bench_function(&format!("full_index_{}_files", size), |b| {
            b.iter(|| index_project(&fixture.path, &Config::default()))
        });
    }
}

fn bench_incremental_reindex(c: &mut Criterion) {
    let fixture = generate_synthetic_repo(5_000);
    let (mut graph, mut cache) = index_project(&fixture.path, &Config::default()).unwrap();

    c.bench_function("incremental_single_file", |b| {
        b.iter(|| {
            // Simulate a single file change
            let file = fixture.random_file();
            reindex_file(&mut graph, &mut cache, &file)
        })
    });
}

fn bench_edge_aggregation(c: &mut Criterion) {
    let fixture = generate_synthetic_repo(5_000);
    let (graph, _) = index_project(&fixture.path, &Config::default()).unwrap();

    c.bench_function("aggregate_edges_top_level", |b| {
        let visible = top_level_node_ids(&graph);
        b.iter(|| aggregate_edges(&graph, &visible))
    });
}
```

### CI Pipeline

```yaml
name: CI
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo test --all
      - run: cargo bench --no-run  # compile check only

  release:
    if: startsWith(github.ref, 'refs/tags/v')
    needs: check
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: aarch64-apple-darwin
            os: macos-latest
          - target: x86_64-pc-windows-msvc
            os: windows-latest
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - run: cargo build --release --target ${{ matrix.target }}
      - uses: softprops/action-gh-release@v2
        with:
          files: target/${{ matrix.target }}/release/canopy*

  publish:
    if: startsWith(github.ref, 'refs/tags/v')
    needs: check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo publish -p canopy-core
      - run: cargo publish -p canopy-indexer
      - run: cargo publish -p canopy-ai
      - run: cargo publish -p canopy-server
      - run: cargo publish -p canopy-watcher
      - run: cargo publish  # root package
```

---

## 16. Build and Distribution

### Single Binary

```bash
cargo build --release
# produces: target/release/canopy
# size: ~20–30MB (includes embedded client assets + tree-sitter grammars)
```

The binary is fully self-contained. No runtime dependencies. No Node.js. No Python. No external database.

### Installation Methods

```bash
# Cargo (from source)
cargo install canopy

# Homebrew (macOS/Linux)
brew install canopy/tap/canopy

# npm wrapper (downloads prebuilt binary)
npx canopy

# Nix flake
nix run github:canopy-dev/canopy

# Arch Linux (AUR)
yay -S canopy-bin

# Direct download (any platform)
curl -fsSL https://canopy.dev/install.sh | sh

# Docker (for CI usage)
docker run --rm -v $(pwd):/repo canopy/canopy --ci
```

### Cross-Compilation Targets

| Target | Platform |
|--------|----------|
| `x86_64-unknown-linux-gnu` | Linux x86_64 |
| `aarch64-unknown-linux-gnu` | Linux ARM64 (AWS Graviton, etc.) |
| `x86_64-apple-darwin` | macOS Intel |
| `aarch64-apple-darwin` | macOS Apple Silicon |
| `x86_64-pc-windows-msvc` | Windows x86_64 |

All targets built in CI on every tagged release using GitHub Actions with `cross`.

---

## 17. Enterprise Tier (Future, Closed Source)

The open source core covers single-repo, single-user, local usage. Enterprise extends with:

| Feature | Description | Value Proposition |
|---------|-------------|-------------------|
| **MCP Server** | Expose the graph as an MCP server. AI agents query "what calls this function" or "what's the blast radius of changing this file" before making changes. | Agents produce better code when they understand architecture. |
| **Multi-repo view** | Aggregate graphs across repositories. Show cross-repo dependencies (shared libraries, API contracts). | Monorepo and microservice teams see the full picture. |
| **Architecture history** | Git-integrated timeline. Replay how the graph evolved commit-by-commit or PR-by-PR. Diff two points in time. | "How did our architecture change this quarter?" |
| **Structural regression CI** | GitHub Action that fails builds on configurable structural rules: no circular imports, forbidden dependency paths, max coupling thresholds. | Architectural guardrails enforced automatically. |
| **Team annotations** | Shared bookmarks, ownership labels, notes on graph nodes. Synced across team members. | Institutional knowledge attached to the code it describes. |
| **PR diff visualization** | When reviewing a PR, see how the graph changes — new edges, removed edges, new dependencies. | Visual code review for architectural impact. |
| **Hosted graph** | Cloud-hosted persistent graph with SSO, RBAC, audit logging. | Enterprise deployment without local setup. |

**The MCP server is the highest-priority enterprise feature.** Implementation sketch:

```rust
// MCP tool definitions exposed by the canopy server:
//
// canopy_get_architecture_summary
//   Returns: top-level packages/modules and their dependency relationships
//
// canopy_get_callers(symbol: string)
//   Returns: all functions/methods that call the given symbol
//
// canopy_get_dependencies(file: string)
//   Returns: all files that this file depends on (transitively)
//
// canopy_get_dependents(file: string)
//   Returns: all files that depend on this file (blast radius)
//
// canopy_get_config_bindings(file: string)
//   Returns: all config keys that affect code in this file
//
// canopy_check_structural_rules(changes: Vec<FileChange>)
//   Returns: whether proposed changes would violate any structural rules
```

---

## 18. Milestone Plan

### M1 — Navigable Hierarchy (5–6 weeks)

**Goal:** The expand/collapse navigation with live updating works beautifully, even with shallow data.

**Deliverables:**
- CLI boots, walks filesystem, detects workspace structure
- Directory + File nodes with `Contains` hierarchy
- Rust + TypeScript language extractors (symbols only, no cross-file resolution)
- ELK-based hierarchical SVG layout in browser
- Expand/collapse with animated edge aggregation/disaggregation
- Zoom, pan, click-to-select, detail panel with file path
- `notify`-based file watcher with WebSocket live diffs
- Change indicators with numbered badges and change trail
- `.env` → code environment variable linking (first config-to-code demo)
- Editor integration (VS Code URI protocol)
- `.canopy/cache.bincode` for fast restarts

**Not in M1:** Cross-file resolution, AI bridge, search, layers, most config parsers, most languages.

**Success criteria:** Run `canopy` on a medium Rust or TS project. See the hierarchy. Expand directories. Watch edges aggregate and split smoothly. Start an AI agent in another terminal. See changes appear in real time with numbered indicators. Click a node, open it in VS Code.

### M2 — Full Structural Analysis (5–6 weeks)

**Deliverables:**
- All Tier 1 language extractors (Python, Go, Java, C/C++)
- Cross-file symbol resolution (imports, calls, type references, inheritance)
- All Tier 2 config parsers (YAML, TOML, JSON, Dockerfile, docker-compose, GitHub Actions, SQL migrations)
- Config-to-code heuristic linking (env vars, config keys, routes, Docker mounts)
- Fuzzy search with ancestor expansion
- Layer views (L1 Files, L2 Modules+Types, L3 Full Detail)
- Incremental tree-sitter parsing with retained trees
- Path finding between two nodes
- Export (JSON, DOT)
- `--ci` mode with structural rule checking

### M3 — AI Bridge + Polish (3–4 weeks)

**Deliverables:**
- AI semantic bridge (Anthropic, OpenAI, Ollama providers)
- Batched prompt construction, response parsing, edge creation
- AI result caching and budget controls
- AI edges visually distinct (dashed, with confidence and reasoning in detail panel)
- AI edge toggle in UI
- Settings panel (editor preference, theme, confidence threshold)
- Keyboard shortcuts
- Performance optimization: profile and fix any layout/rendering bottlenecks
- Syntax-highlighted code panel (syntect)
- Comprehensive test suite (unit, integration, snapshot, browser)

### M4 — Community Release (2–3 weeks)

**Deliverables:**
- README with demo GIFs (record with `vhs` or `asciinema`)
- Documentation site (mdBook or similar)
- `cargo install`, Homebrew tap, `npx` wrapper, install script
- Cross-compiled binaries for all 5 targets
- GitHub release automation
- Contributing guide
- Launch: Hacker News, Reddit r/rust + r/programming, Twitter/X, relevant Discord servers

### M5 — Enterprise Foundations (ongoing)

**Deliverables:**
- MCP server integration (highest priority)
- Structural regression CI GitHub Action
- Architecture history (git log integration)
- Multi-repo view prototype
- PR diff visualization prototype
