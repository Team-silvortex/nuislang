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
use crate::kernel_codegen_table::{
    render_codegen_table, KernelYirCodegenTable, KERNEL_YIR_CODEGEN_TABLE_CONTRACT,
};

pub(crate) fn write_domain_build_unit_stubs(
    output_dir: &Path,
    units: &mut [BuildManifestDomainBuildUnit],
) -> Result<Vec<(String, PathBuf)>, String> {
    write_domain_build_unit_stubs_with_kernel_codegen_table(output_dir, units, None)
}

pub(crate) fn write_domain_build_unit_stubs_with_kernel_codegen_table(
    output_dir: &Path,
    units: &mut [BuildManifestDomainBuildUnit],
    kernel_codegen_table: Option<&KernelYirCodegenTable>,
) -> Result<Vec<(String, PathBuf)>, String> {
    let mut artifacts = Vec::new();
    let mut wrote_kernel_codegen_table = false;
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
            kernel_codegen_table,
        )? {
            artifacts.push(code_asset);
        }
        if unit.domain_family == "kernel"
            && unit.selected_lowering_target.as_deref() == Some("cuda.nvidia-gpu")
            && !wrote_kernel_codegen_table
        {
            if let Some(table) = kernel_codegen_table {
                let table_path = output_dir.join("nuis.domain.kernel.codegen-table.toml");
                fs::write(&table_path, render_codegen_table(table)?).map_err(|error| {
                    format!("failed to write `{}`: {error}", table_path.display())
                })?;
                artifacts.push(("domain_codegen_table_kernel".to_owned(), table_path));
                wrote_kernel_codegen_table = true;
            }
        }
    }
    Ok(artifacts)
}

fn write_registered_domain_code_asset(
    output_dir: &Path,
    domain_family: &str,
    lowering_target: Option<&str>,
    kernel_codegen_table: Option<&KernelYirCodegenTable>,
) -> Result<Option<(String, PathBuf)>, String> {
    if domain_family != "kernel" {
        return Ok(None);
    }
    let Some(asset) = lowering_target.and_then(crate::kernel_code_asset::select_kernel_code_asset)
    else {
        return Ok(None);
    };
    let path = output_dir.join(asset.file_name);
    let generated_bytes = if let Some(table) = kernel_codegen_table {
        if table.contract != KERNEL_YIR_CODEGEN_TABLE_CONTRACT {
            return Err("AOT received an invalid Kernel/YIR codegen table contract".to_owned());
        }
        let entries = table
            .functions
            .iter()
            .map(|function| function.entry.as_str())
            .collect::<Vec<_>>();
        if !entries.starts_with(asset.visible_entries) {
            return Err(format!(
                "Kernel/YIR codegen entries {:?} do not preserve registered code asset entry prefix {:?}",
                entries, asset.visible_entries
            ));
        }
        Some(crate::kernel_ptx_emitter::lower_cuda_ptx(table)?.into_bytes())
    } else {
        None
    };
    let bytes = generated_bytes.as_deref().unwrap_or(asset.bytes);
    fs::write(&path, bytes)
        .map_err(|error| format!("failed to write `{}`: {error}", path.display()))?;
    Ok(Some((format!("domain_code_asset_{domain_family}"), path)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cuda_kernel_unit() -> BuildManifestDomainBuildUnit {
        BuildManifestDomainBuildUnit {
            package_id: "official.kernel".to_owned(),
            domain_family: "kernel".to_owned(),
            abi: Some("kernel.cuda.ptx8_0.v1".to_owned()),
            machine_arch: Some("x86_64".to_owned()),
            machine_os: Some("linux".to_owned()),
            backend_family: Some("cuda".to_owned()),
            vendor: Some("nvidia".to_owned()),
            device_class: Some("nvidia-gpu".to_owned()),
            target_device: Some("nvidia-gpu".to_owned()),
            ir_format: Some("ptx8.0".to_owned()),
            dispatch_abi: Some("cuda-driver".to_owned()),
            backend_priority: Some(100),
            verification: Some("verified".to_owned()),
            selected_lowering_target: Some("cuda.nvidia-gpu".to_owned()),
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: None,
            artifact_ir_sidecar_path: None,
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: None,
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
            contract_family: "nustar.kernel".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
        }
    }

    #[test]
    fn materializes_registered_cuda_ptx_without_external_compiler() {
        let output_dir =
            std::env::temp_dir().join(format!("nuisc-kernel-code-asset-{}", std::process::id()));
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).unwrap();
        let (kind, path) = write_registered_domain_code_asset(
            &output_dir,
            "kernel",
            Some("cuda.nvidia-gpu"),
            None,
        )
        .unwrap()
        .expect("CUDA code asset");
        let asset = crate::kernel_code_asset::select_kernel_code_asset("cuda.nvidia-gpu").unwrap();
        assert_eq!(kind, "domain_code_asset_kernel");
        assert_eq!(path.file_name().unwrap(), asset.file_name);
        assert_eq!(fs::read(&path).unwrap(), asset.bytes);
        assert!(write_registered_domain_code_asset(
            &output_dir,
            "shader",
            Some("cuda.nvidia-gpu"),
            None,
        )
        .unwrap()
        .is_none());
        fs::remove_dir_all(output_dir).unwrap();
    }

    #[test]
    fn project_yir_table_materializes_hashed_sidecar_and_ptx() {
        let output_dir = std::env::temp_dir().join(format!(
            "nuisc-project-kernel-codegen-table-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).unwrap();
        let source = "yir 0.1\n\
resource cpu0 cpu.arm64\n\
resource kernel0 kernel.cuda\n\
function main cpu entry\n\
function-result main i64 value scalar\n\
function-node main input\n\
function-node main scalar\n\
function-node main mapped\n\
kernel.tensor input kernel0 1 4 1,2,3,4\n\
cpu.const_i64 scalar cpu0 10\n\
kernel.add_scalar_axis mapped kernel0 input cols scalar\n\
kernel.target_config target kernel0 x86_64 cuda 1 ptx\n";
        let table =
            crate::kernel_codegen_table::table_from_compiled_project_yir(source, "cuda.nvidia-gpu")
                .unwrap();
        let mut units = [cuda_kernel_unit()];
        let artifacts = write_domain_build_unit_stubs_with_kernel_codegen_table(
            &output_dir,
            &mut units,
            Some(&table),
        )
        .unwrap();

        assert!(artifacts
            .iter()
            .any(|(kind, _)| kind == "domain_codegen_table_kernel"));
        assert!(artifacts
            .iter()
            .any(|(kind, _)| kind == "domain_code_asset_kernel"));
        let sidecar =
            fs::read_to_string(output_dir.join("nuis.domain.kernel.codegen-table.toml")).unwrap();
        assert!(sidecar.contains("source_binding = \"compiled-project-yir\""));
        assert!(sidecar.contains("source_function_count = 1"));
        assert!(sidecar.contains("source_adapted_count = 1"));
        assert!(sidecar.contains("[[source_function]]"));
        assert!(sidecar.contains("[[source_adaptation]]"));
        assert!(sidecar.contains("generated_entry = \"nuis_project_main_mapped_i64\""));
        assert!(sidecar.contains(&format!(
            "source_fnv1a64 = \"{}\"",
            crate::aot_encoding::fnv1a64_hex(source.as_bytes())
        )));
        let ptx = fs::read(output_dir.join("nuis.domain.kernel.cuda.ptx")).unwrap();
        let ptx = std::str::from_utf8(&ptx).unwrap();
        assert!(ptx.contains(".visible .entry nuis_kernel_vector_add_f32"));
        assert!(ptx.contains(".visible .entry nuis_kernel_scale_f32"));
        assert!(ptx.contains(".visible .entry nuis_project_main_mapped_i64"));
        assert!(ptx.contains("add.s64"));
        fs::remove_dir_all(output_dir).unwrap();
    }
}
