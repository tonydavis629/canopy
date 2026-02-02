//! Test utilities for Canopy

use tempfile::TempDir;
use std::fs;
use std::path::Path;

/// Create a temporary test repository with sample files
pub fn create_test_repo() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    
    // Create directory structure
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("src/utils")).unwrap();
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::create_dir_all(root.join(".github/workflows")).unwrap();
    
    // Create Rust source files
    fs::write(root.join("src/main.rs"), r#"
use crate::utils::helper;

fn main() {
    println!("Hello, world!");
    helper::do_something();
}
"#).unwrap();
    
    fs::write(root.join("src/utils/mod.rs"), r#"
pub mod helper;
"#).unwrap();
    
    fs::write(root.join("src/utils/helper.rs"), r#"
pub fn do_something() {
    println!("Helper function");
}
"#).unwrap();
    
    // Create TypeScript file
    fs::write(root.join("src/index.ts"), r#"
import { UserService } from './services/user';

function main() {
    const service = new UserService();
    service.loadUsers();
}

main();
"#).unwrap();
    
    // Create services directory
    fs::create_dir_all(root.join("src/services")).unwrap();
    fs::write(root.join("src/services/user.ts"), r#"
export class UserService {
    private users: User[] = [];
    
    loadUsers() {
        // Implementation
    }
}
"#).unwrap();
    
    // Create config files
    fs::write(root.join("Cargo.toml"), r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
"#).unwrap();
    
    fs::write(root.join("package.json"), r#"
{
  "name": "test-project",
  "version": "0.1.0",
  "dependencies": {
    "typescript": "^5.0.0"
  }
}
"#).unwrap();
    
    fs::write(root.join(".env"), r#"
DATABASE_URL=postgres://localhost/mydb
API_KEY=secret-key
PORT=3000
"#).unwrap();
    
    fs::write(root.join(".github/workflows/ci.yml"), r#"
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run tests
        run: cargo test
"#).unwrap();
    
    temp_dir
}

/// Create a simple test repository with just a few files
pub fn create_simple_repo() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    
    fs::create_dir_all(root.join("src")).unwrap();
    
    fs::write(root.join("main.rs"), r#"
fn main() {
    println!("Hello!");
}
"#).unwrap();
    
    fs::write(root.join("lib.rs"), r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#).unwrap();
    
    temp_dir
}

/// Create a repository with a specific file structure
pub fn create_repo_with_structure(structure: &[(&str, &str)]) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    
    for (path, content) in structure {
        let full_path = root.join(path);
        
        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        
        fs::write(&full_path, content).unwrap();
    }
    
    temp_dir
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_test_repo() {
        let temp_dir = create_test_repo();
        let root = temp_dir.path();
        
        // Check that files were created
        assert!(root.join("src/main.rs").exists());
        assert!(root.join("src/utils/helper.rs").exists());
        assert!(root.join("src/index.ts").exists());
        assert!(root.join("Cargo.toml").exists());
        assert!(root.join("package.json").exists());
        assert!(root.join(".env").exists());
    }
}
