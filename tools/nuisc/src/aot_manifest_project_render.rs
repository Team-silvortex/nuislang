use crate::aot_manifest_types::BuildManifestProjectInfo;
use crate::aot_toml::{escape_toml_string, render_string_array};

pub(crate) fn append_project_manifest_section(
    out: &mut String,
    project: &BuildManifestProjectInfo,
) {
    out.push('\n');
    out.push_str("[project]\n");
    out.push_str(&format!(
        "name = \"{}\"\n",
        escape_toml_string(&project.name)
    ));
    out.push_str(&format!(
        "abi_mode = \"{}\"\n",
        escape_toml_string(&project.abi_mode)
    ));
    if let Some(value) = &project.abi_graph_summary {
        out.push_str(&format!("abi_graph = \"{}\"\n", escape_toml_string(value)));
    }
    if let Some(value) = &project.plan_summary {
        out.push_str(&format!(
            "plan_summary = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    if let Some(value) = &project.effective_input {
        out.push_str(&format!(
            "effective_input = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    out.push_str(&format!(
        "text_handle_rewrite_helper_hits = {}\n",
        project.text_handle_rewrite_helper_hits
    ));
    out.push_str(&format!(
        "text_handle_rewrite_local_hits = {}\n",
        project.text_handle_rewrite_local_hits
    ));
    let mut abi_entries = project
        .abi_entries
        .iter()
        .map(|(domain, abi)| format!("{domain}={abi}"))
        .collect::<Vec<_>>();
    abi_entries.sort();
    out.push_str(&format!("abi = {}\n", render_string_array(&abi_entries)));
    if let Some(value) = &project.manifest_copy_path {
        out.push_str(&format!(
            "manifest_copy = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    if let Some(value) = &project.plan_index_path {
        out.push_str(&format!("plan_index = \"{}\"\n", escape_toml_string(value)));
    }
    if let Some(value) = &project.organization_index_path {
        out.push_str(&format!(
            "organization_index = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    if let Some(value) = &project.exchange_index_path {
        out.push_str(&format!(
            "exchange_index = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    if let Some(value) = &project.modules_index_path {
        out.push_str(&format!(
            "modules_index = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    if let Some(value) = &project.docs_index_path {
        out.push_str(&format!("docs_index = \"{}\"\n", escape_toml_string(value)));
    }
    out.push_str(&format!(
        "docs_module_count = {}\n",
        project.docs_module_count
    ));
    out.push_str(&format!(
        "docs_documented_module_count = {}\n",
        project.docs_documented_module_count
    ));
    out.push_str(&format!(
        "docs_documented_item_count = {}\n",
        project.docs_documented_item_count
    ));
    if let Some(value) = &project.imports_index_path {
        out.push_str(&format!(
            "imports_index = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    out.push_str(&format!(
        "imports_library_count = {}\n",
        project.imports_library_count
    ));
    out.push_str(&format!(
        "imports_visible_library_count = {}\n",
        project.imports_visible_library_count
    ));
    out.push_str(&format!(
        "imports_visible_module_count = {}\n",
        project.imports_visible_module_count
    ));
    out.push_str(&format!(
        "imports_documented_visible_module_count = {}\n",
        project.imports_documented_visible_module_count
    ));
    out.push_str(&format!(
        "imports_documented_visible_item_count = {}\n",
        project.imports_documented_visible_item_count
    ));
    if let Some(value) = &project.galaxy_index_path {
        out.push_str(&format!(
            "galaxy_index = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    out.push_str(&format!("galaxy_count = {}\n", project.galaxy_count));
    out.push_str(&format!(
        "documented_galaxy_count = {}\n",
        project.galaxy_documented_count
    ));
    out.push_str(&format!(
        "documented_galaxy_library_module_count = {}\n",
        project.galaxy_documented_library_module_count
    ));
    out.push_str(&format!(
        "documented_galaxy_item_count = {}\n",
        project.galaxy_documented_item_count
    ));
    if let Some(value) = &project.links_index_path {
        out.push_str(&format!(
            "links_index = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    if let Some(value) = &project.packet_index_path {
        out.push_str(&format!(
            "packet_index = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    if let Some(value) = &project.host_ffi_index_path {
        out.push_str(&format!(
            "host_ffi_index = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    if let Some(value) = &project.abi_index_path {
        out.push_str(&format!("abi_index = \"{}\"\n", escape_toml_string(value)));
    }
}
