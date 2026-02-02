//! Thread-safe parser pool for tree-sitter parsers
//!
//! This module provides a thread-safe way to use tree-sitter parsers in async contexts.
//! Tree-sitter parsers are not Send + Sync, so we use a channel-based approach with
//! dedicated parser threads to work around this limitation.

use std::path::PathBuf;
use anyhow::Result;
use tree_sitter::{Parser, Language};

/// Supported file types for parsing
#[derive(Debug, Clone)]
pub enum FileType {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    Java,
    C,
    Cpp,
    Generic,
}

impl FileType {
    /// Determine file type from file extension
    pub fn from_path(path: &PathBuf) -> Option<Self> {
        let ext = path.extension()?.to_str()?;
        match ext {
            "rs" => Some(FileType::Rust),
            "ts" => Some(FileType::TypeScript),
            "tsx" => Some(FileType::TypeScript),
            "js" => Some(FileType::JavaScript),
            "jsx" => Some(FileType::JavaScript),
            "py" => Some(FileType::Python),
            "go" => Some(FileType::Go),
            "java" => Some(FileType::Java),
            "c" => Some(FileType::C),
            "cpp" | "cc" | "cxx" => Some(FileType::Cpp),
            "h" | "hpp" => Some(FileType::Cpp),
            _ => Some(FileType::Generic),
        }
    }

    /// Get the tree-sitter language for this file type
    pub fn get_language(&self) -> Language {
        match self {
            FileType::Rust => tree_sitter_rust::LANGUAGE.into(),
            FileType::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            FileType::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            FileType::Python => tree_sitter_python::LANGUAGE.into(),
            FileType::Go => tree_sitter_go::LANGUAGE.into(),
            FileType::Java => tree_sitter_java::LANGUAGE.into(),
            FileType::C => tree_sitter_c::LANGUAGE.into(),
            FileType::Cpp => tree_sitter_cpp::LANGUAGE.into(),
            FileType::Generic => tree_sitter_rust::LANGUAGE.into(), // Fallback
        }
    }
}

/// A parsing request sent to the parser pool
#[derive(Debug)]
pub struct ParseRequest {
    pub file_type: FileType,
    pub content: String,
    pub path: PathBuf,
}

/// Result of a parsing operation
#[derive(Debug)]
pub struct ParseResult {
    pub tree: tree_sitter::Tree,
    pub path: PathBuf,
    pub content: String,
}

/// Result of parsing a file with additional metadata
#[derive(Debug)]
pub struct FileParseResult {
    pub language: String,
    pub ast_json: String,
    pub path: PathBuf,
}

/// Internal message for the parser worker
#[derive(Debug)]
struct WorkerRequest {
    request: ParseRequest,
    response_sender: std::sync::mpsc::Sender<Result<ParseResult>>,
}

/// Thread-safe parser pool
pub struct ParserPool {
    sender: std::sync::mpsc::Sender<WorkerRequest>,
}

impl ParserPool {
    /// Create a new parser pool with the specified number of worker threads
    pub fn new(num_workers: usize) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel::<WorkerRequest>();
        let receiver = std::sync::Arc::new(std::sync::Mutex::new(receiver));

        for i in 0..num_workers {
            let receiver = receiver.clone();
            std::thread::spawn(move || {
                Self::worker_thread(i, receiver);
            });
        }

        Self { sender }
    }

    /// Worker thread function that processes parsing requests
    fn worker_thread(
        worker_id: usize,
        receiver: std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<WorkerRequest>>>,
    ) {
        tracing::debug!("Parser worker {} started", worker_id);
        
        let mut parser = Parser::new();
        
        loop {
            let request = match receiver.lock().unwrap().recv() {
                Ok(req) => req,
                Err(_) => {
                    tracing::debug!("Parser worker {} shutting down", worker_id);
                    break;
                }
            };

            let WorkerRequest { request, response_sender } = request;
            
            // Set the language for this parser
            let language = request.file_type.get_language();
            if let Err(e) = parser.set_language(&language) {
                let _ = response_sender.send(Err(anyhow::anyhow!("Failed to set language: {}", e)));
                continue;
            }

            // Parse the content
            let result = match parser.parse(&request.content, None) {
                Some(tree) => Ok(ParseResult {
                    tree,
                    path: request.path,
                    content: request.content,
                }),
                None => Err(anyhow::anyhow!("Failed to parse content")),
            };

            // Send the result back
            if response_sender.send(result).is_err() {
                tracing::warn!("Failed to send parse result back to caller");
            }
        }
    }

    /// Parse content synchronously using the parser pool
    /// Note: This blocks the current thread until parsing is complete
    pub fn parse_blocking(&self, request: ParseRequest) -> Result<ParseResult> {
        let (response_sender, response_receiver) = std::sync::mpsc::channel();
        
        let worker_request = WorkerRequest {
            request,
            response_sender,
        };

        // Send the request to the worker pool
        self.sender.send(worker_request)
            .map_err(|_| anyhow::anyhow!("Parser pool is shut down"))?;

        // Wait for the result
        response_receiver.recv()
            .map_err(|_| anyhow::anyhow!("Parser worker died"))?
    }

    /// Parse content asynchronously using the parser pool
    pub async fn parse(&self, request: ParseRequest) -> Result<ParseResult> {
        // Use spawn_blocking to run the synchronous parse in a blocking context
        let sender = self.sender.clone();
        tokio::task::spawn_blocking(move || {
            let (response_sender, response_receiver) = std::sync::mpsc::channel();
            
            let worker_request = WorkerRequest {
                request,
                response_sender,
            };

            // Send the request to the worker pool
            sender.send(worker_request)
                .map_err(|_| anyhow::anyhow!("Parser pool is shut down"))?;

            // Wait for the result
            response_receiver.recv()
                .map_err(|_| anyhow::anyhow!("Parser worker died"))?
        }).await.map_err(|e| anyhow::anyhow!("Task join error: {}", e))?
    }

    /// Parse a file and return a simplified result with language and AST JSON
    pub async fn parse_file(&self, path: &PathBuf, content: &str) -> Result<FileParseResult> {
        let file_type = FileType::from_path(path)
            .ok_or_else(|| anyhow::anyhow!("Cannot determine file type for: {:?}", path))?;
        
        let request = ParseRequest {
            file_type: file_type.clone(),
            content: content.to_string(),
            path: path.clone(),
        };
        
        let parse_result = self.parse(request).await?;
        
        // Convert AST to JSON representation
        let ast_json = tree_to_json(&parse_result.tree.root_node(), content);
        
        let language = match file_type {
            FileType::Rust => "rust",
            FileType::TypeScript => "typescript",
            FileType::JavaScript => "javascript",
            FileType::Python => "python",
            FileType::Go => "go",
            FileType::Java => "java",
            FileType::C => "c",
            FileType::Cpp => "cpp",
            FileType::Generic => "generic",
        };
        
        Ok(FileParseResult {
            language: language.to_string(),
            ast_json,
            path: path.clone(),
        })
    }
}

/// Convert a tree-sitter tree to a JSON representation
fn tree_to_json(node: &tree_sitter::Node, source: &str) -> String {
    use std::fmt::Write;
    
    fn write_node<W: Write>(writer: &mut W, node: tree_sitter::Node, source: &str, depth: usize) {
        let indent = "  ".repeat(depth);
        let _ = writer.write_str(&indent);
        let _ = write!(writer, "{{\"type\":\"{}\",", node.kind());
        
        // Add text content for leaf nodes
        if node.child_count() == 0 {
            if let Ok(text) = node.utf8_text(source.as_bytes()) {
                let escaped = text.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
                let _ = write!(writer, "\"text\":\"{}\",", escaped);
            }
        }
        
        let _ = write!(writer, "\"start\":{},\"end\":{},", node.start_byte(), node.end_byte());
        
        if node.child_count() > 0 {
            let _ = write!(writer, "\"children\":[");
            let mut cursor = node.walk();
            let mut first = true;
            for child in node.children(&mut cursor) {
                if !first {
                    let _ = write!(writer, ",");
                }
                first = false;
                let _ = write!(writer, "\n");
                write_node(writer, child, source, depth + 1);
            }
            let _ = write!(writer, "\n{}]", indent);
        }
        
        let _ = writer.write_str("}");
    }
    
    let mut result = String::new();
    write_node(&mut result, *node, source, 0);
    result
}

impl Clone for ParserPool {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

/// Convenience function to create a parser pool with default settings
pub fn create_parser_pool() -> ParserPool {
    // Use number of CPU cores as default worker count, but at least 2
    let num_workers = std::thread::available_parallelism()
        .map(|n| n.get().max(2))
        .unwrap_or(2);
    
    ParserPool::new(num_workers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_rust() {
        let pool = create_parser_pool();
        let content = r#"
fn main() {
    println!("Hello, world!");
}
"#.to_string();
        
        let request = ParseRequest {
            file_type: FileType::Rust,
            content,
            path: PathBuf::from("test.rs"),
        };

        let result = pool.parse(request).await.unwrap();
        assert_eq!(result.tree.root_node().kind(), "source_file");
    }

    #[tokio::test]
    async fn test_parse_typescript() {
        let pool = create_parser_pool();
        let content = r#"
class MyClass {
    method() {
        console.log("Hello");
    }
}
"#.to_string();
        
        let request = ParseRequest {
            file_type: FileType::TypeScript,
            content,
            path: PathBuf::from("test.ts"),
        };

        let result = pool.parse(request).await.unwrap();
        assert_eq!(result.tree.root_node().kind(), "program");
    }
}