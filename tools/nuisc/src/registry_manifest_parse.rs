use std::path::{Path, PathBuf};

use crate::registry::{NustarPackageIndexEntry, NustarPackageManifest};

pub fn manifest_path(root: &Path, entry: &NustarPackageIndexEntry) -> PathBuf {
    root.join(&entry.manifest)
}

pub(crate) fn parse_index(
    source: &str,
    path: &Path,
) -> Result<Vec<NustarPackageIndexEntry>, String> {
    let mut entries = Vec::new();
    let mut current = Vec::<String>::new();

    for raw_line in source.lines() {
        let line = raw_line.trim();
        if line == "[[package]]" {
            if !current.is_empty() {
                entries.push(parse_index_entry(&current.join("\n"), path)?);
                current.clear();
            }
            continue;
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        current.push(line.to_owned());
    }

    if !current.is_empty() {
        entries.push(parse_index_entry(&current.join("\n"), path)?);
    }

    entries.sort_by(|lhs, rhs| lhs.package_id.cmp(&rhs.package_id));
    Ok(entries)
}

fn parse_index_entry(source: &str, path: &Path) -> Result<NustarPackageIndexEntry, String> {
    Ok(NustarPackageIndexEntry {
        package_id: parse_required_string(source, "package_id", path)?,
        manifest: parse_required_string(source, "manifest", path)?,
        domain_family: parse_required_string(source, "domain_family", path)?,
    })
}

pub(crate) fn parse_manifest(source: &str, path: &Path) -> Result<NustarPackageManifest, String> {
    let manifest_schema = parse_optional_string(source, "manifest_schema")
        .unwrap_or_else(|| "nustar-manifest-v1".to_owned());
    let package_id = parse_required_string(source, "package_id", path)?;
    let domain_family = parse_required_string(source, "domain_family", path)?;
    let frontend = parse_required_string(source, "frontend", path)?;
    let entry_crate = parse_required_string(source, "entry_crate", path)?;
    let ast_entry = parse_optional_string(source, "ast_entry")
        .unwrap_or_else(|| format!("{}.ast.bootstrap.v1", domain_family));
    let nir_entry = parse_optional_string(source, "nir_entry")
        .unwrap_or_else(|| format!("{}.nir.bootstrap.v1", domain_family));
    let yir_lowering_entry = parse_optional_string(source, "yir_lowering_entry")
        .unwrap_or_else(|| format!("{}.yir.lowering.v1", domain_family));
    let part_verify_entry = parse_optional_string(source, "part_verify_entry")
        .unwrap_or_else(|| format!("{}.verify.partial.v1", domain_family));
    let ast_surface = parse_optional_string_array(source, "ast_surface")
        .unwrap_or_else(|| vec![format!("{domain_family}.mod-ast.v1")]);
    let nir_surface = parse_optional_string_array(source, "nir_surface")
        .unwrap_or_else(|| vec![format!("nir.{domain_family}.surface.v1")]);
    let yir_lowering = parse_optional_string_array(source, "yir_lowering")
        .unwrap_or_else(|| vec![format!("yir.{domain_family}.lowering.v1")]);
    let part_verify = parse_optional_string_array(source, "part_verify")
        .unwrap_or_else(|| vec![format!("verify.{domain_family}.contract.v1")]);
    let binary_extension =
        parse_optional_string(source, "binary_extension").unwrap_or_else(|| "nustar".to_owned());
    let package_layout = parse_optional_string(source, "package_layout")
        .unwrap_or_else(|| "single-envelope".to_owned());
    let machine_abi_policy = parse_optional_string(source, "machine_abi_policy")
        .unwrap_or_else(|| "exact-match".to_owned());
    let abi_profiles = parse_optional_string_array(source, "abi_profiles").unwrap_or_default();
    let abi_capabilities =
        parse_optional_string_array(source, "abi_capabilities").unwrap_or_default();
    let abi_targets = parse_optional_string_array(source, "abi_targets").unwrap_or_default();
    let implementation_kinds = parse_optional_string_array(source, "implementation_kinds")
        .unwrap_or_else(|| vec!["native-stub".to_owned()]);
    let loader_entry = parse_optional_string(source, "loader_entry")
        .unwrap_or_else(|| "nustar.bootstrap.v1".to_owned());
    let loader_abi = parse_optional_string(source, "loader_abi")
        .unwrap_or_else(|| "nustar-loader-v1".to_owned());
    let host_ffi_surface =
        parse_optional_string_array(source, "host_ffi_surface").unwrap_or_default();
    let host_ffi_abis = parse_optional_string_array(source, "host_ffi_abis").unwrap_or_default();
    let host_ffi_bridge =
        parse_optional_string(source, "host_ffi_bridge").unwrap_or_else(|| "none".to_owned());
    let bridge_lane_policy = parse_optional_string(source, "bridge_lane_policy");
    let bridge_surface = parse_optional_string(source, "bridge_surface");
    let bridge_emission_kind = parse_optional_string(source, "bridge_emission_kind");
    let bridge_entry = parse_optional_string(source, "bridge_entry");
    let bridge_kind = parse_optional_string(source, "bridge_kind");
    let bridge_scheduler_binding = parse_optional_string(source, "bridge_scheduler_binding");
    let backend_stub_kind = parse_optional_string(source, "backend_stub_kind");
    let backend_submission_mode = parse_optional_string(source, "backend_submission_mode");
    let backend_wake_policy = parse_optional_string(source, "backend_wake_policy");
    let backend_transport_model = parse_optional_string(source, "backend_transport_model");
    let backend_request_shape = parse_optional_string(source, "backend_request_shape");
    let backend_response_shape = parse_optional_string(source, "backend_response_shape");
    let backend_dispatch_shape = parse_optional_string(source, "backend_dispatch_shape");
    let backend_memory_binding = parse_optional_string(source, "backend_memory_binding");
    let backend_resource_binding = parse_optional_string(source, "backend_resource_binding");
    let backend_completion_model = parse_optional_string(source, "backend_completion_model");
    let phase_bind = parse_optional_string(source, "phase_bind");
    let phase_submit = parse_optional_string(source, "phase_submit");
    let phase_wait = parse_optional_string(source, "phase_wait");
    let phase_finalize = parse_optional_string(source, "phase_finalize");
    let host_bridge_host_ffi_surface =
        parse_optional_string_array(source, "host_bridge_host_ffi_surface");
    let host_bridge_handle_family =
        parse_optional_string_array(source, "host_bridge_handle_family");
    let host_bridge_phase_order = parse_optional_string_array(source, "host_bridge_phase_order");
    let host_bridge_phase_bind_inputs =
        parse_optional_string_array(source, "host_bridge_phase_bind_inputs");
    let host_bridge_phase_bind_outputs =
        parse_optional_string_array(source, "host_bridge_phase_bind_outputs");
    let host_bridge_phase_submit_inputs =
        parse_optional_string_array(source, "host_bridge_phase_submit_inputs");
    let host_bridge_phase_submit_outputs =
        parse_optional_string_array(source, "host_bridge_phase_submit_outputs");
    let host_bridge_phase_wait_inputs =
        parse_optional_string_array(source, "host_bridge_phase_wait_inputs");
    let host_bridge_phase_wait_outputs =
        parse_optional_string_array(source, "host_bridge_phase_wait_outputs");
    let host_bridge_phase_finalize_inputs =
        parse_optional_string_array(source, "host_bridge_phase_finalize_inputs");
    let host_bridge_phase_finalize_outputs =
        parse_optional_string_array(source, "host_bridge_phase_finalize_outputs");
    let host_bridge_phase_bind_wake = parse_optional_string(source, "host_bridge_phase_bind_wake");
    let host_bridge_phase_submit_wake =
        parse_optional_string(source, "host_bridge_phase_submit_wake");
    let host_bridge_phase_wait_wake = parse_optional_string(source, "host_bridge_phase_wait_wake");
    let host_bridge_phase_finalize_wake =
        parse_optional_string(source, "host_bridge_phase_finalize_wake");
    let host_bridge_plan_begin = parse_optional_bool(source, "host_bridge_plan_begin");
    let host_bridge_plan_end = parse_optional_bool(source, "host_bridge_plan_end");
    let support_surface =
        parse_optional_string_array(source, "support_surface").unwrap_or_default();
    let support_profile_slots =
        parse_optional_string_array(source, "support_profile_slots").unwrap_or_default();
    let capability_tags = parse_optional_string_array(source, "capability_tags")
        .unwrap_or_else(|| infer_default_capability_tags(&domain_family));
    let default_lanes = parse_optional_string_array(source, "default_lanes").unwrap_or_default();
    let clock_domain_id = parse_optional_string(source, "clock_domain_id")
        .unwrap_or_else(|| format!("{domain_family}.clock.local.v1"));
    let clock_kind =
        parse_optional_string(source, "clock_kind").unwrap_or_else(|| "local-monotonic".to_owned());
    let clock_epoch_kind = parse_optional_string(source, "clock_epoch_kind")
        .unwrap_or_else(|| "domain-epoch".to_owned());
    let clock_resolution =
        parse_optional_string(source, "clock_resolution").unwrap_or_else(|| "tick:1ns".to_owned());
    let clock_bridge_default =
        parse_optional_string(source, "clock_bridge_default").unwrap_or_else(|| "self".to_owned());
    let profiles = parse_string_array(source, "profiles", path)?;
    let resource_families = parse_string_array(source, "resource_families", path)?;
    let unit_types = parse_optional_string_array(source, "unit_types").unwrap_or_default();
    let lowering_targets = parse_string_array(source, "lowering_targets", path)?;
    let ops = parse_string_array(source, "ops", path)?;

    Ok(NustarPackageManifest {
        manifest_schema,
        package_id,
        domain_family,
        frontend,
        entry_crate,
        ast_entry,
        nir_entry,
        yir_lowering_entry,
        part_verify_entry,
        ast_surface,
        nir_surface,
        yir_lowering,
        part_verify,
        binary_extension,
        package_layout,
        machine_abi_policy,
        abi_profiles,
        abi_capabilities,
        abi_targets,
        implementation_kinds,
        loader_entry,
        loader_abi,
        host_ffi_surface,
        host_ffi_abis,
        host_ffi_bridge,
        bridge_lane_policy,
        bridge_surface,
        bridge_emission_kind,
        bridge_entry,
        bridge_kind,
        bridge_scheduler_binding,
        backend_stub_kind,
        backend_submission_mode,
        backend_wake_policy,
        backend_transport_model,
        backend_request_shape,
        backend_response_shape,
        backend_dispatch_shape,
        backend_memory_binding,
        backend_resource_binding,
        backend_completion_model,
        phase_bind,
        phase_submit,
        phase_wait,
        phase_finalize,
        host_bridge_host_ffi_surface,
        host_bridge_handle_family,
        host_bridge_phase_order,
        host_bridge_phase_bind_inputs,
        host_bridge_phase_bind_outputs,
        host_bridge_phase_submit_inputs,
        host_bridge_phase_submit_outputs,
        host_bridge_phase_wait_inputs,
        host_bridge_phase_wait_outputs,
        host_bridge_phase_finalize_inputs,
        host_bridge_phase_finalize_outputs,
        host_bridge_phase_bind_wake,
        host_bridge_phase_submit_wake,
        host_bridge_phase_wait_wake,
        host_bridge_phase_finalize_wake,
        host_bridge_plan_begin,
        host_bridge_plan_end,
        support_surface,
        support_profile_slots,
        capability_tags,
        default_lanes,
        clock_domain_id,
        clock_kind,
        clock_epoch_kind,
        clock_resolution,
        clock_bridge_default,
        profiles,
        resource_families,
        unit_types,
        lowering_targets,
        ops,
    })
}

fn infer_default_capability_tags(domain_family: &str) -> Vec<String> {
    match domain_family {
        "cpu" => vec![
            "host-execution",
            "native-llvm",
            "memory-runtime",
            "syscall-surface",
        ],
        "data" => vec![
            "fabric-plane",
            "packet-layout",
            "cross-domain-marker",
            "window-transfer",
        ],
        "shader" => vec!["gpu-render", "frame-graph", "shader-ir", "resource-binding"],
        "kernel" => vec![
            "accelerator-compute",
            "tensor-kernel",
            "device-dispatch",
            "buffer-table",
        ],
        "network" => vec![
            "io-reactor",
            "socket-transport",
            "packet-exchange",
            "async-bridge",
        ],
        _ => vec!["custom-domain"],
    }
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn parse_optional_string(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return parse_quoted(rest);
        }
    }
    None
}

fn parse_optional_bool(source: &str, key: &str) -> Option<bool> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return match rest.trim() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            };
        }
    }
    None
}

fn parse_required_string(source: &str, key: &str, path: &Path) -> Result<String, String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return parse_quoted(rest).ok_or_else(|| {
                format!(
                    "manifest `{}` has invalid string value for `{key}`",
                    path.display()
                )
            });
        }
    }

    Err(format!(
        "manifest `{}` is missing required key `{key}`",
        path.display()
    ))
}

fn parse_string_array(source: &str, key: &str, path: &Path) -> Result<Vec<String>, String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return parse_array(rest).ok_or_else(|| {
                format!(
                    "manifest `{}` has invalid array value for `{key}`",
                    path.display()
                )
            });
        }
    }

    Err(format!(
        "manifest `{}` is missing required key `{key}`",
        path.display()
    ))
}

#[cfg(test)]
pub(crate) fn parse_optional_string_array(source: &str, key: &str) -> Option<Vec<String>> {
    parse_optional_string_array_impl(source, key)
}

#[cfg(not(test))]
fn parse_optional_string_array(source: &str, key: &str) -> Option<Vec<String>> {
    parse_optional_string_array_impl(source, key)
}

fn parse_optional_string_array_impl(source: &str, key: &str) -> Option<Vec<String>> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return parse_array(rest);
        }
    }
    None
}

fn parse_quoted(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        Some(trimmed[1..trimmed.len() - 1].to_owned())
    } else {
        None
    }
}

fn parse_array(raw: &str) -> Option<Vec<String>> {
    let trimmed = raw.trim();
    if !(trimmed.starts_with('[') && trimmed.ends_with(']')) {
        return None;
    }

    let inner = &trimmed[1..trimmed.len() - 1];
    if inner.trim().is_empty() {
        return Some(Vec::new());
    }

    let mut items = Vec::new();
    for part in split_quoted_array_items(inner)? {
        items.push(parse_quoted(part.trim())?);
    }
    Some(items)
}

fn split_quoted_array_items(inner: &str) -> Option<Vec<&str>> {
    let mut items = Vec::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut start = 0;
    for (index, ch) in inner.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            ',' if !in_string => {
                items.push(&inner[start..index]);
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    if in_string || escaped {
        return None;
    }
    items.push(&inner[start..]);
    Some(items)
}
