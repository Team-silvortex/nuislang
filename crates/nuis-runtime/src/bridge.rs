use nuis_artifact::{
    BridgeRegistryEntry, BuildManifestDomainBuildUnit, DomainBuildUnitPayloadBlob,
    HostBridgePlanEntry,
};

use crate::{AdapterRegistry, DomainAdapter, LoadedExecutable, RuntimeError};

pub struct PreparedDomainExecution<'a> {
    pub unit: &'a BuildManifestDomainBuildUnit,
    pub payload_blob: Option<&'a DomainBuildUnitPayloadBlob>,
    pub adapter: &'a dyn DomainAdapter,
    pub bridge_registry_entry: Option<&'a BridgeRegistryEntry>,
    pub host_bridge_plan_entry: Option<&'a HostBridgePlanEntry>,
}

impl<'a> PreparedDomainExecution<'a> {
    pub fn phase_order(&self) -> Option<&[String]> {
        self.host_bridge_plan_entry
            .map(|entry| entry.phase_order.as_slice())
    }

    pub fn lowering_plan_text(&self) -> Option<Result<&str, std::str::Utf8Error>> {
        self.payload_blob
            .and_then(|blob| blob.section_text("lowering_plan"))
    }

    pub fn backend_stub_text(&self) -> Option<Result<&str, std::str::Utf8Error>> {
        self.payload_blob
            .and_then(|blob| blob.section_text("backend_stub"))
    }

    pub fn bridge_plan_text(&self) -> Option<Result<&str, std::str::Utf8Error>> {
        self.payload_blob
            .and_then(|blob| blob.section_text("bridge_plan"))
    }

    pub fn ir_sidecar_text(&self) -> Option<Result<&str, std::str::Utf8Error>> {
        self.payload_blob.and_then(|blob| blob.ir_sidecar_text())
    }
}

#[derive(Debug, Default)]
pub struct BridgeExecutor;

impl BridgeExecutor {
    pub fn prepare<'a>(
        &self,
        executable: &'a LoadedExecutable,
        adapters: &'a AdapterRegistry,
        domain_family: &str,
    ) -> Result<PreparedDomainExecution<'a>, RuntimeError> {
        let unit = executable
            .domain_units
            .iter()
            .find(|unit| unit.domain_family == domain_family)
            .ok_or_else(|| RuntimeError::new(format!("unknown domain `{domain_family}`")))?;

        let adapter = adapters.resolve(unit)?;
        let payload_blob = executable.payload_blob_for_domain(domain_family);
        let bridge_registry_entry = executable
            .bridge_registry
            .as_ref()
            .and_then(|registry| registry.find_by_domain_family(domain_family));
        let host_bridge_plan_entry = executable
            .host_bridge_plan_index
            .as_ref()
            .and_then(|index| index.find_by_domain_family(domain_family));

        if unit.is_heterogeneous() {
            if payload_blob.is_none() {
                return Err(RuntimeError::new(format!(
                    "missing domain payload blob for heterogeneous domain `{domain_family}`"
                )));
            }
            if bridge_registry_entry.is_none() {
                return Err(RuntimeError::new(format!(
                    "missing bridge registry entry for heterogeneous domain `{domain_family}`"
                )));
            }
            if host_bridge_plan_entry.is_none() {
                return Err(RuntimeError::new(format!(
                    "missing host bridge plan entry for heterogeneous domain `{domain_family}`"
                )));
            }
        }

        Ok(PreparedDomainExecution {
            unit,
            payload_blob,
            adapter,
            bridge_registry_entry,
            host_bridge_plan_entry,
        })
    }
}

#[cfg(test)]
mod tests {
    use nuis_artifact::{
        BridgeRegistry, BridgeRegistryEntry, BuildManifest, BuildManifestDomainBuildUnit,
        DomainBuildUnitPayloadBlob, HostBridgePlanEntry, HostBridgePlanIndex, NuisCompiledArtifact,
        NuisExecutableEnvelope, NuisLifecycleContract,
    };

    use crate::{AdapterRegistry, DomainAdapter, LoadedExecutable};

    use super::BridgeExecutor;

    struct NetworkAdapter;

    impl DomainAdapter for NetworkAdapter {
        fn adapter_id(&self) -> &'static str {
            "network-test-adapter"
        }

        fn supports(&self, unit: &BuildManifestDomainBuildUnit) -> bool {
            unit.domain_family == "network"
        }
    }

    fn loaded_executable() -> LoadedExecutable {
        let unit = BuildManifestDomainBuildUnit {
            package_id: "official.network".to_owned(),
            domain_family: "network".to_owned(),
            abi: None,
            machine_arch: None,
            machine_os: None,
            backend_family: Some("urlsession".to_owned()),
            vendor: None,
            device_class: None,
            selected_lowering_target: Some("urlsession".to_owned()),
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: Some("/tmp/network.bridge.stub.txt".to_owned()),
            artifact_ir_sidecar_path: None,
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: Some("/tmp/network.payload.bin".to_owned()),
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
            contract_family: "nustar.network".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
        };
        LoadedExecutable {
            artifact: NuisCompiledArtifact {
                schema: "nuis-compiled-artifact-v1".to_owned(),
                packaging_mode: "native-cpu-llvm".to_owned(),
                cpu_target_abi: "cpu.x86_64.sysv64".to_owned(),
                cpu_target_machine_arch: "x86_64".to_owned(),
                cpu_target_machine_os: "linux".to_owned(),
                cpu_target_object_format: "elf".to_owned(),
                cpu_target_calling_abi: "sysv64".to_owned(),
                binary_name: "demo.bin".to_owned(),
                binary_bytes: 3,
                build_manifest_bytes: 0,
                envelope: NuisExecutableEnvelope {
                    schema: "nuis-executable-envelope-v1".to_owned(),
                    executable_kind: "native-cpu-llvm".to_owned(),
                    package_count: 1,
                    domain_families: vec!["network".to_owned()],
                    contract_families: vec!["nustar.network".to_owned()],
                    function_kind: "function-node".to_owned(),
                    graph_kind: "function-graph".to_owned(),
                    default_time_mode: "logical".to_owned(),
                },
                lifecycle: NuisLifecycleContract {
                    schema: "nuis-lifecycle-contract-v1".to_owned(),
                    bootstrap_entry: "nuis.bootstrap.lifecycle.v1".to_owned(),
                    tick_policy: "cooperative".to_owned(),
                    shutdown_policy: "graceful".to_owned(),
                    yalivia_rpc: "yalivia.rpc.bootstrap.v1".to_owned(),
                    hook_surface: vec![],
                    export_surface: vec![],
                    runtime_capability_flags: vec![],
                },
                build_manifest_source: String::new(),
                binary_blob: vec![],
            },
            envelope: NuisExecutableEnvelope {
                schema: "nuis-executable-envelope-v1".to_owned(),
                executable_kind: "native-cpu-llvm".to_owned(),
                package_count: 1,
                domain_families: vec!["network".to_owned()],
                contract_families: vec!["nustar.network".to_owned()],
                function_kind: "function-node".to_owned(),
                graph_kind: "function-graph".to_owned(),
                default_time_mode: "logical".to_owned(),
            },
            manifest: BuildManifest {
                schema: "nuis-build-manifest-v1".to_owned(),
                input: "/tmp/demo.ns".to_owned(),
                output_dir: "/tmp/out".to_owned(),
                packaging_mode: "native-cpu-llvm".to_owned(),
                envelope_path: "/tmp/out/nuis.executable.envelope.toml".to_owned(),
                envelope_schema: "nuis-executable-envelope-v1".to_owned(),
                envelope_package_count: 1,
                artifact_path: "/tmp/out/nuis.compiled.artifact".to_owned(),
                artifact_schema: "nuis-compiled-artifact-v1".to_owned(),
                artifact_binary_name: "demo.bin".to_owned(),
                artifact_binary_bytes: 3,
                lifecycle_schema: "nuis-lifecycle-contract-v1".to_owned(),
                lifecycle_bootstrap_entry: "nuis.bootstrap.lifecycle.v1".to_owned(),
                lifecycle_tick_policy: "cooperative".to_owned(),
                lifecycle_shutdown_policy: "graceful".to_owned(),
                lifecycle_yalivia_rpc: "yalivia.rpc.bootstrap.v1".to_owned(),
                lifecycle_hook_surface: vec![],
                lifecycle_export_surface: vec![],
                lifecycle_runtime_capability_flags: vec![],
                envelope_function_kind: "function-node".to_owned(),
                envelope_graph_kind: "function-graph".to_owned(),
                envelope_default_time_mode: "logical".to_owned(),
                cpu_target_abi: "cpu.x86_64.sysv64".to_owned(),
                cpu_target_machine_arch: "x86_64".to_owned(),
                cpu_target_machine_os: "linux".to_owned(),
                cpu_target_object_format: "elf".to_owned(),
                cpu_target_calling_abi: "sysv64".to_owned(),
                cpu_target_clang: "x86_64-unknown-linux-gnu".to_owned(),
                cpu_target_cross: true,
                compile_cache_status: None,
                compile_cache_key: None,
                compile_cache_root: None,
                project_plan_index: None,
                project_packet_index: None,
                project_plan_summary: None,
                bridge_registry_path: None,
                bridge_registry_schema: Some("nuis-bridge-registry-v1".to_owned()),
                bridge_registry_units: 1,
                bridge_registry_inline: None,
                host_bridge_plan_index_path: None,
                host_bridge_plan_index_schema: Some("nuis-host-bridge-plan-index-v1".to_owned()),
                host_bridge_plan_units: 1,
                host_bridge_plan_index_inline: None,
                artifact_hashes: vec![],
                execution_contract_count: 1,
                domain_build_units: vec![unit.clone()],
            },
            domain_units: vec![unit],
            domain_payload_blobs: vec![DomainBuildUnitPayloadBlob {
                domain_family: "network".to_owned(),
                package_id: "official.network".to_owned(),
                backend_family: Some("urlsession".to_owned()),
                vendor: None,
                device_class: None,
                selected_lowering_target: Some("urlsession".to_owned()),
                contract_family: "nustar.network".to_owned(),
                packaging_role: "hetero-contract".to_owned(),
                payload_kind: "contract-sidecar".to_owned(),
                payload_format: "toml".to_owned(),
                sections: vec![
                    nuis_artifact::DomainBuildUnitPayloadBlobSection {
                        name: "lowering_plan".to_owned(),
                        bytes: b"execution_route = \"foundation-session-reactor\"".to_vec(),
                    },
                    nuis_artifact::DomainBuildUnitPayloadBlobSection {
                        name: "backend_stub".to_owned(),
                        bytes: b"transport_ir = \"foundation-url-request\"".to_vec(),
                    },
                    nuis_artifact::DomainBuildUnitPayloadBlobSection {
                        name: "bridge_plan".to_owned(),
                        bytes: b"phase_submit = \"packet-write-dispatch\"".to_vec(),
                    },
                    nuis_artifact::DomainBuildUnitPayloadBlobSection {
                        name: "network_ir_sidecar".to_owned(),
                        bytes: b"schema = \"nuis-network-ir-sidecar-v1\"".to_vec(),
                    },
                ],
            }],
            bridge_registry: Some(BridgeRegistry {
                schema: "nuis-bridge-registry-v1".to_owned(),
                bridge_count: 1,
                domains: vec!["network".to_owned()],
                entries: vec![BridgeRegistryEntry {
                    domain_family: "network".to_owned(),
                    package_id: "official.network".to_owned(),
                    backend_family: "urlsession".to_owned(),
                    selected_lowering_target: "urlsession".to_owned(),
                    bridge_stub_path: "/tmp/network.bridge.stub.txt".to_owned(),
                    payload_blob_path: "/tmp/network.payload.bin".to_owned(),
                    plan_inline: String::new(),
                }],
            }),
            host_bridge_plan_index: Some(HostBridgePlanIndex {
                schema: "nuis-host-bridge-plan-index-v1".to_owned(),
                plan_count: 1,
                domains: vec!["network".to_owned()],
                entries: vec![HostBridgePlanEntry {
                    domain_family: "network".to_owned(),
                    package_id: "official.network".to_owned(),
                    bridge_stub_path: "/tmp/network.bridge.stub.txt".to_owned(),
                    bridge_surface: "host-ffi.bridge.network".to_owned(),
                    scheduler_binding: "network-poll-bridge".to_owned(),
                    phase_order: vec![
                        "bind".to_owned(),
                        "submit".to_owned(),
                        "wait".to_owned(),
                        "finalize".to_owned(),
                    ],
                    plan_inline: String::new(),
                }],
            }),
        }
    }

    #[test]
    fn prepare_resolves_adapter_and_bridge_metadata() {
        let mut registry = AdapterRegistry::new();
        registry.register(Box::new(NetworkAdapter));
        let executable = loaded_executable();
        let prepared = BridgeExecutor
            .prepare(&executable, &registry, "network")
            .unwrap();
        assert_eq!(prepared.adapter.adapter_id(), "network-test-adapter");
        assert_eq!(prepared.unit.domain_family, "network");
        assert_eq!(prepared.payload_blob.unwrap().domain_family, "network");
        assert_eq!(
            prepared.phase_order().unwrap(),
            &[
                "bind".to_owned(),
                "submit".to_owned(),
                "wait".to_owned(),
                "finalize".to_owned()
            ]
        );
        assert_eq!(
            prepared.lowering_plan_text().unwrap().unwrap(),
            "execution_route = \"foundation-session-reactor\""
        );
        assert_eq!(
            prepared.backend_stub_text().unwrap().unwrap(),
            "transport_ir = \"foundation-url-request\""
        );
        assert_eq!(
            prepared.bridge_plan_text().unwrap().unwrap(),
            "phase_submit = \"packet-write-dispatch\""
        );
        assert_eq!(
            prepared.ir_sidecar_text().unwrap().unwrap(),
            "schema = \"nuis-network-ir-sidecar-v1\""
        );
        assert_eq!(
            prepared.bridge_registry_entry.unwrap().backend_family,
            "urlsession"
        );
        assert_eq!(
            prepared.host_bridge_plan_entry.unwrap().scheduler_binding,
            "network-poll-bridge"
        );
    }
}
