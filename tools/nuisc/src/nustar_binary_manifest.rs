use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::registry::NustarPackageManifest;

pub fn validate_manifest_for_packaging(manifest: &NustarPackageManifest) -> Result<(), String> {
    if manifest.abi_profiles.is_empty() {
        return Err(format!(
            "nustar package `{}` must declare at least one ABI profile in `abi_profiles`",
            manifest.package_id
        ));
    }
    if manifest.abi_capabilities.is_empty() {
        return Err(format!(
            "nustar package `{}` must declare ABI capability mappings in `abi_capabilities`",
            manifest.package_id
        ));
    }

    let profile_set = manifest
        .abi_profiles
        .iter()
        .map(|value| value.trim().to_owned())
        .collect::<BTreeSet<_>>();
    if profile_set.len() != manifest.abi_profiles.len() {
        return Err(format!(
            "nustar package `{}` has duplicated ABI profile entries in `abi_profiles`",
            manifest.package_id
        ));
    }
    let capability_abi_set = manifest
        .abi_profiles
        .iter()
        .chain(manifest.host_ffi_abis.iter())
        .map(|value| value.trim().to_owned())
        .collect::<BTreeSet<_>>();

    for profile in &manifest.abi_profiles {
        crate::registry::validate_manifest_abi(manifest, profile)?;
        crate::registry::validate_abi_capabilities(manifest, profile, &[], &[])?;
        if manifest.domain_family == "cpu" {
            crate::registry::registered_abi_target(manifest, profile)?;
        }
    }

    let mut capabilities_by_abi = BTreeMap::<String, Vec<(String, String)>>::new();
    for raw in &manifest.abi_capabilities {
        let Some((abi, _)) = raw.split_once(':') else {
            return Err(format!(
                "nustar package `{}` has invalid abi_capabilities entry `{}`; expected `abi:kind:value[|kind:value...]`",
                manifest.package_id, raw
            ));
        };
        let abi = abi.trim();
        if !capability_abi_set.contains(abi) {
            return Err(format!(
                "nustar package `{}` has abi_capabilities entry `{}` referencing undeclared ABI or host FFI ABI `{}`",
                manifest.package_id, raw, abi
            ));
        }
        let caps = raw
            .split_once(':')
            .map(|(_, caps)| caps)
            .unwrap_or_default();
        for cap in caps.split('|').map(str::trim).filter(|cap| !cap.is_empty()) {
            if let Some(pattern) = cap.strip_prefix("surface:") {
                capabilities_by_abi
                    .entry(abi.to_owned())
                    .or_default()
                    .push(("surface".to_owned(), pattern.trim().to_owned()));
            } else if let Some(pattern) = cap.strip_prefix("op:") {
                capabilities_by_abi
                    .entry(abi.to_owned())
                    .or_default()
                    .push(("op".to_owned(), pattern.trim().to_owned()));
            }
        }
    }
    validate_domain_capability_policy(manifest, &capabilities_by_abi)?;
    Ok(())
}

fn validate_domain_capability_policy(
    manifest: &NustarPackageManifest,
    capabilities_by_abi: &BTreeMap<String, Vec<(String, String)>>,
) -> Result<(), String> {
    let op_prefix = format!("{}.", manifest.domain_family);
    for profile in &manifest.abi_profiles {
        let profile = profile.trim();
        let caps = capabilities_by_abi
            .get(profile)
            .cloned()
            .unwrap_or_default();
        let mut has_op_capability = false;
        let mut has_surface_capability = false;
        for (kind, pattern) in caps {
            if kind == "op" {
                has_op_capability = true;
                if !pattern.starts_with(&op_prefix) {
                    return Err(format!(
                        "nustar package `{}` ABI `{}` has cross-domain op capability pattern `{}`; expected prefix `{}`",
                        manifest.package_id, profile, pattern, op_prefix
                    ));
                }
            } else if kind == "surface" {
                has_surface_capability = true;
                validate_surface_pattern_for_domain(
                    &manifest.package_id,
                    &manifest.domain_family,
                    profile,
                    &pattern,
                )?;
            }
        }
        if !has_op_capability {
            return Err(format!(
                "nustar package `{}` ABI `{}` must declare at least one `op:` capability",
                manifest.package_id, profile
            ));
        }
        if !manifest.support_surface.is_empty()
            && manifest.domain_family != "cpu"
            && !has_surface_capability
        {
            return Err(format!(
                "nustar package `{}` ABI `{}` must declare at least one `surface:` capability for domain `{}`",
                manifest.package_id, profile, manifest.domain_family
            ));
        }
    }
    Ok(())
}

fn validate_surface_pattern_for_domain(
    package_id: &str,
    domain_family: &str,
    abi: &str,
    pattern: &str,
) -> Result<(), String> {
    let allowed = match domain_family {
        "cpu" => false,
        "data" => pattern.starts_with("data.profile."),
        "kernel" => pattern.starts_with("kernel.profile."),
        "shader" => pattern.starts_with("shader.profile.") || pattern == "shader.inline.wgsl.v1",
        other => pattern.starts_with(&format!("{other}.")),
    };
    if allowed {
        return Ok(());
    }
    Err(format!(
        "nustar package `{}` ABI `{}` has invalid surface capability pattern `{}` for domain `{}`",
        package_id, abi, pattern, domain_family
    ))
}
pub(super) fn render_manifest(manifest: &NustarPackageManifest) -> String {
    format!(
        "manifest_schema = \"{}\"\npackage_id = \"{}\"\ndomain_family = \"{}\"\nfrontend = \"{}\"\nentry_crate = \"{}\"\nast_entry = \"{}\"\nnir_entry = \"{}\"\nyir_lowering_entry = \"{}\"\npart_verify_entry = \"{}\"\nast_surface = {}\nnir_surface = {}\nyir_lowering = {}\npart_verify = {}\nbinary_extension = \"{}\"\npackage_layout = \"{}\"\nmachine_abi_policy = \"{}\"\nabi_profiles = {}\nabi_capabilities = {}\nabi_targets = {}\nimplementation_kinds = {}\nloader_entry = \"{}\"\nloader_abi = \"{}\"\nhost_ffi_surface = {}\nhost_ffi_abis = {}\nhost_ffi_bridge = \"{}\"\nsupport_surface = {}\nsupport_profile_slots = {}\ncapability_tags = {}\ndefault_lanes = {}\nclock_domain_id = \"{}\"\nclock_kind = \"{}\"\nclock_epoch_kind = \"{}\"\nclock_resolution = \"{}\"\nclock_bridge_default = \"{}\"\nprofiles = {}\nresource_families = {}\nunit_types = {}\nlowering_targets = {}\nops = {}\n",
        manifest.manifest_schema,
        manifest.package_id,
        manifest.domain_family,
        manifest.frontend,
        manifest.entry_crate,
        manifest.ast_entry,
        manifest.nir_entry,
        manifest.yir_lowering_entry,
        manifest.part_verify_entry,
        render_array(&manifest.ast_surface),
        render_array(&manifest.nir_surface),
        render_array(&manifest.yir_lowering),
        render_array(&manifest.part_verify),
        manifest.binary_extension,
        manifest.package_layout,
        manifest.machine_abi_policy,
        render_array(&manifest.abi_profiles),
        render_array(&manifest.abi_capabilities),
        render_array(&manifest.abi_targets),
        render_array(&manifest.implementation_kinds),
        manifest.loader_entry,
        manifest.loader_abi,
        render_array(&manifest.host_ffi_surface),
        render_array(&manifest.host_ffi_abis),
        manifest.host_ffi_bridge,
        render_array(&manifest.support_surface),
        render_array(&manifest.support_profile_slots),
        render_array(&manifest.capability_tags),
        render_array(&manifest.default_lanes),
        manifest.clock_domain_id,
        manifest.clock_kind,
        manifest.clock_epoch_kind,
        manifest.clock_resolution,
        manifest.clock_bridge_default,
        render_array(&manifest.profiles),
        render_array(&manifest.resource_families),
        render_array(&manifest.unit_types),
        render_array(&manifest.lowering_targets),
        render_array(&manifest.ops),
    )
}

fn render_array(values: &[String]) -> String {
    let quoted = values
        .iter()
        .map(|value| format!("\"{}\"", value))
        .collect::<Vec<_>>();
    format!("[{}]", quoted.join(", "))
}
fn infer_default_capability_tags(domain_family: &str) -> Vec<String> {
    match domain_family {
        "cpu" => vec!["host-execution", "native-llvm", "memory-runtime"],
        "data" => vec!["fabric-plane", "packet-layout", "cross-domain-marker"],
        "shader" => vec!["gpu-render", "frame-graph", "shader-ir"],
        "kernel" => vec!["accelerator-compute", "tensor-kernel", "device-dispatch"],
        "network" => vec!["io-reactor", "socket-transport", "packet-exchange"],
        _ => vec!["custom-domain"],
    }
    .into_iter()
    .map(str::to_owned)
    .collect()
}

pub(super) fn parse_manifest_text(
    source: &str,
    path: &Path,
) -> Result<NustarPackageManifest, String> {
    let domain_family = parse_required_string(source, "domain_family", path)?;
    Ok(NustarPackageManifest {
        manifest_schema: parse_required_string(source, "manifest_schema", path)?,
        package_id: parse_required_string(source, "package_id", path)?,
        domain_family: domain_family.clone(),
        frontend: parse_required_string(source, "frontend", path)?,
        entry_crate: parse_required_string(source, "entry_crate", path)?,
        ast_entry: parse_required_string(source, "ast_entry", path)?,
        nir_entry: parse_required_string(source, "nir_entry", path)?,
        yir_lowering_entry: parse_required_string(source, "yir_lowering_entry", path)?,
        part_verify_entry: parse_required_string(source, "part_verify_entry", path)?,
        ast_surface: parse_string_array(source, "ast_surface", path)?,
        nir_surface: parse_string_array(source, "nir_surface", path)?,
        yir_lowering: parse_string_array(source, "yir_lowering", path)?,
        part_verify: parse_string_array(source, "part_verify", path)?,
        binary_extension: parse_required_string(source, "binary_extension", path)?,
        package_layout: parse_required_string(source, "package_layout", path)?,
        machine_abi_policy: parse_required_string(source, "machine_abi_policy", path)?,
        abi_profiles: parse_optional_string_array(source, "abi_profiles").unwrap_or_default(),
        abi_capabilities: parse_optional_string_array(source, "abi_capabilities")
            .unwrap_or_default(),
        abi_targets: parse_optional_string_array(source, "abi_targets").unwrap_or_default(),
        implementation_kinds: parse_string_array(source, "implementation_kinds", path)?,
        loader_entry: parse_required_string(source, "loader_entry", path)?,
        loader_abi: parse_required_string(source, "loader_abi", path)?,
        host_ffi_surface: parse_string_array(source, "host_ffi_surface", path)?,
        host_ffi_abis: parse_string_array(source, "host_ffi_abis", path)?,
        host_ffi_bridge: parse_required_string(source, "host_ffi_bridge", path)?,
        bridge_lane_policy: parse_optional_string(source, "bridge_lane_policy"),
        bridge_surface: parse_optional_string(source, "bridge_surface"),
        bridge_emission_kind: parse_optional_string(source, "bridge_emission_kind"),
        bridge_entry: parse_optional_string(source, "bridge_entry"),
        bridge_kind: parse_optional_string(source, "bridge_kind"),
        bridge_scheduler_binding: parse_optional_string(source, "bridge_scheduler_binding"),
        backend_stub_kind: parse_optional_string(source, "backend_stub_kind"),
        backend_submission_mode: parse_optional_string(source, "backend_submission_mode"),
        backend_wake_policy: parse_optional_string(source, "backend_wake_policy"),
        backend_transport_model: parse_optional_string(source, "backend_transport_model"),
        backend_request_shape: parse_optional_string(source, "backend_request_shape"),
        backend_response_shape: parse_optional_string(source, "backend_response_shape"),
        backend_dispatch_shape: parse_optional_string(source, "backend_dispatch_shape"),
        backend_memory_binding: parse_optional_string(source, "backend_memory_binding"),
        backend_resource_binding: parse_optional_string(source, "backend_resource_binding"),
        backend_completion_model: parse_optional_string(source, "backend_completion_model"),
        phase_bind: parse_optional_string(source, "phase_bind"),
        phase_submit: parse_optional_string(source, "phase_submit"),
        phase_wait: parse_optional_string(source, "phase_wait"),
        phase_finalize: parse_optional_string(source, "phase_finalize"),
        host_bridge_host_ffi_surface: None,
        host_bridge_handle_family: None,
        host_bridge_phase_order: None,
        host_bridge_phase_bind_inputs: None,
        host_bridge_phase_bind_outputs: None,
        host_bridge_phase_submit_inputs: None,
        host_bridge_phase_submit_outputs: None,
        host_bridge_phase_wait_inputs: None,
        host_bridge_phase_wait_outputs: None,
        host_bridge_phase_finalize_inputs: None,
        host_bridge_phase_finalize_outputs: None,
        host_bridge_phase_bind_wake: None,
        host_bridge_phase_submit_wake: None,
        host_bridge_phase_wait_wake: None,
        host_bridge_phase_finalize_wake: None,
        host_bridge_plan_begin: None,
        host_bridge_plan_end: None,
        support_surface: parse_optional_string_array(source, "support_surface").unwrap_or_default(),
        support_profile_slots: parse_optional_string_array(source, "support_profile_slots")
            .unwrap_or_default(),
        capability_tags: parse_optional_string_array(source, "capability_tags")
            .unwrap_or_else(|| infer_default_capability_tags(&domain_family)),
        default_lanes: parse_optional_string_array(source, "default_lanes").unwrap_or_default(),
        clock_domain_id: parse_required_string(source, "clock_domain_id", path)?,
        clock_kind: parse_required_string(source, "clock_kind", path)?,
        clock_epoch_kind: parse_required_string(source, "clock_epoch_kind", path)?,
        clock_resolution: parse_required_string(source, "clock_resolution", path)?,
        clock_bridge_default: parse_required_string(source, "clock_bridge_default", path)?,
        profiles: parse_string_array(source, "profiles", path)?,
        resource_families: parse_string_array(source, "resource_families", path)?,
        unit_types: parse_string_array(source, "unit_types", path)?,
        lowering_targets: parse_string_array(source, "lowering_targets", path)?,
        ops: parse_string_array(source, "ops", path)?,
    })
}

fn parse_required_string(source: &str, key: &str, path: &Path) -> Result<String, String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let trimmed = rest.trim();
            if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
                return Ok(trimmed[1..trimmed.len() - 1].to_owned());
            }
            return Err(format!(
                "`{}` has invalid string value for `{key}`",
                path.display()
            ));
        }
    }
    Err(format!(
        "`{}` is missing required key `{key}`",
        path.display()
    ))
}

fn parse_string_array(source: &str, key: &str, path: &Path) -> Result<Vec<String>, String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let trimmed = rest.trim();
            if !(trimmed.starts_with('[') && trimmed.ends_with(']')) {
                return Err(format!(
                    "`{}` has invalid array value for `{key}`",
                    path.display()
                ));
            }
            let inner = &trimmed[1..trimmed.len() - 1];
            if inner.trim().is_empty() {
                return Ok(Vec::new());
            }
            let mut items = Vec::new();
            for part in inner.split(',') {
                let value = part.trim();
                if !(value.starts_with('"') && value.ends_with('"') && value.len() >= 2) {
                    return Err(format!(
                        "`{}` has invalid array item for `{key}`",
                        path.display()
                    ));
                }
                items.push(value[1..value.len() - 1].to_owned());
            }
            return Ok(items);
        }
    }
    Err(format!(
        "`{}` is missing required key `{key}`",
        path.display()
    ))
}

pub(super) fn parse_optional_string_array(source: &str, key: &str) -> Option<Vec<String>> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let trimmed = rest.trim();
            if !(trimmed.starts_with('[') && trimmed.ends_with(']')) {
                return None;
            }
            let inner = &trimmed[1..trimmed.len() - 1];
            if inner.trim().is_empty() {
                return Some(Vec::new());
            }
            let mut values = Vec::new();
            for part in split_quoted_array_items(inner)? {
                let item = part.trim();
                if !(item.starts_with('"') && item.ends_with('"') && item.len() >= 2) {
                    return None;
                }
                values.push(item[1..item.len() - 1].to_owned());
            }
            return Some(values);
        }
    }
    None
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

fn parse_optional_string(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let value = rest.trim();
            if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
                return Some(value[1..value.len() - 1].to_owned());
            }
            return None;
        }
    }
    None
}
