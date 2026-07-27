use std::{
    fs,
    path::{Path, PathBuf},
};

use nuis_artifact::BuildManifestDomainBuildUnit;

use crate::aot_domain_payload_blob::encode_domain_build_unit_payload_blob;
use crate::aot_domain_render::render_domain_build_unit_host_bridge_stub;
use crate::aot_domain_unit_render::{
    render_domain_build_unit_payload, render_domain_build_unit_stub,
};
use crate::aot_encoding::hex_encode_bytes;
use crate::aot_kernel_sidecar::render_domain_build_unit_kernel_ir_sidecar;
use crate::aot_network_sidecar::render_domain_build_unit_network_ir_sidecar;
use crate::aot_shader_sidecar::render_domain_build_unit_shader_ir_sidecar;

pub(crate) fn write_domain_build_unit_stubs(
    output_dir: &Path,
    units: &mut [BuildManifestDomainBuildUnit],
) -> Result<Vec<(String, PathBuf)>, String> {
    let mut artifacts = Vec::new();
    for unit in units {
        if unit.domain_family == "cpu" {
            continue;
        }
        let payload_path =
            output_dir.join(format!("nuis.domain.{}.payload.toml", unit.domain_family));
        let payload_source = render_domain_build_unit_payload(unit)?;
        fs::write(&payload_path, payload_source)
            .map_err(|error| format!("failed to write `{}`: {error}", payload_path.display()))?;
        let payload_blob_path =
            output_dir.join(format!("nuis.domain.{}.payload.bin", unit.domain_family));
        let payload_blob = encode_domain_build_unit_payload_blob(unit, &payload_path)?;
        fs::write(&payload_blob_path, &payload_blob).map_err(|error| {
            format!("failed to write `{}`: {error}", payload_blob_path.display())
        })?;
        let bridge_stub_path = output_dir.join(format!(
            "nuis.domain.{}.bridge.stub.txt",
            unit.domain_family
        ));
        let bridge_stub = render_domain_build_unit_host_bridge_stub(unit);
        fs::write(&bridge_stub_path, &bridge_stub).map_err(|error| {
            format!("failed to write `{}`: {error}", bridge_stub_path.display())
        })?;
        let ir_sidecar_path = if unit.domain_family == "shader"
            || unit.domain_family == "kernel"
            || unit.domain_family == "network"
        {
            let path = output_dir.join(format!(
                "nuis.domain.{}.lowering.ir.txt",
                unit.domain_family
            ));
            let sidecar = match unit.domain_family.as_str() {
                "shader" => render_domain_build_unit_shader_ir_sidecar(unit),
                "kernel" => render_domain_build_unit_kernel_ir_sidecar(unit),
                "network" => render_domain_build_unit_network_ir_sidecar(unit),
                _ => unreachable!(),
            };
            fs::write(&path, sidecar)
                .map_err(|error| format!("failed to write `{}`: {error}", path.display()))?;
            Some(path)
        } else {
            None
        };
        let path = output_dir.join(format!("nuis.domain.{}.artifact.toml", unit.domain_family));
        unit.artifact_payload_path = Some(payload_path.display().to_string());
        unit.artifact_bridge_stub_path = Some(bridge_stub_path.display().to_string());
        unit.artifact_ir_sidecar_path = ir_sidecar_path
            .as_ref()
            .map(|path| path.display().to_string());
        unit.artifact_bridge_stub_inline = Some(bridge_stub.clone());
        unit.artifact_payload_blob_path = Some(payload_blob_path.display().to_string());
        unit.artifact_payload_blob_bytes = Some(payload_blob.len());
        unit.artifact_payload_format = Some("ndpb-v2".to_owned());
        unit.artifact_payload_blob_inline = Some(hex_encode_bytes(&payload_blob));
        let source = render_domain_build_unit_stub(unit);
        fs::write(&path, &source)
            .map_err(|error| format!("failed to write `{}`: {error}", path.display()))?;
        unit.artifact_stub_path = Some(path.display().to_string());
        unit.artifact_stub_inline = Some(source);
        artifacts.push((format!("domain_stub_{}", unit.domain_family), path));
        artifacts.push((
            format!("domain_payload_{}", unit.domain_family),
            payload_path,
        ));
        artifacts.push((
            format!("domain_payload_blob_{}", unit.domain_family),
            payload_blob_path,
        ));
        artifacts.push((
            format!("domain_bridge_stub_{}", unit.domain_family),
            bridge_stub_path,
        ));
        if let Some(ir_sidecar_path) = ir_sidecar_path {
            artifacts.push((
                format!("domain_ir_sidecar_{}", unit.domain_family),
                ir_sidecar_path,
            ));
        }
        if let Some(code_asset) = write_registered_domain_code_asset(
            output_dir,
            &unit.domain_family,
            unit.selected_lowering_target.as_deref(),
        )? {
            artifacts.push(code_asset);
        }
    }
    Ok(artifacts)
}

fn write_registered_domain_code_asset(
    output_dir: &Path,
    domain_family: &str,
    lowering_target: Option<&str>,
) -> Result<Option<(String, PathBuf)>, String> {
    if domain_family != "kernel" {
        return Ok(None);
    }
    let Some(asset) = lowering_target.and_then(crate::kernel_code_asset::select_kernel_code_asset)
    else {
        return Ok(None);
    };
    let path = output_dir.join(asset.file_name);
    fs::write(&path, asset.bytes)
        .map_err(|error| format!("failed to write `{}`: {error}", path.display()))?;
    Ok(Some((format!("domain_code_asset_{domain_family}"), path)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn materializes_registered_cuda_ptx_without_external_compiler() {
        let output_dir =
            std::env::temp_dir().join(format!("nuisc-kernel-code-asset-{}", std::process::id()));
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).unwrap();
        let (kind, path) =
            write_registered_domain_code_asset(&output_dir, "kernel", Some("cuda.nvidia-gpu"))
                .unwrap()
                .expect("CUDA code asset");
        let asset = crate::kernel_code_asset::select_kernel_code_asset("cuda.nvidia-gpu").unwrap();
        assert_eq!(kind, "domain_code_asset_kernel");
        assert_eq!(path.file_name().unwrap(), asset.file_name);
        assert_eq!(fs::read(&path).unwrap(), asset.bytes);
        assert!(
            write_registered_domain_code_asset(&output_dir, "shader", Some("cuda.nvidia-gpu"))
                .unwrap()
                .is_none()
        );
        fs::remove_dir_all(output_dir).unwrap();
    }
}
