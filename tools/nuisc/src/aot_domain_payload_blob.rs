use std::{fs, path::Path};

use nuis_artifact::{
    decode_domain_payload_blob as shared_decode_domain_payload_blob,
    encode_domain_payload_blob as shared_encode_domain_payload_blob, BuildManifestDomainBuildUnit,
    DomainBuildUnitPayloadBlob, DomainBuildUnitPayloadBlobSection,
};

use crate::aot_domain_render::{
    render_domain_build_unit_backend_stub, render_domain_build_unit_bridge_plan,
    render_domain_build_unit_lowering_plan,
};
use crate::aot_kernel_sidecar::render_domain_build_unit_kernel_ir_sidecar;
use crate::aot_network_sidecar::render_domain_build_unit_network_ir_sidecar;
use crate::aot_shader_sidecar::render_domain_build_unit_shader_ir_sidecar;

pub(crate) fn encode_domain_build_unit_payload_blob(
    unit: &BuildManifestDomainBuildUnit,
    payload_path: &Path,
) -> Result<Vec<u8>, String> {
    let payload = fs::read(payload_path)
        .map_err(|error| format!("failed to read `{}`: {error}", payload_path.display()))?;
    let mut sections = vec![
        DomainBuildUnitPayloadBlobSection {
            name: "contract_toml".to_owned(),
            bytes: payload,
        },
        DomainBuildUnitPayloadBlobSection {
            name: "lowering_plan".to_owned(),
            bytes: render_domain_build_unit_lowering_plan(unit).into_bytes(),
        },
        DomainBuildUnitPayloadBlobSection {
            name: "backend_stub".to_owned(),
            bytes: render_domain_build_unit_backend_stub(unit).into_bytes(),
        },
        DomainBuildUnitPayloadBlobSection {
            name: "bridge_plan".to_owned(),
            bytes: render_domain_build_unit_bridge_plan(unit).into_bytes(),
        },
    ];
    if unit.domain_family == "shader" {
        sections.push(DomainBuildUnitPayloadBlobSection {
            name: "shader_ir_sidecar".to_owned(),
            bytes: render_domain_build_unit_shader_ir_sidecar(unit).into_bytes(),
        });
    } else if unit.domain_family == "kernel" {
        sections.push(DomainBuildUnitPayloadBlobSection {
            name: "kernel_ir_sidecar".to_owned(),
            bytes: render_domain_build_unit_kernel_ir_sidecar(unit).into_bytes(),
        });
    } else if unit.domain_family == "network" {
        sections.push(DomainBuildUnitPayloadBlobSection {
            name: "network_ir_sidecar".to_owned(),
            bytes: render_domain_build_unit_network_ir_sidecar(unit).into_bytes(),
        });
    }
    let blob = DomainBuildUnitPayloadBlob::from_domain_unit_and_sections(unit, sections);
    shared_encode_domain_payload_blob(&blob).map_err(|error| error.to_string())
}

pub(crate) fn decode_domain_build_unit_payload_blob(
    bytes: &[u8],
) -> Result<DomainBuildUnitPayloadBlob, String> {
    shared_decode_domain_payload_blob(bytes).map_err(|error| error.to_string())
}
