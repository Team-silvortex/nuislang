use std::{fs, path::Path};

use crate::{ArtifactError, NuisExecutableEnvelope};

mod binary;
mod lowering_index;
mod metadata;
mod section_table;
mod support_materialize;

pub use binary::{decode_nuis_compiled_artifact_binary, encode_nuis_compiled_artifact_binary};
pub use lowering_index::parse_nuis_lowering_index_from_source;
use lowering_index::validate_lowering_index_against_build_manifest;
pub use section_table::{
    decode_nuis_compiled_artifact_section_table_binary,
    encode_nuis_compiled_artifact_section_table,
    encode_nuis_compiled_artifact_section_table_binary, validate_compiled_artifact_section_table,
};
pub use support_materialize::materialize_embedded_artifact_support;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuisLifecycleContract {
    pub schema: String,
    pub bootstrap_entry: String,
    pub tick_policy: String,
    pub shutdown_policy: String,
    pub yalivia_rpc: String,
    pub hook_surface: Vec<String>,
    pub export_surface: Vec<String>,
    pub runtime_capability_flags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuisCompiledArtifact {
    pub schema: String,
    pub packaging_mode: String,
    pub cpu_target_abi: String,
    pub cpu_target_machine_arch: String,
    pub cpu_target_machine_os: String,
    pub cpu_target_object_format: String,
    pub cpu_target_calling_abi: String,
    pub binary_name: String,
    pub binary_bytes: usize,
    pub build_manifest_bytes: usize,
    pub envelope: NuisExecutableEnvelope,
    pub lifecycle: NuisLifecycleContract,
    pub build_manifest_source: String,
    pub binary_blob: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuisCompiledArtifactSection {
    pub name: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuisCompiledArtifactSectionTable {
    pub sections: Vec<NuisCompiledArtifactSection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuisLoweringIndex {
    pub schema: String,
    pub packaging_mode: String,
    pub domain_unit_count: usize,
    pub units: Vec<NuisLoweringIndexUnit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuisLoweringIndexUnit {
    pub package_id: String,
    pub domain_family: String,
    pub backend_family: Option<String>,
    pub target_device: Option<String>,
    pub ir_format: Option<String>,
    pub dispatch_abi: Option<String>,
    pub backend_priority: Option<usize>,
    pub verification: Option<String>,
    pub selected_lowering_target: Option<String>,
    pub artifact_ir_sidecar_path: Option<String>,
    pub contract_family: String,
    pub packaging_role: String,
}

fn encode_u32_len(len: usize, what: &str) -> Result<[u8; 4], ArtifactError> {
    let len = u32::try_from(len)
        .map_err(|_| ArtifactError::new(format!("{what} exceeds 4 GiB and cannot be encoded")))?;
    Ok(len.to_le_bytes())
}

pub fn write_nuis_compiled_artifact(
    path: &Path,
    artifact: &NuisCompiledArtifact,
) -> Result<(), ArtifactError> {
    let out = encode_nuis_compiled_artifact_binary(artifact)?;
    fs::write(path, out).map_err(|error| {
        ArtifactError::new(format!("failed to write `{}`: {error}", path.display()))
    })
}

pub fn parse_nuis_compiled_artifact(path: &Path) -> Result<NuisCompiledArtifact, ArtifactError> {
    let bytes = fs::read(path).map_err(|error| {
        ArtifactError::new(format!("failed to read `{}`: {error}", path.display()))
    })?;
    decode_nuis_compiled_artifact_binary(&bytes)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        decode_nuis_compiled_artifact_binary, decode_nuis_compiled_artifact_section_table_binary,
        encode_domain_payload_blob, encode_nuis_compiled_artifact_section_table,
        encode_nuis_compiled_artifact_section_table_binary, materialize_embedded_artifact_support,
        parse_nuis_lowering_index_from_source,
        protocol::{
            COMPILED_ARTIFACT_SECTION_BUILD_MANIFEST_TOML,
            COMPILED_ARTIFACT_SECTION_ENVELOPE_BINARY, COMPILED_ARTIFACT_SECTION_HOST_BINARY,
            COMPILED_ARTIFACT_SECTION_LIFECYCLE_TOML,
            COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML, COMPILED_ARTIFACT_SECTION_METADATA_TOML,
        },
        DomainBuildUnitPayloadBlob, DomainBuildUnitPayloadBlobSection, NuisCompiledArtifact,
        NuisExecutableEnvelope, NuisLifecycleContract,
    };

    fn temp_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("nuis_artifact_{label}_{unique}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn hex_encode_bytes(bytes: &[u8]) -> String {
        let mut out = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            use std::fmt::Write as _;
            let _ = write!(&mut out, "{byte:02x}");
        }
        out
    }

    fn sample_compiled_artifact() -> NuisCompiledArtifact {
        let output_dir = temp_dir("sample_compiled_artifact");
        let input_path = output_dir.join("demo.ns");
        let envelope_path = output_dir.join("nuis.executable.envelope.toml");
        let artifact_path = output_dir.join("nuis.compiled.artifact");
        let build_manifest_source = format!(
            r#"manifest_schema = "nuis-build-manifest-v1"
input = "{input_path}"
output_dir = "{output_dir}"
packaging_mode = "native-cpu-llvm"
path = "{manifest_path}"
schema = "nuis-executable-envelope-v1"
package_count = 1
artifact_path = "{artifact_path}"
artifact_schema = "nuis-compiled-artifact-v1"
artifact_binary_name = "demo"
artifact_binary_bytes = 3
lifecycle_schema = "nuis-lifecycle-contract-v1"
lifecycle_bootstrap_entry = "nuis.bootstrap.lifecycle.v1"
lifecycle_tick_policy = "cooperative"
lifecycle_shutdown_policy = "graceful"
lifecycle_yalivia_rpc = "disabled"
lifecycle_hook_surface = ["on_bootstrap"]
lifecycle_export_surface = ["main"]
lifecycle_runtime_capability_flags = ["runtime.tick"]
function_kind = "function-node"
graph_kind = "function-graph"
default_time_mode = "logical"
cpu_target_abi = "cpu.arm64.apple_aapcs64"
cpu_target_machine_arch = "arm64"
cpu_target_machine_os = "darwin"
cpu_target_object_format = "mach-o"
cpu_target_calling_abi = "aapcs64-darwin"
cpu_target_clang = "arm64-apple-darwin"
cpu_target_cross = false

[[execution_contract]]
package_id = "official.cpu"
domain_family = "cpu"

[[domain_build_unit]]
package_id = "official.cpu"
domain_family = "cpu"
backend_family = "llvm"
selected_lowering_target = "llvm"
contract_family = "nustar.cpu"
packaging_role = "host-binary"
"#,
            input_path = input_path.display(),
            output_dir = output_dir.display(),
            manifest_path = envelope_path.display(),
            artifact_path = artifact_path.display()
        )
        .to_owned();
        NuisCompiledArtifact {
            schema: "nuis-compiled-artifact-v1".to_owned(),
            packaging_mode: "native-cpu-llvm".to_owned(),
            cpu_target_abi: "cpu.arm64.apple_aapcs64".to_owned(),
            cpu_target_machine_arch: "arm64".to_owned(),
            cpu_target_machine_os: "darwin".to_owned(),
            cpu_target_object_format: "mach-o".to_owned(),
            cpu_target_calling_abi: "aapcs64-darwin".to_owned(),
            binary_name: "demo".to_owned(),
            binary_bytes: 3,
            build_manifest_bytes: build_manifest_source.len(),
            envelope: NuisExecutableEnvelope {
                schema: "nuis-executable-envelope-v1".to_owned(),
                executable_kind: "native".to_owned(),
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
                yalivia_rpc: "disabled".to_owned(),
                hook_surface: vec!["on_bootstrap".to_owned()],
                export_surface: vec!["main".to_owned()],
                runtime_capability_flags: vec!["runtime.tick".to_owned()],
            },
            build_manifest_source,
            binary_blob: b"bin".to_vec(),
        }
    }

    #[test]
    fn section_table_artifact_roundtrips_through_generic_decoder() {
        let artifact = sample_compiled_artifact();

        let encoded = encode_nuis_compiled_artifact_section_table_binary(&artifact).unwrap();
        let table = decode_nuis_compiled_artifact_section_table_binary(&encoded).unwrap();
        let decoded = decode_nuis_compiled_artifact_binary(&encoded).unwrap();
        let section_names = table
            .sections
            .iter()
            .map(|section| section.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            section_names,
            vec![
                COMPILED_ARTIFACT_SECTION_METADATA_TOML,
                COMPILED_ARTIFACT_SECTION_ENVELOPE_BINARY,
                COMPILED_ARTIFACT_SECTION_LIFECYCLE_TOML,
                COMPILED_ARTIFACT_SECTION_BUILD_MANIFEST_TOML,
                COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML,
                COMPILED_ARTIFACT_SECTION_HOST_BINARY,
            ]
        );
        let lowering_index = table
            .section_utf8(COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML)
            .unwrap();
        assert!(lowering_index.contains("schema = \"nuis-lowering-index-v1\""));
        assert!(lowering_index.contains("domain_unit_count = 1"));
        assert!(lowering_index.contains("backend_family = \"llvm\""));
        assert!(lowering_index.contains("selected_lowering_target = \"llvm\""));
        assert_eq!(decoded, artifact);
    }

    #[test]
    fn lowering_index_parser_validates_generated_section() {
        let artifact = sample_compiled_artifact();
        let encoded = encode_nuis_compiled_artifact_section_table_binary(&artifact).unwrap();
        let table = decode_nuis_compiled_artifact_section_table_binary(&encoded).unwrap();
        let lowering_index_source = table
            .section_utf8(COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML)
            .unwrap();
        let lowering_index = parse_nuis_lowering_index_from_source(
            lowering_index_source,
            std::path::Path::new("<test-lowering-index>"),
        )
        .unwrap();

        assert_eq!(lowering_index.schema, "nuis-lowering-index-v1");
        assert_eq!(lowering_index.packaging_mode, "native-cpu-llvm");
        assert_eq!(lowering_index.domain_unit_count, 1);
        assert_eq!(lowering_index.units[0].domain_family, "cpu");
        assert_eq!(
            lowering_index.units[0].selected_lowering_target.as_deref(),
            Some("llvm")
        );
    }

    #[test]
    fn section_table_rejects_lowering_index_build_manifest_drift() {
        let artifact = sample_compiled_artifact();
        let encoded = encode_nuis_compiled_artifact_section_table_binary(&artifact).unwrap();
        let mut table = decode_nuis_compiled_artifact_section_table_binary(&encoded).unwrap();
        let lowering_section = table
            .sections
            .iter_mut()
            .find(|section| section.name == COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML)
            .unwrap();
        let drifted = std::str::from_utf8(&lowering_section.bytes)
            .unwrap()
            .replace(
                "selected_lowering_target = \"llvm\"",
                "selected_lowering_target = \"shader-msl\"",
            );
        lowering_section.bytes = drifted.into_bytes();

        let encoded_with_drift = encode_nuis_compiled_artifact_section_table(&table).unwrap();
        let error = decode_nuis_compiled_artifact_binary(&encoded_with_drift).unwrap_err();

        assert!(error
            .to_string()
            .contains("field `selected_lowering_target` value `shader-msl`"));
    }

    #[test]
    fn section_table_rejects_missing_required_section() {
        let artifact = sample_compiled_artifact();
        let encoded = encode_nuis_compiled_artifact_section_table_binary(&artifact).unwrap();
        let mut table = decode_nuis_compiled_artifact_section_table_binary(&encoded).unwrap();
        table
            .sections
            .retain(|section| section.name != COMPILED_ARTIFACT_SECTION_HOST_BINARY);

        let encoded_without_host = encode_nuis_compiled_artifact_section_table(&table).unwrap();
        let error =
            decode_nuis_compiled_artifact_section_table_binary(&encoded_without_host).unwrap_err();

        assert!(error
            .to_string()
            .contains("missing required section `host_binary`"));
    }

    #[test]
    fn section_table_rejects_duplicate_section_names() {
        let artifact = sample_compiled_artifact();
        let encoded = encode_nuis_compiled_artifact_section_table_binary(&artifact).unwrap();
        let mut table = decode_nuis_compiled_artifact_section_table_binary(&encoded).unwrap();
        table.sections.push(table.sections[0].clone());

        let encoded_with_duplicate = encode_nuis_compiled_artifact_section_table(&table).unwrap();
        let error = decode_nuis_compiled_artifact_section_table_binary(&encoded_with_duplicate)
            .unwrap_err();

        assert!(error
            .to_string()
            .contains("duplicate section `metadata_toml`"));
    }

    #[test]
    fn section_table_exposes_lookup_helpers_for_linker_consumers() {
        let artifact = sample_compiled_artifact();
        let encoded = encode_nuis_compiled_artifact_section_table_binary(&artifact).unwrap();
        let table = decode_nuis_compiled_artifact_section_table_binary(&encoded).unwrap();

        assert!(table.contains_section(COMPILED_ARTIFACT_SECTION_ENVELOPE_BINARY));
        assert!(table
            .section_names()
            .contains(&COMPILED_ARTIFACT_SECTION_BUILD_MANIFEST_TOML));
        assert_eq!(
            table
                .section_utf8(COMPILED_ARTIFACT_SECTION_BUILD_MANIFEST_TOML)
                .unwrap(),
            artifact.build_manifest_source
        );
        assert_eq!(
            table
                .section_bytes(COMPILED_ARTIFACT_SECTION_HOST_BINARY)
                .unwrap(),
            artifact.binary_blob
        );
    }

    #[test]
    fn materializes_embedded_heterogeneous_support_files() {
        let blob = DomainBuildUnitPayloadBlob {
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
                    bytes: b"bridge-plan".to_vec(),
                },
                DomainBuildUnitPayloadBlobSection {
                    name: "network_ir_sidecar".to_owned(),
                    bytes: b"schema = \"nuis-network-ir-sidecar-v1\"".to_vec(),
                },
            ],
        };
        let encoded_blob = encode_domain_payload_blob(&blob).unwrap();
        let output_dir = temp_dir("sample_window_aot_bundle");
        let input_path = output_dir.join("demo.ns");
        let envelope_path = output_dir.join("nuis.executable.envelope.toml");
        let artifact_path = output_dir.join("nuis.compiled.artifact");
        let bridge_registry_path = output_dir.join("nuis.bridge.registry.toml");
        let host_bridge_plan_index_path = output_dir.join("nuis.host-bridge.plan-index.toml");
        let network_artifact_path = output_dir.join("nuis.domain.network.artifact.toml");
        let network_payload_path = output_dir.join("nuis.domain.network.payload.toml");
        let network_bridge_stub_path = output_dir.join("nuis.domain.network.bridge.stub.txt");
        let network_ir_sidecar_path = output_dir.join("nuis.domain.network.lowering.ir.txt");
        let network_payload_blob_path = output_dir.join("nuis.domain.network.payload.bin");
        let manifest = format!(
            r#"manifest_schema = "nuis-build-manifest-v1"
input = "{input_path}"
output_dir = "{output_dir}"
packaging_mode = "window-aot-bundle"
path = "{manifest_path}"
schema = "nuis-executable-envelope-v1"
package_count = 2
artifact_path = "{artifact_path}"
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
bridge_registry_path = "{bridge_registry_path}"
bridge_registry_schema = "nuis-bridge-registry-v1"
bridge_registry_units = 1
bridge_registry_inline = "schema = \"nuis-bridge-registry-v1\"\nbridge_count = 1\ndomains = [\"network\"]\n"
host_bridge_plan_index_path = "{host_bridge_plan_index_path}"
host_bridge_plan_index_schema = "nuis-host-bridge-plan-index-v1"
host_bridge_plan_units = 1
host_bridge_plan_index_inline = "schema = \"nuis-host-bridge-plan-index-v1\"\nplan_count = 1\ndomains = [\"network\"]\n"

[[artifact_hash]]
kind = "artifact"
path = "{artifact_path}"
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
artifact_stub_path = "{artifact_stub_path}"
artifact_stub_inline = "schema = \"nuis-domain-build-unit-v1\""
artifact_payload_path = "{artifact_payload_path}"
artifact_bridge_stub_path = "{artifact_bridge_stub_path}"
artifact_ir_sidecar_path = "{artifact_ir_sidecar_path}"
artifact_bridge_stub_inline = "schema = \"nuis-host-bridge-spec-v1\""
artifact_payload_blob_path = "{artifact_payload_blob_path}"
artifact_payload_blob_bytes = {blob_bytes}
artifact_payload_format = "ndpb-v2"
artifact_payload_blob_inline = "{blob_hex}"
contract_family = "nustar.network"
packaging_role = "hetero-contract"
"#,
            input_path = input_path.display(),
            output_dir = output_dir.display(),
            manifest_path = envelope_path.display(),
            artifact_path = artifact_path.display(),
            bridge_registry_path = bridge_registry_path.display(),
            host_bridge_plan_index_path = host_bridge_plan_index_path.display(),
            artifact_stub_path = network_artifact_path.display(),
            artifact_payload_path = network_payload_path.display(),
            artifact_bridge_stub_path = network_bridge_stub_path.display(),
            artifact_ir_sidecar_path = network_ir_sidecar_path.display(),
            artifact_payload_blob_path = network_payload_blob_path.display(),
            blob_bytes = encoded_blob.len(),
            blob_hex = hex_encode_bytes(&encoded_blob),
        );
        let artifact = NuisCompiledArtifact {
            schema: "nuis-compiled-artifact-v1".to_owned(),
            packaging_mode: "window-aot-bundle".to_owned(),
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
                executable_kind: "window-aot-bundle".to_owned(),
                package_count: 2,
                domain_families: vec!["cpu".to_owned(), "network".to_owned()],
                contract_families: vec!["nustar.cpu".to_owned(), "nustar.network".to_owned()],
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
            build_manifest_source: manifest,
            binary_blob: b"bin".to_vec(),
        };

        let out = temp_dir("materialize_support");
        let written = materialize_embedded_artifact_support(&artifact, &out).unwrap();

        assert!(written
            .iter()
            .any(|path| path.ends_with("nuis.bridge.registry.toml")));
        assert!(written
            .iter()
            .any(|path| path.ends_with("nuis.host-bridge.plan-index.toml")));
        assert!(written
            .iter()
            .any(|path| path.ends_with("nuis.domain.network.artifact.toml")));
        assert!(written
            .iter()
            .any(|path| path.ends_with("nuis.domain.network.payload.toml")));
        assert!(written
            .iter()
            .any(|path| path.ends_with("nuis.domain.network.payload.bin")));
        assert!(written
            .iter()
            .any(|path| path.ends_with("nuis.domain.network.bridge.stub.txt")));
        assert!(written
            .iter()
            .any(|path| path.ends_with("nuis.domain.network.lowering.ir.txt")));
        assert_eq!(
            fs::read(out.join("nuis.domain.network.payload.bin")).unwrap(),
            encoded_blob
        );
        assert_eq!(
            fs::read(out.join("nuis.domain.network.payload.toml")).unwrap(),
            br#"schema = "nuis-domain-build-payload-v1""#
        );
        assert_eq!(
            fs::read_to_string(out.join("nuis.domain.network.bridge.stub.txt")).unwrap(),
            r#"schema = "nuis-host-bridge-spec-v1""#
        );
        assert_eq!(
            fs::read_to_string(out.join("nuis.domain.network.lowering.ir.txt")).unwrap(),
            r#"schema = "nuis-network-ir-sidecar-v1""#
        );
    }
}
