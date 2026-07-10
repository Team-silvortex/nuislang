use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use nuis_artifact::{
    encode_domain_payload_blob, DomainBuildUnitPayloadBlob, DomainBuildUnitPayloadBlobSection,
    NuisCompiledArtifact, NuisExecutableEnvelope, NuisLifecycleContract,
};

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

[clock_protocol]
clock_protocol_path = "/tmp/out/nuis.clock-protocol.toml"
clock_protocol_schema = "nuis-clock-protocol-v1"
clock_protocol_domains = 1
clock_protocol_inline = "schema = \"nuis-clock-protocol-v1\"\nmode = \"host-lifecycle-clock\"\nsource = \"test\"\ndefault_time_mode = \"logical\"\nlifecycle_tick_policy = \"cooperative\"\n[validation]\nchecked = 8\nvalid = true\nissues = []\n[[clock_domain]]\nindex = 0\ndomain_family = \"cpu\"\npackage_id = \"official.cpu\"\nclock_domain_id = \"cpu.clock.host.v1\"\nclock_kind = \"host-monotonic\"\nclock_epoch_kind = \"host-epoch\"\nclock_resolution = \"cpu.tick_i64\"\nclock_bridge_default = \"global->monotonic:bridge\"\nlifecycle_hook = \"on_scheduler_tick\"\n[[clock_edge]]\nindex = 0\nfrom = \"global.clock.root.v1\"\nto = \"cpu.clock.host.v1\"\nrelation = \"global->monotonic:bridge\"\nsource = \"test\"\n"

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
    assert_eq!(loaded.domain_payload_blobs.len(), 0);
    assert_eq!(loaded.bridge_registry, None);
    assert_eq!(loaded.host_bridge_plan_index, None);
    let clock_protocol = loaded
        .clock_protocol
        .as_ref()
        .expect("embedded clock protocol should load");
    assert_eq!(clock_protocol.schema, "nuis-clock-protocol-v1");
    assert_eq!(clock_protocol.domains.len(), 1);
    assert_eq!(
        clock_protocol.find_domain("cpu").unwrap().clock_domain_id,
        "cpu.clock.host.v1"
    );
    assert_eq!(loaded.clock_protocol_summary().unwrap().edges, 1);
    assert_eq!(loaded.domain_units[0].domain_family, "cpu");
    assert_eq!(
        loaded.domain_units[0].selected_lowering_target.as_deref(),
        Some("llvm")
    );
    assert_eq!(loaded.manifest.execution_contract_count, 1);
    assert_eq!(loaded.manifest.artifact_hashes.len(), 1);
}

#[test]
fn loader_rejects_invalid_embedded_clock_protocol() {
    let manifest = minimal_manifest_with_clock_protocol(
        1,
        "schema = \"nuis-clock-protocol-v1\"\nmode = \"host-lifecycle-clock\"\nsource = \"test\"\ndefault_time_mode = \"logical\"\nlifecycle_tick_policy = \"cooperative\"\n[validation]\nchecked = 1\nvalid = false\nissues = [\"broken\"]\n[[clock_domain]]\nindex = 0\ndomain_family = \"cpu\"\npackage_id = \"official.cpu\"\nclock_domain_id = \"cpu.clock.host.v1\"\nclock_kind = \"host-monotonic\"\nclock_epoch_kind = \"host-epoch\"\nclock_resolution = \"cpu.tick_i64\"\nclock_bridge_default = \"global->monotonic:bridge\"\nlifecycle_hook = \"on_scheduler_tick\"\n",
    );
    let artifact = minimal_artifact_with_manifest(&manifest);

    let error = RuntimeLoader
        .load_from_compiled_artifact(artifact)
        .expect_err("runtime loader should reject invalid clock protocol");

    assert!(error
        .to_string()
        .contains("clock protocol validation flag is false"));
}

#[test]
fn loader_rejects_clock_protocol_domain_count_mismatch() {
    let manifest = minimal_manifest_with_clock_protocol(
        2,
        "schema = \"nuis-clock-protocol-v1\"\nmode = \"host-lifecycle-clock\"\nsource = \"test\"\ndefault_time_mode = \"logical\"\nlifecycle_tick_policy = \"cooperative\"\n[validation]\nchecked = 1\nvalid = true\nissues = []\n[[clock_domain]]\nindex = 0\ndomain_family = \"cpu\"\npackage_id = \"official.cpu\"\nclock_domain_id = \"cpu.clock.host.v1\"\nclock_kind = \"host-monotonic\"\nclock_epoch_kind = \"host-epoch\"\nclock_resolution = \"cpu.tick_i64\"\nclock_bridge_default = \"global->monotonic:bridge\"\nlifecycle_hook = \"on_scheduler_tick\"\n",
    );
    let artifact = minimal_artifact_with_manifest(&manifest);

    let error = RuntimeLoader
        .load_from_compiled_artifact(artifact)
        .expect_err("runtime loader should reject clock domain count mismatch");

    assert!(error
        .to_string()
        .contains("clock protocol domain count mismatch"));
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
    let payload_blob = DomainBuildUnitPayloadBlob {
        domain_family: "network".to_owned(),
        package_id: "official.network".to_owned(),
        backend_family: Some("urlsession".to_owned()),
        vendor: Some("apple".to_owned()),
        device_class: Some("socket-io".to_owned()),
        target_device: Some("urlsession-stack".to_owned()),
        ir_format: Some("host-ffi-plan".to_owned()),
        dispatch_abi: Some("nuis-host-call".to_owned()),
        backend_priority: Some(700),
        verification: Some("contract-only".to_owned()),
        selected_lowering_target: Some("urlsession".to_owned()),
        contract_family: "nustar.network".to_owned(),
        packaging_role: "hetero-contract".to_owned(),
        payload_kind: "contract-sidecar".to_owned(),
        payload_format: "toml".to_owned(),
        sections: vec![
            DomainBuildUnitPayloadBlobSection {
                name: "contract_toml".to_owned(),
                bytes: br#"schema = "nuis-domain-build-payload-v1""#.to_vec(),
            },
            DomainBuildUnitPayloadBlobSection {
                name: "lowering_plan".to_owned(),
                bytes: b"lowering".to_vec(),
            },
            DomainBuildUnitPayloadBlobSection {
                name: "backend_stub".to_owned(),
                bytes: b"backend".to_vec(),
            },
            DomainBuildUnitPayloadBlobSection {
                name: "bridge_plan".to_owned(),
                bytes: b"bridge".to_vec(),
            },
        ],
    };
    let payload_blob_hex = hex_encode_bytes(&encode_domain_payload_blob(&payload_blob).unwrap());
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
artifact_payload_blob_inline = "{payload_blob_hex}"
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
        payload_blob_hex = payload_blob_hex,
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
    assert_eq!(loaded.domain_payload_blobs.len(), 1);
    assert_eq!(
        loaded
            .payload_blob_for_domain("network")
            .unwrap()
            .backend_family
            .as_deref(),
        Some("urlsession")
    );
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

fn minimal_manifest_with_clock_protocol(clock_domains: usize, clock_protocol: &str) -> String {
    format!(
        r#"manifest_schema = "nuis-build-manifest-v1"
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

[clock_protocol]
clock_protocol_schema = "nuis-clock-protocol-v1"
clock_protocol_domains = {clock_domains}
clock_protocol_inline = "{clock_protocol}"

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
"#,
        clock_protocol = escape_toml_test_string(clock_protocol)
    )
}

fn minimal_artifact_with_manifest(manifest: &str) -> NuisCompiledArtifact {
    NuisCompiledArtifact {
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
        envelope: NuisExecutableEnvelope {
            schema: "nuis-executable-envelope-v1".to_owned(),
            executable_kind: "native-cpu-llvm".to_owned(),
            package_count: 1,
            domain_families: vec!["cpu".to_owned()],
            contract_families: vec!["nustar.cpu".to_owned()],
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
            hook_surface: vec!["on_bootstrap".to_owned()],
            export_surface: vec!["tick_export".to_owned()],
            runtime_capability_flags: vec!["runtime.tick".to_owned()],
        },
        build_manifest_source: manifest.to_owned(),
        binary_blob: b"bin".to_vec(),
    }
}

fn escape_toml_test_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn hex_encode_bytes(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}
