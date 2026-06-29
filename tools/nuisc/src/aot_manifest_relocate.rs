use std::{fs, path::Path};

use nuis_artifact::{
    parse_domain_build_unit_blocks as shared_parse_domain_build_unit_blocks, NuisCompiledArtifact,
};

use crate::aot_domain_artifact_writer::write_domain_build_unit_stubs;
use crate::aot_domain_index_render::{
    append_relocated_bridge_registry_manifest_section,
    append_relocated_domain_lowering_plan_index_manifest_section,
    append_relocated_host_bridge_plan_index_manifest_section, write_domain_bridge_registry,
    write_domain_lowering_plan_index, write_host_bridge_plan_index,
};
use crate::aot_domain_unit_render::render_domain_build_unit_manifest_block;
use crate::aot_encoding::fnv1a64_hex;
use crate::aot_toml::{escape_toml_string, render_string_array};

pub fn render_relocated_unpacked_build_manifest(
    artifact: &NuisCompiledArtifact,
    output_dir: &Path,
    envelope_path: &Path,
    artifact_path: &Path,
    binary_path: &Path,
) -> Result<String, String> {
    let mut out = String::new();
    let source = &artifact.build_manifest_source;
    let mut domain_build_units =
        shared_parse_domain_build_unit_blocks(source, Path::new("<artifact>"))
            .map_err(|error| error.to_string())?;
    write_domain_build_unit_stubs(output_dir, &mut domain_build_units)?;
    let bridge_registry_path = write_domain_bridge_registry(output_dir, &domain_build_units)?;
    let host_bridge_plan_index_path =
        write_host_bridge_plan_index(output_dir, &domain_build_units)?;
    let lowering_plan_index_path =
        write_domain_lowering_plan_index(output_dir, &domain_build_units)?;
    let mut skip_section = false;
    let strip_project_path_keys = [
        "manifest_copy = ",
        "plan_index = ",
        "organization_index = ",
        "exchange_index = ",
        "modules_index = ",
        "links_index = ",
        "packet_index = ",
        "host_ffi_index = ",
        "abi_index = ",
    ];

    for raw in source.lines() {
        let line = raw.trim();
        if line == "[nuis_envelope]" || line == "[nuis_artifact]" || line == "[artifacts]" {
            skip_section = true;
            continue;
        }
        if line == "[bridge_registry]"
            || line == "[host_bridge_plan_index]"
            || line == "[domain_lowering_plan_index]"
        {
            skip_section = true;
            continue;
        }
        if line == "[[domain_build_unit]]" {
            skip_section = true;
            continue;
        }
        if line == "[[artifact_hash]]" {
            skip_section = true;
            continue;
        }
        if skip_section && line.starts_with('[') {
            skip_section = false;
        }
        if skip_section {
            continue;
        }
        if line.starts_with("output_dir = ") {
            out.push_str(&format!(
                "output_dir = \"{}\"\n",
                escape_toml_string(&output_dir.display().to_string())
            ));
            continue;
        }
        if strip_project_path_keys
            .iter()
            .any(|prefix| line.starts_with(prefix))
        {
            continue;
        }
        out.push_str(raw);
        out.push('\n');
    }

    if !out.ends_with('\n') {
        out.push('\n');
    }
    if !out.ends_with("\n\n") {
        out.push('\n');
    }

    for unit in &domain_build_units {
        out.push_str(&render_domain_build_unit_manifest_block(unit));
    }
    append_relocated_bridge_registry_manifest_section(
        &mut out,
        bridge_registry_path.as_deref(),
        &domain_build_units,
    );
    append_relocated_host_bridge_plan_index_manifest_section(
        &mut out,
        host_bridge_plan_index_path.as_deref(),
        &domain_build_units,
    );
    append_relocated_domain_lowering_plan_index_manifest_section(
        &mut out,
        lowering_plan_index_path.as_deref(),
        &domain_build_units,
    );

    out.push_str("[nuis_envelope]\n");
    out.push_str(&format!(
        "path = \"{}\"\n",
        escape_toml_string(&envelope_path.display().to_string())
    ));
    out.push_str(&format!(
        "schema = \"{}\"\n",
        escape_toml_string(&artifact.envelope.schema)
    ));
    out.push_str(&format!(
        "executable_kind = \"{}\"\n",
        escape_toml_string(&artifact.envelope.executable_kind)
    ));
    out.push_str(&format!(
        "package_count = {}\n",
        artifact.envelope.package_count
    ));
    out.push_str(&format!(
        "domain_families = {}\n",
        render_string_array(&artifact.envelope.domain_families)
    ));
    out.push_str(&format!(
        "contract_families = {}\n",
        render_string_array(&artifact.envelope.contract_families)
    ));
    out.push_str(&format!(
        "function_kind = \"{}\"\n",
        escape_toml_string(&artifact.envelope.function_kind)
    ));
    out.push_str(&format!(
        "graph_kind = \"{}\"\n",
        escape_toml_string(&artifact.envelope.graph_kind)
    ));
    out.push_str(&format!(
        "default_time_mode = \"{}\"\n",
        escape_toml_string(&artifact.envelope.default_time_mode)
    ));
    out.push('\n');

    out.push_str("[nuis_artifact]\n");
    out.push_str(&format!(
        "artifact_path = \"{}\"\n",
        escape_toml_string(&artifact_path.display().to_string())
    ));
    out.push_str(&format!(
        "artifact_schema = \"{}\"\n",
        escape_toml_string(&artifact.schema)
    ));
    out.push_str(&format!(
        "artifact_binary_name = \"{}\"\n",
        escape_toml_string(&artifact.binary_name)
    ));
    out.push_str(&format!(
        "artifact_binary_bytes = {}\n",
        artifact.binary_bytes
    ));
    out.push('\n');

    out.push_str("[artifacts]\n");
    out.push_str(&format!(
        "binary = \"{}\"\n",
        escape_toml_string(&binary_path.display().to_string())
    ));
    out.push_str(&format!(
        "envelope = \"{}\"\n",
        escape_toml_string(&envelope_path.display().to_string())
    ));
    out.push('\n');

    for (kind, path) in [("binary", binary_path), ("envelope", envelope_path)] {
        let bytes = fs::read(path).map_err(|error| {
            format!(
                "failed to read unpacked artifact `{}`: {error}",
                path.display()
            )
        })?;
        out.push_str("[[artifact_hash]]\n");
        out.push_str(&format!("kind = \"{}\"\n", escape_toml_string(kind)));
        out.push_str(&format!(
            "path = \"{}\"\n",
            escape_toml_string(&path.display().to_string())
        ));
        out.push_str(&format!("bytes = {}\n", bytes.len()));
        out.push_str(&format!("fnv1a64 = \"{}\"\n", fnv1a64_hex(&bytes)));
        out.push('\n');
    }

    Ok(out)
}
