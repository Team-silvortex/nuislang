use std::{collections::BTreeSet, path::Path};

use crate::registry::{NustarPackageManifest, NustarRegistryIssue, NustarRegistryIssueKind};

fn parse_backend_family_from_abi_target(raw: &str) -> Option<String> {
    let (_, fields) = raw.split_once(':')?;
    for field in fields
        .split('|')
        .map(str::trim)
        .filter(|field| !field.is_empty())
    {
        let (key, value) = field.split_once('=')?;
        if key.trim() == "backend" {
            return Some(value.trim().to_owned());
        }
    }
    None
}

fn validate_shader_domain_contract(
    manifest: &NustarPackageManifest,
    manifest_path: &Path,
) -> Vec<NustarRegistryIssue> {
    let mut issues = Vec::new();
    let lowering = manifest
        .lowering_targets
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let backend_families = manifest
        .abi_targets
        .iter()
        .filter_map(|raw| parse_backend_family_from_abi_target(raw))
        .collect::<BTreeSet<_>>();
    let missing_lowering = backend_families
        .iter()
        .filter(|backend| !lowering.contains(*backend))
        .cloned()
        .collect::<Vec<_>>();
    if !missing_lowering.is_empty() {
        issues.push(NustarRegistryIssue {
            kind: NustarRegistryIssueKind::DomainContractMismatch,
            package: Some(manifest.package_id.clone()),
            domain: Some(manifest.domain_family.clone()),
            manifest_path: Some(manifest_path.display().to_string()),
            message: format!(
                "shader ABI backends must be represented in lowering_targets; missing: {}",
                missing_lowering.join(", ")
            ),
        });
    }
    if lowering.contains("metal")
        && !manifest
            .support_surface
            .iter()
            .any(|surface| surface == "shader.inline.wgsl.v1")
    {
        issues.push(NustarRegistryIssue {
            kind: NustarRegistryIssueKind::DomainContractMismatch,
            package: Some(manifest.package_id.clone()),
            domain: Some(manifest.domain_family.clone()),
            manifest_path: Some(manifest_path.display().to_string()),
            message:
                "shader lowering_targets containing `metal` must expose `shader.inline.wgsl.v1`"
                    .to_owned(),
        });
    }
    if !manifest
        .support_profile_slots
        .iter()
        .any(|slot| slot == "target")
        || !manifest
            .support_profile_slots
            .iter()
            .any(|slot| slot == "viewport")
        || !manifest
            .support_profile_slots
            .iter()
            .any(|slot| slot == "pipeline")
    {
        issues.push(NustarRegistryIssue {
            kind: NustarRegistryIssueKind::DomainContractMismatch,
            package: Some(manifest.package_id.clone()),
            domain: Some(manifest.domain_family.clone()),
            manifest_path: Some(manifest_path.display().to_string()),
            message: "shader domain must expose target/viewport/pipeline support_profile_slots"
                .to_owned(),
        });
    }
    issues
}

fn validate_kernel_domain_contract(
    manifest: &NustarPackageManifest,
    manifest_path: &Path,
) -> Vec<NustarRegistryIssue> {
    let mut issues = Vec::new();
    let lowering = manifest
        .lowering_targets
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let backend_families = manifest
        .abi_targets
        .iter()
        .filter_map(|raw| parse_backend_family_from_abi_target(raw))
        .collect::<BTreeSet<_>>();
    let missing_lowering = backend_families
        .iter()
        .filter(|backend| !lowering.contains(*backend))
        .cloned()
        .collect::<Vec<_>>();
    if !missing_lowering.is_empty() {
        issues.push(NustarRegistryIssue {
            kind: NustarRegistryIssueKind::DomainContractMismatch,
            package: Some(manifest.package_id.clone()),
            domain: Some(manifest.domain_family.clone()),
            manifest_path: Some(manifest_path.display().to_string()),
            message: format!(
                "kernel ABI backends must be represented in lowering_targets; missing: {}",
                missing_lowering.join(", ")
            ),
        });
    }
    for required_slot in ["bind_core", "queue_depth", "batch_lanes", "entry"] {
        if !manifest
            .support_profile_slots
            .iter()
            .any(|slot| slot == required_slot)
        {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::DomainContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "kernel domain must expose `{}` in support_profile_slots",
                    required_slot
                ),
            });
        }
    }
    if lowering.contains("coreml") || lowering.contains("ane") {
        let has_apple_resource = manifest
            .resource_families
            .iter()
            .any(|family| family.starts_with("kernel.apple") || family.starts_with("npu.apple"));
        if !has_apple_resource {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::DomainContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message:
                    "kernel lowering_targets containing `coreml` or `ane` must expose apple/npu resource families"
                        .to_owned(),
            });
        }
    }
    issues
}

fn validate_network_domain_contract(
    manifest: &NustarPackageManifest,
    manifest_path: &Path,
) -> Vec<NustarRegistryIssue> {
    let mut issues = Vec::new();
    let lowering = manifest
        .lowering_targets
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for required_target in ["socket-abi", "urlsession", "winsock"] {
        if !lowering.contains(required_target) {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::DomainContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "network domain must keep `{}` in lowering_targets",
                    required_target
                ),
            });
        }
    }
    for required_surface in [
        "network.profile.connect.v1",
        "network.profile.accept.v1",
        "network.profile.send.v1",
        "network.profile.recv.v1",
        "network.profile.close.v1",
        "network.profile.protocol.v1",
    ] {
        if !manifest
            .support_surface
            .iter()
            .any(|surface| surface == required_surface)
        {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::DomainContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "network domain must expose `{}` in support_surface",
                    required_surface
                ),
            });
        }
    }
    for required_slot in [
        "endpoint_kind",
        "transport_family",
        "protocol_kind",
        "protocol_version",
        "protocol_header_bytes",
    ] {
        if !manifest
            .support_profile_slots
            .iter()
            .any(|slot| slot == required_slot)
        {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::DomainContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "network domain must expose `{}` in support_profile_slots",
                    required_slot
                ),
            });
        }
    }
    for required_op in [
        "network.connect",
        "network.accept",
        "network.send",
        "network.recv",
        "network.close",
        "network.poll",
    ] {
        if !manifest.ops.iter().any(|op| op == required_op) {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::DomainContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!("network domain must expose `{}` in ops", required_op),
            });
        }
    }
    issues
}

pub(crate) fn validate_domain_specific_contracts(
    manifest: &NustarPackageManifest,
    manifest_path: &Path,
) -> Vec<NustarRegistryIssue> {
    match manifest.domain_family.as_str() {
        "shader" => validate_shader_domain_contract(manifest, manifest_path),
        "kernel" => validate_kernel_domain_contract(manifest, manifest_path),
        "network" => validate_network_domain_contract(manifest, manifest_path),
        _ => Vec::new(),
    }
}

pub(crate) fn validate_build_contract_fields(
    manifest: &NustarPackageManifest,
    manifest_path: &Path,
) -> Vec<NustarRegistryIssue> {
    let mut issues = Vec::new();
    let has_bridge_contract = manifest.bridge_lane_policy.is_some()
        || manifest.bridge_surface.is_some()
        || manifest.bridge_emission_kind.is_some()
        || manifest.bridge_entry.is_some()
        || manifest.bridge_kind.is_some()
        || manifest.bridge_scheduler_binding.is_some()
        || manifest.backend_stub_kind.is_some()
        || manifest.backend_submission_mode.is_some()
        || manifest.backend_wake_policy.is_some()
        || manifest.backend_transport_model.is_some()
        || manifest.backend_request_shape.is_some()
        || manifest.backend_response_shape.is_some()
        || manifest.backend_dispatch_shape.is_some()
        || manifest.backend_memory_binding.is_some()
        || manifest.backend_resource_binding.is_some()
        || manifest.backend_completion_model.is_some()
        || manifest.phase_bind.is_some()
        || manifest.phase_submit.is_some()
        || manifest.phase_wait.is_some()
        || manifest.phase_finalize.is_some();
    if has_bridge_contract {
        let mut missing = Vec::new();
        for (name, value) in [
            ("bridge_lane_policy", manifest.bridge_lane_policy.as_deref()),
            ("bridge_surface", manifest.bridge_surface.as_deref()),
            (
                "bridge_emission_kind",
                manifest.bridge_emission_kind.as_deref(),
            ),
            ("bridge_entry", manifest.bridge_entry.as_deref()),
            ("bridge_kind", manifest.bridge_kind.as_deref()),
            (
                "bridge_scheduler_binding",
                manifest.bridge_scheduler_binding.as_deref(),
            ),
            ("backend_stub_kind", manifest.backend_stub_kind.as_deref()),
            (
                "backend_submission_mode",
                manifest.backend_submission_mode.as_deref(),
            ),
            (
                "backend_wake_policy",
                manifest.backend_wake_policy.as_deref(),
            ),
            ("phase_bind", manifest.phase_bind.as_deref()),
            ("phase_submit", manifest.phase_submit.as_deref()),
            ("phase_wait", manifest.phase_wait.as_deref()),
            ("phase_finalize", manifest.phase_finalize.as_deref()),
        ] {
            if value.is_none() {
                missing.push(name.to_owned());
            }
        }
        if !missing.is_empty() {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::DomainContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "bridge/build contract must declare a complete minimum skeleton; missing: {}",
                    missing.join(", ")
                ),
            });
        }
    }

    let has_host_bridge_contract = manifest.host_bridge_host_ffi_surface.is_some()
        || manifest.host_bridge_handle_family.is_some()
        || manifest.host_bridge_phase_order.is_some()
        || manifest.host_bridge_phase_bind_inputs.is_some()
        || manifest.host_bridge_phase_bind_outputs.is_some()
        || manifest.host_bridge_phase_submit_inputs.is_some()
        || manifest.host_bridge_phase_submit_outputs.is_some()
        || manifest.host_bridge_phase_wait_inputs.is_some()
        || manifest.host_bridge_phase_wait_outputs.is_some()
        || manifest.host_bridge_phase_finalize_inputs.is_some()
        || manifest.host_bridge_phase_finalize_outputs.is_some()
        || manifest.host_bridge_phase_bind_wake.is_some()
        || manifest.host_bridge_phase_submit_wake.is_some()
        || manifest.host_bridge_phase_wait_wake.is_some()
        || manifest.host_bridge_phase_finalize_wake.is_some()
        || manifest.host_bridge_plan_begin.is_some()
        || manifest.host_bridge_plan_end.is_some();
    if has_host_bridge_contract {
        let mut missing = Vec::new();
        for (name, present) in [
            (
                "host_bridge_host_ffi_surface",
                manifest.host_bridge_host_ffi_surface.is_some(),
            ),
            (
                "host_bridge_handle_family",
                manifest.host_bridge_handle_family.is_some(),
            ),
            (
                "host_bridge_phase_order",
                manifest.host_bridge_phase_order.is_some(),
            ),
            (
                "host_bridge_phase_bind_inputs",
                manifest.host_bridge_phase_bind_inputs.is_some(),
            ),
            (
                "host_bridge_phase_bind_outputs",
                manifest.host_bridge_phase_bind_outputs.is_some(),
            ),
            (
                "host_bridge_phase_submit_inputs",
                manifest.host_bridge_phase_submit_inputs.is_some(),
            ),
            (
                "host_bridge_phase_submit_outputs",
                manifest.host_bridge_phase_submit_outputs.is_some(),
            ),
            (
                "host_bridge_phase_wait_inputs",
                manifest.host_bridge_phase_wait_inputs.is_some(),
            ),
            (
                "host_bridge_phase_wait_outputs",
                manifest.host_bridge_phase_wait_outputs.is_some(),
            ),
            (
                "host_bridge_phase_finalize_inputs",
                manifest.host_bridge_phase_finalize_inputs.is_some(),
            ),
            (
                "host_bridge_phase_finalize_outputs",
                manifest.host_bridge_phase_finalize_outputs.is_some(),
            ),
            (
                "host_bridge_phase_bind_wake",
                manifest.host_bridge_phase_bind_wake.is_some(),
            ),
            (
                "host_bridge_phase_submit_wake",
                manifest.host_bridge_phase_submit_wake.is_some(),
            ),
            (
                "host_bridge_phase_wait_wake",
                manifest.host_bridge_phase_wait_wake.is_some(),
            ),
            (
                "host_bridge_phase_finalize_wake",
                manifest.host_bridge_phase_finalize_wake.is_some(),
            ),
            (
                "host_bridge_plan_begin",
                manifest.host_bridge_plan_begin.is_some(),
            ),
            (
                "host_bridge_plan_end",
                manifest.host_bridge_plan_end.is_some(),
            ),
        ] {
            if !present {
                missing.push(name.to_owned());
            }
        }
        if !missing.is_empty() {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::DomainContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "host bridge contract must declare every phase field; missing: {}",
                    missing.join(", ")
                ),
            });
        } else {
            if manifest
                .host_bridge_host_ffi_surface
                .as_ref()
                .is_some_and(Vec::is_empty)
            {
                issues.push(NustarRegistryIssue {
                    kind: NustarRegistryIssueKind::DomainContractMismatch,
                    package: Some(manifest.package_id.clone()),
                    domain: Some(manifest.domain_family.clone()),
                    manifest_path: Some(manifest_path.display().to_string()),
                    message: "host bridge contract must expose at least one host_ffi surface"
                        .to_owned(),
                });
            }
            if manifest
                .host_bridge_handle_family
                .as_ref()
                .is_some_and(Vec::is_empty)
            {
                issues.push(NustarRegistryIssue {
                    kind: NustarRegistryIssueKind::DomainContractMismatch,
                    package: Some(manifest.package_id.clone()),
                    domain: Some(manifest.domain_family.clone()),
                    manifest_path: Some(manifest_path.display().to_string()),
                    message: "host bridge contract must expose at least one handle family"
                        .to_owned(),
                });
            }
            if manifest.host_bridge_phase_order.as_deref()
                != Some(
                    &[
                        "bind".to_owned(),
                        "submit".to_owned(),
                        "wait".to_owned(),
                        "finalize".to_owned(),
                    ][..],
                )
            {
                issues.push(NustarRegistryIssue {
                    kind: NustarRegistryIssueKind::DomainContractMismatch,
                    package: Some(manifest.package_id.clone()),
                    domain: Some(manifest.domain_family.clone()),
                    manifest_path: Some(manifest_path.display().to_string()),
                    message:
                        "host bridge phase_order must be [\"bind\", \"submit\", \"wait\", \"finalize\"]"
                            .to_owned(),
                });
            }
        }
    }

    issues
}
