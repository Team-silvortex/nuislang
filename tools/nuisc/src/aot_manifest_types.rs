use crate::aot_cpu_target::CpuBuildTarget;

pub struct CompileArtifacts {
    pub ast_path: String,
    pub nir_path: String,
    pub yir_path: String,
    pub llvm_ir_path: String,
    pub binary_path: String,
    pub packaging_mode: String,
}

pub struct BuildManifestProjectInfo {
    pub name: String,
    pub abi_mode: String,
    pub abi_graph_summary: Option<String>,
    pub abi_entries: Vec<(String, String)>,
    pub plan_summary: Option<String>,
    pub effective_input: Option<String>,
    pub text_handle_rewrite_helper_hits: usize,
    pub text_handle_rewrite_local_hits: usize,
    pub manifest_copy_path: Option<String>,
    pub plan_index_path: Option<String>,
    pub organization_index_path: Option<String>,
    pub exchange_index_path: Option<String>,
    pub modules_index_path: Option<String>,
    pub docs_index_path: Option<String>,
    pub docs_module_count: usize,
    pub docs_documented_module_count: usize,
    pub docs_documented_item_count: usize,
    pub imports_index_path: Option<String>,
    pub imports_library_count: usize,
    pub imports_visible_library_count: usize,
    pub imports_visible_module_count: usize,
    pub imports_documented_visible_module_count: usize,
    pub imports_documented_visible_item_count: usize,
    pub galaxy_index_path: Option<String>,
    pub galaxy_count: usize,
    pub galaxy_documented_count: usize,
    pub galaxy_documented_library_module_count: usize,
    pub galaxy_documented_item_count: usize,
    pub links_index_path: Option<String>,
    pub packet_index_path: Option<String>,
    pub host_ffi_index_path: Option<String>,
    pub abi_index_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildManifestDocIndexInfo {
    pub path: String,
    pub module_count: usize,
    pub documented_item_count: usize,
}

pub struct BuildManifestContext {
    pub input_path: String,
    pub output_dir: String,
    pub loaded_nustar: Vec<String>,
    pub compile_cache: Option<BuildManifestCacheInfo>,
    pub project: Option<BuildManifestProjectInfo>,
    pub doc_index: Option<BuildManifestDocIndexInfo>,
    pub cpu_target: CpuBuildTarget,
}

pub struct BuildManifestCacheInfo {
    pub status: String,
    pub key: String,
    pub root: String,
}
