use crate::aot_manifest_domain_model::BuildManifestExecutionContract;
use crate::aot_toml::{escape_toml_string, render_string_array};

pub(crate) fn append_execution_contract_manifest_sections(
    out: &mut String,
    contracts: &[BuildManifestExecutionContract],
) {
    for contract in contracts {
        out.push('\n');
        out.push_str("[[execution_contract]]\n");
        out.push_str(&format!(
            "package_id = \"{}\"\n",
            escape_toml_string(&contract.package_id)
        ));
        out.push_str(&format!(
            "domain_family = \"{}\"\n",
            escape_toml_string(&contract.domain_family)
        ));
        out.push_str(&format!(
            "skeleton_version = \"{}\"\n",
            escape_toml_string(&contract.execution.skeleton_version)
        ));
        out.push_str(&format!(
            "function_kind = \"{}\"\n",
            escape_toml_string(&contract.execution.function_kind)
        ));
        out.push_str(&format!(
            "graph_kind = \"{}\"\n",
            escape_toml_string(&contract.execution.graph_kind)
        ));
        out.push_str(&format!(
            "execution_domain = \"{}\"\n",
            escape_toml_string(&contract.execution.execution_domain)
        ));
        out.push_str(&format!(
            "default_time_mode = \"{}\"\n",
            escape_toml_string(&contract.execution.default_time_mode)
        ));
        out.push_str(&format!(
            "contract_family = \"{}\"\n",
            escape_toml_string(&contract.execution.contract_family)
        ));
        out.push_str(&format!(
            "lowering_targets = {}\n",
            render_string_array(&contract.execution.lowering_targets)
        ));
    }
}
