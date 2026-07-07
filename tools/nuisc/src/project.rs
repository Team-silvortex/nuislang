use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;

use nuis_semantics::model::AstExpr;
#[cfg(test)]
use nuis_semantics::model::{NirDataFlowState, NirResultStage};
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
mod profile_refs_data;
mod profile_targets;
mod profile_usage;
mod rendering;
mod runtime_validation;
mod shader_validation;
mod support_contracts;
mod type_contracts;
mod types;
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

pub use types::*;
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
