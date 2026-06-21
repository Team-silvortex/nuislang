use std::path::Path;

use nuis_artifact::{
    parse_bridge_registry, parse_build_manifest_from_source, parse_host_bridge_plan_index,
    parse_nuis_compiled_artifact, BridgeRegistry, BuildManifest, HostBridgePlanIndex,
    NuisCompiledArtifact,
};

use crate::{LoadedExecutable, RuntimeError};

#[derive(Debug, Default)]
pub struct RuntimeLoader;

impl RuntimeLoader {
    pub fn load_from_artifact_path(&self, artifact_path: &Path) -> Result<LoadedExecutable, RuntimeError> {
        let artifact = parse_nuis_compiled_artifact(artifact_path)
            .map_err(|error| RuntimeError::new(format!("failed to load compiled artifact: {error}")))?;
        self.load_from_compiled_artifact(artifact)
    }

    pub fn load_from_compiled_artifact(
        &self,
        artifact: NuisCompiledArtifact,
    ) -> Result<LoadedExecutable, RuntimeError> {
        let manifest = self.load_embedded_manifest(&artifact)?;
        let bridge_registry = self.load_bridge_registry(&manifest)?;
        let host_bridge_plan_index = self.load_host_bridge_plan_index(&manifest)?;
        Ok(LoadedExecutable {
            envelope: artifact.envelope.clone(),
            domain_units: manifest.domain_build_units.clone(),
            bridge_registry,
            host_bridge_plan_index,
            artifact,
            manifest,
        })
    }

    fn load_embedded_manifest(
        &self,
        artifact: &NuisCompiledArtifact,
    ) -> Result<BuildManifest, RuntimeError> {
        parse_build_manifest_from_source(
            &artifact.build_manifest_source,
            Path::new("<embedded-build-manifest>"),
        )
        .map_err(|error| RuntimeError::new(format!("failed to parse embedded build manifest: {error}")))
    }

    fn load_bridge_registry(
        &self,
        manifest: &BuildManifest,
    ) -> Result<Option<BridgeRegistry>, RuntimeError> {
        if let Some(source) = &manifest.bridge_registry_inline {
            return nuis_artifact::parse_bridge_registry_from_source(
                source,
                Path::new("<embedded-bridge-registry>"),
            )
            .map(Some)
            .map_err(|error| {
                RuntimeError::new(format!("failed to parse embedded bridge registry: {error}"))
            });
        }
        let Some(path) = &manifest.bridge_registry_path else {
            return Ok(None);
        };
        parse_bridge_registry(Path::new(path))
            .map(Some)
            .map_err(|error| RuntimeError::new(format!("failed to parse bridge registry: {error}")))
    }

    fn load_host_bridge_plan_index(
        &self,
        manifest: &BuildManifest,
    ) -> Result<Option<HostBridgePlanIndex>, RuntimeError> {
        if let Some(source) = &manifest.host_bridge_plan_index_inline {
            return nuis_artifact::parse_host_bridge_plan_index_from_source(
                source,
                Path::new("<embedded-host-bridge-plan-index>"),
            )
            .map(Some)
            .map_err(|error| {
                RuntimeError::new(format!(
                    "failed to parse embedded host bridge plan index: {error}"
                ))
            });
        }
        let Some(path) = &manifest.host_bridge_plan_index_path else {
            return Ok(None);
        };
        parse_host_bridge_plan_index(Path::new(path)).map(Some).map_err(|error| {
            RuntimeError::new(format!(
                "failed to parse host bridge plan index: {error}"
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, time::{SystemTime, UNIX_EPOCH}};

    use nuis_artifact::{NuisCompiledArtifact, NuisExecutableEnvelope, NuisLifecycleContract};

    use super::RuntimeLoader;

    fn temp_dir(label: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("nuis_runtime_{label}_{unique}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn loader_reads_embedded_manifest_into_loaded_executable() {
        let manifest = r#"
manifest_schema = "nuis-build-manifest-v1"
input = "/tmp/demo.ns"
output_dir = "/tmp/out"
packaging_mode = "native-cpu-llvm"
path = "/tmp/out/nuis.executable.envelope.toml"
schema = "nuis-executable-envelope-v1"
package_count = 1
artifact_path = "/tmp/out/nuis.compiled.artifact"
artifact_schema = "nuis-compiled-artifact-v1"
artifact_binary_name = "demo.bin"
artifact_binary_bytes = 3
lifecycle_schema = "nuis-lifecycle-contract-v1"
lifecycle_bootstrap_entry = "nuis.bootstrap.lifecycle.v1"
lifecycle_tick_policy = "cooperative"
lifecycle_shutdown_policy = "graceful"
lifecycle_yalivia_rpc = "yalivia.rpc.bootstrap.v1"
lifecycle_hook_surface = ["on_bootstrap"]
lifecycle_export_surface = ["tick_export"]
lifecycle_runtime_capability_flags = ["runtime.tick"]
function_kind = "function-node"
graph_kind = "function-graph"
default_time_mode = "logical"
cpu_target_abi = "cpu.x86_64.sysv64"
cpu_target_machine_arch = "x86_64"
cpu_target_machine_os = "linux"
cpu_target_object_format = "elf"
cpu_target_calling_abi = "sysv64"
cpu_target_clang = "x86_64-unknown-linux-gnu"
cpu_target_cross = true

[[artifact_hash]]
kind = "artifact"
path = "/tmp/out/nuis.compiled.artifact"
bytes = 3
fnv1a64 = "0x0000000000000000"

[[execution_contract]]
package_id = "official.cpu"
domain_family = "cpu"

[[domain_build_unit]]
package_id = "official.cpu"
domain_family = "cpu"
selected_lowering_target = "llvm"
contract_family = "nustar.cpu"
packaging_role = "host-binary"
"#;
        let envelope = NuisExecutableEnvelope {
            schema: "nuis-executable-envelope-v1".to_owned(),
            executable_kind: "native-cpu-llvm".to_owned(),
            package_count: 1,
            domain_families: vec!["cpu".to_owned()],
            contract_families: vec!["nustar.cpu".to_owned()],
            function_kind: "function-node".to_owned(),
            graph_kind: "function-graph".to_owned(),
            default_time_mode: "logical".to_owned(),
        };
        let artifact = NuisCompiledArtifact {
            schema: "nuis-compiled-artifact-v1".to_owned(),
            packaging_mode: "native-cpu-llvm".to_owned(),
            cpu_target_abi: "cpu.x86_64.sysv64".to_owned(),
            cpu_target_machine_arch: "x86_64".to_owned(),
            cpu_target_machine_os: "linux".to_owned(),
            cpu_target_object_format: "elf".to_owned(),
            cpu_target_calling_abi: "sysv64".to_owned(),
            binary_name: "demo.bin".to_owned(),
            binary_bytes: 3,
            build_manifest_bytes: manifest.len(),
            envelope: envelope.clone(),
            lifecycle: NuisLifecycleContract {
                schema: "nuis-lifecycle-contract-v1".to_owned(),
                bootstrap_entry: "nuis.bootstrap.lifecycle.v1".to_owned(),
                tick_policy: "cooperative".to_owned(),
                shutdown_policy: "graceful".to_owned(),
                yalivia_rpc: "yalivia.rpc.bootstrap.v1".to_owned(),
                hook_surface: vec!["on_bootstrap".to_owned()],
                export_surface: vec!["tick_export".to_owned()],
                runtime_capability_flags: vec!["runtime.tick".to_owned()],
            },
            build_manifest_source: manifest.to_owned(),
            binary_blob: b"bin".to_vec(),
        };

        let loaded = RuntimeLoader
            .load_from_compiled_artifact(artifact)
            .expect("runtime loader should accept embedded manifest");
        assert_eq!(loaded.envelope, envelope);
        assert_eq!(loaded.manifest.schema, "nuis-build-manifest-v1");
        assert_eq!(loaded.domain_units.len(), 1);
        assert_eq!(loaded.bridge_registry, None);
        assert_eq!(loaded.host_bridge_plan_index, None);
        assert_eq!(loaded.domain_units[0].domain_family, "cpu");
        assert_eq!(
            loaded.domain_units[0].selected_lowering_target.as_deref(),
            Some("llvm")
        );
        assert_eq!(loaded.manifest.execution_contract_count, 1);
        assert_eq!(loaded.manifest.artifact_hashes.len(), 1);
    }

    #[test]
    fn loader_reads_bridge_registry_and_host_plan_index() {
        let dir = temp_dir("bridge_assets");
        let bridge_registry_source = r#"schema = "nuis-bridge-registry-v1"
bridge_count = 1
domains = ["network"]

[[bridge]]
domain_family = "network"
package_id = "official.network"
backend_family = "urlsession"
selected_lowering_target = "urlsession"
bridge_stub_path = "/tmp/network.bridge.stub.txt"
payload_blob_path = "/tmp/network.payload.bin"
"#;
        let host_plan_source = r#"schema = "nuis-host-bridge-plan-index-v1"
plan_count = 1
domains = ["network"]

[[plan]]
domain_family = "network"
package_id = "official.network"
bridge_stub_path = "/tmp/network.bridge.stub.txt"
bridge_surface = "host-ffi.bridge.network"
scheduler_binding = "network-poll-bridge"
phase_order = ["bind", "submit", "wait", "finalize"]
plan_inline = "bridge_kind = \"managed-lifecycle-bridge\""
"#;
        let manifest = format!(
            r#"manifest_schema = "nuis-build-manifest-v1"
input = "/tmp/demo.ns"
output_dir = "{output_dir}"
packaging_mode = "native-cpu-llvm"
path = "/tmp/out/nuis.executable.envelope.toml"
schema = "nuis-executable-envelope-v1"
package_count = 2
artifact_path = "/tmp/out/nuis.compiled.artifact"
artifact_schema = "nuis-compiled-artifact-v1"
artifact_binary_name = "demo.bin"
artifact_binary_bytes = 3
lifecycle_schema = "nuis-lifecycle-contract-v1"
lifecycle_bootstrap_entry = "nuis.bootstrap.lifecycle.v1"
lifecycle_tick_policy = "cooperative"
lifecycle_shutdown_policy = "graceful"
lifecycle_yalivia_rpc = "yalivia.rpc.bootstrap.v1"
lifecycle_hook_surface = ["on_bootstrap"]
lifecycle_export_surface = ["tick_export"]
lifecycle_runtime_capability_flags = ["runtime.tick"]
function_kind = "function-node"
graph_kind = "function-graph"
default_time_mode = "logical"
cpu_target_abi = "cpu.x86_64.sysv64"
cpu_target_machine_arch = "x86_64"
cpu_target_machine_os = "linux"
cpu_target_object_format = "elf"
cpu_target_calling_abi = "sysv64"
cpu_target_clang = "x86_64-unknown-linux-gnu"
cpu_target_cross = true
bridge_registry_path = "/tmp/missing.bridge.registry.toml"
bridge_registry_schema = "nuis-bridge-registry-v1"
bridge_registry_units = 1
bridge_registry_inline = "{bridge_registry_source}"
host_bridge_plan_index_path = "/tmp/missing.host-bridge.plan-index.toml"
host_bridge_plan_index_schema = "nuis-host-bridge-plan-index-v1"
host_bridge_plan_units = 1
host_bridge_plan_index_inline = "{host_plan_source}"

[[artifact_hash]]
kind = "artifact"
path = "/tmp/out/nuis.compiled.artifact"
bytes = 3
fnv1a64 = "0x0000000000000000"

[[execution_contract]]
package_id = "official.cpu"
domain_family = "cpu"

[[execution_contract]]
package_id = "official.network"
domain_family = "network"

[[domain_build_unit]]
package_id = "official.cpu"
domain_family = "cpu"
selected_lowering_target = "llvm"
contract_family = "nustar.cpu"
packaging_role = "host-binary"

[[domain_build_unit]]
package_id = "official.network"
domain_family = "network"
backend_family = "urlsession"
selected_lowering_target = "urlsession"
artifact_bridge_stub_path = "/tmp/network.bridge.stub.txt"
artifact_payload_blob_path = "/tmp/network.payload.bin"
contract_family = "nustar.network"
packaging_role = "hetero-contract"
"#,
            output_dir = dir.display(),
            bridge_registry_source = bridge_registry_source
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n"),
            host_plan_source = host_plan_source
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n"),
        );
        let envelope = NuisExecutableEnvelope {
            schema: "nuis-executable-envelope-v1".to_owned(),
            executable_kind: "native-cpu-llvm".to_owned(),
            package_count: 2,
            domain_families: vec!["cpu".to_owned(), "network".to_owned()],
            contract_families: vec!["nustar.cpu".to_owned(), "nustar.network".to_owned()],
            function_kind: "function-node".to_owned(),
            graph_kind: "function-graph".to_owned(),
            default_time_mode: "logical".to_owned(),
        };
        let artifact = NuisCompiledArtifact {
            schema: "nuis-compiled-artifact-v1".to_owned(),
            packaging_mode: "native-cpu-llvm".to_owned(),
            cpu_target_abi: "cpu.x86_64.sysv64".to_owned(),
            cpu_target_machine_arch: "x86_64".to_owned(),
            cpu_target_machine_os: "linux".to_owned(),
            cpu_target_object_format: "elf".to_owned(),
            cpu_target_calling_abi: "sysv64".to_owned(),
            binary_name: "demo.bin".to_owned(),
            binary_bytes: 3,
            build_manifest_bytes: manifest.len(),
            envelope,
            lifecycle: NuisLifecycleContract {
                schema: "nuis-lifecycle-contract-v1".to_owned(),
                bootstrap_entry: "nuis.bootstrap.lifecycle.v1".to_owned(),
                tick_policy: "cooperative".to_owned(),
                shutdown_policy: "graceful".to_owned(),
                yalivia_rpc: "yalivia.rpc.bootstrap.v1".to_owned(),
                hook_surface: vec!["on_bootstrap".to_owned()],
                export_surface: vec!["tick_export".to_owned()],
                runtime_capability_flags: vec!["runtime.tick".to_owned()],
            },
            build_manifest_source: manifest,
            binary_blob: b"bin".to_vec(),
        };

        let loaded = RuntimeLoader
            .load_from_compiled_artifact(artifact)
            .expect("runtime loader should load bridge metadata");
        assert_eq!(loaded.domain_units.len(), 2);
        assert_eq!(
            loaded
                .bridge_registry
                .as_ref()
                .unwrap()
                .find_by_domain_family("network")
                .unwrap()
                .backend_family,
            "urlsession"
        );
        let plan = loaded
            .host_bridge_plan_index
            .as_ref()
            .unwrap()
            .find_by_domain_family("network")
            .unwrap();
        assert_eq!(plan.bridge_surface, "host-ffi.bridge.network");
        assert_eq!(
            plan.phase_order,
            vec![
                "bind".to_owned(),
                "submit".to_owned(),
                "wait".to_owned(),
                "finalize".to_owned()
            ]
        );
    }
}
