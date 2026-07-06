use crate::ArtifactError;

pub const NUIS_BINARY_FORMAT_PROTOCOL: &str = "nuis-binary-format-protocol-v1";

pub const BUILD_MANIFEST_SCHEMA_V1: &str = "nuis-build-manifest-v1";
pub const COMPILED_ARTIFACT_SCHEMA_V1: &str = "nuis-compiled-artifact-v1";
pub const EXECUTABLE_ENVELOPE_SCHEMA_V1: &str = "nuis-executable-envelope-v1";
pub const LIFECYCLE_CONTRACT_SCHEMA_V1: &str = "nuis-lifecycle-contract-v1";

pub const COMPILED_ARTIFACT_MAGIC: &[u8; 4] = b"NART";
pub const COMPILED_ARTIFACT_BINARY_VERSION: u16 = 1;
pub const COMPILED_ARTIFACT_SECTION_TABLE_BINARY_VERSION: u16 = 2;
pub const EXECUTABLE_ENVELOPE_MAGIC: &[u8; 4] = b"NENV";
pub const EXECUTABLE_ENVELOPE_BINARY_VERSION: u16 = 1;
pub const DOMAIN_PAYLOAD_BLOB_MAGIC: &[u8; 4] = b"NDPB";
pub const DOMAIN_PAYLOAD_BLOB_BINARY_VERSION: u16 = 3;

pub const DOMAIN_PAYLOAD_SECTION_CONTRACT_TOML: &str = "contract_toml";
pub const DOMAIN_PAYLOAD_SECTION_LOWERING_PLAN: &str = "lowering_plan";
pub const DOMAIN_PAYLOAD_SECTION_BACKEND_STUB: &str = "backend_stub";
pub const DOMAIN_PAYLOAD_SECTION_BRIDGE_PLAN: &str = "bridge_plan";
pub const DOMAIN_PAYLOAD_SECTION_SHADER_IR_SIDECAR: &str = "shader_ir_sidecar";
pub const DOMAIN_PAYLOAD_SECTION_KERNEL_IR_SIDECAR: &str = "kernel_ir_sidecar";
pub const DOMAIN_PAYLOAD_SECTION_NETWORK_IR_SIDECAR: &str = "network_ir_sidecar";

pub const COMPILED_ARTIFACT_SECTION_METADATA_TOML: &str = "metadata_toml";
pub const COMPILED_ARTIFACT_SECTION_ENVELOPE_BINARY: &str = "envelope_binary";
pub const COMPILED_ARTIFACT_SECTION_LIFECYCLE_TOML: &str = "lifecycle_toml";
pub const COMPILED_ARTIFACT_SECTION_BUILD_MANIFEST_TOML: &str = "build_manifest_toml";
pub const COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML: &str = "lowering_index_toml";
pub const COMPILED_ARTIFACT_SECTION_HOST_BINARY: &str = "host_binary";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NuisBinaryContainerProtocol {
    pub protocol: &'static str,
    pub compiled_artifact_magic: &'static [u8; 4],
    pub compiled_artifact_binary_version: u16,
    pub compiled_artifact_section_table_binary_version: u16,
    pub compiled_artifact_schema: &'static str,
    pub executable_envelope_magic: &'static [u8; 4],
    pub executable_envelope_binary_version: u16,
    pub executable_envelope_schema: &'static str,
    pub domain_payload_blob_magic: &'static [u8; 4],
    pub domain_payload_blob_binary_version: u16,
    pub build_manifest_schema: &'static str,
    pub lifecycle_contract_schema: &'static str,
}

pub const CURRENT_BINARY_CONTAINER_PROTOCOL: NuisBinaryContainerProtocol =
    NuisBinaryContainerProtocol {
        protocol: NUIS_BINARY_FORMAT_PROTOCOL,
        compiled_artifact_magic: COMPILED_ARTIFACT_MAGIC,
        compiled_artifact_binary_version: COMPILED_ARTIFACT_BINARY_VERSION,
        compiled_artifact_section_table_binary_version:
            COMPILED_ARTIFACT_SECTION_TABLE_BINARY_VERSION,
        compiled_artifact_schema: COMPILED_ARTIFACT_SCHEMA_V1,
        executable_envelope_magic: EXECUTABLE_ENVELOPE_MAGIC,
        executable_envelope_binary_version: EXECUTABLE_ENVELOPE_BINARY_VERSION,
        executable_envelope_schema: EXECUTABLE_ENVELOPE_SCHEMA_V1,
        domain_payload_blob_magic: DOMAIN_PAYLOAD_BLOB_MAGIC,
        domain_payload_blob_binary_version: DOMAIN_PAYLOAD_BLOB_BINARY_VERSION,
        build_manifest_schema: BUILD_MANIFEST_SCHEMA_V1,
        lifecycle_contract_schema: LIFECYCLE_CONTRACT_SCHEMA_V1,
    };

pub fn require_schema(actual: &str, expected: &str, label: &str) -> Result<(), ArtifactError> {
    if actual == expected {
        return Ok(());
    }
    Err(ArtifactError::new(format!(
        "unsupported nuis {label} schema `{actual}`; expected `{expected}`"
    )))
}

pub fn supported_domain_payload_sections() -> &'static [&'static str] {
    &[
        DOMAIN_PAYLOAD_SECTION_CONTRACT_TOML,
        DOMAIN_PAYLOAD_SECTION_LOWERING_PLAN,
        DOMAIN_PAYLOAD_SECTION_BACKEND_STUB,
        DOMAIN_PAYLOAD_SECTION_BRIDGE_PLAN,
        DOMAIN_PAYLOAD_SECTION_SHADER_IR_SIDECAR,
        DOMAIN_PAYLOAD_SECTION_KERNEL_IR_SIDECAR,
        DOMAIN_PAYLOAD_SECTION_NETWORK_IR_SIDECAR,
    ]
}

pub fn supported_compiled_artifact_sections() -> &'static [&'static str] {
    &[
        COMPILED_ARTIFACT_SECTION_METADATA_TOML,
        COMPILED_ARTIFACT_SECTION_ENVELOPE_BINARY,
        COMPILED_ARTIFACT_SECTION_LIFECYCLE_TOML,
        COMPILED_ARTIFACT_SECTION_BUILD_MANIFEST_TOML,
        COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML,
        COMPILED_ARTIFACT_SECTION_HOST_BINARY,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_protocol_descriptor_matches_exported_constants() {
        let protocol = CURRENT_BINARY_CONTAINER_PROTOCOL;

        assert_eq!(protocol.protocol, NUIS_BINARY_FORMAT_PROTOCOL);
        assert_eq!(protocol.compiled_artifact_magic, COMPILED_ARTIFACT_MAGIC);
        assert_eq!(
            protocol.compiled_artifact_binary_version,
            COMPILED_ARTIFACT_BINARY_VERSION
        );
        assert_eq!(
            protocol.compiled_artifact_section_table_binary_version,
            COMPILED_ARTIFACT_SECTION_TABLE_BINARY_VERSION
        );
        assert_eq!(
            protocol.compiled_artifact_schema,
            COMPILED_ARTIFACT_SCHEMA_V1
        );
        assert_eq!(
            protocol.executable_envelope_magic,
            EXECUTABLE_ENVELOPE_MAGIC
        );
        assert_eq!(
            protocol.executable_envelope_binary_version,
            EXECUTABLE_ENVELOPE_BINARY_VERSION
        );
        assert_eq!(
            protocol.executable_envelope_schema,
            EXECUTABLE_ENVELOPE_SCHEMA_V1
        );
        assert_eq!(
            protocol.domain_payload_blob_magic,
            DOMAIN_PAYLOAD_BLOB_MAGIC
        );
        assert_eq!(
            protocol.domain_payload_blob_binary_version,
            DOMAIN_PAYLOAD_BLOB_BINARY_VERSION
        );
        assert_eq!(protocol.build_manifest_schema, BUILD_MANIFEST_SCHEMA_V1);
        assert_eq!(
            protocol.lifecycle_contract_schema,
            LIFECYCLE_CONTRACT_SCHEMA_V1
        );
    }

    #[test]
    fn supported_payload_sections_include_domain_sidecars() {
        let sections = supported_domain_payload_sections();

        assert!(sections.contains(&DOMAIN_PAYLOAD_SECTION_CONTRACT_TOML));
        assert!(sections.contains(&DOMAIN_PAYLOAD_SECTION_SHADER_IR_SIDECAR));
        assert!(sections.contains(&DOMAIN_PAYLOAD_SECTION_KERNEL_IR_SIDECAR));
        assert!(sections.contains(&DOMAIN_PAYLOAD_SECTION_NETWORK_IR_SIDECAR));
    }

    #[test]
    fn supported_compiled_artifact_sections_include_linker_inputs() {
        let sections = supported_compiled_artifact_sections();

        assert!(sections.contains(&COMPILED_ARTIFACT_SECTION_METADATA_TOML));
        assert!(sections.contains(&COMPILED_ARTIFACT_SECTION_ENVELOPE_BINARY));
        assert!(sections.contains(&COMPILED_ARTIFACT_SECTION_LIFECYCLE_TOML));
        assert!(sections.contains(&COMPILED_ARTIFACT_SECTION_BUILD_MANIFEST_TOML));
        assert!(sections.contains(&COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML));
        assert!(sections.contains(&COMPILED_ARTIFACT_SECTION_HOST_BINARY));
    }
}
