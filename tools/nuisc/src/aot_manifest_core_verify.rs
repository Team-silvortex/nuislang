use std::{fs, path::Path};

use nuis_artifact::BuildManifestDomainBuildUnit;

use crate::aot_artifact::{
    parse_nuis_compiled_artifact, parse_nuis_executable_envelope, validate_artifact_binary_name,
};
use crate::aot_artifact_hash::{artifact_hash_fallback_bytes, parse_artifact_hash_blocks};
use crate::aot_encoding::fnv1a64_hex;
use crate::aot_manifest_path::validate_manifest_path_in_output_dir;
use crate::aot_toml::{
    parse_required_toml_string, parse_required_toml_string_array, parse_required_toml_usize,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ManifestCoreVerification {
    pub schema: String,
    pub input: String,
    pub output_dir: String,
    pub packaging_mode: String,
    pub envelope_path: String,
    pub envelope_schema: String,
    pub envelope_package_count: usize,
    pub artifact_path: String,
    pub artifact_schema: String,
    pub artifact_binary_name: String,
    pub artifact_binary_bytes: usize,
    pub lifecycle_schema: String,
    pub lifecycle_bootstrap_entry: String,
    pub lifecycle_tick_policy: String,
    pub lifecycle_shutdown_policy: String,
    pub lifecycle_yalivia_rpc: String,
    pub lifecycle_hook_surface: Vec<String>,
    pub lifecycle_export_surface: Vec<String>,
    pub lifecycle_runtime_capability_flags: Vec<String>,
}

pub(crate) fn verify_manifest_core(
    source: &str,
    path: &Path,
) -> Result<ManifestCoreVerification, String> {
    let schema = parse_required_toml_string(source, "manifest_schema", path)?;
    if schema != "nuis-build-manifest-v1" {
        return Err(format!(
            "`{}` has unsupported manifest schema `{}`; expected `nuis-build-manifest-v1`",
            path.display(),
            schema
        ));
    }
    let input = parse_required_toml_string(source, "input", path)?;
    let output_dir = parse_required_toml_string(source, "output_dir", path)?;
    let packaging_mode = parse_required_toml_string(source, "packaging_mode", path)?;
    let envelope_path = parse_required_toml_string(source, "path", path)?;
    let envelope_schema = parse_required_toml_string(source, "schema", path)?;
    if envelope_schema != "nuis-executable-envelope-v1" {
        return Err(format!(
            "`{}` has unsupported nuis envelope schema `{}`; expected `nuis-executable-envelope-v1`",
            path.display(),
            envelope_schema
        ));
    }
    let envelope_package_count = parse_required_toml_usize(source, "package_count", path)?;
    let artifact_path = parse_required_toml_string(source, "artifact_path", path)?;
    let artifact_schema = parse_required_toml_string(source, "artifact_schema", path)?;
    if artifact_schema != "nuis-compiled-artifact-v1" {
        return Err(format!(
            "`{}` has unsupported nuis artifact schema `{}`; expected `nuis-compiled-artifact-v1`",
            path.display(),
            artifact_schema
        ));
    }
    let artifact_binary_name = parse_required_toml_string(source, "artifact_binary_name", path)?;
    validate_artifact_binary_name("artifact_binary_name", &artifact_binary_name, path)?;
    validate_manifest_path_in_output_dir("nuis_envelope.path", &envelope_path, &output_dir, path)?;
    validate_manifest_path_in_output_dir(
        "nuis_artifact.artifact_path",
        &artifact_path,
        &output_dir,
        path,
    )?;
    let artifact_binary_bytes = parse_required_toml_usize(source, "artifact_binary_bytes", path)?;
    let lifecycle_schema = parse_required_toml_string(source, "lifecycle_schema", path)?;
    if lifecycle_schema != "nuis-lifecycle-contract-v1" {
        return Err(format!(
            "`{}` has unsupported lifecycle schema `{}`; expected `nuis-lifecycle-contract-v1`",
            path.display(),
            lifecycle_schema
        ));
    }
    let lifecycle_bootstrap_entry =
        parse_required_toml_string(source, "lifecycle_bootstrap_entry", path)?;
    let lifecycle_tick_policy = parse_required_toml_string(source, "lifecycle_tick_policy", path)?;
    let lifecycle_shutdown_policy =
        parse_required_toml_string(source, "lifecycle_shutdown_policy", path)?;
    let lifecycle_yalivia_rpc = parse_required_toml_string(source, "lifecycle_yalivia_rpc", path)?;
    let lifecycle_hook_surface =
        parse_required_toml_string_array(source, "lifecycle_hook_surface", path)?;
    let lifecycle_export_surface =
        parse_required_toml_string_array(source, "lifecycle_export_surface", path)?;
    let lifecycle_runtime_capability_flags =
        parse_required_toml_string_array(source, "lifecycle_runtime_capability_flags", path)?;
    let envelope_function_kind = parse_required_toml_string(source, "function_kind", path)?;
    if envelope_function_kind != "function-node" {
        return Err(format!(
            "`{}` has unsupported nuis envelope function_kind `{}`; expected `function-node`",
            path.display(),
            envelope_function_kind
        ));
    }
    let envelope_graph_kind = parse_required_toml_string(source, "graph_kind", path)?;
    if envelope_graph_kind != "function-graph" {
        return Err(format!(
            "`{}` has unsupported nuis envelope graph_kind `{}`; expected `function-graph`",
            path.display(),
            envelope_graph_kind
        ));
    }
    let envelope_time_mode = parse_required_toml_string(source, "default_time_mode", path)?;
    if envelope_time_mode.is_empty() {
        return Err(format!(
            "`{}` has empty nuis envelope default_time_mode",
            path.display()
        ));
    }

    Ok(ManifestCoreVerification {
        schema,
        input,
        output_dir,
        packaging_mode,
        envelope_path,
        envelope_schema,
        envelope_package_count,
        artifact_path,
        artifact_schema,
        artifact_binary_name,
        artifact_binary_bytes,
        lifecycle_schema,
        lifecycle_bootstrap_entry,
        lifecycle_tick_policy,
        lifecycle_shutdown_policy,
        lifecycle_yalivia_rpc,
        lifecycle_hook_surface,
        lifecycle_export_surface,
        lifecycle_runtime_capability_flags,
    })
}

pub(crate) fn verify_manifest_artifacts(
    source: &str,
    path: &Path,
    core: &ManifestCoreVerification,
    domain_build_units: &[BuildManifestDomainBuildUnit],
    bridge_registry_inline: Option<&str>,
    host_bridge_plan_index_inline: Option<&str>,
    lowering_plan_index_inline: Option<&str>,
) -> Result<usize, String> {
    let artifacts = parse_artifact_hash_blocks(source, path)?;
    if artifacts.is_empty() {
        return Err(format!(
            "`{}` does not contain any `[[artifact_hash]]` blocks",
            path.display()
        ));
    }
    for item in &artifacts {
        validate_manifest_path_in_output_dir(
            "artifact_hash.path",
            &item.path,
            &core.output_dir,
            path,
        )?;
    }

    let parsed_envelope = parse_nuis_executable_envelope(Path::new(&core.envelope_path))?;
    if parsed_envelope.schema != core.envelope_schema {
        return Err(format!(
            "`{}` nuis envelope schema mismatch between manifest and `{}`",
            path.display(),
            core.envelope_path
        ));
    }
    if parsed_envelope.package_count != core.envelope_package_count {
        return Err(format!(
            "`{}` nuis envelope package_count mismatch between manifest and `{}`",
            path.display(),
            core.envelope_path
        ));
    }
    if parsed_envelope.executable_kind != core.packaging_mode {
        return Err(format!(
            "`{}` nuis envelope executable_kind mismatch between manifest and `{}`",
            path.display(),
            core.envelope_path
        ));
    }
    let parsed_artifact = parse_nuis_compiled_artifact(Path::new(&core.artifact_path))?;
    if parsed_artifact.schema != core.artifact_schema {
        return Err(format!(
            "`{}` nuis artifact schema mismatch between manifest and `{}`",
            path.display(),
            core.artifact_path
        ));
    }
    if parsed_artifact.packaging_mode != core.packaging_mode {
        return Err(format!(
            "`{}` nuis artifact packaging_mode mismatch between manifest and `{}`",
            path.display(),
            core.artifact_path
        ));
    }
    if parsed_artifact.binary_name != core.artifact_binary_name {
        return Err(format!(
            "`{}` nuis artifact binary_name mismatch between manifest and `{}`",
            path.display(),
            core.artifact_path
        ));
    }
    if parsed_artifact.binary_bytes != core.artifact_binary_bytes {
        return Err(format!(
            "`{}` nuis artifact binary_bytes mismatch between manifest and `{}`",
            path.display(),
            core.artifact_path
        ));
    }
    if parsed_artifact.build_manifest_source != source {
        return Err(format!(
            "`{}` nuis artifact embedded build manifest does not match manifest source",
            path.display()
        ));
    }
    if parsed_artifact.envelope != parsed_envelope {
        return Err(format!(
            "`{}` nuis artifact envelope mismatch between manifest and `{}`",
            path.display(),
            core.artifact_path
        ));
    }
    if parsed_artifact.lifecycle.schema != core.lifecycle_schema {
        return Err(format!(
            "`{}` nuis artifact lifecycle schema mismatch: expected `{}`, found `{}`",
            path.display(),
            core.lifecycle_schema,
            parsed_artifact.lifecycle.schema
        ));
    }
    if parsed_artifact.lifecycle.bootstrap_entry != core.lifecycle_bootstrap_entry
        || parsed_artifact.lifecycle.tick_policy != core.lifecycle_tick_policy
        || parsed_artifact.lifecycle.shutdown_policy != core.lifecycle_shutdown_policy
        || parsed_artifact.lifecycle.yalivia_rpc != core.lifecycle_yalivia_rpc
        || parsed_artifact.lifecycle.hook_surface != core.lifecycle_hook_surface
        || parsed_artifact.lifecycle.export_surface != core.lifecycle_export_surface
        || parsed_artifact.lifecycle.runtime_capability_flags
            != core.lifecycle_runtime_capability_flags
    {
        return Err(format!(
            "`{}` nuis artifact lifecycle contract mismatch between manifest and `{}`",
            path.display(),
            core.artifact_path
        ));
    }

    for item in &artifacts {
        let bytes = match fs::read(&item.path) {
            Ok(bytes) => bytes,
            Err(_) => artifact_hash_fallback_bytes(
                &item.kind,
                domain_build_units,
                bridge_registry_inline,
                host_bridge_plan_index_inline,
                lowering_plan_index_inline,
            )
            .ok_or_else(|| format!("failed to read artifact `{}`", item.path))?,
        };
        if bytes.len() != item.bytes {
            return Err(format!(
                "artifact `{}` bytes mismatch for kind `{}`: manifest={}, actual={}",
                item.path,
                item.kind,
                item.bytes,
                bytes.len()
            ));
        }
        let actual_hash = fnv1a64_hex(&bytes);
        if actual_hash != item.fnv1a64 {
            return Err(format!(
                "artifact `{}` hash mismatch for kind `{}`: manifest={}, actual={}",
                item.path, item.kind, item.fnv1a64, actual_hash
            ));
        }
    }

    Ok(artifacts.len())
}
