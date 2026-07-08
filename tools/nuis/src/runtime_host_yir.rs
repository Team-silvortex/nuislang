use std::{fs, path::Path};

use crate::{
    json_bool_field, json_field, json_i64_field, json_optional_string_field, json_usize_field,
};

pub(crate) fn runtime_host_yir_json_fields(
    artifact_path: Option<&Path>,
    artifact_verified: bool,
) -> Vec<String> {
    if !artifact_verified {
        return unavailable(false, None, None, None);
    }
    let Some(path) = artifact_path else {
        return unavailable(false, None, None, None);
    };
    match summary(path) {
        Ok(Some((yir_path, summary))) => vec![
            json_bool_field("runtime_host_yir_attempted", true),
            json_bool_field("runtime_host_yir_ok", true),
            json_optional_string_field("runtime_host_yir_error", None),
            json_optional_string_field("runtime_host_yir_skip_reason", None),
            json_field("runtime_host_yir_path", &yir_path),
            json_usize_field("runtime_host_yir_nodes", summary.nodes_executed),
            json_usize_field(
                "runtime_host_yir_kernel_nodes",
                summary.kernel_nodes_executed,
            ),
            json_usize_field("runtime_host_yir_tensor_values", summary.tensor_values),
            json_usize_field("runtime_host_yir_scalar_values", summary.scalar_values),
            json_usize_field("runtime_host_yir_frame_values", summary.frame_values),
            json_i64_field(
                "runtime_host_yir_integer_checksum",
                summary.integer_checksum,
            ),
            json_i64_field(
                "runtime_host_yir_kernel_integer_checksum",
                summary.kernel_integer_checksum,
            ),
        ],
        Ok(None) => unavailable(
            false,
            None,
            Some("host_ffi_externs_present_or_no_yir"),
            None,
        ),
        Err(error) => unavailable(true, Some(&error), None, None),
    }
}

pub(crate) fn summary(
    artifact_path: &Path,
) -> Result<Option<(String, nuis_runtime::HostYirExecutionSummary)>, String> {
    let loaded = nuis_runtime::RuntimeLoader
        .load_from_artifact_path(artifact_path)
        .map_err(|error| error.to_string())?;
    let Some(yir_path) = loaded
        .manifest
        .artifact_hashes
        .iter()
        .find(|entry| entry.kind == "yir")
        .map(|entry| entry.path.clone())
    else {
        return Ok(None);
    };
    let source = fs::read_to_string(&yir_path)
        .map_err(|error| format!("failed to read host YIR `{yir_path}`: {error}"))?;
    if source
        .lines()
        .any(|line| line.starts_with("cpu.extern_call"))
    {
        return Ok(None);
    }
    let summary = nuis_runtime::execute_host_yir_source(&source)
        .map_err(|error| format!("failed to execute host YIR `{}`: {}", yir_path, error))?;
    Ok(Some((yir_path, summary)))
}

fn unavailable(
    attempted: bool,
    error: Option<&str>,
    skip_reason: Option<&str>,
    yir_path: Option<&str>,
) -> Vec<String> {
    vec![
        json_bool_field("runtime_host_yir_attempted", attempted),
        json_bool_field("runtime_host_yir_ok", false),
        json_optional_string_field("runtime_host_yir_error", error),
        json_optional_string_field("runtime_host_yir_skip_reason", skip_reason),
        json_optional_string_field("runtime_host_yir_path", yir_path),
        json_usize_field("runtime_host_yir_nodes", 0),
        json_usize_field("runtime_host_yir_kernel_nodes", 0),
        json_usize_field("runtime_host_yir_tensor_values", 0),
        json_usize_field("runtime_host_yir_scalar_values", 0),
        json_usize_field("runtime_host_yir_frame_values", 0),
        json_i64_field("runtime_host_yir_integer_checksum", 0),
        json_i64_field("runtime_host_yir_kernel_integer_checksum", 0),
    ]
}
