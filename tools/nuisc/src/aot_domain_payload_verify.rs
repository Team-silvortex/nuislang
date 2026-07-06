use std::{fs, path::Path};

use nuis_artifact::BuildManifestDomainBuildUnit;

use crate::aot_domain_payload_blob::decode_domain_build_unit_payload_blob;
use crate::aot_domain_render::{
    render_domain_build_unit_backend_stub, render_domain_build_unit_bridge_plan,
    render_domain_build_unit_host_bridge_stub, render_domain_build_unit_lowering_plan,
};
use crate::aot_encoding::hex_decode_bytes;
use crate::aot_kernel_sidecar::render_domain_build_unit_kernel_ir_sidecar;
use crate::aot_network_sidecar::render_domain_build_unit_network_ir_sidecar;
use crate::aot_shader_sidecar::render_domain_build_unit_shader_ir_sidecar;

pub(crate) struct DomainPayloadVerifyReport {
    pub domain_payload_blobs_checked: usize,
    pub domain_payload_blob_sections_checked: usize,
    pub domain_payload_contract_sections_checked: usize,
    pub domain_payload_lowering_plans_checked: usize,
    pub domain_payload_backend_stubs_checked: usize,
    pub domain_payload_bridge_plans_checked: usize,
    pub domain_bridge_stubs_checked: usize,
}

pub(crate) fn verify_domain_payload_blobs(
    manifest_path: &Path,
    domain_build_units: &[BuildManifestDomainBuildUnit],
) -> Result<DomainPayloadVerifyReport, String> {
    let mut report = DomainPayloadVerifyReport {
        domain_payload_blobs_checked: 0,
        domain_payload_blob_sections_checked: 0,
        domain_payload_contract_sections_checked: 0,
        domain_payload_lowering_plans_checked: 0,
        domain_payload_backend_stubs_checked: 0,
        domain_payload_bridge_plans_checked: 0,
        domain_bridge_stubs_checked: 0,
    };
    for unit in domain_build_units {
        if unit.domain_family == "cpu" {
            if unit.artifact_payload_blob_path.is_some()
                || unit.artifact_payload_blob_bytes.is_some()
                || unit.artifact_payload_format.is_some()
                || unit.artifact_payload_blob_inline.is_some()
            {
                return Err(format!(
                    "`{}` cpu domain_build_unit must not declare hetero payload blob fields",
                    manifest_path.display()
                ));
            }
            continue;
        }
        verify_domain_payload_blob(manifest_path, unit, &mut report)?;
    }
    Ok(report)
}

fn verify_domain_payload_blob(
    manifest_path: &Path,
    unit: &BuildManifestDomainBuildUnit,
    report: &mut DomainPayloadVerifyReport,
) -> Result<(), String> {
    let blob_path = unit.artifact_payload_blob_path.as_ref().ok_or_else(|| {
        format!(
            "`{}` domain_build_unit `{}` is missing `artifact_payload_blob_path`",
            manifest_path.display(),
            unit.domain_family
        )
    })?;
    let blob_bytes_declared = unit.artifact_payload_blob_bytes.ok_or_else(|| {
        format!(
            "`{}` domain_build_unit `{}` is missing `artifact_payload_blob_bytes`",
            manifest_path.display(),
            unit.domain_family
        )
    })?;
    let blob_format = unit.artifact_payload_format.as_deref().ok_or_else(|| {
        format!(
            "`{}` domain_build_unit `{}` is missing `artifact_payload_format`",
            manifest_path.display(),
            unit.domain_family
        )
    })?;
    if blob_format != "ndpb-v2" {
        return Err(format!(
            "`{}` domain_build_unit `{}` has unsupported artifact_payload_format `{}`; expected `ndpb-v2`",
            manifest_path.display(),
            unit.domain_family,
            blob_format
        ));
    }
    let (blob, blob_label) = match fs::read(blob_path) {
        Ok(blob) => (blob, blob_path.clone()),
        Err(_) => {
            let inline = unit.artifact_payload_blob_inline.as_ref().ok_or_else(|| {
                format!(
                    "failed to read domain payload blob `{}` referenced by `{}` and no `artifact_payload_blob_inline` fallback is available",
                    blob_path,
                    manifest_path.display()
                )
            })?;
            (
                hex_decode_bytes(inline).map_err(|error| {
                    format!(
                        "invalid `artifact_payload_blob_inline` for domain `{}` in `{}`: {error}",
                        unit.domain_family,
                        manifest_path.display()
                    )
                })?,
                format!("<embedded-domain-payload-blob:{}>", unit.domain_family),
            )
        }
    };
    if blob.len() != blob_bytes_declared {
        return Err(format!(
            "domain payload blob `{}` byte length mismatch for `{}`: manifest={}, actual={}",
            blob_label,
            unit.domain_family,
            blob_bytes_declared,
            blob.len()
        ));
    }
    let decoded_blob = decode_domain_build_unit_payload_blob(&blob)
        .map_err(|error| format!("invalid domain payload blob `{}`: {error}", blob_label))?;
    verify_payload_blob_header(&blob_label, unit, &decoded_blob)?;
    let payload_path = unit.artifact_payload_path.as_ref().ok_or_else(|| {
        format!(
            "`{}` domain_build_unit `{}` is missing `artifact_payload_path`",
            manifest_path.display(),
            unit.domain_family
        )
    })?;
    let bridge_stub_path = unit.artifact_bridge_stub_path.as_ref().ok_or_else(|| {
        format!(
            "`{}` domain_build_unit `{}` is missing `artifact_bridge_stub_path`",
            manifest_path.display(),
            unit.domain_family
        )
    })?;
    let expected_section_count = if unit.domain_family == "shader"
        || unit.domain_family == "kernel"
        || unit.domain_family == "network"
    {
        5
    } else {
        4
    };
    if decoded_blob.sections.len() != expected_section_count {
        return Err(format!(
            "domain payload blob `{}` section count mismatch: expected {}, found {}",
            blob_label,
            expected_section_count,
            decoded_blob.sections.len()
        ));
    }
    verify_payload_sections(
        &blob_label,
        unit,
        payload_path,
        bridge_stub_path,
        &decoded_blob,
        report,
    )
}

fn verify_payload_blob_header(
    blob_label: &str,
    unit: &BuildManifestDomainBuildUnit,
    decoded_blob: &nuis_artifact::DomainBuildUnitPayloadBlob,
) -> Result<(), String> {
    if decoded_blob.domain_family != unit.domain_family {
        return Err(format!(
            "domain payload blob `{}` domain mismatch: manifest={}, blob={}",
            blob_label, unit.domain_family, decoded_blob.domain_family
        ));
    }
    if decoded_blob.package_id != unit.package_id {
        return Err(format!(
            "domain payload blob `{}` package mismatch: manifest={}, blob={}",
            blob_label, unit.package_id, decoded_blob.package_id
        ));
    }
    if decoded_blob.backend_family != unit.backend_family {
        return Err(format!(
            "domain payload blob `{}` backend_family mismatch for `{}`",
            blob_label, unit.domain_family
        ));
    }
    if decoded_blob.target_device != unit.target_device {
        return Err(format!(
            "domain payload blob `{}` target_device mismatch for `{}`",
            blob_label, unit.domain_family
        ));
    }
    if decoded_blob.ir_format != unit.ir_format {
        return Err(format!(
            "domain payload blob `{}` ir_format mismatch for `{}`",
            blob_label, unit.domain_family
        ));
    }
    if decoded_blob.dispatch_abi != unit.dispatch_abi {
        return Err(format!(
            "domain payload blob `{}` dispatch_abi mismatch for `{}`",
            blob_label, unit.domain_family
        ));
    }
    if decoded_blob.backend_priority != unit.backend_priority {
        return Err(format!(
            "domain payload blob `{}` backend_priority mismatch for `{}`",
            blob_label, unit.domain_family
        ));
    }
    if decoded_blob.verification != unit.verification {
        return Err(format!(
            "domain payload blob `{}` verification mismatch for `{}`",
            blob_label, unit.domain_family
        ));
    }
    if decoded_blob.selected_lowering_target != unit.selected_lowering_target {
        return Err(format!(
            "domain payload blob `{}` selected_lowering_target mismatch for `{}`",
            blob_label, unit.domain_family
        ));
    }
    if decoded_blob.contract_family != unit.contract_family {
        return Err(format!(
            "domain payload blob `{}` contract_family mismatch: manifest={}, blob={}",
            blob_label, unit.contract_family, decoded_blob.contract_family
        ));
    }
    if decoded_blob.packaging_role != unit.packaging_role {
        return Err(format!(
            "domain payload blob `{}` packaging_role mismatch: manifest={}, blob={}",
            blob_label, unit.packaging_role, decoded_blob.packaging_role
        ));
    }
    if decoded_blob.payload_kind != "contract-sidecar" {
        return Err(format!(
            "domain payload blob `{}` payload_kind mismatch: expected `contract-sidecar`, found `{}`",
            blob_label,
            decoded_blob.payload_kind
        ));
    }
    if decoded_blob.payload_format != "toml" {
        return Err(format!(
            "domain payload blob `{}` payload_format mismatch: expected `toml`, found `{}`",
            blob_label, decoded_blob.payload_format
        ));
    }
    Ok(())
}

fn verify_payload_sections(
    blob_label: &str,
    unit: &BuildManifestDomainBuildUnit,
    payload_path: &str,
    bridge_stub_path: &str,
    decoded_blob: &nuis_artifact::DomainBuildUnitPayloadBlob,
    report: &mut DomainPayloadVerifyReport,
) -> Result<(), String> {
    let contract_section = &decoded_blob.sections[0];
    if contract_section.name != "contract_toml" {
        return Err(format!(
            "domain payload blob `{}` section name mismatch: expected `contract_toml`, found `{}`",
            blob_label, contract_section.name
        ));
    }
    let payload = fs::read(payload_path).unwrap_or_else(|_| contract_section.bytes.clone());
    if contract_section.bytes != payload {
        return Err(format!(
            "domain payload blob `{}` payload content mismatch against `{}`",
            blob_label, payload_path
        ));
    }
    report.domain_payload_contract_sections_checked += 1;
    verify_named_section(
        blob_label,
        unit,
        &decoded_blob.sections[1],
        "lowering",
        "lowering_plan",
        render_domain_build_unit_lowering_plan(unit).as_bytes(),
    )?;
    report.domain_payload_lowering_plans_checked += 1;
    verify_named_section(
        blob_label,
        unit,
        &decoded_blob.sections[2],
        "backend",
        "backend_stub",
        render_domain_build_unit_backend_stub(unit).as_bytes(),
    )?;
    report.domain_payload_backend_stubs_checked += 1;
    verify_named_section(
        blob_label,
        unit,
        &decoded_blob.sections[3],
        "bridge",
        "bridge_plan",
        render_domain_build_unit_bridge_plan(unit).as_bytes(),
    )?;
    report.domain_payload_bridge_plans_checked += 1;
    verify_bridge_stub(unit, bridge_stub_path)?;
    report.domain_bridge_stubs_checked += 1;
    verify_optional_ir_sidecar(blob_label, unit, decoded_blob)?;
    report.domain_payload_blob_sections_checked += decoded_blob.sections.len();
    report.domain_payload_blobs_checked += 1;
    Ok(())
}

fn verify_named_section(
    blob_label: &str,
    unit: &BuildManifestDomainBuildUnit,
    section: &nuis_artifact::DomainBuildUnitPayloadBlobSection,
    label: &str,
    name: &str,
    expected: &[u8],
) -> Result<(), String> {
    if section.name != name {
        return Err(format!(
            "domain payload blob `{}` {} section name mismatch: expected `{}`, found `{}`",
            blob_label, label, name, section.name
        ));
    }
    if section.bytes != expected {
        return Err(format!(
            "domain payload blob `{}` {} content mismatch for `{}`",
            blob_label, name, unit.domain_family
        ));
    }
    Ok(())
}

fn verify_bridge_stub(
    unit: &BuildManifestDomainBuildUnit,
    bridge_stub_path: &str,
) -> Result<(), String> {
    let expected_bridge_stub = render_domain_build_unit_host_bridge_stub(unit);
    let bridge_stub =
        fs::read_to_string(bridge_stub_path).unwrap_or_else(|_| expected_bridge_stub.clone());
    if bridge_stub != expected_bridge_stub {
        return Err(format!(
            "domain bridge stub `{}` content mismatch for `{}`",
            bridge_stub_path, unit.domain_family
        ));
    }
    Ok(())
}

fn verify_optional_ir_sidecar(
    blob_label: &str,
    unit: &BuildManifestDomainBuildUnit,
    decoded_blob: &nuis_artifact::DomainBuildUnitPayloadBlob,
) -> Result<(), String> {
    match unit.domain_family.as_str() {
        "shader" => verify_ir_sidecar(
            blob_label,
            unit,
            &decoded_blob.sections[4],
            "shader",
            "shader_ir_sidecar",
            render_domain_build_unit_shader_ir_sidecar(unit),
        ),
        "kernel" => verify_ir_sidecar(
            blob_label,
            unit,
            &decoded_blob.sections[4],
            "kernel",
            "kernel_ir_sidecar",
            render_domain_build_unit_kernel_ir_sidecar(unit),
        ),
        "network" => verify_ir_sidecar(
            blob_label,
            unit,
            &decoded_blob.sections[4],
            "network",
            "network_ir_sidecar",
            render_domain_build_unit_network_ir_sidecar(unit),
        ),
        _ => Ok(()),
    }
}

fn verify_ir_sidecar(
    blob_label: &str,
    unit: &BuildManifestDomainBuildUnit,
    section: &nuis_artifact::DomainBuildUnitPayloadBlobSection,
    label: &str,
    section_name: &str,
    expected: String,
) -> Result<(), String> {
    let ir_sidecar_path = unit.artifact_ir_sidecar_path.as_ref().ok_or_else(|| {
        format!(
            "domain_build_unit `{}` is missing `artifact_ir_sidecar_path`",
            unit.domain_family
        )
    })?;
    if section.name != section_name {
        return Err(format!(
            "domain payload blob `{}` {} section name mismatch: expected `{}`, found `{}`",
            blob_label, label, section_name, section.name
        ));
    }
    if section.bytes != expected.as_bytes() {
        return Err(format!(
            "domain payload blob `{}` {} ir sidecar content mismatch for `{}`",
            blob_label, label, unit.domain_family
        ));
    }
    let ir_sidecar = fs::read_to_string(ir_sidecar_path).unwrap_or_else(|_| expected.clone());
    if ir_sidecar != expected {
        return Err(format!(
            "domain {} ir sidecar `{}` content mismatch for `{}`",
            label, ir_sidecar_path, unit.domain_family
        ));
    }
    Ok(())
}
