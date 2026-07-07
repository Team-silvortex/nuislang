use crate::aot_manifest_domain_model::BuildManifestExecutionContract;
use crate::aot_toml::{escape_toml_string, render_string_array};
use std::fmt::Write as _;

pub(crate) fn append_execution_contract_manifest_sections(
    out: &mut String,
    contracts: &[BuildManifestExecutionContract],
) {
    for contract in contracts {
        out.push('\n');
        out.push_str("[[execution_contract]]\n");
        writeln!(
            out,
            "package_id = \"{}\"",
            escape_toml_string(&contract.package_id)
        )
        .unwrap();
        writeln!(
            out,
            "domain_family = \"{}\"",
            escape_toml_string(&contract.domain_family)
        )
        .unwrap();
        writeln!(
            out,
            "skeleton_version = \"{}\"",
            escape_toml_string(&contract.execution.skeleton_version)
        )
        .unwrap();
        writeln!(
            out,
            "function_kind = \"{}\"",
            escape_toml_string(&contract.execution.function_kind)
        )
        .unwrap();
        writeln!(
            out,
            "graph_kind = \"{}\"",
            escape_toml_string(&contract.execution.graph_kind)
        )
        .unwrap();
        writeln!(
            out,
            "execution_domain = \"{}\"",
            escape_toml_string(&contract.execution.execution_domain)
        )
        .unwrap();
        writeln!(
            out,
            "default_time_mode = \"{}\"",
            escape_toml_string(&contract.execution.default_time_mode)
        )
        .unwrap();
        writeln!(
            out,
            "contract_family = \"{}\"",
            escape_toml_string(&contract.execution.contract_family)
        )
        .unwrap();
        writeln!(
            out,
            "lowering_targets = {}",
            render_string_array(&contract.execution.lowering_targets)
        )
        .unwrap();
    }
}
