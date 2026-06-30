use std::{fs, path::Path};

use crate::aot;

#[path = "linker_alignment.rs"]
mod linker_alignment;
#[path = "linker_clock_protocol.rs"]
mod linker_clock_protocol;
#[path = "linker_final_stage.rs"]
mod linker_final_stage;
#[path = "linker_hetero_calculate.rs"]
mod linker_hetero_calculate;
#[path = "linker_render.rs"]
mod linker_render;
#[path = "linker_types.rs"]
mod linker_types;

pub use linker_alignment::build_artifact_lowering_alignment_summary;
use linker_final_stage::derive_final_stage;
pub use linker_hetero_calculate::render_hetero_calculate_plan_toml;
pub use linker_render::render_link_plan_summary;
pub use linker_types::{
    ArtifactLoweringAlignmentCheck, ArtifactLoweringAlignmentSummary, LinkPlan, LinkPlanArtifact,
    LinkPlanClockDomain, LinkPlanClockEdge, LinkPlanClockProtocol, LinkPlanClockValidationSummary,
    LinkPlanCpuTarget, LinkPlanDataSegment, LinkPlanDomainUnit, LinkPlanEnvelope,
    LinkPlanFinalStage, LinkPlanHeteroCalculate, LinkPlanHeteroNode,
    LinkPlanHeteroValidationSummary, LinkPlanLifecycle, LINK_PLAN_SCHEMA,
};

pub fn build_clock_protocol(
    lifecycle: &LinkPlanLifecycle,
    default_time_mode: &str,
    domain_units: &[LinkPlanDomainUnit],
    hetero_calculate: &LinkPlanHeteroCalculate,
) -> LinkPlanClockProtocol {
    linker_clock_protocol::derive_clock_protocol(
        lifecycle,
        default_time_mode,
        domain_units,
        hetero_calculate,
    )
}

pub fn render_clock_protocol_toml(plan: &LinkPlanClockProtocol) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "schema = \"{}\"\n",
        crate::aot_toml::escape_toml_string(&plan.schema)
    ));
    out.push_str(&format!(
        "mode = \"{}\"\n",
        crate::aot_toml::escape_toml_string(&plan.mode)
    ));
    out.push_str(&format!(
        "source = \"{}\"\n",
        crate::aot_toml::escape_toml_string(&plan.source)
    ));
    out.push_str(&format!(
        "default_time_mode = \"{}\"\n",
        crate::aot_toml::escape_toml_string(&plan.default_time_mode)
    ));
    out.push_str(&format!(
        "lifecycle_tick_policy = \"{}\"\n",
        crate::aot_toml::escape_toml_string(&plan.lifecycle_tick_policy)
    ));
    out.push_str("[validation]\n");
    out.push_str(&format!("checked = {}\n", plan.validation.checked));
    out.push_str(&format!("valid = {}\n", plan.validation.valid));
    out.push_str(&format!(
        "issues = {}\n",
        crate::aot_toml::render_string_array(&plan.validation.issues)
    ));
    for domain in &plan.domains {
        out.push_str("[[clock_domain]]\n");
        out.push_str(&format!("index = {}\n", domain.index));
        out.push_str(&format!(
            "domain_family = \"{}\"\n",
            crate::aot_toml::escape_toml_string(&domain.domain_family)
        ));
        out.push_str(&format!(
            "package_id = \"{}\"\n",
            crate::aot_toml::escape_toml_string(&domain.package_id)
        ));
        out.push_str(&format!(
            "clock_domain_id = \"{}\"\n",
            crate::aot_toml::escape_toml_string(&domain.clock_domain_id)
        ));
        out.push_str(&format!(
            "clock_kind = \"{}\"\n",
            crate::aot_toml::escape_toml_string(&domain.clock_kind)
        ));
        out.push_str(&format!(
            "clock_epoch_kind = \"{}\"\n",
            crate::aot_toml::escape_toml_string(&domain.clock_epoch_kind)
        ));
        out.push_str(&format!(
            "clock_resolution = \"{}\"\n",
            crate::aot_toml::escape_toml_string(&domain.clock_resolution)
        ));
        out.push_str(&format!(
            "clock_bridge_default = \"{}\"\n",
            crate::aot_toml::escape_toml_string(&domain.clock_bridge_default)
        ));
        out.push_str(&format!(
            "lifecycle_hook = \"{}\"\n",
            crate::aot_toml::escape_toml_string(&domain.lifecycle_hook)
        ));
    }
    for edge in &plan.edges {
        out.push_str("[[clock_edge]]\n");
        out.push_str(&format!("index = {}\n", edge.index));
        out.push_str(&format!(
            "from = \"{}\"\n",
            crate::aot_toml::escape_toml_string(&edge.from)
        ));
        out.push_str(&format!(
            "to = \"{}\"\n",
            crate::aot_toml::escape_toml_string(&edge.to)
        ));
        out.push_str(&format!(
            "relation = \"{}\"\n",
            crate::aot_toml::escape_toml_string(&edge.relation)
        ));
        out.push_str(&format!(
            "source = \"{}\"\n",
            crate::aot_toml::escape_toml_string(&edge.source)
        ));
    }
    out
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
            artifact_ir_sidecar_path: unit.artifact_ir_sidecar_path.clone(),
            artifact_bridge_stub_inline: unit.artifact_bridge_stub_inline.clone(),
            artifact_payload_blob_path: unit.artifact_payload_blob_path.clone(),
            artifact_payload_blob_bytes: unit.artifact_payload_blob_bytes,
            artifact_payload_format: unit.artifact_payload_format.clone(),
            artifact_payload_blob_inline: unit.artifact_payload_blob_inline.clone(),
        })
        .collect::<Vec<_>>();

    let artifact_container =
        aot::inspect_nuis_compiled_artifact_container(Path::new(&report.artifact_path)).ok();

    let compiled_artifact = LinkPlanArtifact {
        path: report.artifact_path.clone(),
        binary_name: artifact.binary_name.clone(),
        binary_path: binary_path.clone(),
        binary_bytes: artifact.binary_bytes,
        build_manifest_bytes: artifact.build_manifest_bytes,
        container_kind: artifact_container
            .as_ref()
            .map(|container| container.container_kind.clone()),
        container_version: artifact_container
            .as_ref()
            .map(|container| container.binary_version),
        section_count: artifact_container
            .as_ref()
            .map(|container| container.section_count),
        section_names: artifact_container
            .as_ref()
            .map(|container| container.section_names.clone())
            .unwrap_or_default(),
        section_table_valid: artifact_container
            .as_ref()
            .map(|container| container.section_table_valid),
        lowering_unit_count: artifact_container
            .as_ref()
            .map(|container| container.lowering_unit_count),
        lowering_domain_families: artifact_container
            .as_ref()
            .map(|container| container.lowering_domain_families.clone())
            .unwrap_or_default(),
        lowering_targets: artifact_container
            .as_ref()
            .map(|container| container.lowering_targets.clone())
            .unwrap_or_default(),
        lowering_units: artifact_container
            .as_ref()
            .map(|container| container.lowering_units.clone())
            .unwrap_or_default(),
    };
    let artifact_lowering_alignment =
        build_artifact_lowering_alignment_summary(&compiled_artifact, &domain_units);

    let lifecycle = LinkPlanLifecycle {
        bootstrap_entry: report.lifecycle_bootstrap_entry.clone(),
        tick_policy: report.lifecycle_tick_policy.clone(),
        shutdown_policy: report.lifecycle_shutdown_policy.clone(),
        yalivia_rpc: report.lifecycle_yalivia_rpc.clone(),
        hook_surface: report.lifecycle_hook_surface.clone(),
        export_surface: report.lifecycle_export_surface.clone(),
        runtime_capability_flags: report.lifecycle_runtime_capability_flags.clone(),
    };
    let hetero_calculate =
        linker_hetero_calculate::derive_hetero_calculate_plan(&lifecycle, &domain_units);
    let clock_protocol = linker_clock_protocol::derive_clock_protocol(
        &lifecycle,
        &artifact.envelope.default_time_mode,
        &domain_units,
        &hetero_calculate,
    );

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
        lifecycle,
        envelope: LinkPlanEnvelope {
            schema: artifact.envelope.schema.clone(),
            package_count: artifact.envelope.package_count,
            contract_families: artifact.envelope.contract_families.clone(),
            domain_families: artifact.envelope.domain_families.clone(),
            function_kind: artifact.envelope.function_kind.clone(),
            graph_kind: artifact.envelope.graph_kind.clone(),
            default_time_mode: artifact.envelope.default_time_mode.clone(),
        },
        compiled_artifact,
        bridge_registry_path: report.bridge_registry_path.clone(),
        host_bridge_plan_index_path: report.host_bridge_plan_index_path.clone(),
        lowering_plan_index_path: report.lowering_plan_index_path.clone(),
        domain_units,
        artifact_lowering_alignment,
        clock_protocol,
        hetero_calculate,
        final_stage: derive_final_stage(report, &binary_path),
    }
}

pub fn build_link_plan_from_manifest(path: &Path) -> Result<LinkPlan, String> {
    let report = aot::verify_build_manifest(path)?;
    let artifact = aot::parse_nuis_compiled_artifact(Path::new(&report.artifact_path))?;
    Ok(build_link_plan(&report, &artifact))
}

pub fn build_hetero_calculate_plan(
    lifecycle: &LinkPlanLifecycle,
    domain_units: &[LinkPlanDomainUnit],
) -> LinkPlanHeteroCalculate {
    linker_hetero_calculate::derive_hetero_calculate_plan(lifecycle, domain_units)
}

pub fn write_hetero_calculate_plan(plan: &LinkPlan, path: &Path) -> Result<(), String> {
    let source = render_hetero_calculate_plan_toml(&plan.hetero_calculate);
    fs::write(path, source).map_err(|error| {
        format!(
            "failed to write hetero calculate link plan `{}`: {error}",
            path.display()
        )
    })
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
            lifecycle_hook_surface: vec!["on_scheduler_tick".to_owned()],
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
            lowering_plan_index_path: Some("out/nuis.lowering.plan-index.toml".to_owned()),
            lowering_plan_units: 1,
            lowering_plan_index_checked: 1,
            lowering_plan_entries_checked: 1,
            clock_protocol_path: Some("out/nuis.clock-protocol.toml".to_owned()),
            clock_protocol_domains: 2,
            clock_protocol_checked: 1,
            clock_protocol_entries_checked: 2,
            hetero_calculate_plan_path: Some("out/nuis.hetero-calculate.plan.toml".to_owned()),
            hetero_calculate_plan_units: 1,
            hetero_calculate_plan_checked: 1,
            hetero_calculate_plan_entries_checked: 1,
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
        assert_eq!(plan.hetero_calculate.mode, "host-only");
        assert!(plan.hetero_calculate.static_link);
        assert!(plan.hetero_calculate.lifecycle_driven);
        assert!(plan.hetero_calculate.nodes.is_empty());
        assert_eq!(plan.artifact_lowering_alignment.checked, 0);
        assert!(plan.artifact_lowering_alignment.consistent);
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
        assert_eq!(
            plan.lowering_plan_index_path.as_deref(),
            Some("out/nuis.lowering.plan-index.toml")
        );
        assert!(plan
            .final_stage
            .inputs
            .iter()
            .any(|input| input == "out/nuis.lowering.plan-index.toml"));
        assert_eq!(plan.domain_units.len(), 2);
        assert_eq!(plan.domain_units[1].kind, "heterogeneous");
        assert_eq!(plan.hetero_calculate.mode, "heterogeneous-static-lifecycle");
        assert_eq!(
            plan.hetero_calculate.time_order_model,
            "timestamped-partial-order"
        );
        assert_eq!(
            plan.hetero_calculate.data_order_model,
            "deterministic-segment-order"
        );
        assert_eq!(
            plan.hetero_calculate.c_world_policy,
            "wrapped-ordinary-node-no-linker-fast-path"
        );
        assert_eq!(plan.hetero_calculate.nodes.len(), 1);
        assert!(plan.hetero_calculate.validation.valid);
        assert!(plan.hetero_calculate.validation.issues.is_empty());
        assert_eq!(plan.clock_protocol.schema, "nuis-clock-protocol-v1");
        assert_eq!(plan.clock_protocol.mode, "heterogeneous-lifecycle-clock");
        assert!(plan.clock_protocol.validation.valid);
        assert!(plan
            .clock_protocol
            .domains
            .iter()
            .any(|domain| domain.clock_domain_id == "shader.clock.frame.v1"));
        assert!(plan
            .clock_protocol
            .edges
            .iter()
            .any(|edge| edge.to == "t0001.shader" && edge.relation == "happens-before"));
        assert_eq!(plan.hetero_calculate.nodes[0].timestamp, "t0001.shader");
        assert_eq!(
            plan.hetero_calculate.nodes[0].lifecycle_hook,
            "on_hetero_submission_progress"
        );
        assert_eq!(
            plan.hetero_calculate.nodes[0].wait_on,
            vec!["t0000.nustar.bootstrap.v1".to_owned()]
        );
        assert!(plan.hetero_calculate.nodes[0].c_world_wrapper);
        assert_eq!(plan.hetero_calculate.data_segments.len(), 1);
        assert_eq!(
            plan.hetero_calculate.data_segments[0].order_key,
            "data:0001:shader"
        );
        assert_eq!(
            plan.domain_units[1].artifact_payload_blob_path.as_deref(),
            Some("out/shader.ndpb")
        );
        let lines = render_link_plan_summary(&plan);
        assert!(lines.iter().any(
            |line| line.contains("hetero_calculate: schema=nuis-hetero-calculate-link-plan-v1")
        ));
        assert!(lines
            .iter()
            .any(|line| line.contains("hetero_validation: checked=")));
        assert!(lines
            .iter()
            .any(|line| line.contains("valid=true issues=none")));
        assert!(lines
            .iter()
            .any(|line| line.contains("hetero_node: index=0 timestamp=t0001.shader")));
        assert!(lines
            .iter()
            .any(|line| line.contains("data_segment: index=0 id=seg0001.shader")));
        assert!(lines
            .iter()
            .any(|line| line.contains("clock_protocol: schema=nuis-clock-protocol-v1")));
        assert!(lines
            .iter()
            .any(|line| line.contains("clock_domain: index=1 domain=shader")));
        assert!(lines
            .iter()
            .any(|line| line.contains("clock_edge: index=2 from=t0000.nustar.bootstrap.v1")));
        let hetero_toml = render_hetero_calculate_plan_toml(&plan.hetero_calculate);
        assert!(hetero_toml.contains("schema = \"nuis-hetero-calculate-link-plan-v1\""));
        assert!(hetero_toml.contains("static_link = true"));
        assert!(hetero_toml.contains("lifecycle_driven = true"));
        assert!(hetero_toml.contains("[validation]"));
        assert!(hetero_toml.contains("valid = true"));
        assert!(hetero_toml.contains("[[node]]"));
        assert!(hetero_toml.contains("timestamp = \"t0001.shader\""));
        assert!(hetero_toml.contains("wait_on = [\"t0000.nustar.bootstrap.v1\"]"));
        assert!(hetero_toml.contains("[[data_segment]]"));
        assert!(hetero_toml.contains("order_key = \"data:0001:shader\""));
        let out_path = std::env::temp_dir().join("nuis-linker-hetero-calculate-test.toml");
        write_hetero_calculate_plan(&plan, &out_path).unwrap();
        let written = std::fs::read_to_string(&out_path).unwrap();
        assert_eq!(written, hetero_toml);
        let _ = std::fs::remove_file(out_path);
        assert_eq!(plan.artifact_lowering_alignment.checked, 0);
        assert!(plan.artifact_lowering_alignment.consistent);
    }

    #[test]
    fn hetero_calculate_validation_rejects_broken_time_order() {
        let report = sample_report(
            "window-aot-bundle",
            vec![aot::BuildManifestDomainBuildUnit {
                package_id: "official.kernel".to_owned(),
                domain_family: "kernel".to_owned(),
                abi: Some("kernel.vulkan".to_owned()),
                machine_arch: Some("gpu".to_owned()),
                machine_os: Some("darwin".to_owned()),
                backend_family: Some("vulkan".to_owned()),
                vendor: Some("cross-vendor".to_owned()),
                device_class: Some("discrete-or-integrated-gpu".to_owned()),
                selected_lowering_target: Some("vulkan.discrete-or-integrated-gpu".to_owned()),
                artifact_stub_path: Some("out/kernel.stub.toml".to_owned()),
                artifact_stub_inline: None,
                artifact_payload_path: Some("out/kernel.payload.toml".to_owned()),
                artifact_bridge_stub_path: Some("out/kernel.bridge.c".to_owned()),
                artifact_ir_sidecar_path: Some("out/kernel.lowering.ir.txt".to_owned()),
                artifact_bridge_stub_inline: None,
                artifact_payload_blob_path: Some("out/kernel.ndpb".to_owned()),
                artifact_payload_blob_bytes: Some(128),
                artifact_payload_format: Some("ndpb-v2".to_owned()),
                artifact_payload_blob_inline: None,
                contract_family: "nustar.kernel".to_owned(),
                packaging_role: "hetero-payload".to_owned(),
            }],
        );
        let artifact = sample_artifact();
        let mut plan = build_link_plan(&report, &artifact);
        plan.hetero_calculate.nodes[0].timestamp = "t9999.kernel".to_owned();

        let validation =
            linker_hetero_calculate::validate_hetero_calculate_plan(&plan.hetero_calculate);

        assert!(!validation.valid);
        assert!(validation
            .issues
            .iter()
            .any(|issue| issue.contains("node timestamp mismatch")));
    }
}
