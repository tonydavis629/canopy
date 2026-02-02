//! Filesystem monitoring

pub mod watcher;

pub use watcher::{FileWatcher, WatchEvent, WatcherService};