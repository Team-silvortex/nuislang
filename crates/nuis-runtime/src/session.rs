use nuis_artifact::{
    BridgeRegistry, BuildManifest, BuildManifestDomainBuildUnit, DomainBuildUnitPayloadBlob,
    HostBridgePlanIndex, NuisCompiledArtifact, NuisExecutableEnvelope,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostConsumableDomainUnit {
    pub domain_family: String,
    pub package_id: String,
    pub backend_family: Option<String>,
    pub selected_lowering_target: Option<String>,
    pub payload_blob_loaded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostConsumableSummary {
    pub heterogeneous_units: usize,
    pub payload_backed_units: usize,
    pub cpu_fallback_units: usize,
    pub host_consumable_units: usize,
    pub units: Vec<HostConsumableDomainUnit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedExecutable {
    pub artifact: NuisCompiledArtifact,
    pub envelope: NuisExecutableEnvelope,
    pub manifest: BuildManifest,
    pub domain_units: Vec<BuildManifestDomainBuildUnit>,
    pub domain_payload_blobs: Vec<DomainBuildUnitPayloadBlob>,
    pub bridge_registry: Option<BridgeRegistry>,
    pub host_bridge_plan_index: Option<HostBridgePlanIndex>,
}

impl LoadedExecutable {
    pub fn heterogeneous_units(&self) -> impl Iterator<Item = &BuildManifestDomainBuildUnit> {
        self.domain_units
            .iter()
            .filter(|unit| unit.is_heterogeneous())
    }

    pub fn payload_blob_for_domain(
        &self,
        domain_family: &str,
    ) -> Option<&DomainBuildUnitPayloadBlob> {
        self.domain_payload_blobs
            .iter()
            .find(|blob| blob.domain_family == domain_family)
    }

    pub fn host_consumable_summary(&self) -> HostConsumableSummary {
        let mut heterogeneous_units = 0usize;
        let mut payload_backed_units = 0usize;
        let mut cpu_fallback_units = 0usize;
        let mut units = Vec::new();

        for unit in self.heterogeneous_units() {
            heterogeneous_units += 1;
            let payload_blob_loaded = self.payload_blob_for_domain(&unit.domain_family).is_some();
            let cpu_fallback = is_cpu_fallback_unit(unit);
            if payload_blob_loaded {
                payload_backed_units += 1;
            }
            if cpu_fallback {
                cpu_fallback_units += 1;
            }
            if payload_blob_loaded && cpu_fallback {
                units.push(HostConsumableDomainUnit {
                    domain_family: unit.domain_family.clone(),
                    package_id: unit.package_id.clone(),
                    backend_family: unit.backend_family.clone(),
                    selected_lowering_target: unit.selected_lowering_target.clone(),
                    payload_blob_loaded,
                });
            }
        }

        HostConsumableSummary {
            heterogeneous_units,
            payload_backed_units,
            cpu_fallback_units,
            host_consumable_units: units.len(),
            units,
        }
    }
}

fn is_cpu_fallback_unit(unit: &BuildManifestDomainBuildUnit) -> bool {
    let backend = unit.backend_family.as_deref().unwrap_or("");
    let target = unit.selected_lowering_target.as_deref().unwrap_or("");
    backend == "cpu-fallback"
        || backend == "cpu-host"
        || target == "cpu-fallback"
        || target == "cpu-host"
        || target.starts_with("cpu-fallback.")
        || target.ends_with(".cpu-host")
}

#[cfg(test)]
mod tests {
    use nuis_artifact::{
        ArtifactHashEntry, BuildManifest, BuildManifestDomainBuildUnit, DomainBuildUnitPayloadBlob,
        NuisCompiledArtifact, NuisExecutableEnvelope, NuisLifecycleContract,
    };

    use super::LoadedExecutable;

    fn domain_unit(
        domain_family: &str,
        backend_family: Option<&str>,
        selected_lowering_target: Option<&str>,
    ) -> BuildManifestDomainBuildUnit {
        BuildManifestDomainBuildUnit {
            package_id: format!("official.{domain_family}"),
            domain_family: domain_family.to_owned(),
            abi: None,
            machine_arch: None,
            machine_os: None,
            backend_family: backend_family.map(str::to_owned),
            vendor: None,
            device_class: None,
            selected_lowering_target: selected_lowering_target.map(str::to_owned),
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: None,
            artifact_ir_sidecar_path: None,
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: None,
            artifact_payload_blob_bytes: None,
            artifact_payload_format: Some("ndpb-v2".to_owned()),
            artifact_payload_blob_inline: None,
            contract_family: format!("nustar.{domain_family}"),
            packaging_role: if domain_family == "cpu" {
                "host-binary".to_owned()
            } else {
                "hetero-contract".to_owned()
            },
        }
    }

    fn payload_blob(unit: &BuildManifestDomainBuildUnit) -> DomainBuildUnitPayloadBlob {
        DomainBuildUnitPayloadBlob {
            domain_family: unit.domain_family.clone(),
            package_id: unit.package_id.clone(),
            backend_family: unit.backend_family.clone(),
            vendor: None,
            device_class: None,
            selected_lowering_target: unit.selected_lowering_target.clone(),
            contract_family: unit.contract_family.clone(),
            packaging_role: unit.packaging_role.clone(),
            payload_kind: "contract-sidecar".to_owned(),
            payload_format: "ndpb-v2".to_owned(),
            sections: Vec::new(),
        }
    }

    fn loaded_executable(
        domain_units: Vec<BuildManifestDomainBuildUnit>,
        domain_payload_blobs: Vec<DomainBuildUnitPayloadBlob>,
    ) -> LoadedExecutable {
        let envelope = NuisExecutableEnvelope {
            schema: "nuis-executable-envelope-v1".to_owned(),
            executable_kind: "native-cpu-llvm".to_owned(),
            package_count: domain_units.len(),
            domain_families: domain_units
                .iter()
                .map(|unit| unit.domain_family.clone())
                .collect(),
            contract_families: domain_units
                .iter()
                .map(|unit| unit.contract_family.clone())
                .collect(),
            function_kind: "function-node".to_owned(),
            graph_kind: "function-graph".to_owned(),
            default_time_mode: "logical".to_owned(),
        };
        let lifecycle = NuisLifecycleContract {
            schema: "nuis-lifecycle-contract-v1".to_owned(),
            bootstrap_entry: "nuis.bootstrap.lifecycle.v1".to_owned(),
            tick_policy: "cooperative".to_owned(),
            shutdown_policy: "graceful".to_owned(),
            yalivia_rpc: "yalivia.rpc.bootstrap.v1".to_owned(),
            hook_surface: Vec::new(),
            export_surface: Vec::new(),
            runtime_capability_flags: Vec::new(),
        };
        LoadedExecutable {
            artifact: NuisCompiledArtifact {
                schema: "nuis-compiled-artifact-v1".to_owned(),
                packaging_mode: "native-cpu-llvm".to_owned(),
                cpu_target_abi: "cpu.host.v1".to_owned(),
                cpu_target_machine_arch: "host".to_owned(),
                cpu_target_machine_os: "host".to_owned(),
                cpu_target_object_format: "host".to_owned(),
                cpu_target_calling_abi: "host".to_owned(),
                binary_name: "demo".to_owned(),
                binary_bytes: 4,
                build_manifest_bytes: 0,
                envelope: envelope.clone(),
                lifecycle: lifecycle.clone(),
                build_manifest_source: String::new(),
                binary_blob: Vec::new(),
            },
            envelope,
            manifest: BuildManifest {
                schema: "nuis-build-manifest-v1".to_owned(),
                input: "/tmp/demo.ns".to_owned(),
                output_dir: "/tmp/out".to_owned(),
                packaging_mode: "native-cpu-llvm".to_owned(),
                envelope_path: "/tmp/out/nuis.executable.envelope.toml".to_owned(),
                envelope_schema: "nuis-executable-envelope-v1".to_owned(),
                envelope_package_count: domain_units.len(),
                artifact_path: "/tmp/out/nuis.compiled.artifact".to_owned(),
                artifact_schema: "nuis-compiled-artifact-v1".to_owned(),
                artifact_binary_name: "demo".to_owned(),
                artifact_binary_bytes: 4,
                lifecycle_schema: lifecycle.schema,
                lifecycle_bootstrap_entry: lifecycle.bootstrap_entry,
                lifecycle_tick_policy: lifecycle.tick_policy,
                lifecycle_shutdown_policy: lifecycle.shutdown_policy,
                lifecycle_yalivia_rpc: lifecycle.yalivia_rpc,
                lifecycle_hook_surface: lifecycle.hook_surface,
                lifecycle_export_surface: lifecycle.export_surface,
                lifecycle_runtime_capability_flags: lifecycle.runtime_capability_flags,
                envelope_function_kind: "function-node".to_owned(),
                envelope_graph_kind: "function-graph".to_owned(),
                envelope_default_time_mode: "logical".to_owned(),
                cpu_target_abi: "cpu.host.v1".to_owned(),
                cpu_target_machine_arch: "host".to_owned(),
                cpu_target_machine_os: "host".to_owned(),
                cpu_target_object_format: "host".to_owned(),
                cpu_target_calling_abi: "host".to_owned(),
                cpu_target_clang: "host".to_owned(),
                cpu_target_cross: false,
                compile_cache_status: None,
                compile_cache_key: None,
                compile_cache_root: None,
                project_plan_index: None,
                project_packet_index: None,
                project_plan_summary: None,
                bridge_registry_path: None,
                bridge_registry_schema: None,
                bridge_registry_units: 0,
                bridge_registry_inline: None,
                host_bridge_plan_index_path: None,
                host_bridge_plan_index_schema: None,
                host_bridge_plan_units: 0,
                host_bridge_plan_index_inline: None,
                artifact_hashes: vec![ArtifactHashEntry {
                    kind: "binary".to_owned(),
                    path: "/tmp/out/demo".to_owned(),
                    bytes: 4,
                    fnv1a64: "0x0000000000000000".to_owned(),
                }],
                execution_contract_count: domain_units.len(),
                domain_build_units: domain_units.clone(),
            },
            domain_units,
            domain_payload_blobs,
            bridge_registry: None,
            host_bridge_plan_index: None,
        }
    }

    #[test]
    fn host_consumable_summary_counts_only_payload_backed_cpu_fallback_domains() {
        let cpu = domain_unit("cpu", Some("llvm"), Some("llvm"));
        let shader = domain_unit(
            "shader",
            Some("cpu-fallback"),
            Some("cpu-fallback.cpu-host"),
        );
        let kernel = domain_unit("kernel", Some("coreml"), Some("coreml.apple-ane"));
        let loaded = loaded_executable(
            vec![cpu, shader.clone(), kernel.clone()],
            vec![payload_blob(&shader), payload_blob(&kernel)],
        );

        let summary = loaded.host_consumable_summary();

        assert_eq!(summary.heterogeneous_units, 2);
        assert_eq!(summary.payload_backed_units, 2);
        assert_eq!(summary.cpu_fallback_units, 1);
        assert_eq!(summary.host_consumable_units, 1);
        assert_eq!(summary.units[0].domain_family, "shader");
        assert_eq!(
            summary.units[0].selected_lowering_target.as_deref(),
            Some("cpu-fallback.cpu-host")
        );
    }

    #[test]
    fn host_consumable_summary_rejects_cpu_fallback_without_payload_blob() {
        let shader = domain_unit(
            "shader",
            Some("cpu-fallback"),
            Some("cpu-fallback.cpu-host"),
        );
        let loaded = loaded_executable(vec![shader], Vec::new());

        let summary = loaded.host_consumable_summary();

        assert_eq!(summary.heterogeneous_units, 1);
        assert_eq!(summary.payload_backed_units, 0);
        assert_eq!(summary.cpu_fallback_units, 1);
        assert_eq!(summary.host_consumable_units, 0);
        assert!(summary.units.is_empty());
    }
}
