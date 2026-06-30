use std::{
    fmt, fs,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::project::{LoadedProject, ProjectCompilationPlan};

const COMPILE_CACHE_EPOCH: &str = "2026-06-22-aot-streaming-cache-fingerprint-v1";

#[path = "cache_entry.rs"]
mod cache_entry;
#[path = "cache_fingerprint.rs"]
mod cache_fingerprint;
#[path = "cache_fs.rs"]
mod cache_fs;
#[path = "cache_key.rs"]
mod cache_key;
#[path = "cache_maintenance.rs"]
mod cache_maintenance;
#[cfg(test)]
#[path = "cache_tests.rs"]
mod cache_tests;
#[path = "cache_types.rs"]
mod cache_types;

pub use cache_entry::{
    compile_cache_status, compile_cache_status_with_plan, lookup_compile_cache,
    restore_compile_cache, store_compile_cache,
};
pub use cache_key::{compute_compile_cache_key, compute_compile_cache_key_with_plan};
pub use cache_maintenance::{
    clean_compile_cache, clean_compile_cache_summary, clean_compile_cache_with_plan,
    compile_cache_inventory, compile_cache_inventory_summary, prune_compile_cache,
    prune_compile_cache_summary, prune_compile_cache_with_plan,
};
pub use cache_types::{
    CleanedCompileCache, CleanedCompileCacheSummary, CompileCacheEntry, CompileCacheInventory,
    CompileCacheInventoryEntry, CompileCacheInventorySummary, CompileCacheKey, CompileCacheStatus,
    PrunedCompileCache, PrunedCompileCacheSummary,
};

use cache_fingerprint::{fingerprint_records, CacheFingerprintRecord};
use cache_fs::{
    collect_cache_entry_stats, copy_directory_recursive, discover_compile_cache_roots,
    sanitize_path_label, summarize_cache_entries, summarize_directory,
};
use cache_key::cache_root;
