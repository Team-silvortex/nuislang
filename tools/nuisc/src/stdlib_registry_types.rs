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
    pub schema: String,
    pub name: String,
    pub default_entry: String,
    pub modules: Vec<StdlibIndexModule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdlibIndexModule {
    pub name: String,
    pub version: String,
    pub kind: String,
    pub path: String,
    pub package_id: String,
    pub depends_on: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalaxyResolutionProviderDescriptor {
    pub provider_id: String,
    pub provider_kind: String,
    pub root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalaxyResolutionProviderRequirement {
    pub name: String,
    pub version_requirement: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalaxyResolutionCandidateSetReport {
    pub contract: String,
    pub status: String,
    pub generation: u64,
    pub index_sha256: String,
    pub candidate_sha256: String,
    pub response_sha256: String,
    pub signature_count: usize,
    pub signer_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalaxyResolutionProviderRequest {
    pub contract: String,
    pub provider_id: String,
    pub provider_kind: String,
    pub request_sha256: String,
    pub candidate_set: GalaxyResolutionCandidateSetReport,
    pub requirements: Vec<GalaxyResolutionProviderRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalaxyResolutionProviderSelection {
    pub name: String,
    pub version: String,
    pub package_id: String,
    pub relative_path: String,
    pub direct: bool,
    pub requested_by: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalaxyResolutionProviderReport {
    pub contract: String,
    pub status: String,
    pub provider_id: String,
    pub provider_kind: String,
    pub request_sha256: String,
    pub selection_sha256: String,
    pub candidate_count: usize,
    pub selected_count: usize,
    pub candidate_set: GalaxyResolutionCandidateSetReport,
    pub requirements: Vec<GalaxyResolutionProviderRequirement>,
    pub selections: Vec<GalaxyResolutionProviderSelection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalaxyResolutionProviderResolution {
    pub request: GalaxyResolutionProviderRequest,
    pub report: GalaxyResolutionProviderReport,
    pub dependencies: Vec<ResolvedGalaxyDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdlibModuleManifest {
    pub name: String,
    pub package_id: String,
    pub tier: String,
    pub depends_on: Vec<String>,
    pub summary: String,
    pub surfaces: Vec<String>,
    pub code_assets: Vec<String>,
    pub source_modules: Vec<String>,
    pub library_modules: Vec<String>,
    pub library_import_policy: StdlibLibraryImportPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedGalaxyContentIdentity {
    pub logical_path: String,
    pub bytes: usize,
    pub sha256: String,
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
    pub manifest_content_identity: ResolvedGalaxyContentIdentity,
    pub depends_on: Vec<String>,
    pub surfaces: Vec<String>,
    pub code_assets: Vec<String>,
    pub source_modules: Vec<String>,
    pub resolved_source_paths: Vec<PathBuf>,
    pub source_content_identities: Vec<ResolvedGalaxyContentIdentity>,
    pub library_modules: Vec<String>,
    pub resolved_library_paths: Vec<PathBuf>,
    pub library_content_identities: Vec<ResolvedGalaxyContentIdentity>,
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
