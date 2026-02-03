//! AI semantic bridge for Canopy
//!
//! This crate provides AI-powered semantic analysis of code,
//! including relationship inference, code summarization, and
//! natural language querying of the codebase.

pub mod bridge;
pub mod prompt;
pub mod providers;
pub mod cache;
pub mod budget;

#[cfg(test)]
pub mod tests;

pub use bridge::*;
pub use budget::Budget;
pub use cache::AnalysisCache;