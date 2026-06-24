use std::path::Path;

use crate::aot;

pub const LINK_PLAN_SCHEMA: &str = "nuis-link-plan-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkPlan {
    pub schema: String,
    pub input: String,
    pub output_dir: String,
    pub packaging_mode: String,
    pub cpu_target: LinkPlanCpuTarget,
    pub lifecycle: LinkPlanLifecycle,
    pub envelope: LinkPlanEnvelope,
    pub compiled_artifact: LinkPlanArtifact,
    pub bridge_registry_path: Option<String>,
    pub host_bridge_plan_index_path: Option<String>,
    pub domain_units: Vec<LinkPlanDomainUnit>,
    pub final_stage: LinkPlanFinalStage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkPlanCpuTarget {
    pub abi: String,
    pub machine_arch: String,
    pub machine_os: String,
    pub object_format: String,
    pub calling_abi: String,
    pub clang_target: String,
    pub cross_compile: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkPlanLifecycle {
    pub bootstrap_entry: String,
    pub tick_policy: String,
    pub shutdown_policy: String,
    pub yalivia_rpc: String,
    pub hook_surface: Vec<String>,
    pub export_surface: Vec<String>,
    pub runtime_capability_flags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkPlanEnvelope {
    pub schema: String,
    pub package_count: usize,
    pub contract_families: Vec<String>,
    pub domain_families: Vec<String>,
    pub function_kind: String,
    pub graph_kind: String,
    pub default_time_mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkPlanArtifact {
    pub path: String,
    pub binary_name: String,
    pub binary_path: String,
    pub binary_bytes: usize,
    pub build_manifest_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkPlanDomainUnit {
    pub kind: String,
    pub package_id: String,
    pub domain_family: String,
    pub abi: Option<String>,
    pub machine_arch: Option<String>,
    pub machine_os: Option<String>,
    pub backend_family: Option<String>,
    pub vendor: Option<String>,
    pub device_class: Option<String>,
    pub selected_lowering_target: Option<String>,
    pub contract_family: String,
    pub packaging_role: String,
    pub artifact_stub_path: Option<String>,
    pub artifact_stub_inline: Option<String>,
    pub artifact_payload_path: Option<String>,
    pub artifact_bridge_stub_path: Option<String>,
    pub artifact_bridge_stub_inline: Option<String>,
    pub artifact_payload_blob_path: Option<String>,
    pub artifact_payload_blob_bytes: Option<usize>,
    pub artifact_payload_format: Option<String>,
    pub artifact_payload_blob_inline: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkPlanFinalStage {
    pub kind: String,
    pub driver: String,
    pub link_mode: String,
    pub output_path: String,
    pub inputs: Vec<String>,
    pub notes: Vec<String>,
}

pub fn build_link_plan(
    report: &aot::BuildManifestVerifyReport,
    artifact: &aot::NuisCompiledArtifact,
) -> LinkPlan {
    let binary_path = Path::new(&report.output_dir)
        .join(&artifact.binary_name)
        .display()
        .to_string();
    let domain_units = report
        .domain_build_units
        .iter()
        .map(|unit| LinkPlanDomainUnit {
            kind: if unit.is_heterogeneous() {
                "heterogeneous".to_owned()
            } else {
                "host".to_owned()
            },
            package_id: unit.package_id.clone(),
            domain_family: unit.domain_family.clone(),
            abi: unit.abi.clone(),
            machine_arch: unit.machine_arch.clone(),
            machine_os: unit.machine_os.clone(),
            backend_family: unit.backend_family.clone(),
            vendor: unit.vendor.clone(),
            device_class: unit.device_class.clone(),
            selected_lowering_target: unit.selected_lowering_target.clone(),
            contract_family: unit.contract_family.clone(),
            packaging_role: unit.packaging_role.clone(),
            artifact_stub_path: unit.artifact_stub_path.clone(),
            artifact_stub_inline: unit.artifact_stub_inline.clone(),
            artifact_payload_path: unit.artifact_payload_path.clone(),
            artifact_bridge_stub_path: unit.artifact_bridge_stub_path.clone(),
            artifact_bridge_stub_inline: unit.artifact_bridge_stub_inline.clone(),
            artifact_payload_blob_path: unit.artifact_payload_blob_path.clone(),
            artifact_payload_blob_bytes: unit.artifact_payload_blob_bytes,
            artifact_payload_format: unit.artifact_payload_format.clone(),
            artifact_payload_blob_inline: unit.artifact_payload_blob_inline.clone(),
        })
        .collect::<Vec<_>>();

    LinkPlan {
        schema: LINK_PLAN_SCHEMA.to_owned(),
        input: report.input.clone(),
        output_dir: report.output_dir.clone(),
        packaging_mode: report.packaging_mode.clone(),
        cpu_target: LinkPlanCpuTarget {
            abi: report.cpu_target_abi.clone(),
            machine_arch: report.cpu_target_machine_arch.clone(),
            machine_os: report.cpu_target_machine_os.clone(),
            object_format: report.cpu_target_object_format.clone(),
            calling_abi: report.cpu_target_calling_abi.clone(),
            clang_target: report.cpu_target_clang.clone(),
            cross_compile: report.cpu_target_cross,
        },
        lifecycle: LinkPlanLifecycle {
            bootstrap_entry: report.lifecycle_bootstrap_entry.clone(),
            tick_policy: report.lifecycle_tick_policy.clone(),
            shutdown_policy: report.lifecycle_shutdown_policy.clone(),
            yalivia_rpc: report.lifecycle_yalivia_rpc.clone(),
            hook_surface: report.lifecycle_hook_surface.clone(),
            export_surface: report.lifecycle_export_surface.clone(),
            runtime_capability_flags: report.lifecycle_runtime_capability_flags.clone(),
        },
        envelope: LinkPlanEnvelope {
            schema: artifact.envelope.schema.clone(),
            package_count: artifact.envelope.package_count,
            contract_families: artifact.envelope.contract_families.clone(),
            domain_families: artifact.envelope.domain_families.clone(),
            function_kind: artifact.envelope.function_kind.clone(),
            graph_kind: artifact.envelope.graph_kind.clone(),
            default_time_mode: artifact.envelope.default_time_mode.clone(),
        },
        compiled_artifact: LinkPlanArtifact {
            path: report.artifact_path.clone(),
            binary_name: artifact.binary_name.clone(),
            binary_path: binary_path.clone(),
            binary_bytes: artifact.binary_bytes,
            build_manifest_bytes: artifact.build_manifest_bytes,
        },
        bridge_registry_path: report.bridge_registry_path.clone(),
        host_bridge_plan_index_path: report.host_bridge_plan_index_path.clone(),
        domain_units,
        final_stage: derive_final_stage(report, &binary_path),
    }
}

pub fn build_link_plan_from_manifest(path: &Path) -> Result<LinkPlan, String> {
    let report = aot::verify_build_manifest(path)?;
    let artifact = aot::parse_nuis_compiled_artifact(Path::new(&report.artifact_path))?;
    Ok(build_link_plan(&report, &artifact))
}

pub fn render_link_plan_summary(plan: &LinkPlan) -> Vec<String> {
    let mut lines = vec![
        format!("schema: {}", plan.schema),
        format!("input: {}", plan.input),
        format!("output_dir: {}", plan.output_dir),
        format!("packaging_mode: {}", plan.packaging_mode),
        format!(
            "cpu_target: abi={} arch={} os={} object={} calling={} clang={} cross={}",
            plan.cpu_target.abi,
            plan.cpu_target.machine_arch,
            plan.cpu_target.machine_os,
            plan.cpu_target.object_format,
            plan.cpu_target.calling_abi,
            plan.cpu_target.clang_target,
            plan.cpu_target.cross_compile
        ),
        format!(
            "envelope: schema={} packages={} families={} domains={}",
            plan.envelope.schema,
            plan.envelope.package_count,
            plan.envelope.contract_families.join(","),
            plan.envelope.domain_families.join(",")
        ),
        format!(
            "artifact: path={} binary={} bytes={}",
            plan.compiled_artifact.path,
            plan.compiled_artifact.binary_path,
            plan.compiled_artifact.binary_bytes
        ),
        format!(
            "final_stage: kind={} driver={} link_mode={} output={}",
            plan.final_stage.kind,
            plan.final_stage.driver,
            plan.final_stage.link_mode,
            plan.final_stage.output_path
        ),
    ];
    if let Some(path) = &plan.bridge_registry_path {
        lines.push(format!("bridge_registry: {path}"));
    }
    if let Some(path) = &plan.host_bridge_plan_index_path {
        lines.push(format!("host_bridge_plan_index: {path}"));
    }
    for unit in &plan.domain_units {
        lines.push(format!(
            "domain_unit: kind={} domain={} package={} lowering={} backend={} role={}",
            unit.kind,
            unit.domain_family,
            unit.package_id,
            unit.selected_lowering_target.as_deref().unwrap_or("none"),
            unit.backend_family.as_deref().unwrap_or("none"),
            unit.packaging_role
        ));
    }
    lines
}

fn derive_final_stage(
    report: &aot::BuildManifestVerifyReport,
    binary_path: &str,
) -> LinkPlanFinalStage {
    let mut inputs = vec![report.artifact_path.clone(), report.envelope_path.clone()];
    if let Some(path) = &report.bridge_registry_path {
        inputs.push(path.clone());
    }
    if let Some(path) = &report.host_bridge_plan_index_path {
        inputs.push(path.clone());
    }
    let (kind, driver, link_mode, mut notes) = match report.packaging_mode.as_str() {
        "native-cpu-llvm" => (
            "host-native-link".to_owned(),
            "clang".to_owned(),
            "host-toolchain-finalize".to_owned(),
            vec![
                "nuisc currently lowers host CPU code to LLVM IR and delegates the final machine link to the host clang toolchain".to_owned(),
                "this stage is the temporary backend-facing tail of the larger nuis federated linking model".to_owned(),
            ],
        ),
        "window-aot-bundle" => (
            "heterogeneous-bundle-pack".to_owned(),
            "yir-pack-aot".to_owned(),
            "bundle-packaging".to_owned(),
            vec![
                "heterogeneous window packaging is currently assembled as an AOT bundle rather than a plain native executable link".to_owned(),
            ],
        ),
        other => (
            "custom-finalize".to_owned(),
            "custom".to_owned(),
            "custom".to_owned(),
            vec![format!(
                "packaging mode `{other}` requires an explicit finalization backend"
            )],
        ),
    };
    if report.cpu_target_cross {
        notes.push("cross-compilation target selected; final backend must honor the requested target ABI exactly".to_owned());
    }
    LinkPlanFinalStage {
        kind,
        driver,
        link_mode,
        output_path: binary_path.to_owned(),
        inputs,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_report(
        packaging_mode: &str,
        domain_build_units: Vec<aot::BuildManifestDomainBuildUnit>,
    ) -> aot::BuildManifestVerifyReport {
        aot::BuildManifestVerifyReport {
            schema: "nuis-build-manifest-v1".to_owned(),
            input: "main.ns".to_owned(),
            output_dir: "out".to_owned(),
            packaging_mode: packaging_mode.to_owned(),
            envelope_path: "out/nuis.executable.envelope.toml".to_owned(),
            envelope_schema: "nuis-executable-envelope-v1".to_owned(),
            envelope_package_count: domain_build_units.len(),
            artifact_path: "out/nuis.compiled.artifact".to_owned(),
            artifact_schema: "nuis-compiled-artifact-v1".to_owned(),
            artifact_binary_name: "demo".to_owned(),
            artifact_binary_bytes: 7,
            lifecycle_schema: "nuis-lifecycle-contract-v1".to_owned(),
            lifecycle_bootstrap_entry: "nustar.bootstrap.v1".to_owned(),
            lifecycle_tick_policy: "poll".to_owned(),
            lifecycle_shutdown_policy: "flush".to_owned(),
            lifecycle_yalivia_rpc: "disabled".to_owned(),
            lifecycle_hook_count: 1,
            lifecycle_hook_surface: vec!["tick".to_owned()],
            lifecycle_export_count: 1,
            lifecycle_export_surface: vec!["main".to_owned()],
            lifecycle_runtime_capability_flags: vec!["cpu".to_owned()],
            execution_contracts_checked: 1,
            domain_build_unit_count: domain_build_units.len(),
            heterogeneous_domain_count: domain_build_units
                .iter()
                .filter(|unit| unit.is_heterogeneous())
                .count(),
            domain_payload_blobs_checked: 0,
            domain_payload_blob_sections_checked: 0,
            domain_payload_contract_sections_checked: 0,
            domain_payload_lowering_plans_checked: 0,
            domain_payload_backend_stubs_checked: 0,
            domain_payload_bridge_plans_checked: 0,
            domain_bridge_stubs_checked: 0,
            domain_build_units,
            cpu_target_abi: "cpu.arm64.apple_aapcs64".to_owned(),
            cpu_target_machine_arch: "arm64".to_owned(),
            cpu_target_machine_os: "darwin".to_owned(),
            cpu_target_object_format: "mach-o".to_owned(),
            cpu_target_calling_abi: "aapcs64-darwin".to_owned(),
            cpu_target_clang: "aarch64-apple-darwin".to_owned(),
            cpu_target_cross: false,
            loaded_nustar: vec!["official.cpu".to_owned()],
            compile_cache_status: None,
            compile_cache_key: None,
            compile_cache_root: None,
            doc_index_path: None,
            doc_index_module_count: 0,
            doc_index_documented_item_count: 0,
            doc_index_checked: 0,
            project_text_handle_rewrite_helper_hits: 0,
            project_text_handle_rewrite_local_hits: 0,
            project_plan_index: None,
            project_docs_index: None,
            project_docs_module_count: 0,
            project_docs_documented_module_count: 0,
            project_docs_documented_item_count: 0,
            project_imports_index: None,
            project_imports_library_count: 0,
            project_imports_visible_library_count: 0,
            project_imports_visible_module_count: 0,
            project_imports_documented_visible_module_count: 0,
            project_imports_documented_visible_item_count: 0,
            project_galaxy_index: None,
            project_galaxy_count: 0,
            project_documented_galaxy_count: 0,
            project_documented_galaxy_library_module_count: 0,
            project_documented_galaxy_item_count: 0,
            project_packet_index: None,
            bridge_registry_path: Some("out/nuis.bridge.registry.toml".to_owned()),
            bridge_registry_units: 1,
            bridge_registry_checked: 1,
            bridge_registry_entries_checked: 1,
            host_bridge_plan_index_path: Some("out/nuis.host-bridge.plan-index.toml".to_owned()),
            host_bridge_plan_units: 1,
            host_bridge_plan_checked: 1,
            host_bridge_plan_entries_checked: 1,
            artifacts_checked: 1,
            project_metadata_checked: 0,
        }
    }

    fn sample_artifact() -> aot::NuisCompiledArtifact {
        aot::NuisCompiledArtifact {
            schema: "nuis-compiled-artifact-v1".to_owned(),
            packaging_mode: "native-cpu-llvm".to_owned(),
            cpu_target_abi: "cpu.arm64.apple_aapcs64".to_owned(),
            cpu_target_machine_arch: "arm64".to_owned(),
            cpu_target_machine_os: "darwin".to_owned(),
            cpu_target_object_format: "mach-o".to_owned(),
            cpu_target_calling_abi: "aapcs64-darwin".to_owned(),
            binary_name: "demo".to_owned(),
            binary_bytes: 7,
            build_manifest_bytes: 13,
            envelope: aot::NuisExecutableEnvelope {
                schema: "nuis-executable-envelope-v1".to_owned(),
                executable_kind: "native".to_owned(),
                package_count: 2,
                domain_families: vec!["cpu".to_owned(), "shader".to_owned()],
                contract_families: vec!["nustar.cpu".to_owned(), "nustar.shader".to_owned()],
                function_kind: "federated-function".to_owned(),
                graph_kind: "federated-graph".to_owned(),
                default_time_mode: "global".to_owned(),
            },
            lifecycle: aot::NuisLifecycleContract {
                schema: "nuis-lifecycle-contract-v1".to_owned(),
                bootstrap_entry: "nustar.bootstrap.v1".to_owned(),
                tick_policy: "poll".to_owned(),
                shutdown_policy: "flush".to_owned(),
                yalivia_rpc: "disabled".to_owned(),
                hook_surface: vec!["tick".to_owned()],
                export_surface: vec!["main".to_owned()],
                runtime_capability_flags: vec!["cpu".to_owned()],
            },
            build_manifest_source: String::new(),
            binary_blob: vec![1, 2, 3],
        }
    }

    #[test]
    fn builds_native_link_plan_with_host_clang_final_stage() {
        let report = sample_report(
            "native-cpu-llvm",
            vec![aot::BuildManifestDomainBuildUnit {
                package_id: "official.cpu".to_owned(),
                domain_family: "cpu".to_owned(),
                abi: Some("cpu.arm64.apple_aapcs64".to_owned()),
                machine_arch: Some("arm64".to_owned()),
                machine_os: Some("darwin".to_owned()),
                backend_family: Some("llvm".to_owned()),
                vendor: None,
                device_class: None,
                selected_lowering_target: Some("llvm".to_owned()),
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
                contract_family: "nustar.cpu".to_owned(),
                packaging_role: "host-binary".to_owned(),
            }],
        );
        let artifact = sample_artifact();

        let plan = build_link_plan(&report, &artifact);

        assert_eq!(plan.schema, LINK_PLAN_SCHEMA);
        assert_eq!(plan.final_stage.driver, "clang");
        assert_eq!(plan.final_stage.kind, "host-native-link");
        assert_eq!(plan.compiled_artifact.binary_path, "out/demo");
        assert_eq!(plan.domain_units.len(), 1);
        assert_eq!(plan.domain_units[0].kind, "host");
    }

    #[test]
    fn builds_bundle_link_plan_with_heterogeneous_domain_units() {
        let report = sample_report(
            "window-aot-bundle",
            vec![
                aot::BuildManifestDomainBuildUnit {
                    package_id: "official.cpu".to_owned(),
                    domain_family: "cpu".to_owned(),
                    abi: Some("cpu.arm64.apple_aapcs64".to_owned()),
                    machine_arch: Some("arm64".to_owned()),
                    machine_os: Some("darwin".to_owned()),
                    backend_family: Some("llvm".to_owned()),
                    vendor: None,
                    device_class: None,
                    selected_lowering_target: Some("llvm".to_owned()),
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
                    contract_family: "nustar.cpu".to_owned(),
                    packaging_role: "host-binary".to_owned(),
                },
                aot::BuildManifestDomainBuildUnit {
                    package_id: "official.shader".to_owned(),
                    domain_family: "shader".to_owned(),
                    abi: Some("shader.apple.metal".to_owned()),
                    machine_arch: Some("apple-gpu".to_owned()),
                    machine_os: Some("darwin".to_owned()),
                    backend_family: Some("metal".to_owned()),
                    vendor: Some("apple".to_owned()),
                    device_class: Some("apple-silicon-gpu".to_owned()),
                    selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
                    artifact_stub_path: Some("out/shader.stub.toml".to_owned()),
                    artifact_stub_inline: None,
                    artifact_payload_path: Some("out/shader.payload.toml".to_owned()),
                    artifact_bridge_stub_path: Some("out/shader.bridge.c".to_owned()),
                    artifact_ir_sidecar_path: Some("out/shader.lowering.ir.txt".to_owned()),
                    artifact_bridge_stub_inline: None,
                    artifact_payload_blob_path: Some("out/shader.ndpb".to_owned()),
                    artifact_payload_blob_bytes: Some(128),
                    artifact_payload_format: Some("ndpb-v2".to_owned()),
                    artifact_payload_blob_inline: None,
                    contract_family: "nustar.shader".to_owned(),
                    packaging_role: "hetero-payload".to_owned(),
                },
            ],
        );
        let artifact = sample_artifact();

        let plan = build_link_plan(&report, &artifact);

        assert_eq!(plan.final_stage.driver, "yir-pack-aot");
        assert_eq!(plan.final_stage.kind, "heterogeneous-bundle-pack");
        assert_eq!(plan.domain_units.len(), 2);
        assert_eq!(plan.domain_units[1].kind, "heterogeneous");
        assert_eq!(
            plan.domain_units[1].artifact_payload_blob_path.as_deref(),
            Some("out/shader.ndpb")
        );
    }
}
