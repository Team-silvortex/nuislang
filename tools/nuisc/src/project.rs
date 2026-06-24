use std::path::{Path, PathBuf};

use nuis_semantics::model::{
    AstExpr, AstModule, NirAttributeValue, NirDataFlowState, NirResultStage, NirTypeRef,
};
#[cfg(test)]
use yir_core::YirModule;

mod abi;
mod bridge_contracts;
mod data_bridge_directions;
mod data_contract_types;
mod data_validation;
mod kernel_validation;
mod manifest;
mod network_validation;
mod packet;
mod planning;
mod profile_apply;
mod profile_refs;
mod profile_targets;
mod profile_usage;
mod rendering;
mod runtime_validation;
mod shader_validation;
mod support_contracts;
mod type_contracts;
mod validation_core;

pub(crate) use abi::{
    backend_family_for_registered_abi_target, backend_features_for_registered_abi_target,
    selected_lowering_target_for_registered_abi_target,
};
pub use abi::{
    ensure_project_abi_selections_valid, project_abi_selection_check_json,
    render_project_abi_selection_check_lines, resolve_project_abi, validate_project_abi_selections,
    write_project_abi_selection_check_lines,
};
#[cfg(test)]
use abi::{host_calling_abi, host_object_format, recommend_abi_profile_for_host};
use bridge_contracts::{
    build_project_link_bridge_contract, materialize_project_bridge_contract_nodes,
    required_project_link_stage_contract, validate_project_link_stage_contract,
};
use data_contract_types::{
    find_profile_call_declared_type, infer_data_handle_table_schema,
    infer_project_route_payload_type, merge_project_payload_contract, payload_class_marker_name,
    payload_shape_marker_name, require_marker_semantic_payload_name, require_profile_semantic_type,
};
#[cfg(test)]
use data_validation::validate_data_profile_token_types;
#[cfg(test)]
use kernel_validation::{
    validate_kernel_profile_slot_contract, validate_kernel_target_config_contract,
};
use manifest::{parse_project_manifest, sanitize_ident};
#[cfg(test)]
use network_validation::validate_network_target_projection;
pub use packet::render_project_packet_index;
pub use planning::{
    build_project_compilation_plan, describe_project, describe_project_compilation_plan,
    describe_project_dependency_categories, describe_project_exchange_route_classes,
    describe_project_output_intent_categories, load_project, project_docs_summary,
    project_galaxy_summary, render_project_compilation_plan_index,
    write_project_compilation_plan_index, write_project_metadata,
};
use profile_apply::{
    apply_support_module_profile, collect_profile_int_bindings, ensure_project_resource,
    extract_profile_call, push_profile_node,
};
use profile_refs::push_project_dependency_edge_if_missing;
use profile_targets::resolve_project_profile_target_name;
pub use rendering::{
    describe_project_abi_graph, ensure_project_lowering_selections_valid, organize_project,
    organize_project_exchanges, project_abi_selection_view_json, project_abi_selection_views,
    project_imports_summary, project_lowering_selection_json, render_project_abi_graph_line,
    render_project_abi_selection_lines, render_project_abi_selection_view_lines,
    render_project_import_index, render_project_lowering_selection_lines,
    validate_project_lowering_selections, write_project_abi_selection_lines,
    write_project_abi_selection_view_lines, write_project_lowering_selection_lines,
};
#[cfg(test)]
use rendering::{
    render_project_abi_index, render_project_exchange_index, render_project_host_ffi_index,
    render_project_organization_index,
};
use rendering::{
    write_project_abi_index, write_project_exchange_index, write_project_host_ffi_index,
    write_project_import_index, write_project_organization_index,
};
pub use runtime_validation::{
    apply_project_links_to_yir, apply_project_support_modules_to_yir,
    prune_project_topology_for_codegen, validate_project_abi_against_yir,
    validate_project_links_against_nir, validate_project_links_against_yir,
};
use shader_validation::{has_edge_to, infer_shader_packet_contract};
#[cfg(test)]
use shader_validation::{
    shader_packet_support_surface_contract, validate_shader_packet_contract,
    validate_shader_target_projection, ShaderPacketContract,
};
use support_contracts::{
    data_profile_required_slots_for_link, kernel_profile_slot_targets,
    require_declared_profile_slot, support_profile_slots_for_domain,
};
use type_contracts::materialize_project_type_contract_nodes;
use validation_core::{
    split_domain_unit, validate_project_abi_requirements, validate_project_links,
    validate_project_modules, validate_project_unit_bindings, validate_project_uses,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuisProjectManifest {
    pub name: String,
    pub entry: String,
    pub modules: Vec<String>,
    pub tests: Vec<String>,
    pub links: Vec<ProjectLink>,
    pub abi_requirements: Vec<ProjectAbiRequirement>,
    pub galaxy_dependencies: Vec<ProjectGalaxyDependency>,
    pub galaxy_imports: Vec<ProjectGalaxyImport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectLink {
    pub from: String,
    pub to: String,
    pub via: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectAbiRequirement {
    pub domain: String,
    pub abi: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectGalaxyDependency {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectGalaxyImport {
    pub galaxy: String,
    pub library_module: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectModuleOrigin {
    LocalProject {
        manifest_spec: String,
    },
    AutoInjectedGalaxy {
        galaxy: String,
        package_id: String,
        library_module: String,
        import_policy: String,
    },
    ExplicitGalaxyImport {
        galaxy: String,
        package_id: String,
        library_module: String,
        import_policy: String,
    },
}

impl ProjectModuleOrigin {
    pub fn source_kind(&self) -> &'static str {
        match self {
            Self::LocalProject { .. } => "project-local",
            Self::AutoInjectedGalaxy { .. } => "galaxy-auto-inject",
            Self::ExplicitGalaxyImport { .. } => "galaxy-explicit-import",
        }
    }

    pub fn source_detail(&self) -> String {
        match self {
            Self::LocalProject { manifest_spec } => format!("manifest_spec={manifest_spec}"),
            Self::AutoInjectedGalaxy {
                galaxy,
                package_id,
                library_module,
                import_policy,
            } => format!(
                "galaxy={galaxy}\tpackage={package_id}\tlibrary_module={library_module}\timport_policy={import_policy}"
            ),
            Self::ExplicitGalaxyImport {
                galaxy,
                package_id,
                library_module,
                import_policy,
            } => format!(
                "galaxy={galaxy}\tpackage={package_id}\tlibrary_module={library_module}\timport_policy={import_policy}"
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectModule {
    pub path: PathBuf,
    pub ast: AstModule,
    pub origin: ProjectModuleOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedProject {
    pub root: PathBuf,
    pub manifest_path: PathBuf,
    pub manifest: NuisProjectManifest,
    pub entry_path: PathBuf,
    pub entry_source: String,
    pub modules: Vec<ProjectModule>,
    pub resolved_galaxies: Vec<crate::stdlib_registry::ResolvedGalaxyDependency>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProjectTextHandleRewriteSummary {
    pub helper_hits: usize,
    pub local_hits: usize,
}

impl ProjectTextHandleRewriteSummary {
    pub fn total_hits(self) -> usize {
        self.helper_hits + self.local_hits
    }
}

pub fn summarize_project_text_handle_rewrites(
    project: &LoadedProject,
) -> Result<ProjectTextHandleRewriteSummary, String> {
    let mut summary = ProjectTextHandleRewriteSummary::default();
    for module in &project.modules {
        let helper_modules = project
            .modules
            .iter()
            .filter(|candidate| candidate.path != module.path)
            .map(|candidate| candidate.ast.clone())
            .collect::<Vec<_>>();
        let nir = crate::frontend::lower_project_ast_to_nir(&module.ast, &helper_modules)?;
        for function in &nir.functions {
            for annotation in &function.annotations {
                if annotation.name != "__nuisc_text_handle_rewrite" {
                    continue;
                }
                for arg in &annotation.args {
                    let Some(name) = arg.name.as_deref() else {
                        continue;
                    };
                    let NirAttributeValue::Int(value) = arg.value else {
                        continue;
                    };
                    if value <= 0 {
                        continue;
                    }
                    match name {
                        "helper" => summary.helper_hits += value as usize,
                        "local" => summary.local_hits += value as usize,
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(summary)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDocsSummary {
    pub modules: usize,
    pub documented_modules: usize,
    pub documented_items: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectImportsSummary {
    pub libraries: usize,
    pub visible_libraries: usize,
    pub visible_modules: usize,
    pub documented_visible_modules: usize,
    pub documented_visible_items: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectGalaxySummary {
    pub galaxies: usize,
    pub documented_galaxies: usize,
    pub documented_library_modules: usize,
    pub documented_items: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectBuildMetadata {
    pub manifest_copy_path: String,
    pub plan_index_path: String,
    pub organization_index_path: String,
    pub exchange_index_path: String,
    pub modules_index_path: String,
    pub docs_index_path: String,
    pub docs_summary: ProjectDocsSummary,
    pub imports_index_path: String,
    pub imports_summary: ProjectImportsSummary,
    pub galaxy_index_path: String,
    pub galaxy_summary: ProjectGalaxySummary,
    pub links_index_path: String,
    pub packet_index_path: String,
    pub host_ffi_index_path: String,
    pub abi_index_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectOrganization {
    pub entry: String,
    pub domains: Vec<String>,
    pub modules: Vec<ProjectOrganizationModule>,
    pub links: Vec<ProjectOrganizationLink>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectOrganizationModule {
    pub path: String,
    pub domain: String,
    pub unit: String,
    pub is_entry: bool,
    pub source_kind: String,
    pub source_detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectOrganizationLink {
    pub from: String,
    pub to: String,
    pub via: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectExchangeOrganization {
    pub routes: Vec<ProjectExchangeRoute>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectExchangeRoute {
    pub from: String,
    pub to: String,
    pub via: Option<String>,
    pub mode: String,
    pub class: String,
    pub domains: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectAbiResolution {
    pub requirements: Vec<ProjectAbiRequirement>,
    pub explicit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectAbiSelectionView {
    pub domain: String,
    pub abi: String,
    pub machine_arch: Option<String>,
    pub machine_os: Option<String>,
    pub object_format: Option<String>,
    pub calling_abi: Option<String>,
    pub clang_target: Option<String>,
    pub backend_family: Option<String>,
    pub vendor: Option<String>,
    pub device_class: Option<String>,
    pub host_adaptive: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectAbiIssueKind {
    MissingExplicitDomainAbi,
    UnusedExplicitDomainAbi,
    DomainNotRegistered,
    AbiNotRegistered,
}

impl ProjectAbiIssueKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MissingExplicitDomainAbi => "missing_explicit_domain_abi",
            Self::UnusedExplicitDomainAbi => "unused_explicit_domain_abi",
            Self::DomainNotRegistered => "domain_not_registered",
            Self::AbiNotRegistered => "abi_not_registered",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::MissingExplicitDomainAbi => "ABI001",
            Self::UnusedExplicitDomainAbi => "ABI002",
            Self::DomainNotRegistered => "ABI003",
            Self::AbiNotRegistered => "ABI004",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectAbiIssue {
    pub kind: ProjectAbiIssueKind,
    pub message: String,
}

impl ProjectAbiIssue {
    pub fn summary(&self) -> String {
        format!(
            "{} {}: {}",
            self.kind.code(),
            self.kind.as_str(),
            self.message
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectAbiSelectionCheck {
    pub domain: String,
    pub abi: Option<String>,
    pub source: String,
    pub abi_registered: bool,
    pub ok: bool,
    pub issues: Vec<ProjectAbiIssue>,
}

impl ProjectAbiSelectionCheck {
    pub fn issue_count(&self) -> usize {
        self.issues.len()
    }

    pub fn summary_line(&self) -> String {
        format!(
            "{} (source={}, abi={}): {}",
            self.domain,
            self.source,
            self.abi.as_deref().unwrap_or("<none>"),
            if self.issues.is_empty() {
                "ok".to_owned()
            } else {
                self.issues
                    .iter()
                    .map(ProjectAbiIssue::summary)
                    .collect::<Vec<_>>()
                    .join("; ")
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectLoweringIssueKind {
    DomainNotRegistered,
    NoRegisteredLoweringTargets,
    AbiTargetResolutionFailed,
    SelectedLoweringTargetNotRegistered,
}

impl ProjectLoweringIssueKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DomainNotRegistered => "domain_not_registered",
            Self::NoRegisteredLoweringTargets => "no_registered_lowering_targets",
            Self::AbiTargetResolutionFailed => "abi_target_resolution_failed",
            Self::SelectedLoweringTargetNotRegistered => "selected_lowering_target_not_registered",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::DomainNotRegistered => "NLT001",
            Self::NoRegisteredLoweringTargets => "NLT002",
            Self::AbiTargetResolutionFailed => "NLT003",
            Self::SelectedLoweringTargetNotRegistered => "NLT004",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectLoweringIssue {
    pub kind: ProjectLoweringIssueKind,
    pub message: String,
}

impl ProjectLoweringIssue {
    pub fn summary(&self) -> String {
        format!(
            "{} {}: {}",
            self.kind.code(),
            self.kind.as_str(),
            self.message
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectLoweringSelectionView {
    pub domain: String,
    pub abi: Option<String>,
    pub registered_lowering_targets: Vec<String>,
    pub selected_lowering_target: Option<String>,
    pub ok: bool,
    pub issues: Vec<ProjectLoweringIssue>,
}

impl ProjectLoweringSelectionView {
    pub fn issue_count(&self) -> usize {
        self.issues.len()
    }

    pub fn summary_line(&self) -> String {
        format!(
            "{} (abi={}, selected={}): {}",
            self.domain,
            self.abi.as_deref().unwrap_or("<none>"),
            self.selected_lowering_target.as_deref().unwrap_or("<none>"),
            if self.issues.is_empty() {
                "ok".to_owned()
            } else {
                self.issues
                    .iter()
                    .map(ProjectLoweringIssue::summary)
                    .collect::<Vec<_>>()
                    .join("; ")
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectCompilationDependency {
    pub category: String,
    pub name: String,
    pub version: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSyntheticInput {
    pub kind: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectOutputIntent {
    pub category: String,
    pub kind: String,
    pub path_hint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectCompilationPlan {
    pub project_name: String,
    pub entry: String,
    pub organization: ProjectOrganization,
    pub exchanges: ProjectExchangeOrganization,
    pub abi_resolution: ProjectAbiResolution,
    pub dependencies: Vec<ProjectCompilationDependency>,
    pub synthetic_input: ProjectSyntheticInput,
    pub output_intents: Vec<ProjectOutputIntent>,
    pub effective_input_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ProjectLinkStageContract {
    uplink: NirResultStage,
    downlink: NirResultStage,
}

impl ProjectLinkStageContract {
    pub(super) fn windowed_data_bridge() -> Self {
        Self {
            uplink: NirResultStage::Data(NirDataFlowState::Windowed),
            downlink: NirResultStage::Data(NirDataFlowState::Windowed),
        }
    }

    pub(super) fn is_windowed_data_bridge(&self) -> bool {
        self.uplink == NirResultStage::Data(NirDataFlowState::Windowed)
            && self.downlink == NirResultStage::Data(NirDataFlowState::Windowed)
    }

    pub(super) fn directions(self) -> [(&'static str, NirResultStage); 2] {
        [("uplink", self.uplink), ("downlink", self.downlink)]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProjectLinkBridgeContract {
    stages: ProjectLinkStageContract,
    payloads: [Option<NirTypeRef>; 2],
}

impl ProjectLinkBridgeContract {
    pub(super) fn payload(&self, is_uplink: bool) -> Option<&NirTypeRef> {
        self.payloads[bridge_direction_index(is_uplink)].as_ref()
    }

    pub(super) fn into_payload(self, is_uplink: bool) -> Option<NirTypeRef> {
        self.payloads[bridge_direction_index(is_uplink)].clone()
    }
}

fn bridge_direction_index(is_uplink: bool) -> usize {
    if is_uplink {
        0
    } else {
        1
    }
}

pub fn is_project_input(path: &Path) -> bool {
    path.is_dir() || path.file_name().and_then(|name| name.to_str()) == Some("nuis.toml")
}

pub(super) fn lower_project_module_to_nir(
    project: &LoadedProject,
    project_module: &ProjectModule,
) -> Result<nuis_semantics::model::NirModule, String> {
    let sibling_modules = project
        .modules
        .iter()
        .filter(|module| module.path != project_module.path)
        .map(|module| module.ast.clone())
        .collect::<Vec<_>>();
    crate::frontend::lower_project_ast_to_nir(&project_module.ast, &sibling_modules)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Shared filesystem/in-memory test builders live in `test_support`; feature-specific
    // helpers stay in each test file until they prove reusable across files.
    #[path = "abi_recommendation.rs"]
    mod abi_recommendation;
    #[path = "galaxy_resolution.rs"]
    mod galaxy_resolution;
    #[path = "multidomain_async.rs"]
    mod multidomain_async;
    #[path = "packet_data_contracts.rs"]
    mod packet_data_contracts;
    #[path = "planning_kernel.rs"]
    mod planning_kernel;
    #[path = "shader_nova_contracts.rs"]
    mod shader_nova_contracts;
    #[path = "test_support.rs"]
    mod test_support;

    fn project_with_modules(modules: Vec<(&str, &str)>) -> LoadedProject {
        LoadedProject {
            root: PathBuf::from("."),
            manifest_path: PathBuf::from("nuis.toml"),
            manifest: NuisProjectManifest {
                name: "test".to_owned(),
                entry: "main.ns".to_owned(),
                modules: vec![],
                tests: vec![],
                links: vec![],
                abi_requirements: vec![],
                galaxy_dependencies: vec![],
                galaxy_imports: vec![],
            },
            entry_path: PathBuf::from("main.ns"),
            entry_source: String::new(),
            modules: modules
                .into_iter()
                .map(|(path, source)| ProjectModule {
                    path: PathBuf::from(path),
                    ast: crate::frontend::parse_nuis_ast(source).unwrap(),
                    origin: ProjectModuleOrigin::LocalProject {
                        manifest_spec: path.to_owned(),
                    },
                })
                .collect(),
            resolved_galaxies: vec![],
        }
    }
}
