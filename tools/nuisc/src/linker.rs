use std::{fs, path::Path};

#[path = "linker_alignment.rs"]
mod linker_alignment;
#[path = "linker_clock_protocol.rs"]
mod linker_clock_protocol;
#[path = "linker_clock_render.rs"]
mod linker_clock_render;
#[path = "linker_final_stage.rs"]
mod linker_final_stage;
#[path = "linker_hetero_calculate.rs"]
mod linker_hetero_calculate;
#[path = "linker_host_ffi.rs"]
mod linker_host_ffi;
#[path = "linker_plan_build.rs"]
mod linker_plan_build;
#[path = "linker_render.rs"]
mod linker_render;
#[path = "linker_types.rs"]
mod linker_types;

pub use linker_alignment::build_artifact_lowering_alignment_summary;
pub use linker_clock_render::render_clock_protocol_toml;
pub use linker_hetero_calculate::render_hetero_calculate_plan_toml;
pub use linker_plan_build::{build_link_plan, build_link_plan_from_manifest};
pub use linker_render::{render_link_plan_json, render_link_plan_summary};
pub use linker_types::{
    ArtifactLoweringAlignmentCheck, ArtifactLoweringAlignmentSummary, LinkPlan, LinkPlanArtifact,
    LinkPlanClockDomain, LinkPlanClockEdge, LinkPlanClockProtocol, LinkPlanClockValidationSummary,
    LinkPlanCpuTarget, LinkPlanDataSegment, LinkPlanDomainUnit, LinkPlanEnvelope,
    LinkPlanFinalStage, LinkPlanHeteroCalculate, LinkPlanHeteroNode,
    LinkPlanHeteroValidationSummary, LinkPlanHostFfiAbiEntry, LinkPlanHostFfiAbiGroup,
    LinkPlanHostFfiEntry, LinkPlanHostFfiFootprint, LinkPlanHostFfiValidationSummary,
    LinkPlanLifecycle, LINK_PLAN_SCHEMA,
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
    use crate::aot;

    use super::linker_host_ffi::{derive_host_ffi_abi_groups, validate_host_ffi_footprint};
    use super::*;

    fn sample_host_ffi_entry(symbol: &str) -> LinkPlanHostFfiEntry {
        sample_host_ffi_entry_with_signature(symbol, "i64(i64)")
    }

    fn sample_host_ffi_entry_with_signature(
        symbol: &str,
        signature_pattern: &str,
    ) -> LinkPlanHostFfiEntry {
        LinkPlanHostFfiEntry {
            abi: "c".to_owned(),
            symbol: symbol.to_owned(),
            signature_pattern: signature_pattern.to_owned(),
            signature_hash: "fnv1a64:test".to_owned(),
            policy: crate::aot_ffi_bridge::SIGNATURE_WHITELIST_POLICY.to_owned(),
        }
    }

    #[test]
    fn host_ffi_validation_rejects_duplicate_whitelist_entries() {
        let entries = vec![
            sample_host_ffi_entry("host_sleep_ns"),
            sample_host_ffi_entry("host_sleep_ns"),
        ];

        let validation = validate_host_ffi_footprint(
            2,
            2,
            crate::aot_ffi_bridge::SIGNATURE_WHITELIST_POLICY,
            &entries,
        );

        assert_eq!(validation.checked, 2);
        assert!(!validation.valid);
        assert!(!validation.link_allowed);
        assert!(validation
            .issues
            .iter()
            .any(|issue| issue.contains("duplicate whitelist entry")));
    }

    #[test]
    fn host_ffi_validation_notes_multi_signature_symbol_without_rejecting() {
        let entries = vec![
            sample_host_ffi_entry_with_signature("host_probe", "i64()"),
            sample_host_ffi_entry_with_signature("host_probe", "i64(i64)"),
        ];

        let validation = validate_host_ffi_footprint(
            2,
            2,
            crate::aot_ffi_bridge::SIGNATURE_WHITELIST_POLICY,
            &entries,
        );

        assert_eq!(validation.checked, 2);
        assert!(validation.valid);
        assert!(validation.link_allowed);
        assert!(validation.issues.is_empty());
        assert!(validation
            .notes
            .iter()
            .any(|note| note.contains("has 2 whitelisted signatures")));
    }

    #[test]
    fn host_ffi_abi_group_validation_tracks_local_notes() {
        let entries = vec![
            sample_host_ffi_entry_with_signature("host_probe", "i64()"),
            sample_host_ffi_entry_with_signature("host_probe", "i64(i64)"),
        ];

        let groups = derive_host_ffi_abi_groups(&entries);

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].abi, "c");
        assert_eq!(groups[0].entries.len(), 2);
        assert!(groups[0].validation.valid);
        assert!(groups[0].validation.link_allowed);
        assert!(groups[0]
            .validation
            .notes
            .iter()
            .any(|note| note.contains("has 2 whitelisted signatures")));
    }

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
            project_host_ffi_index: None,
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
                target_device: Some("host-cpu".to_owned()),
                ir_format: Some("llvm-bitcode".to_owned()),
                dispatch_abi: Some("nuis-host-call".to_owned()),
                backend_priority: Some(100),
                verification: Some("contract-only".to_owned()),
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
                    target_device: Some("host-cpu".to_owned()),
                    ir_format: Some("llvm-bitcode".to_owned()),
                    dispatch_abi: Some("nuis-host-call".to_owned()),
                    backend_priority: Some(100),
                    verification: Some("contract-only".to_owned()),
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
                    target_device: Some("apple-gpu".to_owned()),
                    ir_format: Some("msl".to_owned()),
                    dispatch_abi: Some("metal-render-pipeline".to_owned()),
                    backend_priority: Some(10),
                    verification: Some("contract-only".to_owned()),
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
        assert!(plan.clock_protocol.edges.iter().any(|edge| {
            edge.from == "t0001.shader.complete"
                && edge.to == "t0001.shader.data_commit"
                && edge.relation == "data-segment-commit"
        }));
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
            plan.hetero_calculate.data_segments[0].wait_event,
            "t0001.shader.complete"
        );
        assert_eq!(
            plan.hetero_calculate.data_segments[0].commit_event,
            "t0001.shader.data_commit"
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
        assert!(lines.iter().any(|line| {
            line.contains("clock_edge:")
                && line.contains("from=t0001.shader.complete")
                && line.contains("relation=data-segment-commit")
        }));
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
        assert!(hetero_toml.contains("wait_event = \"t0001.shader.complete\""));
        assert!(hetero_toml.contains("commit_event = \"t0001.shader.data_commit\""));
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
                target_device: Some("vulkan-device".to_owned()),
                ir_format: Some("spirv".to_owned()),
                dispatch_abi: Some("vulkan-compute-pipeline".to_owned()),
                backend_priority: Some(30),
                verification: Some("contract-only".to_owned()),
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
