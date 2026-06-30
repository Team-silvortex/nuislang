use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileCacheKey {
    pub root: PathBuf,
    pub key: String,
    pub input_labels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileCacheEntry {
    pub root: PathBuf,
    pub key: String,
    pub entry_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileCacheStatus {
    pub root: PathBuf,
    pub key: String,
    pub input_labels: Vec<String>,
    pub entry_exists: bool,
    pub entry_dir: PathBuf,
    pub file_count: usize,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileCacheInventoryEntry {
    pub key: String,
    pub entry_dir: PathBuf,
    pub file_count: usize,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileCacheInventory {
    pub root: PathBuf,
    pub entry_count: usize,
    pub total_files: usize,
    pub total_bytes: u64,
    pub entries: Vec<CompileCacheInventoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileCacheInventorySummary {
    pub workspace_root: PathBuf,
    pub roots: Vec<CompileCacheInventory>,
    pub total_entries: usize,
    pub total_files: usize,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanedCompileCache {
    pub root: PathBuf,
    pub removed_entries: usize,
    pub removed_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanedCompileCacheSummary {
    pub workspace_root: PathBuf,
    pub cleaned_roots: Vec<CleanedCompileCache>,
    pub removed_entries: usize,
    pub removed_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrunedCompileCache {
    pub root: PathBuf,
    pub kept_entries: usize,
    pub removed_entries: usize,
    pub removed_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrunedCompileCacheSummary {
    pub workspace_root: PathBuf,
    pub pruned_roots: Vec<PrunedCompileCache>,
    pub kept_entries: usize,
    pub removed_entries: usize,
    pub removed_bytes: u64,
}
