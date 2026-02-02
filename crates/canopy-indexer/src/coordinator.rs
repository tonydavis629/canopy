//! Orchestrates parallel indexing

use anyhow::Result;

pub struct Coordinator;

impl Coordinator {
    pub fn new() -> Self {
        Coordinator
    }

    pub fn run_full_index(&self) -> Result<()> {
        todo!("Implement full indexing")
    }
}
