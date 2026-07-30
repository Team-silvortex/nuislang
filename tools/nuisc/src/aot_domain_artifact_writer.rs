use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use nuis_artifact::BuildManifestDomainBuildUnit;

use crate::aot_code_asset_contribution::{
    kernel_asset_contribution, registered_asset_contribution, render_code_asset_contribution_table,
    required_registered_asset_contribution, shader_sidecar_contribution,
    DomainCodeAssetContribution,
};
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

pub(crate) fn write_domain_build_unit_stubs_with_kernel_codegen_table(
    output_dir: &Path,
    units: &mut [BuildManifestDomainBuildUnit],
    kernel_codegen_table: Option<&KernelYirCodegenTable>,
    required_code_assets: &[String],
) -> Result<Vec<(String, PathBuf)>, String> {
    let mut artifacts = Vec::new();
    let mut code_asset_contributions = Vec::<DomainCodeAssetContribution>::new();
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
            fs::write(&path, &sidecar)
                .map_err(|error| format!("failed to write `{}`: {error}", path.display()))?;
            if let Some(contribution) =
                shader_sidecar_contribution(unit, &path, sidecar.as_bytes())?
            {
                code_asset_contributions.push(contribution);
            }
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
            if let Some(contribution) = kernel_asset_contribution(
                unit,
                &code_asset.1,
                &fs::read(&code_asset.1).map_err(|error| {
                    format!("failed to read `{}`: {error}", code_asset.1.display())
                })?,
                kernel_codegen_table,
            )? {
                code_asset_contributions.push(contribution);
            }
            artifacts.push(code_asset);
        }
        for (kind, path, registration) in write_registered_nustar_code_assets(output_dir, unit)? {
            code_asset_contributions.push(registered_asset_contribution(
                unit,
                &registration,
                &path,
            )?);
            artifacts.push((kind, path));
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
    write_required_nustar_code_assets(
        output_dir,
        required_code_assets,
        &mut code_asset_contributions,
        &mut artifacts,
    )?;
    if !code_asset_contributions.is_empty() {
        let path = output_dir.join("nuis.domain.code-asset-contributions.toml");
        fs::write(
            &path,
            render_code_asset_contribution_table(&code_asset_contributions)?,
        )
        .map_err(|error| format!("failed to write `{}`: {error}", path.display()))?;
        artifacts.push(("domain_code_asset_contribution_table".to_owned(), path));
    }
    Ok(artifacts)
}

fn write_required_nustar_code_assets(
    output_dir: &Path,
    required_ids: &[String],
    contributions: &mut Vec<DomainCodeAssetContribution>,
    artifacts: &mut Vec<(String, PathBuf)>,
) -> Result<(), String> {
    let root = Path::new(crate::NUSTAR_REGISTRY_ROOT);
    let mut available = BTreeMap::new();
    for manifest in crate::registry::load_all_manifests(root)? {
        for asset in crate::registry::code_asset_registrations(root, &manifest)? {
            if available.insert(asset.asset_id.clone(), asset).is_some() {
                return Err("Nustar code asset registry contains duplicate asset IDs".to_owned());
            }
        }
    }
    let existing = contributions
        .iter()
        .map(|row| row.asset_id.clone())
        .collect::<BTreeSet<_>>();
    for id in required_ids.iter().collect::<BTreeSet<_>>() {
        if existing.contains(id) {
            continue;
        }
        let asset = available
            .get(id)
            .ok_or_else(|| format!("Galaxy requires unknown Nustar code asset `{id}`"))?;
        let path = output_dir.join(&asset.file_name);
        fs::write(&path, &asset.bytes)
            .map_err(|error| format!("failed to write `{}`: {error}", path.display()))?;
        contributions.push(required_registered_asset_contribution(asset, &path)?);
        artifacts.push((format!("domain_code_asset_{}", asset.domain_family), path));
    }
    Ok(())
}

fn write_registered_nustar_code_assets(
    output_dir: &Path,
    unit: &BuildManifestDomainBuildUnit,
) -> Result<
    Vec<(
        String,
        PathBuf,
        crate::registry::NustarCodeAssetRegistration,
    )>,
    String,
> {
    let Some(lowering_target) = unit.selected_lowering_target.as_deref() else {
        return Ok(Vec::new());
    };
    let manifest = crate::registry::load_manifest_for_domain(
        Path::new(crate::NUSTAR_REGISTRY_ROOT),
        &unit.domain_family,
    )?;
    let registrations = crate::registry::code_asset_registrations(
        Path::new(crate::NUSTAR_REGISTRY_ROOT),
        &manifest,
    )?;
    registrations
        .into_iter()
        .filter(|asset| asset.lowering_target == lowering_target)
        .map(|asset| {
            let path = output_dir.join(&asset.file_name);
            fs::write(&path, &asset.bytes)
                .map_err(|error| format!("failed to write `{}`: {error}", path.display()))?;
            Ok((
                format!("domain_code_asset_{}", unit.domain_family),
                path,
                asset,
            ))
        })
        .collect()
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

    fn vulkan_shader_unit() -> BuildManifestDomainBuildUnit {
        BuildManifestDomainBuildUnit {
            package_id: "official.shader".to_owned(),
            domain_family: "shader".to_owned(),
            abi: Some("shader.vulkan.spv1_6".to_owned()),
            machine_arch: Some("x86_64".to_owned()),
            machine_os: Some("linux".to_owned()),
            backend_family: Some("vulkan".to_owned()),
            vendor: Some("cross-vendor".to_owned()),
            device_class: Some("discrete-or-integrated-gpu".to_owned()),
            target_device: Some("discrete-or-integrated-gpu".to_owned()),
            ir_format: Some("spirv1.6".to_owned()),
            dispatch_abi: Some("vulkan-compute-pipeline".to_owned()),
            backend_priority: Some(90),
            verification: Some("verified".to_owned()),
            selected_lowering_target: Some("vulkan.discrete-or-integrated-gpu".to_owned()),
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
            contract_family: "nustar.shader".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
        }
    }

    fn metal_shader_unit() -> BuildManifestDomainBuildUnit {
        BuildManifestDomainBuildUnit {
            package_id: "official.shader".to_owned(),
            domain_family: "shader".to_owned(),
            abi: Some("shader.metal.msl2_4".to_owned()),
            machine_arch: Some("arm64".to_owned()),
            machine_os: Some("darwin".to_owned()),
            backend_family: Some("metal".to_owned()),
            vendor: Some("apple".to_owned()),
            device_class: Some("apple-silicon-gpu".to_owned()),
            target_device: Some("apple-silicon-gpu".to_owned()),
            ir_format: Some("msl2.4".to_owned()),
            dispatch_abi: Some("metal-render-pipeline".to_owned()),
            backend_priority: Some(100),
            verification: Some("verified".to_owned()),
            selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
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
            contract_family: "nustar.shader".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
        }
    }

    #[test]
    fn materializes_registered_vulkan_spirv_and_contribution_table() {
        let output_dir =
            std::env::temp_dir().join(format!("nuisc-shader-spirv-{}", std::process::id()));
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).unwrap();
        let mut units = [vulkan_shader_unit()];
        let artifacts = write_domain_build_unit_stubs_with_kernel_codegen_table(
            &output_dir,
            &mut units,
            None,
            &[],
        )
        .unwrap();

        let spirv_path = output_dir.join("nuis.shader.vulkan.copy-u32.spv");
        assert!(artifacts
            .iter()
            .any(|(kind, path)| kind == "domain_code_asset_shader" && path == &spirv_path));
        let spirv = fs::read(&spirv_path).unwrap();
        assert_eq!(
            u32::from_le_bytes(spirv[0..4].try_into().unwrap()),
            0x0723_0203
        );
        assert_eq!(
            u32::from_le_bytes(spirv[4..8].try_into().unwrap()),
            0x0001_0600
        );
        let table =
            fs::read_to_string(output_dir.join("nuis.domain.code-asset-contributions.toml"))
                .unwrap();
        assert!(table.contains("contribution_count = 2"));
        assert!(table.contains("owner_package_id = \"official.shader\""));
        assert!(table.contains("asset_id = \"shader.vulkan.copy-u32.spirv\""));
        assert!(table.contains("format = \"spirv-binary\""));
        assert!(table.contains("lowering_target = \"vulkan.discrete-or-integrated-gpu\""));
        assert!(table.contains("target = \"vulkan1.3-spirv1.6\""));
        assert!(table.contains("entries = [\"nuis_vulkan_copy_u32\"]"));
        assert!(table.contains("path = \"nuis.shader.vulkan.copy-u32.spv\""));
        fs::remove_dir_all(output_dir).unwrap();
    }

    #[test]
    fn materializes_registered_msl_from_canonical_wgsl_source() {
        let output_dir =
            std::env::temp_dir().join(format!("nuisc-shader-msl-{}", std::process::id()));
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).unwrap();
        let mut units = [metal_shader_unit()];
        let artifacts = write_domain_build_unit_stubs_with_kernel_codegen_table(
            &output_dir,
            &mut units,
            None,
            &[],
        )
        .unwrap();

        let msl_path = output_dir.join("nuis.shader.metal.copy-u32.metal");
        assert!(artifacts
            .iter()
            .any(|(kind, path)| kind == "domain_code_asset_shader" && path == &msl_path));
        let msl = fs::read_to_string(&msl_path).unwrap();
        assert!(msl.contains(
            "// nuis-module-lowering-plan contract=nuis-yir.shader.backend-lowering-plan.v1"
        ));
        assert!(msl.contains("// nuis-module-lowering-target msl:metal-gpu"));
        assert!(msl.contains("// nuis-module-native-ir msl2.4"));
        assert!(msl.contains(
            "// nuis-module-stage kind=compute execution_model=kernel binding_slot_model=argument-buffer-slot"
        ));
        assert!(msl.contains("kernel void nuis_metal_copy_u32("));
        assert!(msl.contains("output_values[gid] = value;"));
        let table =
            fs::read_to_string(output_dir.join("nuis.domain.code-asset-contributions.toml"))
                .unwrap();
        assert!(table.contains("asset_id = \"shader.metal.copy-u32.msl\""));
        assert!(table.contains("format = \"metal-source\""));
        assert!(table.contains("target = \"msl2.4\""));
        assert!(table.contains("entries = [\"nuis_metal_copy_u32\"]"));
        fs::remove_dir_all(output_dir).unwrap();
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
            &[],
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
        assert!(sidecar.contains(
            "project_code_asset_identity_contract = \"nuis-kernel-project-code-asset-identity-v1\""
        ));
        assert!(sidecar.contains("project_code_asset_id = \"kernel.cuda.project."));
        assert!(sidecar.contains("project_code_asset_entries = [\"nuis_project_main_mapped_i64\"]"));
        assert!(sidecar.contains("project_code_asset_identity_hash = \"0x"));
        assert!(sidecar.contains(
            "project_code_asset_identity_set_contract = \"nuis-provider-code-asset-identity-set-v1\""
        ));
        assert!(sidecar.contains("project_code_asset_identity_set_count = 1"));
        assert!(sidecar.contains("project_code_asset_identity_set_root_hash = \"0x"));
        let project_asset_id = sidecar
            .lines()
            .find_map(|line| {
                line.strip_prefix("project_code_asset_id = \"")
                    .and_then(|value| value.strip_suffix('"'))
            })
            .unwrap();
        let contribution_table =
            fs::read_to_string(output_dir.join("nuis.domain.code-asset-contributions.toml"))
                .unwrap();
        assert!(contribution_table.contains(&format!("asset_id = \"{project_asset_id}\"")));
        assert!(contribution_table.contains("lowering_target = \"cuda.nvidia-gpu\""));
        assert!(contribution_table.contains("target = \"sm_80\""));
        assert!(sidecar.contains("[[source_function]]"));
        assert!(sidecar.contains("[[source_adaptation]]"));
        assert!(sidecar.contains("generated_entry = \"nuis_project_main_mapped_i64\""));
        assert!(sidecar.contains(&format!(
            "source_fnv1a64 = \"{}\"",
            crate::aot_encoding::fnv1a64_hex(source.as_bytes())
        )));
        let ptx = fs::read(output_dir.join("nuis.domain.kernel.cuda.ptx")).unwrap();
        let ptx = std::str::from_utf8(&ptx).unwrap();
        assert!(!ptx.contains(".visible .entry nuis_kernel_vector_add_f32"));
        assert!(!ptx.contains(".visible .entry nuis_kernel_scale_f32"));
        assert!(ptx.contains(".visible .entry nuis_project_main_mapped_i64"));
        assert!(ptx.contains("add.s64"));
        fs::remove_dir_all(output_dir).unwrap();
    }

    #[test]
    fn writes_one_table_for_cross_domain_galaxy_assets() {
        let output_dir = std::env::temp_dir().join(format!(
            "nuisc-domain-code-asset-contributions-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).unwrap();
        let mut units = [cuda_kernel_unit()];
        let artifacts = write_domain_build_unit_stubs_with_kernel_codegen_table(
            &output_dir,
            &mut units,
            None,
            &[
                "shader.witsage.vector-bias.metal".to_owned(),
                "shader.witsage.argmax.metal".to_owned(),
            ],
        )
        .unwrap();
        assert!(artifacts
            .iter()
            .any(|(kind, _)| kind == "domain_code_asset_contribution_table"));
        let table =
            fs::read_to_string(output_dir.join("nuis.domain.code-asset-contributions.toml"))
                .unwrap();
        assert!(table.contains("protocol = \"nuis-domain-code-asset-contribution-table-v1\""));
        assert!(table.contains("contribution_count = 3"));
        assert!(table.contains("owner_package_id = \"official.kernel\""));
        assert!(table.contains("owner_package_id = \"official.shader\""));
        assert!(table.contains("path = \"nuis.domain.kernel.cuda.ptx\""));
        assert!(table.contains("asset_id = \"shader.witsage.vector-bias.metal\""));
        assert!(table.contains("path = \"nuis.witsage.vector-bias.metal\""));
        assert!(table.contains("entries = [\"nuis_witsage_argmax_f32\"]"));
        assert!(
            table.find("domain_family = \"kernel\"").unwrap()
                < table.find("domain_family = \"shader\"").unwrap()
        );
        assert!(table.contains("identity_set_root_hash = \"0x"));
        assert!(table.contains("table_hash = \"0x"));
        fs::remove_dir_all(output_dir).unwrap();
    }
}
