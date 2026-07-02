use std::path::Path;

use crate::aot_toml::{
    parse_optional_toml_string, parse_optional_toml_usize, parse_required_toml_bool,
    parse_required_toml_string, parse_required_toml_string_array,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ManifestFieldVerification {
    pub cpu_target_abi: String,
    pub cpu_target_machine_arch: String,
    pub cpu_target_machine_os: String,
    pub cpu_target_object_format: String,
    pub cpu_target_calling_abi: String,
    pub cpu_target_clang: String,
    pub cpu_target_cross: bool,
    pub loaded_nustar: Vec<String>,
    pub compile_cache_status: Option<String>,
    pub compile_cache_key: Option<String>,
    pub compile_cache_root: Option<String>,
    pub doc_index_path: Option<String>,
    pub doc_index_module_count: usize,
    pub doc_index_documented_item_count: usize,
    pub project_text_handle_rewrite_helper_hits: usize,
    pub project_text_handle_rewrite_local_hits: usize,
    pub project_plan_index: Option<String>,
    pub project_plan_summary: Option<String>,
    pub project_docs_index: Option<String>,
    pub project_docs_module_count: usize,
    pub project_docs_documented_module_count: usize,
    pub project_docs_documented_item_count: usize,
    pub project_imports_index: Option<String>,
    pub project_imports_library_count: usize,
    pub project_imports_visible_library_count: usize,
    pub project_imports_visible_module_count: usize,
    pub project_imports_documented_visible_module_count: usize,
    pub project_imports_documented_visible_item_count: usize,
    pub project_galaxy_index: Option<String>,
    pub project_galaxy_count: usize,
    pub project_documented_galaxy_count: usize,
    pub project_documented_galaxy_library_module_count: usize,
    pub project_documented_galaxy_item_count: usize,
    pub project_packet_index: Option<String>,
    pub project_host_ffi_index: Option<String>,
    pub bridge_registry_path: Option<String>,
    pub bridge_registry_schema: Option<String>,
    pub bridge_registry_units: usize,
    pub bridge_registry_inline: Option<String>,
    pub host_bridge_plan_index_path: Option<String>,
    pub host_bridge_plan_index_schema: Option<String>,
    pub host_bridge_plan_units: usize,
    pub host_bridge_plan_index_inline: Option<String>,
    pub lowering_plan_index_path: Option<String>,
    pub lowering_plan_index_schema: Option<String>,
    pub lowering_plan_units: usize,
    pub lowering_plan_index_inline: Option<String>,
    pub clock_protocol_path: Option<String>,
    pub clock_protocol_schema: Option<String>,
    pub clock_protocol_domains: usize,
    pub clock_protocol_inline: Option<String>,
    pub hetero_calculate_plan_path: Option<String>,
    pub hetero_calculate_plan_schema: Option<String>,
    pub hetero_calculate_plan_units: usize,
    pub hetero_calculate_plan_inline: Option<String>,
}

pub(crate) fn verify_manifest_fields(
    source: &str,
    path: &Path,
) -> Result<ManifestFieldVerification, String> {
    Ok(ManifestFieldVerification {
        cpu_target_abi: parse_required_toml_string(source, "cpu_target_abi", path)?,
        cpu_target_machine_arch: parse_required_toml_string(
            source,
            "cpu_target_machine_arch",
            path,
        )?,
        cpu_target_machine_os: parse_required_toml_string(source, "cpu_target_machine_os", path)?,
        cpu_target_object_format: parse_required_toml_string(
            source,
            "cpu_target_object_format",
            path,
        )?,
        cpu_target_calling_abi: parse_required_toml_string(source, "cpu_target_calling_abi", path)?,
        cpu_target_clang: parse_required_toml_string(source, "cpu_target_clang", path)?,
        cpu_target_cross: parse_required_toml_bool(source, "cpu_target_cross", path)?,
        loaded_nustar: parse_required_toml_string_array(source, "loaded_nustar", path)?,
        compile_cache_status: parse_optional_toml_string(source, "compile_cache_status"),
        compile_cache_key: parse_optional_toml_string(source, "compile_cache_key"),
        compile_cache_root: parse_optional_toml_string(source, "compile_cache_root"),
        doc_index_path: parse_optional_toml_string(source, "doc_index_path"),
        doc_index_module_count: parse_optional_toml_usize(source, "doc_index_module_count")
            .unwrap_or(0),
        doc_index_documented_item_count: parse_optional_toml_usize(
            source,
            "doc_index_documented_item_count",
        )
        .unwrap_or(0),
        project_text_handle_rewrite_helper_hits: parse_optional_toml_usize(
            source,
            "text_handle_rewrite_helper_hits",
        )
        .unwrap_or(0),
        project_text_handle_rewrite_local_hits: parse_optional_toml_usize(
            source,
            "text_handle_rewrite_local_hits",
        )
        .unwrap_or(0),
        project_plan_index: parse_optional_toml_string(source, "plan_index"),
        project_plan_summary: parse_optional_toml_string(source, "plan_summary"),
        project_docs_index: parse_optional_toml_string(source, "docs_index"),
        project_docs_module_count: parse_optional_toml_usize(source, "docs_module_count")
            .unwrap_or(0),
        project_docs_documented_module_count: parse_optional_toml_usize(
            source,
            "docs_documented_module_count",
        )
        .unwrap_or(0),
        project_docs_documented_item_count: parse_optional_toml_usize(
            source,
            "docs_documented_item_count",
        )
        .unwrap_or(0),
        project_imports_index: parse_optional_toml_string(source, "imports_index"),
        project_imports_library_count: parse_optional_toml_usize(source, "imports_library_count")
            .unwrap_or(0),
        project_imports_visible_library_count: parse_optional_toml_usize(
            source,
            "imports_visible_library_count",
        )
        .unwrap_or(0),
        project_imports_visible_module_count: parse_optional_toml_usize(
            source,
            "imports_visible_module_count",
        )
        .unwrap_or(0),
        project_imports_documented_visible_module_count: parse_optional_toml_usize(
            source,
            "imports_documented_visible_module_count",
        )
        .unwrap_or(0),
        project_imports_documented_visible_item_count: parse_optional_toml_usize(
            source,
            "imports_documented_visible_item_count",
        )
        .unwrap_or(0),
        project_galaxy_index: parse_optional_toml_string(source, "galaxy_index"),
        project_galaxy_count: parse_optional_toml_usize(source, "galaxy_count").unwrap_or(0),
        project_documented_galaxy_count: parse_optional_toml_usize(
            source,
            "documented_galaxy_count",
        )
        .unwrap_or(0),
        project_documented_galaxy_library_module_count: parse_optional_toml_usize(
            source,
            "documented_galaxy_library_module_count",
        )
        .unwrap_or(0),
        project_documented_galaxy_item_count: parse_optional_toml_usize(
            source,
            "documented_galaxy_item_count",
        )
        .unwrap_or(0),
        project_packet_index: parse_optional_toml_string(source, "packet_index"),
        project_host_ffi_index: parse_optional_toml_string(source, "host_ffi_index"),
        bridge_registry_path: parse_optional_toml_string(source, "bridge_registry_path"),
        bridge_registry_schema: parse_optional_toml_string(source, "bridge_registry_schema"),
        bridge_registry_units: parse_optional_toml_usize(source, "bridge_registry_units")
            .unwrap_or(0),
        bridge_registry_inline: parse_optional_toml_string(source, "bridge_registry_inline"),
        host_bridge_plan_index_path: parse_optional_toml_string(
            source,
            "host_bridge_plan_index_path",
        ),
        host_bridge_plan_index_schema: parse_optional_toml_string(
            source,
            "host_bridge_plan_index_schema",
        ),
        host_bridge_plan_units: parse_optional_toml_usize(source, "host_bridge_plan_units")
            .unwrap_or(0),
        host_bridge_plan_index_inline: parse_optional_toml_string(
            source,
            "host_bridge_plan_index_inline",
        ),
        lowering_plan_index_path: parse_optional_toml_string(source, "lowering_plan_index_path"),
        lowering_plan_index_schema: parse_optional_toml_string(
            source,
            "lowering_plan_index_schema",
        ),
        lowering_plan_units: parse_optional_toml_usize(source, "lowering_plan_units").unwrap_or(0),
        lowering_plan_index_inline: parse_optional_toml_string(
            source,
            "lowering_plan_index_inline",
        ),
        clock_protocol_path: parse_optional_toml_string(source, "clock_protocol_path"),
        clock_protocol_schema: parse_optional_toml_string(source, "clock_protocol_schema"),
        clock_protocol_domains: parse_optional_toml_usize(source, "clock_protocol_domains")
            .unwrap_or(0),
        clock_protocol_inline: parse_optional_toml_string(source, "clock_protocol_inline"),
        hetero_calculate_plan_path: parse_optional_toml_string(
            source,
            "hetero_calculate_plan_path",
        ),
        hetero_calculate_plan_schema: parse_optional_toml_string(
            source,
            "hetero_calculate_plan_schema",
        ),
        hetero_calculate_plan_units: parse_optional_toml_usize(
            source,
            "hetero_calculate_plan_units",
        )
        .unwrap_or(0),
        hetero_calculate_plan_inline: parse_optional_toml_string(
            source,
            "hetero_calculate_plan_inline",
        ),
    })
}
