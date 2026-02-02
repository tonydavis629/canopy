//! Workspace/monorepo detection

use std::path::Path;

/// Detect if this is a workspace (Cargo, npm, etc.)
pub fn detect_workspace(root: &Path) -> Option<WorkspaceType> {
    if root.join("Cargo.toml").exists() {
        Some(WorkspaceType::Cargo)
    } else if root.join("package.json").exists() {
        Some(WorkspaceType::Npm)
    } else if root.join("go.mod").exists() {
        Some(WorkspaceType::GoModules)
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceType {
    Cargo,
    Npm,
    GoModules,
    Maven,  // pom.xml
    Gradle, // build.gradle
}
