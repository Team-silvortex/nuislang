use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StdlibLibraryImportPolicy {
    ProjectAuto,
    ManualOnly,
}

impl StdlibLibraryImportPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ProjectAuto => "project-auto",
            Self::ManualOnly => "manual-only",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdlibLayout {
    pub name: String,
    pub default_entry: String,
    pub modules: Vec<StdlibIndexModule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdlibIndexModule {
    pub name: String,
    pub kind: String,
    pub path: String,
    pub package_id: String,
    pub depends_on: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdlibModuleManifest {
    pub name: String,
    pub package_id: String,
    pub tier: String,
    pub depends_on: Vec<String>,
    pub summary: String,
    pub surfaces: Vec<String>,
    pub source_modules: Vec<String>,
    pub library_modules: Vec<String>,
    pub library_import_policy: StdlibLibraryImportPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedGalaxyDependency {
    pub name: String,
    pub version: String,
    pub package_id: String,
    pub direct: bool,
    pub requested_by: Vec<String>,
    pub module_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub depends_on: Vec<String>,
    pub surfaces: Vec<String>,
    pub source_modules: Vec<String>,
    pub resolved_source_paths: Vec<PathBuf>,
    pub library_modules: Vec<String>,
    pub resolved_library_paths: Vec<PathBuf>,
    pub library_import_policy: StdlibLibraryImportPolicy,
    pub auto_injectable: bool,
    pub auto_inject_blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedGalaxyDocSummary {
    pub(crate) documented_library_modules: usize,
    pub(crate) documented_items: usize,
    pub(crate) library_module_items: Vec<(String, usize)>,
}
